//! Orchestrator · Market One-Shot: size → market entry with a pip bracket → manage → close.

use super::{manage, size_lot, RiskPreset, StrategyPreset};
use crate::{MT4Error, MT4Sugar};

/// The simplest orchestrator. Management honours the `RiskPreset`: auto-breakeven once
/// `be_trigger_pips` is reached, and trailing starting at that same trigger. Returns the ticket.
pub async fn run_market_one_shot(sugar: &MT4Sugar, sp: &StrategyPreset, rp: &RiskPreset) -> Result<u64, MT4Error> {
    println!("── Market One-Shot · {} ──", sp.symbol);

    let lots = size_lot(sugar, sp, rp).await?;
    let r = sugar.buy_market(&sp.symbol, lots, None, None).await?;
    let ticket = r.ticket;
    println!("entry : BUY #{ticket} {lots:.2} lot @ {:.5}", r.price);

    // pip-based SL/TP bracket
    sugar.modify_sl_tp_by_pips(ticket, Some(rp.sl_pips), rp.tp_pips).await?;

    // manage a few seconds: breakeven, then trail (both from the preset)
    let trail_from = rp.be_trigger_pips.unwrap_or(0.0);
    manage(sugar, ticket, &sp.symbol, rp, 5, trail_from).await?;

    println!("close : #{ticket}");
    sugar.close(ticket, None).await?;
    Ok(ticket)
}
