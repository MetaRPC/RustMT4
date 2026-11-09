//! `MT4Sugar` — the high-level convenience API over [`MT4Service`] (Layer 3).
//!
//! One-liners and business logic: quick prices, symbol math (point/pip/normalize), risk-based lot
//! sizing, order construction from `symbol`/side/`lots`, order management, and account snapshots.
//! Reuses the plain DTOs in [`crate::models`] (`Quote`, `Bar`, `OrderRequest`, `OrderResult`, …).

use std::time::Duration;

use tokio::sync::mpsc;

use crate::error::MT4Error;
use crate::models::*;
use crate::mt4::service::MT4Service;
use crate::pb::mt4_term_api as pb;

const SORT_OPEN_ASC: pb::EnumOpenedOrderSortType = pb::EnumOpenedOrderSortType::SortByOpenTimeAsc;

pub struct MT4Sugar {
    svc: MT4Service,
    magic: i32,
    slippage: u32,
    comment: String,
}

impl MT4Sugar {
    /// Wrap a service.
    pub fn new(svc: MT4Service) -> Self {
        Self { svc, magic: 0, slippage: 20, comment: String::new() }
    }

    /// Set the default magic / slippage (points) / comment used by the order-entry helpers.
    pub fn set_defaults(mut self, magic: i32, slippage: u32, comment: impl Into<String>) -> Self {
        self.magic = magic;
        self.slippage = slippage;
        self.comment = comment.into();
        self
    }

    pub fn service(&self) -> &MT4Service {
        &self.svc
    }
    pub fn service_mut(&mut self) -> &mut MT4Service {
        &mut self.svc
    }

    // ===== 1. Connection & defaults =====

    pub async fn connect_by_server_name(
        &self,
        login: u64,
        password: &str,
        server: &str,
        name: Option<&str>,
        timeout_seconds: u32,
    ) -> Result<(), MT4Error> {
        self.svc.connect_by_server_name(login, password, server, name, timeout_seconds).await.map(|_| ())
    }
    pub async fn reconnect(&self) -> Result<(), MT4Error> {
        self.svc.reconnect().await
    }
    pub async fn disconnect(&self, delete: bool) -> Result<(), MT4Error> {
        self.svc.disconnect(delete).await.map(|_| ())
    }
    /// Liveness check — true if an account snapshot succeeds.
    pub async fn ping(&self) -> bool {
        self.svc.account_summary().await.is_ok()
    }

    // ----- internal: symbol params -----

    async fn sym(&self, symbol: &str) -> Result<pb::SymbolParamsManyInfo, MT4Error> {
        self.svc
            .symbol_params_many(Some(symbol))
            .await?
            .symbol_infos
            .into_iter()
            .find(|x| x.symbol_name == symbol)
            .ok_or_else(|| MT4Error::Protocol(format!("symbol {symbol} not in SymbolParamsMany reply")))
    }

    // ===== 2. Prices & quotes =====

    /// Latest [`Quote`] for a symbol.
    pub async fn last_quote(&self, symbol: &str) -> Result<Quote, MT4Error> {
        Ok(Quote::from_quote(&self.svc.quote(symbol).await?))
    }
    /// Latest quotes for several symbols.
    pub async fn quotes(&self, symbols: &[&str]) -> Result<Vec<Quote>, MT4Error> {
        Ok(self.svc.quote_many(symbols).await?.quotes.iter().map(Quote::from_quote).collect())
    }
    /// Mid price `(bid + ask) / 2`.
    pub async fn mid_price(&self, symbol: &str) -> Result<f64, MT4Error> {
        let q = self.last_quote(symbol).await?;
        Ok((q.bid + q.ask) / 2.0)
    }
    /// Current spread in pips.
    pub async fn spread_pips(&self, symbol: &str) -> Result<f64, MT4Error> {
        let q = self.last_quote(symbol).await?;
        let pip = self.pip_size(symbol).await?;
        Ok(if pip > 0.0 { (q.ask - q.bid) / pip } else { 0.0 })
    }
    /// Select a symbol, then poll until it publishes a usable two-sided quote (or time out).
    pub async fn ensure_symbol(&self, symbol: &str, wait: Duration) -> Result<Quote, MT4Error> {
        self.svc.symbol_select(symbol, true).await?;
        let deadline = std::time::Instant::now() + wait;
        let mut last: Option<Quote> = None;
        loop {
            if let Ok(q) = self.last_quote(symbol).await {
                if q.is_usable() {
                    return Ok(q);
                }
                last = Some(q);
            }
            if std::time::Instant::now() >= deadline {
                return Err(MT4Error::Protocol(format!("{symbol}: no usable quote within {wait:?} (last={last:?})")));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
    /// The newest `count` bars of `timeframe` (M1 = 0 … MN1 = 8), oldest first.
    pub async fn bars(&self, symbol: &str, timeframe: i32, count: usize) -> Result<Vec<Bar>, MT4Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let tf = pb_timeframe_seconds(timeframe);
        let from = now - tf * count as i64 * 3;
        let hist = self
            .svc
            .quote_history(symbol, timeframe, Some(ts(from)), Some(ts(now + 3600)))
            .await?;
        let mut bars: Vec<Bar> = hist.historical_quotes.iter().map(Bar::from).collect();
        bars.sort_by_key(|b| b.time);
        let start = bars.len().saturating_sub(count);
        Ok(bars[start..].to_vec())
    }

    // ===== 3. Symbol math =====

    /// Point size (`SYMBOL_POINT`).
    pub async fn point(&self, symbol: &str) -> Result<f64, MT4Error> {
        Ok(self.sym(symbol).await?.point)
    }
    /// Price decimal places.
    pub async fn digits(&self, symbol: &str) -> Result<i32, MT4Error> {
        Ok(self.sym(symbol).await?.digits)
    }
    /// Pip size — `point × 10` on 3/5-digit symbols, else `point`.
    pub async fn pip_size(&self, symbol: &str) -> Result<f64, MT4Error> {
        let s = self.sym(symbol).await?;
        Ok(if s.digits == 3 || s.digits == 5 { s.point * 10.0 } else { s.point })
    }
    /// Convert a pip distance into a price distance.
    pub async fn pips_to_price(&self, symbol: &str, pips: f64) -> Result<f64, MT4Error> {
        Ok(pips * self.pip_size(symbol).await?)
    }
    /// Convert a price distance into pips.
    pub async fn price_to_pips(&self, symbol: &str, price_delta: f64) -> Result<f64, MT4Error> {
        let pip = self.pip_size(symbol).await?;
        Ok(if pip > 0.0 { price_delta / pip } else { 0.0 })
    }
    /// Round a lot size to the symbol's volume step and clamp to [min, max].
    pub async fn normalize_lot(&self, symbol: &str, lot: f64) -> Result<f64, MT4Error> {
        let s = self.sym(symbol).await?;
        let step = if s.volume_step > 0.0 { s.volume_step } else { 0.01 };
        let mut v = (lot / step).round() * step;
        if v < s.volume_min {
            v = s.volume_min;
        } else if s.volume_max > 0.0 && v > s.volume_max {
            v = s.volume_max;
        }
        Ok(v)
    }
    /// Round a price to the symbol's digits.
    pub async fn normalize_price(&self, symbol: &str, price: f64) -> Result<f64, MT4Error> {
        let d = self.sym(symbol).await?.digits.max(0) as u32;
        let f = 10f64.powi(d as i32);
        Ok((price * f).round() / f)
    }
    /// Money value of one tick per lot (`SYMBOL_TRADE_TICK_VALUE`).
    pub async fn tick_value(&self, symbol: &str) -> Result<f64, MT4Error> {
        Ok(self.sym(symbol).await?.trade_tick_value)
    }

    // ===== 4. Risk sizing =====

    /// Cash at risk for `lots` over an `sl_pips` stop.
    pub async fn calc_cash_risk(&self, symbol: &str, lots: f64, sl_pips: f64) -> Result<f64, MT4Error> {
        Ok(lots * sl_pips * self.pip_value_per_lot(symbol).await?)
    }
    /// Lot size that risks `risk_cash` of the deposit currency over an `sl_pips` stop, normalized.
    pub async fn calc_lot_by_risk(&self, symbol: &str, risk_cash: f64, sl_pips: f64) -> Result<f64, MT4Error> {
        let pv = self.pip_value_per_lot(symbol).await?;
        if sl_pips <= 0.0 || pv <= 0.0 {
            return self.normalize_lot(symbol, 0.0).await;
        }
        let raw = risk_cash / (sl_pips * pv);
        self.normalize_lot(symbol, raw).await
    }

    async fn pip_value_per_lot(&self, symbol: &str) -> Result<f64, MT4Error> {
        let s = self.sym(symbol).await?;
        let pip = if s.digits == 3 || s.digits == 5 { s.point * 10.0 } else { s.point };
        if s.trade_tick_size > 0.0 {
            Ok(s.trade_tick_value * (pip / s.trade_tick_size))
        } else {
            Ok(s.trade_tick_value)
        }
    }

    // ===== 5. Order entry =====

    /// Send a fully-built [`OrderRequest`]; maps MQL failures to [`MT4Error::TradeError`].
    pub async fn place_order(&self, req: OrderRequest) -> Result<OrderResult, MT4Error> {
        match self.svc.order_send(req.to_proto()).await {
            Ok(d) => Ok(OrderResult::from_proto(d)),
            Err(MT4Error::Api { code, message, mql_code, mql_desc, .. }) if mql_code != 0 => {
                Err(MT4Error::TradeError(mql_code as u32, format!("{code}: {message} ({mql_desc})")))
            }
            Err(e) => Err(e),
        }
    }

    fn base(&self, symbol: &str, side: OrderType, lots: f64) -> OrderRequest {
        let mut r = OrderRequest::market(symbol, side, lots).with_slippage(self.slippage);
        if self.magic != 0 {
            r = r.with_magic(self.magic);
        }
        if !self.comment.is_empty() {
            r = r.with_comment(&self.comment);
        }
        r
    }

    /// Market BUY.
    pub async fn buy_market(&self, symbol: &str, lots: f64, sl: Option<f64>, tp: Option<f64>) -> Result<OrderResult, MT4Error> {
        self.place_order(self.base(symbol, OrderType::Buy, lots).with_sl_tp(sl, tp)).await
    }
    /// Market SELL.
    pub async fn sell_market(&self, symbol: &str, lots: f64, sl: Option<f64>, tp: Option<f64>) -> Result<OrderResult, MT4Error> {
        self.place_order(self.base(symbol, OrderType::Sell, lots).with_sl_tp(sl, tp)).await
    }
    /// BUY LIMIT pending order at `price`.
    pub async fn buy_limit(&self, symbol: &str, lots: f64, price: f64, sl: Option<f64>, tp: Option<f64>) -> Result<OrderResult, MT4Error> {
        let mut r = self.base(symbol, OrderType::BuyLimit, lots).with_sl_tp(sl, tp);
        r.price = Some(price);
        self.place_order(r).await
    }
    /// SELL LIMIT pending order at `price`.
    pub async fn sell_limit(&self, symbol: &str, lots: f64, price: f64, sl: Option<f64>, tp: Option<f64>) -> Result<OrderResult, MT4Error> {
        let mut r = self.base(symbol, OrderType::SellLimit, lots).with_sl_tp(sl, tp);
        r.price = Some(price);
        self.place_order(r).await
    }
    /// BUY STOP pending order at `price`.
    pub async fn buy_stop(&self, symbol: &str, lots: f64, price: f64, sl: Option<f64>, tp: Option<f64>) -> Result<OrderResult, MT4Error> {
        let mut r = self.base(symbol, OrderType::BuyStop, lots).with_sl_tp(sl, tp);
        r.price = Some(price);
        self.place_order(r).await
    }
    /// SELL STOP pending order at `price`.
    pub async fn sell_stop(&self, symbol: &str, lots: f64, price: f64, sl: Option<f64>, tp: Option<f64>) -> Result<OrderResult, MT4Error> {
        let mut r = self.base(symbol, OrderType::SellStop, lots).with_sl_tp(sl, tp);
        r.price = Some(price);
        self.place_order(r).await
    }

    // ===== 6. Order management =====

    /// Close a market order fully (`lots = None`) or partially, or delete a pending.
    pub async fn close(&self, ticket: u64, lots: Option<f64>) -> Result<CloseResult, MT4Error> {
        let d = self.svc.order_close_delete(ticket, lots, self.slippage as i32).await?;
        Ok(CloseResult { mode: d.mode, history_order_comment: d.history_order_comment })
    }
    /// Close part of a market order by volume.
    pub async fn close_partial(&self, ticket: u64, lots: f64) -> Result<CloseResult, MT4Error> {
        self.close(ticket, Some(lots)).await
    }
    /// Close an order by an opposite one.
    pub async fn close_by(&self, ticket: u64, opposite_ticket: u64) -> Result<pb::OrderCloseByData, MT4Error> {
        self.svc.order_close_by(ticket, opposite_ticket).await
    }
    /// Close every open **market** order (optionally only on `symbol`); returns how many closed.
    pub async fn close_all(&self, symbol: Option<&str>) -> Result<usize, MT4Error> {
        let mut n = 0;
        for o in self.positions().await? {
            if let Some(s) = symbol {
                if o.symbol != s {
                    continue;
                }
            }
            if self.close(o.ticket as u64, None).await.is_ok() {
                n += 1;
            }
        }
        Ok(n)
    }
    /// Delete every pending order (optionally only on `symbol`); returns how many deleted.
    pub async fn cancel_pendings(&self, symbol: Option<&str>) -> Result<usize, MT4Error> {
        let mut n = 0;
        for o in self.orders().await? {
            let is_pending = OrderType::from_i32(o.order_type).map(|t| !t.is_market()).unwrap_or(false);
            if !is_pending {
                continue;
            }
            if let Some(s) = symbol {
                if o.symbol != s {
                    continue;
                }
            }
            if self.close(o.ticket as u64, None).await.is_ok() {
                n += 1;
            }
        }
        Ok(n)
    }
    /// Set a position's SL/TP to absolute prices.
    pub async fn modify_sl_tp_by_price(&self, ticket: u64, sl: Option<f64>, tp: Option<f64>) -> Result<(), MT4Error> {
        self.svc.order_modify(ticket, sl, tp).await.map(|_| ())
    }
    /// Set a position's SL/TP as pip distances from its open price (direction-aware).
    pub async fn modify_sl_tp_by_pips(&self, ticket: u64, sl_pips: Option<f64>, tp_pips: Option<f64>) -> Result<(), MT4Error> {
        let o = self
            .find_ticket(ticket)
            .await?
            .ok_or_else(|| MT4Error::Protocol(format!("order {ticket} not open")))?;
        let pip = self.pip_size(&o.symbol).await?;
        let buy = OrderType::from_i32(o.order_type).map(|t| t.is_buy()).unwrap_or(true);
        let sign = if buy { 1.0 } else { -1.0 };
        let sl = sl_pips.map(|p| o.open_price - sign * p * pip);
        let tp = tp_pips.map(|p| o.open_price + sign * p * pip);
        self.modify_sl_tp_by_price(ticket, sl, tp).await
    }

    // ===== 9. Order info =====

    /// All open market + pending orders.
    pub async fn orders(&self) -> Result<Vec<pb::OpenedOrderInfo>, MT4Error> {
        self.svc.opened_orders(SORT_OPEN_ASC).await
    }
    /// Open **market** orders only (MT4's equivalent of "positions").
    pub async fn positions(&self) -> Result<Vec<pb::OpenedOrderInfo>, MT4Error> {
        Ok(self
            .orders()
            .await?
            .into_iter()
            .filter(|o| OrderType::from_i32(o.order_type).map(|t| t.is_market()).unwrap_or(false))
            .collect())
    }
    /// Open orders on a symbol.
    pub async fn orders_for_symbol(&self, symbol: &str) -> Result<Vec<pb::OpenedOrderInfo>, MT4Error> {
        Ok(self.orders().await?.into_iter().filter(|o| o.symbol == symbol).collect())
    }
    /// Open orders tagged with a magic number.
    pub async fn orders_by_magic(&self, magic: i32) -> Result<Vec<pb::OpenedOrderInfo>, MT4Error> {
        Ok(self.orders().await?.into_iter().filter(|o| o.magic_number == magic).collect())
    }
    /// One open order by ticket (if present).
    pub async fn find_ticket(&self, ticket: u64) -> Result<Option<pb::OpenedOrderInfo>, MT4Error> {
        Ok(self.orders().await?.into_iter().find(|o| o.ticket as u64 == ticket))
    }
    /// Total floating profit across open positions (optionally on a symbol).
    pub async fn total_profit(&self, symbol: Option<&str>) -> Result<f64, MT4Error> {
        Ok(self
            .positions()
            .await?
            .iter()
            .filter(|o| symbol.map(|s| o.symbol == s).unwrap_or(true))
            .map(|o| o.profit + o.swap + o.commision)
            .sum())
    }
    /// A compact account snapshot.
    pub async fn get_account_info(&self) -> Result<AccountInfo, MT4Error> {
        let d = self.svc.account_summary().await?;
        Ok(AccountInfo::from_summary(&d, ""))
    }
    /// Total floating value of positions (optionally filtered by symbol and/or magic), incl. swap
    /// and commission.
    pub async fn positions_value(&self, symbol: Option<&str>, magic: Option<i32>) -> Result<f64, MT4Error> {
        Ok(self
            .positions()
            .await?
            .iter()
            .filter(|o| symbol.map(|s| o.symbol == s).unwrap_or(true))
            .filter(|o| magic.map(|m| o.magic_number == m).unwrap_or(true))
            .map(|o| o.profit + o.swap + o.commision)
            .sum())
    }
    /// Net lots per symbol across open market positions (buy positive, sell negative).
    pub async fn exposure_summary(&self) -> Result<Vec<(String, f64)>, MT4Error> {
        use std::collections::BTreeMap;
        let mut m: BTreeMap<String, f64> = BTreeMap::new();
        for o in self.positions().await? {
            let buy = OrderType::from_i32(o.order_type).map(|t| t.is_buy()).unwrap_or(true);
            *m.entry(o.symbol.clone()).or_insert(0.0) += if buy { o.lots } else { -o.lots };
        }
        Ok(m.into_iter().collect())
    }

    // ===== 7. Trailing & breakeven =====

    /// If the position is at least `trigger_pips` in profit, move its stop to `entry ± plus_pips`
    /// (never loosening). Returns whether the stop was moved. Call it on a timer to keep it active.
    pub async fn auto_breakeven(&self, ticket: u64, trigger_pips: f64, plus_pips: f64) -> Result<bool, MT4Error> {
        let o = self.find_ticket(ticket).await?.ok_or_else(|| MT4Error::Protocol(format!("order {ticket} not open")))?;
        let pip = self.pip_size(&o.symbol).await?;
        let q = self.last_quote(&o.symbol).await?;
        let buy = OrderType::from_i32(o.order_type).map(|t| t.is_buy()).unwrap_or(true);
        let profit_pips = if buy { (q.bid - o.open_price) / pip } else { (o.open_price - q.ask) / pip };
        if profit_pips < trigger_pips {
            return Ok(false);
        }
        let new_sl = if buy { o.open_price + plus_pips * pip } else { o.open_price - plus_pips * pip };
        let better = o.stop_loss == 0.0 || if buy { new_sl > o.stop_loss } else { new_sl < o.stop_loss };
        if !better {
            return Ok(false);
        }
        self.modify_sl_tp_by_price(ticket, Some(new_sl), None).await?;
        Ok(true)
    }

    /// Ratchet the stop to `distance_pips` behind the current price (only ever tightening). Returns
    /// whether the stop was moved. Call it on a timer to trail continuously.
    pub async fn trail_stop(&self, ticket: u64, distance_pips: f64) -> Result<bool, MT4Error> {
        let o = self.find_ticket(ticket).await?.ok_or_else(|| MT4Error::Protocol(format!("order {ticket} not open")))?;
        let pip = self.pip_size(&o.symbol).await?;
        let q = self.last_quote(&o.symbol).await?;
        let buy = OrderType::from_i32(o.order_type).map(|t| t.is_buy()).unwrap_or(true);
        let desired = if buy { q.bid - distance_pips * pip } else { q.ask + distance_pips * pip };
        let better = o.stop_loss == 0.0 || if buy { desired > o.stop_loss } else { desired < o.stop_loss };
        if !better {
            return Ok(false);
        }
        self.modify_sl_tp_by_price(ticket, Some(desired), None).await?;
        Ok(true)
    }

    // ===== 8. Watchers & waiters =====

    /// Subscribe to live ticks for `symbols`, delivered as [`Quote`]s on an mpsc channel. The
    /// background task ends on the first stream error or when the receiver is dropped.
    pub async fn subscribe_quotes(&self, symbols: Vec<String>) -> Result<mpsc::Receiver<Quote>, MT4Error> {
        let refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let mut stream = self.svc.stream_ticks(&refs).await?;
        let (tx, rx) = mpsc::channel(1024);
        tokio::spawn(async move {
            loop {
                match stream.message().await {
                    Ok(Some(reply)) => match reply.response {
                        Some(pb::on_symbol_tick_reply::Response::Data(d)) => {
                            if let Some(t) = d.symbol_tick {
                                if tx.send(Quote::from_subscription_tick(&t)).await.is_err() {
                                    break;
                                }
                            }
                        }
                        _ => break,
                    },
                    _ => break,
                }
            }
        });
        Ok(rx)
    }

    /// Stream per-order floating profit every `interval_ms`, on an mpsc channel.
    pub async fn watch_profit(&self, interval_ms: i32) -> Result<mpsc::Receiver<pb::OnOpenedOrdersProfitData>, MT4Error> {
        let mut stream = self.svc.stream_opened_profit(interval_ms).await?;
        let (tx, rx) = mpsc::channel(256);
        tokio::spawn(async move {
            loop {
                match stream.message().await {
                    Ok(Some(reply)) => match reply.response {
                        Some(pb::on_opened_orders_profit_reply::Response::Data(d)) => {
                            if tx.send(d).await.is_err() {
                                break;
                            }
                        }
                        _ => break,
                    },
                    _ => break,
                }
            }
        });
        Ok(rx)
    }

    /// Stream the open-order ticket list every `interval_ms`, on an mpsc channel.
    pub async fn watch_tickets(&self, interval_ms: i32) -> Result<mpsc::Receiver<pb::OnOpenedOrdersTicketsData>, MT4Error> {
        let mut stream = self.svc.stream_opened_tickets(interval_ms).await?;
        let (tx, rx) = mpsc::channel(256);
        tokio::spawn(async move {
            loop {
                match stream.message().await {
                    Ok(Some(reply)) => match reply.response {
                        Some(pb::on_opened_orders_tickets_reply::Response::Data(d)) => {
                            if tx.send(d).await.is_err() {
                                break;
                            }
                        }
                        _ => break,
                    },
                    _ => break,
                }
            }
        });
        Ok(rx)
    }

    /// Poll until a symbol's price reaches `target` (`above = true` waits for bid ≥ target, else
    /// ask ≤ target), or time out.
    pub async fn wait_price(&self, symbol: &str, target: f64, above: bool, timeout: Duration) -> Result<Quote, MT4Error> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let q = self.last_quote(symbol).await?;
            let hit = if above { q.bid >= target } else { q.ask <= target };
            if hit {
                return Ok(q);
            }
            if std::time::Instant::now() >= deadline {
                return Err(MT4Error::Protocol(format!("{symbol}: price did not reach {target} within {timeout:?}")));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    /// Poll until a pending order `ticket` becomes a market order (filled). `true` on fill, `false`
    /// on timeout.
    pub async fn wait_filled(&self, ticket: u64, timeout: Duration) -> Result<bool, MT4Error> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if let Some(o) = self.find_ticket(ticket).await? {
                if OrderType::from_i32(o.order_type).map(|t| t.is_market()).unwrap_or(false) {
                    return Ok(true);
                }
            }
            if std::time::Instant::now() >= deadline {
                return Ok(false);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    /// Poll until `ticket` is no longer open (closed/deleted). `true` on close, `false` on timeout.
    pub async fn wait_closed(&self, ticket: u64, timeout: Duration) -> Result<bool, MT4Error> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if self.find_ticket(ticket).await?.is_none() {
                return Ok(true);
            }
            if std::time::Instant::now() >= deadline {
                return Ok(false);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}

// ----- free helpers -----

fn ts(seconds: i64) -> prost_types::Timestamp {
    prost_types::Timestamp { seconds, nanos: 0 }
}

/// Seconds per `EnumQuoteHistoryTimeframe` value (M1 = 0 … MN1 = 8).
fn pb_timeframe_seconds(tf: i32) -> i64 {
    match tf {
        0 => 60,
        1 => 300,
        2 => 900,
        3 => 1800,
        4 => 3600,
        5 => 14400,
        6 => 86400,
        7 => 604800,
        8 => 2592000,
        _ => 60,
    }
}
