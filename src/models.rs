use serde::{Deserialize, Serialize};

use crate::pb::mt4_term_api as pb;

/// `AccountSummaryData.account_trade_mode` decoded. Fleet runners refuse to trade unless `Demo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeMode {
    Demo,
    Contest,
    Real,
    Unknown(i32),
}

impl TradeMode {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => TradeMode::Demo,
            1 => TradeMode::Contest,
            2 => TradeMode::Real,
            other => TradeMode::Unknown(other),
        }
    }
    pub fn is_demo(&self) -> bool {
        matches!(self, TradeMode::Demo)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub login: u64,
    pub currency: String,
    pub balance: f64,
    pub equity: f64,
    pub credit: f64,
    /// MT4 `AccountSummary` has no margin / free-margin fields.
    pub leverage: u32,
    pub name: String,
    pub company: String,
    pub server: String,
    pub trade_mode: TradeMode,
    pub is_investor: bool,
    /// Broker server time (unix seconds) at the moment of the reply.
    pub server_time: i64,
    /// Broker server time minus UTC, in minutes. History windows and all history timestamps are
    /// in server time.
    pub utc_shift_minutes: i64,
}

impl AccountInfo {
    pub fn from_summary(d: &pb::AccountSummaryData, server: &str) -> Self {
        AccountInfo {
            login: d.account_login as u64,
            currency: d.account_currency.clone(),
            balance: d.account_balance,
            equity: d.account_equity,
            credit: d.account_credit,
            leverage: d.account_leverage as u32,
            name: d.account_user_name.clone(),
            company: d.account_company_name.clone(),
            server: server.to_string(),
            trade_mode: TradeMode::from_i32(d.account_trade_mode),
            is_investor: d.is_investor,
            server_time: d.server_time.as_ref().map(|t| t.seconds).unwrap_or(0),
            utc_shift_minutes: d.utc_server_time_shift_minutes as i64,
        }
    }
}

/// One quote from `MarketInfo/Quote` or the `OnSymbolTick` stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    /// Unix seconds (0 when the gateway sent no timestamp).
    pub time: i64,
    /// Unix milliseconds.
    pub time_msc: i64,
    /// Session high / low (`Quote` only, 0 for stream ticks).
    pub high: f64,
    pub low: f64,
}

impl Quote {
    pub fn from_quote(d: &pb::QuoteData) -> Self {
        Quote {
            symbol: d.symbol.clone(),
            bid: d.bid,
            ask: d.ask,
            time: d.date_time.as_ref().map(|t| t.seconds).unwrap_or(0),
            time_msc: d
                .date_time
                .as_ref()
                .map(|t| t.seconds * 1000 + (t.nanos / 1_000_000) as i64)
                .unwrap_or(0),
            high: d.high,
            low: d.low,
        }
    }
    pub fn from_subscription_tick(t: &pb::OnSymbolMqlTickInfo) -> Self {
        Quote {
            symbol: t.symbol.clone(),
            bid: t.bid,
            ask: t.ask,
            time: t.time.as_ref().map(|x| x.seconds).unwrap_or(t.time_msc / 1000),
            time_msc: t.time_msc,
            high: 0.0,
            low: 0.0,
        }
    }
    /// Demo feeds intermittently publish `bid == ask` or zero; such quotes are unusable.
    pub fn is_usable(&self) -> bool {
        self.bid > 0.0 && self.ask > 0.0 && self.ask > self.bid
    }
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }
}

/// MT4 order operation. Market = `Buy` / `Sell`; the rest are pending orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Buy,
    Sell,
    BuyLimit,
    SellLimit,
    BuyStop,
    SellStop,
}

impl OrderType {
    /// Maps to `OrderSendOperationType` (`OC_OP_BUY` = 0 ... `OC_OP_SELLSTOP` = 5), the same
    /// numbering as MQL4 `OP_BUY`..`OP_SELLSTOP` and as `OpenedOrderType` in opened/history orders.
    pub fn to_operation(self) -> pb::OrderSendOperationType {
        use pb::OrderSendOperationType as T;
        match self {
            OrderType::Buy => T::OcOpBuy,
            OrderType::Sell => T::OcOpSell,
            OrderType::BuyLimit => T::OcOpBuylimit,
            OrderType::SellLimit => T::OcOpSelllimit,
            OrderType::BuyStop => T::OcOpBuystop,
            OrderType::SellStop => T::OcOpSellstop,
        }
    }
    /// Decode an `OpenedOrderType` / MQL4 `OrderType()` integer.
    pub fn from_i32(v: i32) -> Option<Self> {
        Some(match v {
            0 => OrderType::Buy,
            1 => OrderType::Sell,
            2 => OrderType::BuyLimit,
            3 => OrderType::SellLimit,
            4 => OrderType::BuyStop,
            5 => OrderType::SellStop,
            _ => return None,
        })
    }
    pub fn is_market(self) -> bool {
        matches!(self, OrderType::Buy | OrderType::Sell)
    }
    pub fn is_buy(self) -> bool {
        matches!(self, OrderType::Buy | OrderType::BuyLimit | OrderType::BuyStop)
    }
}

/// Alias kept for symmetry with the MT5 SDK (`TradeAction::Buy` == `OrderType::Buy`).
pub type TradeAction = OrderType;

/// High-level order request. For market orders leave `price` as `None` (the gateway fills at
/// the current quote). `magic` is the MQL4 magic number, which is how orders are filtered per
/// strategy (`MT4Client::orders_by_magic`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub order_type: OrderType,
    pub lots: f64,
    pub price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    /// Max slippage in points.
    pub slippage: Option<u32>,
    pub comment: Option<String>,
    #[serde(default)]
    pub magic: Option<i32>,
    /// Pending orders only: unix seconds (server time) when the order expires.
    #[serde(default)]
    pub expiration: Option<i64>,
}

impl OrderRequest {
    pub fn market(symbol: &str, order_type: OrderType, lots: f64) -> Self {
        OrderRequest {
            symbol: symbol.to_string(),
            order_type,
            lots,
            price: None,
            stop_loss: None,
            take_profit: None,
            slippage: Some(20),
            comment: None,
            magic: None,
            expiration: None,
        }
    }
    pub fn pending(symbol: &str, order_type: OrderType, lots: f64, price: f64) -> Self {
        let mut r = Self::market(symbol, order_type, lots);
        r.price = Some(price);
        r
    }
    pub fn with_magic(mut self, magic: i32) -> Self {
        self.magic = Some(magic);
        self
    }
    pub fn with_comment(mut self, c: &str) -> Self {
        self.comment = Some(c.to_string());
        self
    }
    pub fn with_sl_tp(mut self, sl: Option<f64>, tp: Option<f64>) -> Self {
        self.stop_loss = sl;
        self.take_profit = tp;
        self
    }
    pub fn with_slippage(mut self, points: u32) -> Self {
        self.slippage = Some(points);
        self
    }

    pub fn to_proto(&self) -> pb::OrderSendRequest {
        pb::OrderSendRequest {
            symbol: self.symbol.clone(),
            operation_type: self.order_type.to_operation() as i32,
            volume: self.lots,
            price: self.price,
            slippage: self.slippage.map(|s| s as i32),
            stoploss: self.stop_loss,
            takeprofit: self.take_profit,
            comment: self.comment.clone(),
            magic_number: self.magic,
            expiration: self.expiration.map(|s| prost_types::Timestamp { seconds: s, nanos: 0 }),
        }
    }
}

/// Result of a successful `OrderSend`. In MT4 the **order ticket is the position**: use it for
/// `order_modify` and `order_close`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResult {
    pub ticket: u64,
    /// Always 0 on success (failures are returned as `MT4Error::Api` / `MT4Error::TradeError`).
    pub error_code: u32,
    pub price: f64,
    pub lots: f64,
    /// Unix seconds (server time) of the fill.
    pub open_time: i64,
    pub message: String,
}

impl OrderResult {
    pub fn from_proto(d: pb::OrderSendData) -> Self {
        OrderResult {
            ticket: d.ticket as u64,
            error_code: 0,
            price: d.price,
            lots: d.volume,
            open_time: d.open_time.as_ref().map(|t| t.seconds).unwrap_or(0),
            message: "Order placed".to_string(),
        }
    }
}

/// Result of `OrderCloseDelete`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseResult {
    /// `0` = a market order was closed, `1` = a pending order was deleted.
    pub mode: i32,
    pub history_order_comment: Option<String>,
}

/// Minimal bar type (mirrors `pb::HistoryQuote`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bar {
    /// Unix seconds (bar open time, broker server time zone as the terminal reports it).
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub tick_volume: i64,
    pub real_volume: i64,
    pub spread: i32,
}

impl From<&pb::HistoryQuote> for Bar {
    fn from(b: &pb::HistoryQuote) -> Self {
        Bar {
            time: b.time.as_ref().map(|t| t.seconds).unwrap_or(0),
            open: b.open,
            high: b.high,
            low: b.low,
            close: b.close,
            tick_volume: b.tick_volume,
            real_volume: b.real_volume,
            spread: b.spread,
        }
    }
}
