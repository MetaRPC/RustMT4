//! `MT4Service` — the mid-level facade over [`MT4Account`] (Layer 2).
//!
//! Friendly arguments (`&str` symbols, tickets, typed sort enums) and clean return values —
//! building the protobuf `…Request` for you and unwrapping simple payloads. Order construction,
//! pip math and risk sizing live one layer up in [`crate::mt4::sugar`].

use prost_types::Timestamp;
use tonic::Streaming;

use crate::error::MT4Error;
use crate::mt4::account::MT4Account;
use crate::pb::mt4_term_api as pb;

pub struct MT4Service {
    acc: MT4Account,
}

impl MT4Service {
    /// Wrap a low-level account.
    pub fn new(acc: MT4Account) -> Self {
        Self { acc }
    }
    /// Borrow the underlying [`MT4Account`].
    pub fn account(&self) -> &MT4Account {
        &self.acc
    }
    /// Mutably borrow the underlying [`MT4Account`].
    pub fn account_mut(&mut self) -> &mut MT4Account {
        &mut self.acc
    }

    // ===== connection =====

    pub async fn connect_by_server_name(
        &self,
        login: u64,
        password: &str,
        server: &str,
        name: Option<&str>,
        timeout_seconds: u32,
    ) -> Result<pb::ConnectData, MT4Error> {
        self.acc.connect_by_server_name(login, password, server, name, timeout_seconds).await
    }
    pub async fn reconnect(&self) -> Result<(), MT4Error> {
        self.acc.reconnect().await
    }
    pub async fn disconnect(&self, delete: bool) -> Result<pb::DisconnectData, MT4Error> {
        self.acc.disconnect(delete).await
    }

    // ===== 1. Account information =====

    /// Full account snapshot.
    pub async fn account_summary(&self) -> Result<pb::AccountSummaryData, MT4Error> {
        self.acc.account_summary().await
    }
    /// Aggregated trade statistics (optionally from a server-time `yyyy-MM-ddTHH:mm:ss`).
    pub async fn trade_stats(&self, from: Option<String>) -> Result<pb::StatsData, MT4Error> {
        self.acc.trade_stats(pb::TradeStatsRequest { from, ..Default::default() }).await
    }
    /// Stats + equity points + trades.
    pub async fn trade_stats_equity_history(
        &self,
        from: Option<String>,
        exclude_same_bars: Option<bool>,
    ) -> Result<pb::StatsWithChartsData, MT4Error> {
        self.acc
            .trade_stats_equity_history(pb::TradeStatsEquityHistoryRequest { from, exclude_same_bars, ..Default::default() })
            .await
    }
    /// The equity-curve points.
    pub async fn equity_history(
        &self,
        from: Option<String>,
        exclude_same_bars: Option<bool>,
    ) -> Result<pb::EquityHistoryData, MT4Error> {
        self.acc
            .equity_history(pb::EquityHistoryRequest { from, exclude_same_bars, ..Default::default() })
            .await
    }
    /// Unrealized-P/L extrema per open order.
    pub async fn trade_unrealized_pl(&self, from: Option<String>) -> Result<pb::TradeUnrealizedPlData, MT4Error> {
        self.acc.trade_unrealized_pl(pb::TradeUnrealizedPlRequest { from, ..Default::default() }).await
    }

    // ===== 2. Orders & history =====

    /// Open market + pending orders, in the requested sort order.
    pub async fn opened_orders(&self, sort: pb::EnumOpenedOrderSortType) -> Result<Vec<pb::OpenedOrderInfo>, MT4Error> {
        Ok(self.acc.opened_orders(pb::OpenedOrdersRequest { sort_type: sort as i32 }).await?.order_infos)
    }
    /// Just the ticket numbers of open/pending orders.
    pub async fn opened_orders_tickets(&self) -> Result<Vec<i32>, MT4Error> {
        Ok(self.acc.opened_orders_tickets().await?.tickets)
    }
    /// Closed/cancelled order history over a window, with sort and paging defaults.
    pub async fn orders_history(
        &self,
        sort: pb::EnumOrderHistorySortType,
        from: Option<Timestamp>,
        to: Option<Timestamp>,
    ) -> Result<pb::OrdersHistoryData, MT4Error> {
        self.acc
            .orders_history(pb::OrdersHistoryRequest {
                input_sort_mode: sort as i32,
                input_from: from,
                input_to: to,
                ..Default::default()
            })
            .await
    }

    // ===== 3. Symbols & quotes =====

    /// Every symbol name the terminal knows.
    pub async fn symbols(&self) -> Result<Vec<pb::SymbolNameInfo>, MT4Error> {
        Ok(self.acc.symbols().await?.symbol_name_infos)
    }
    /// Add/remove a symbol in Market Watch; returns whether it succeeded.
    pub async fn symbol_select(&self, symbol: &str, select: bool) -> Result<bool, MT4Error> {
        Ok(self.acc.symbol_select(pb::SymbolSelectRequest { symbol: symbol.to_string(), select }).await?.success)
    }
    /// Full parameter sets for one symbol (`Some(name)`) or all (`None`).
    pub async fn symbol_params_many(&self, name_filter: Option<&str>) -> Result<pb::SymbolParamsManyData, MT4Error> {
        self.acc
            .symbol_params_many(pb::SymbolParamsManyRequest { symbol_name: name_filter.map(|s| s.to_string()) })
            .await
    }
    /// Tick value / size / contract size for the given symbols.
    pub async fn tick_value_with_size(&self, symbols: &[&str]) -> Result<pb::TickValueWithSizeData, MT4Error> {
        self.acc
            .tick_value_with_size(pb::TickValueWithSizeRequest { symbol_names: symbols.iter().map(|s| s.to_string()).collect() })
            .await
    }
    /// The latest tick (bid/ask + session high/low) for one symbol.
    pub async fn quote(&self, symbol: &str) -> Result<pb::QuoteData, MT4Error> {
        self.acc.quote(pb::QuoteRequest { symbol: symbol.to_string() }).await
    }
    /// Latest ticks for several symbols.
    pub async fn quote_many(&self, symbols: &[&str]) -> Result<pb::QuoteManyData, MT4Error> {
        self.acc.quote_many(pb::QuoteManyRequest { symbols: symbols.iter().map(|s| s.to_string()).collect() }).await
    }
    /// OHLC bars for a symbol/timeframe. `timeframe` is an `EnumQuoteHistoryTimeframe` value
    /// (M1 = 0 … MN1 = 8); `from`/`to` are **server-time** bounds.
    pub async fn quote_history(
        &self,
        symbol: &str,
        timeframe: i32,
        from: Option<Timestamp>,
        to: Option<Timestamp>,
    ) -> Result<pb::QuoteHistoryData, MT4Error> {
        self.acc
            .quote_history(pb::QuoteHistoryRequest {
                symbol: symbol.to_string(),
                timeframe,
                from_time: from,
                to_time: to,
                ..Default::default()
            })
            .await
    }
    /// The broker server's UTC offset.
    pub async fn server_timezone(&self) -> Result<pb::ServerTimezoneData, MT4Error> {
        self.acc.server_timezone().await
    }

    // ===== 4. Trading operations =====

    /// Send a market or pending order (raw request — see [`crate::mt4::sugar`] for buy/sell helpers).
    pub async fn order_send(&self, request: pb::OrderSendRequest) -> Result<pb::OrderSendData, MT4Error> {
        self.acc.order_send(request).await
    }
    /// Change an order's SL/TP (`None` leaves a side untouched; `Some(0.0)` removes a stop).
    pub async fn order_modify(
        &self,
        ticket: u64,
        stop_loss: Option<f64>,
        take_profit: Option<f64>,
    ) -> Result<pb::OrderModifyData, MT4Error> {
        self.acc
            .order_modify(pb::OrderModifyRequest {
                order_ticket: ticket as i32,
                new_stop_loss: stop_loss,
                new_take_profit: take_profit,
                ..Default::default()
            })
            .await
    }
    /// Close a market order (`lots = None` closes fully, `Some(x)` partially) or delete a pending.
    pub async fn order_close_delete(
        &self,
        ticket: u64,
        lots: Option<f64>,
        slippage: i32,
    ) -> Result<pb::OrderCloseDeleteData, MT4Error> {
        self.acc
            .order_close_delete(pb::OrderCloseDeleteRequest {
                order_ticket: ticket as i32,
                lots,
                closing_price: None,
                slippage: Some(slippage),
            })
            .await
    }
    /// Close an order by an opposite one.
    pub async fn order_close_by(&self, ticket: u64, opposite_ticket: u64) -> Result<pb::OrderCloseByData, MT4Error> {
        self.acc
            .order_close_by(pb::OrderCloseByRequest {
                ticket_to_close: ticket as i32,
                opposite_ticket_closing_by: opposite_ticket as i32,
            })
            .await
    }

    // ===== 5. Streaming =====

    /// Live tick stream for the given symbols.
    pub async fn stream_ticks(&self, symbols: &[&str]) -> Result<Streaming<pb::OnSymbolTickReply>, MT4Error> {
        self.acc
            .on_symbol_tick(pb::OnSymbolTickRequest { symbol_names: symbols.iter().map(|s| s.to_string()).collect() })
            .await
    }
    /// Live trade-event stream.
    pub async fn stream_trades(&self) -> Result<Streaming<pb::OnTradeReply>, MT4Error> {
        self.acc.on_trade(pb::OnTradeRequest {}).await
    }
    /// Live open-order ticket list, emitted every `interval_ms`.
    pub async fn stream_opened_tickets(&self, interval_ms: i32) -> Result<Streaming<pb::OnOpenedOrdersTicketsReply>, MT4Error> {
        self.acc
            .on_opened_orders_tickets(pb::OnOpenedOrdersTicketsRequest { pull_interval_milliseconds: interval_ms })
            .await
    }
    /// Live per-order floating profit, emitted every `interval_ms`.
    pub async fn stream_opened_profit(&self, interval_ms: i32) -> Result<Streaming<pb::OnOpenedOrdersProfitReply>, MT4Error> {
        self.acc
            .on_opened_orders_profit(pb::OnOpenedOrdersProfitRequest { timer_period_milliseconds: interval_ms })
            .await
    }
    /// Live terminal connection-state events.
    pub async fn stream_connect_state(&self) -> Result<Streaming<pb::OnConnectStateReply>, MT4Error> {
        self.acc.on_connect_state(pb::OnConnectStateRequest { terminal_ids: vec![] }).await
    }
}
