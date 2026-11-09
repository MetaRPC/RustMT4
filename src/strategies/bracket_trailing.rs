//! Orchestrator · Bracket + Trailing: enter with a bracket, start trailing only after a cushion.

use super::{manage, size_lot, RiskPreset, StrategyPreset};
use crate::{MT4Error, MT4Sugar};

/// Enter with a bracket, then **only start trailing once the trade is `activate_at_pips` in
/// profit**. Auto-breakeven (from `be_trigger_pips`) still applies earlier. Returns the ticket.
pub async fn run_bracket_trailing(sugar: &MT4Sugar, sp: &StrategyPreset, rp: &RiskPreset, activate_at_pips: f64) -> Result<u64, MT4Error> {
    println!("── Bracket + Trailing · {} ──", sp.symbol);

    let lots = size_lot(sugar, sp, rp).await?;
    let r = sugar.buy_market(&sp.symbol, lots, None, None).await?;
    let ticket = r.ticket;
    sugar.modify_sl_tp_by_pips(ticket, Some(rp.sl_pips), rp.tp_pips).await?;
    println!("entry : BUY #{ticket} @ {:.5}; trailing starts at +{activate_at_pips:.0}p", r.price);

    manage(sugar, ticket, &sp.symbol, rp, 10, activate_at_pips).await?;

    println!("close : #{ticket}");
    sugar.close(ticket, None).await?;
    Ok(ticket)
}
