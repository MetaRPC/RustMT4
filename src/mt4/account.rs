//! `MT4Account` — the low-level async gRPC client (Layer 1).
//!
//! Direct calls to the MetaRPC MT4 gateway: builds each protobuf `…Request`, attaches the
//! `id` + `apikey` metadata, and unwraps the reply's `oneof { data, error }` into
//! `Result<…Data, MT4Error>`. Friendly argument shapes live in [`crate::mt4::service`] and the
//! high-level helpers in [`crate::mt4::sugar`].
//!
//! Every call takes `&self`; the client is `Send + Sync` and can be shared behind an `Arc`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tonic::{Request, Streaming};

use crate::error::MT4Error;
use crate::pb::mt4_term_api as pb;
use pb::account_helper_client::AccountHelperClient;
use pb::connection_client::ConnectionClient;
use pb::market_info_client::MarketInfoClient;
use pb::server_timezone_client::ServerTimezoneClient;
use pb::subscription_service_client::SubscriptionServiceClient;
use pb::trade_stats_client::TradeStatsClient;
use pb::trading_helper_client::TradingHelperClient;

/// Unwrap a `…Reply { response: oneof { Data, Error } }` into `Result<Data, MT4Error>`.
macro_rules! unwrap_reply {
    ($reply:expr, $module:ident) => {
        match $reply.into_inner().response {
            Some(pb::$module::Response::Data(d)) => Ok(d),
            Some(pb::$module::Response::Error(e)) => Err(MT4Error::from_api(e)),
            None => Err(MT4Error::Protocol(concat!("empty ", stringify!($module)).to_string())),
        }
    };
}

/// Stored connection parameters, reused by `reconnect`.
#[derive(Debug, Clone)]
struct ConnectParams {
    login: u64,
    password: String,
    server: String,
    name: Option<String>,
    timeout_seconds: u32,
}

/// Low-level MT4 gRPC client.
pub struct MT4Account {
    host: String,
    port: u16,
    api_key: Option<String>,
    id: Option<String>,
    call_timeout: Duration,
    max_retries: u32,
    channel: Mutex<Option<Channel>>,
    terminal_guid: Mutex<Option<String>>,
    params: Mutex<Option<ConnectParams>>,
    connected: AtomicBool,
    server_utc_shift: Mutex<Option<i64>>,
}

impl MT4Account {
    // ===== construction =====

    /// New client for `host:port` (e.g. `"mt4.mrpc.pro", 443`). API key from `MRPC_API_KEY`.
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self::build(host.into(), port, std::env::var("MRPC_API_KEY").ok(), None)
    }

    /// New client with an explicit API key.
    pub fn with_api_key(host: impl Into<String>, port: u16, api_key: impl Into<String>) -> Self {
        Self::build(host.into(), port, Some(api_key.into()), None)
    }

    fn build(host: String, port: u16, api_key: Option<String>, id: Option<String>) -> Self {
        MT4Account {
            host,
            port,
            api_key,
            id,
            call_timeout: Duration::from_secs(60),
            max_retries: 3,
            channel: Mutex::new(None),
            terminal_guid: Mutex::new(None),
            params: Mutex::new(None),
            connected: AtomicBool::new(false),
            server_utc_shift: Mutex::new(None),
        }
    }

    /// Stable terminal id from a label (UUID v5) — the same label reuses the same terminal across
    /// runs, so restarts don't orphan a new terminal (the gateway caps terminals per key).
    pub fn stable_terminal_id(label: &str) -> String {
        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, label.as_bytes()).to_string()
    }

    /// Set a fixed terminal `id` (pass a [`stable_terminal_id`](Self::stable_terminal_id)).
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the per-call deadline (default 60 s).
    pub fn with_call_timeout(mut self, t: Duration) -> Self {
        self.call_timeout = t;
        self
    }

    /// The id the gateway knows this terminal by (returned `terminalInstanceGuid`, else `id`).
    pub fn terminal_id(&self) -> Option<String> {
        self.terminal_guid.lock().unwrap().clone().or_else(|| self.id.clone())
    }

    /// Whether a successful `ConnectEx` has completed.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Broker server time minus UTC in seconds, once known (after `account_summary`).
    pub fn server_utc_shift_secs(&self) -> Option<i64> {
        *self.server_utc_shift.lock().unwrap()
    }

    // ===== transport =====

    fn channel(&self) -> Result<Channel, MT4Error> {
        let mut guard = self.channel.lock().unwrap();
        if let Some(ch) = guard.as_ref() {
            return Ok(ch.clone());
        }
        let uri = format!("https://{}:{}", self.host, self.port);
        let endpoint = Endpoint::from_shared(uri)
            .map_err(|e| MT4Error::Transport(e.to_string()))?
            .tls_config(ClientTlsConfig::new().with_webpki_roots().domain_name(self.host.clone()))
            .map_err(|e| MT4Error::Transport(e.to_string()))?
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(20))
            .keep_alive_while_idle(true)
            .connect_timeout(Duration::from_secs(20));
        let ch = endpoint.connect_lazy();
        *guard = Some(ch.clone());
        Ok(ch)
    }

    /// Wrap a message in a `tonic::Request` with `id` + `apikey` metadata and an optional deadline.
    fn request<T>(&self, msg: T, timeout: Option<Duration>) -> Result<Request<T>, MT4Error> {
        let mut req = Request::new(msg);
        if let Some(t) = timeout {
            req.set_timeout(t);
        }
        let md = req.metadata_mut();
        if let Some(id) = self.terminal_id() {
            md.insert("id", id.parse().map_err(|_| MT4Error::Protocol("bad id".into()))?);
        }
        let key = self.api_key.clone().unwrap_or_else(|| "TRIAL".to_string());
        md.insert("apikey", key.parse().map_err(|_| MT4Error::Protocol("bad api key".into()))?);
        Ok(req)
    }

    fn req<T>(&self, msg: T) -> Result<Request<T>, MT4Error> {
        self.request(msg, Some(self.call_timeout))
    }

    /// Run `op`, retrying transient transport errors with back-off and re-`ConnectEx`-ing once per
    /// attempt on `TERMINAL_INSTANCE_NOT_FOUND`. An invalid login is never retried.
    async fn call<T>(&self, op: impl AsyncFn() -> Result<T, MT4Error>) -> Result<T, MT4Error> {
        let mut attempt: u32 = 0;
        loop {
            match op().await {
                Ok(v) => return Ok(v),
                Err(e) => {
                    attempt += 1;
                    if attempt > self.max_retries || !e.is_retryable() {
                        return Err(e);
                    }
                    if e.is_terminal_not_found() {
                        if let Err(re) = self.reconnect().await {
                            if matches!(re, MT4Error::InvalidLogin(_)) {
                                return Err(re);
                            }
                        }
                    }
                    let backoff = Duration::from_millis(500 * (1u64 << attempt.min(5)));
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    // ===== connection =====

    /// `Connection/ConnectEx` by broker **server name** (`mt_cluster_name`, e.g. `"FreshForex-Demo"`).
    /// Blocks until the terminal has logged in (up to `timeout_seconds`, default 120 when 0).
    pub async fn connect_by_server_name(
        &self,
        login: u64,
        password: &str,
        server: &str,
        name: Option<&str>,
        timeout_seconds: u32,
    ) -> Result<pb::ConnectData, MT4Error> {
        let params = ConnectParams {
            login,
            password: password.to_string(),
            server: server.to_string(),
            name: name.map(|s| s.to_string()),
            timeout_seconds,
        };
        *self.params.lock().unwrap() = Some(params.clone());
        self.do_connect(&params).await
    }

    async fn do_connect(&self, p: &ConnectParams) -> Result<pb::ConnectData, MT4Error> {
        if self.id.is_none() && self.terminal_guid.lock().unwrap().is_none() {
            let fresh = uuid::Uuid::new_v4().to_string();
            *self.terminal_guid.lock().unwrap() = Some(fresh);
        }
        let timeout = if p.timeout_seconds == 0 { 120 } else { p.timeout_seconds };
        let msg = pb::ConnectExRequest {
            user: p.login,
            password: p.password.clone(),
            mt_cluster_name: p.server.clone(),
            experts_to_add: vec![],
            timeout_seconds: Some(timeout),
            name: p.name.clone(),
            expiration: None,
        };
        let req = self.request(msg, Some(Duration::from_secs(timeout as u64 + 30)))?;
        let mut client = ConnectionClient::new(self.channel()?);
        let reply = client.connect_ex(req).await?;
        let data: pb::ConnectData = unwrap_reply!(reply, connect_ex_reply)?;
        if !data.terminal_instance_guid.is_empty() {
            *self.terminal_guid.lock().unwrap() = Some(data.terminal_instance_guid.clone());
        }
        self.connected.store(true, Ordering::SeqCst);
        Ok(data)
    }

    /// Re-run `ConnectEx` with the stored parameters (same terminal id).
    pub async fn reconnect(&self) -> Result<(), MT4Error> {
        let p = self.params.lock().unwrap().clone().ok_or(MT4Error::NotConnected)?;
        self.do_connect(&p).await.map(|_| ())
    }

    /// `Connection/CheckConnect` — cheap liveness probe of the terminal.
    pub async fn check_connect(&self) -> Result<pb::CheckConnectData, MT4Error> {
        let mut c = ConnectionClient::new(self.channel()?);
        let r = c.check_connect(self.req(pb::CheckConnectRequest {})?).await?;
        unwrap_reply!(r, check_connect_reply)
    }

    /// `Connection/Disconnect`. `delete = true` removes the terminal permanently; `false` stops it
    /// (reusable by a later `ConnectEx` with the same id).
    pub async fn disconnect(&self, delete: bool) -> Result<pb::DisconnectData, MT4Error> {
        let mut c = ConnectionClient::new(self.channel()?);
        let msg = pb::DisconnectRequest { reason: Some("client disconnect".into()), delete: Some(delete) };
        let r = c.disconnect(self.req(msg)?).await?;
        self.connected.store(false, Ordering::SeqCst);
        unwrap_reply!(r, disconnect_reply)
    }

    // ===== 1. Account information =====

    /// `AccountHelper/AccountSummary` — login, balance, equity, currency, leverage, trade mode.
    pub async fn account_summary(&self) -> Result<pb::AccountSummaryData, MT4Error> {
        self.call(async || {
            let mut c = AccountHelperClient::new(self.channel()?);
            let r = c.account_summary(self.req(pb::AccountSummaryRequest {})?).await?;
            let d: pb::AccountSummaryData = unwrap_reply!(r, account_summary_reply)?;
            *self.server_utc_shift.lock().unwrap() = Some(d.utc_server_time_shift_minutes as i64 * 60);
            Ok(d)
        })
        .await
    }

    /// `TradeStats/TradeStats` — aggregated trade statistics.
    pub async fn trade_stats(&self, request: pb::TradeStatsRequest) -> Result<pb::StatsData, MT4Error> {
        self.call(async || {
            let mut c = TradeStatsClient::new(self.channel()?);
            let r = c.trade_stats(self.req(request.clone())?).await?;
            unwrap_reply!(r, trade_stats_reply)
        })
        .await
    }

    /// `TradeStats/TradeStatsEquityHistory` — stats + equity points + trades.
    pub async fn trade_stats_equity_history(
        &self,
        request: pb::TradeStatsEquityHistoryRequest,
    ) -> Result<pb::StatsWithChartsData, MT4Error> {
        self.call(async || {
            let mut c = TradeStatsClient::new(self.channel()?);
            let r = c.trade_stats_equity_history(self.req(request.clone())?).await?;
            unwrap_reply!(r, trade_stats_equity_history_reply)
        })
        .await
    }

    /// `TradeStats/EquityHistory` — the equity-curve points.
    pub async fn equity_history(&self, request: pb::EquityHistoryRequest) -> Result<pb::EquityHistoryData, MT4Error> {
        self.call(async || {
            let mut c = TradeStatsClient::new(self.channel()?);
            let r = c.equity_history(self.req(request.clone())?).await?;
            unwrap_reply!(r, equity_history_reply)
        })
        .await
    }

    /// `TradeStats/TradeUnrealizedPL` — unrealized-P/L extrema per open order.
    pub async fn trade_unrealized_pl(
        &self,
        request: pb::TradeUnrealizedPlRequest,
    ) -> Result<pb::TradeUnrealizedPlData, MT4Error> {
        self.call(async || {
            let mut c = TradeStatsClient::new(self.channel()?);
            let r = c.trade_unrealized_pl(self.req(request.clone())?).await?;
            unwrap_reply!(r, trade_unrealized_pl_reply)
        })
        .await
    }

    // ===== 2. Orders & history =====

    /// `AccountHelper/OpenedOrders` — all currently open and pending orders.
    pub async fn opened_orders(&self, request: pb::OpenedOrdersRequest) -> Result<pb::OpenedOrdersData, MT4Error> {
        self.call(async || {
            let mut c = AccountHelperClient::new(self.channel()?);
            let r = c.opened_orders(self.req(request.clone())?).await?;
            unwrap_reply!(r, opened_orders_reply)
        })
        .await
    }

    /// `AccountHelper/OpenedOrdersTickets` — just the ticket numbers of open/pending orders.
    pub async fn opened_orders_tickets(&self) -> Result<pb::OpenedOrdersTicketsData, MT4Error> {
        self.call(async || {
            let mut c = AccountHelperClient::new(self.channel()?);
            let r = c.opened_orders_tickets(self.req(pb::OpenedOrdersTicketsRequest {})?).await?;
            unwrap_reply!(r, opened_orders_tickets_reply)
        })
        .await
    }

    /// `AccountHelper/OrdersHistory` — closed/cancelled orders over a window.
    pub async fn orders_history(&self, request: pb::OrdersHistoryRequest) -> Result<pb::OrdersHistoryData, MT4Error> {
        self.call(async || {
            let mut c = AccountHelperClient::new(self.channel()?);
            let r = c.orders_history(self.req(request.clone())?).await?;
            unwrap_reply!(r, orders_history_reply)
        })
        .await
    }

    // ===== 3. Symbols & quotes =====

    /// `MarketInfo/Symbols` — the broker's symbol list.
    pub async fn symbols(&self) -> Result<pb::SymbolsData, MT4Error> {
        self.call(async || {
            let mut c = MarketInfoClient::new(self.channel()?);
            let r = c.symbols(self.req(pb::SymbolsRequest {})?).await?;
            unwrap_reply!(r, symbols_reply)
        })
        .await
    }

    /// `MarketInfo/SymbolSelect` — add/remove a symbol in Market Watch.
    pub async fn symbol_select(&self, request: pb::SymbolSelectRequest) -> Result<pb::SymbolSelectData, MT4Error> {
        self.call(async || {
            let mut c = MarketInfoClient::new(self.channel()?);
            let r = c.symbol_select(self.req(request.clone())?).await?;
            unwrap_reply!(r, symbol_select_reply)
        })
        .await
    }

    /// `AccountHelper/SymbolParamsMany` — full parameter sets for symbols.
    pub async fn symbol_params_many(
        &self,
        request: pb::SymbolParamsManyRequest,
    ) -> Result<pb::SymbolParamsManyData, MT4Error> {
        self.call(async || {
            let mut c = AccountHelperClient::new(self.channel()?);
            let r = c.symbol_params_many(self.req(request.clone())?).await?;
            unwrap_reply!(r, symbol_params_many_reply)
        })
        .await
    }

    /// `AccountHelper/TickValueWithSize` — tick value / size / contract size per symbol.
    pub async fn tick_value_with_size(
        &self,
        request: pb::TickValueWithSizeRequest,
    ) -> Result<pb::TickValueWithSizeData, MT4Error> {
        self.call(async || {
            let mut c = AccountHelperClient::new(self.channel()?);
            let r = c.tick_value_with_size(self.req(request.clone())?).await?;
            unwrap_reply!(r, tick_value_with_size_reply)
        })
        .await
    }

    /// `MarketInfo/Quote` — the latest tick for one symbol.
    pub async fn quote(&self, request: pb::QuoteRequest) -> Result<pb::QuoteData, MT4Error> {
        self.call(async || {
            let mut c = MarketInfoClient::new(self.channel()?);
            let r = c.quote(self.req(request.clone())?).await?;
            unwrap_reply!(r, quote_reply)
        })
        .await
    }

    /// `MarketInfo/QuoteMany` — latest ticks for several symbols.
    pub async fn quote_many(&self, request: pb::QuoteManyRequest) -> Result<pb::QuoteManyData, MT4Error> {
        self.call(async || {
            let mut c = MarketInfoClient::new(self.channel()?);
            let r = c.quote_many(self.req(request.clone())?).await?;
            unwrap_reply!(r, quote_many_reply)
        })
        .await
    }

    /// `MarketInfo/QuoteHistory` — OHLC bars for a symbol/timeframe range.
    pub async fn quote_history(&self, request: pb::QuoteHistoryRequest) -> Result<pb::QuoteHistoryData, MT4Error> {
        self.call(async || {
            let mut c = MarketInfoClient::new(self.channel()?);
            let r = c.quote_history(self.req(request.clone())?).await?;
            unwrap_reply!(r, quote_history_reply)
        })
        .await
    }

    /// `ServerTimezone/ServerTimezone` — the broker server's UTC offset.
    pub async fn server_timezone(&self) -> Result<pb::ServerTimezoneData, MT4Error> {
        self.call(async || {
            let mut c = ServerTimezoneClient::new(self.channel()?);
            let r = c.server_timezone(self.req(pb::ServerTimezoneRequest {})?).await?;
            unwrap_reply!(r, server_timezone_reply)
        })
        .await
    }

    // ===== 4. Trading operations =====

    /// `TradingHelper/OrderSend` — open a market/pending order.
    pub async fn order_send(&self, request: pb::OrderSendRequest) -> Result<pb::OrderSendData, MT4Error> {
        let mut c = TradingHelperClient::new(self.channel()?);
        let r = c.order_send(self.req(request)?).await?;
        unwrap_reply!(r, order_send_reply)
    }

    /// `TradingHelper/OrderModify` — change SL/TP (open order) or price/SL/TP/expiration (pending).
    pub async fn order_modify(&self, request: pb::OrderModifyRequest) -> Result<pb::OrderModifyData, MT4Error> {
        let mut c = TradingHelperClient::new(self.channel()?);
        let r = c.order_modify(self.req(request)?).await?;
        unwrap_reply!(r, order_modify_reply)
    }

    /// `TradingHelper/OrderCloseDelete` — close a market order or delete a pending one.
    pub async fn order_close_delete(
        &self,
        request: pb::OrderCloseDeleteRequest,
    ) -> Result<pb::OrderCloseDeleteData, MT4Error> {
        let mut c = TradingHelperClient::new(self.channel()?);
        let r = c.order_close_delete(self.req(request)?).await?;
        unwrap_reply!(r, order_close_delete_reply)
    }

    /// `TradingHelper/OrderCloseBy` — close an order by an opposite one.
    pub async fn order_close_by(&self, request: pb::OrderCloseByRequest) -> Result<pb::OrderCloseByData, MT4Error> {
        let mut c = TradingHelperClient::new(self.channel()?);
        let r = c.order_close_by(self.req(request)?).await?;
        unwrap_reply!(r, order_close_by_reply)
    }

    // ===== 5. Streaming =====

    /// `SubscriptionService/OnSymbolTick` — live ticks for the requested symbols.
    pub async fn on_symbol_tick(
        &self,
        request: pb::OnSymbolTickRequest,
    ) -> Result<Streaming<pb::OnSymbolTickReply>, MT4Error> {
        let mut c = SubscriptionServiceClient::new(self.channel()?);
        let r = c.on_symbol_tick(self.request(request, None)?).await?;
        Ok(r.into_inner())
    }

    /// `SubscriptionService/OnTrade` — order/trade events.
    pub async fn on_trade(&self, request: pb::OnTradeRequest) -> Result<Streaming<pb::OnTradeReply>, MT4Error> {
        let mut c = SubscriptionServiceClient::new(self.channel()?);
        let r = c.on_trade(self.request(request, None)?).await?;
        Ok(r.into_inner())
    }

    /// `SubscriptionService/OnOpenedOrdersTickets` — the open-order ticket list on a timer.
    pub async fn on_opened_orders_tickets(
        &self,
        request: pb::OnOpenedOrdersTicketsRequest,
    ) -> Result<Streaming<pb::OnOpenedOrdersTicketsReply>, MT4Error> {
        let mut c = SubscriptionServiceClient::new(self.channel()?);
        let r = c.on_opened_orders_tickets(self.request(request, None)?).await?;
        Ok(r.into_inner())
    }

    /// `SubscriptionService/OnOpenedOrdersProfit` — per-order floating profit on a timer.
    pub async fn on_opened_orders_profit(
        &self,
        request: pb::OnOpenedOrdersProfitRequest,
    ) -> Result<Streaming<pb::OnOpenedOrdersProfitReply>, MT4Error> {
        let mut c = SubscriptionServiceClient::new(self.channel()?);
        let r = c.on_opened_orders_profit(self.request(request, None)?).await?;
        Ok(r.into_inner())
    }

    /// `SubscriptionService/OnConnectState` — terminal connection-state events.
    pub async fn on_connect_state(
        &self,
        request: pb::OnConnectStateRequest,
    ) -> Result<Streaming<pb::OnConnectStateReply>, MT4Error> {
        let mut c = ConnectionClient::new(self.channel()?);
        let r = c.on_connect_state(self.request(request, None)?).await?;
        Ok(r.into_inner())
    }
}
