//! Ready-made **orchestrators** and **presets** (demonstration templates — DEMO accounts only).
//!
//! Each orchestrator is its own `async fn run_*`, driven by [`MT4Sugar`]. A [`StrategyPreset`] says
//! *what* to trade; a [`RiskPreset`] says *how much* and how to protect.

use std::time::{Duration, Instant};

use crate::error::MT4Error;
use crate::MT4Sugar;

mod bracket_trailing;
mod market_one_shot;
mod pending_bracket;

pub use bracket_trailing::run_bracket_trailing;
pub use market_one_shot::run_market_one_shot;
pub use pending_bracket::run_pending_bracket;

/// WHAT to trade.
#[derive(Debug, Clone)]
pub struct StrategyPreset {
    pub symbol: String,
    pub use_market: bool,
    pub entry_price: Option<f64>,
    pub lots: Option<f64>,
    pub magic: i32,
    pub comment: String,
}

impl StrategyPreset {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self { symbol: symbol.into(), use_market: true, entry_price: None, lots: None, magic: 0, comment: String::new() }
    }
    pub fn market(mut self) -> Self {
        self.use_market = true;
        self.entry_price = None;
        self
    }
    pub fn limit_at(mut self, price: f64) -> Self {
        self.use_market = false;
        self.entry_price = Some(price);
        self
    }
    pub fn lots(mut self, lots: f64) -> Self {
        self.lots = Some(lots);
        self
    }
    pub fn magic(mut self, magic: i32) -> Self {
        self.magic = magic;
        self
    }
    pub fn comment(mut self, c: impl Into<String>) -> Self {
        self.comment = c.into();
        self
    }
}

/// HOW MUCH to risk and how to protect.
#[derive(Debug, Clone)]
pub struct RiskPreset {
    pub risk_percent: f64,
    pub sl_pips: f64,
    pub tp_pips: Option<f64>,
    pub trailing_pips: Option<f64>,
    pub be_trigger_pips: Option<f64>,
    pub be_plus_pips: f64,
}

impl RiskPreset {
    pub fn new(risk_percent: f64, sl_pips: f64) -> Self {
        Self { risk_percent, sl_pips, tp_pips: None, trailing_pips: None, be_trigger_pips: None, be_plus_pips: 0.0 }
    }
    pub fn tp(mut self, pips: f64) -> Self {
        self.tp_pips = Some(pips);
        self
    }
    pub fn trailing(mut self, pips: f64) -> Self {
        self.trailing_pips = Some(pips);
        self
    }
    pub fn breakeven(mut self, trigger_pips: f64, plus_pips: f64) -> Self {
        self.be_trigger_pips = Some(trigger_pips);
        self.be_plus_pips = plus_pips;
        self
    }
}

// ----- shared helpers used by the orchestrators -----

/// Lot size — fixed from the strategy, or by risk% over the stop.
pub(crate) async fn size_lot(sugar: &MT4Sugar, sp: &StrategyPreset, rp: &RiskPreset) -> Result<f64, MT4Error> {
    match sp.lots {
        Some(l) => Ok(l),
        None => {
            let bal = sugar.get_account_info().await?.balance;
            sugar.calc_lot_by_risk(&sp.symbol, bal * rp.risk_percent / 100.0, rp.sl_pips).await
        }
    }
}

/// Manage an open **BUY** order for `manage_secs`: auto-breakeven (from `be_trigger_pips`) and a
/// trailing stop (from `trailing_pips`, once at least `trail_activate_pips` in profit). Returns
/// early if the order is no longer open (hit SL/TP). Both protections are optional.
pub(crate) async fn manage(
    sugar: &MT4Sugar,
    ticket: u64,
    symbol: &str,
    rp: &RiskPreset,
    manage_secs: u64,
    trail_activate_pips: f64,
) -> Result<(), MT4Error> {
    let pip = if rp.trailing_pips.is_some() { sugar.pip_size(symbol).await? } else { 0.0 };
    let deadline = Instant::now() + Duration::from_secs(manage_secs);
    while Instant::now() < deadline {
        if sugar.find_ticket(ticket).await?.is_none() {
            break; // closed by SL/TP
        }
        if let Some(t) = rp.be_trigger_pips {
            let _ = sugar.auto_breakeven(ticket, t, rp.be_plus_pips).await;
        }
        if let Some(tr) = rp.trailing_pips {
            if let Some(o) = sugar.find_ticket(ticket).await? {
                let q = sugar.last_quote(symbol).await?;
                let profit_pips = if o.order_type == 0 { (q.bid - o.open_price) / pip } else { (o.open_price - q.ask) / pip };
                if profit_pips >= trail_activate_pips {
                    let _ = sugar.trail_stop(ticket, tr).await;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

// ===== Presets catalog =====

/// Market EURUSD (magic 777).
pub fn market_eurusd() -> StrategyPreset {
    StrategyPreset::new("EURUSD").magic(777).comment("market EURUSD")
}
/// BUY LIMIT EURUSD at `price` (magic 778).
pub fn limit_eurusd(price: f64) -> StrategyPreset {
    StrategyPreset::new("EURUSD").limit_at(price).magic(778).comment("limit EURUSD")
}
/// 0.5% risk, SL 25, TP 50.
pub fn conservative() -> RiskPreset {
    RiskPreset::new(0.5, 25.0).tp(50.0)
}
/// 1.0% risk, SL 20, TP 40, trail 15, breakeven 10/+2.
pub fn balanced() -> RiskPreset {
    RiskPreset::new(1.0, 20.0).tp(40.0).trailing(15.0).breakeven(10.0, 2.0)
}
/// 2.0% risk, SL 15, TP 45, trail 10, breakeven 8/+1.
pub fn aggressive() -> RiskPreset {
    RiskPreset::new(2.0, 15.0).tp(45.0).trailing(10.0).breakeven(8.0, 1.0)
}
