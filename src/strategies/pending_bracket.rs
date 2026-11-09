//! Orchestrator · Pending Bracket: place a BUY LIMIT, wait for the fill (or cancel), then manage.

use std::time::Duration;

use super::{manage, size_lot, RiskPreset, StrategyPreset};
use crate::{MT4Error, MT4Sugar};

/// Place a BUY LIMIT with a bracket, wait for the fill, then manage; if it isn't filled within
/// `timeout_seconds` the pending order is cancelled. Returns the ticket.
pub async fn run_pending_bracket(sugar: &MT4Sugar, sp: &StrategyPreset, rp: &RiskPreset, timeout_seconds: u64) -> Result<u64, MT4Error> {
    println!("── Pending Bracket · {} ──", sp.symbol);
    let pip = sugar.pip_size(&sp.symbol).await?;

    // entry price: explicit, or 5 pips below market for the demo
    let entry = match sp.entry_price {
        Some(p) => p,
        None => sugar.last_quote(&sp.symbol).await?.bid - 5.0 * pip,
    };
    let lots = size_lot(sugar, sp, rp).await?;
    let p = sugar.buy_limit(&sp.symbol, lots, entry, None, None).await?;
    let ticket = p.ticket;
    println!("placed : BUY LIMIT #{ticket} {lots:.2} lot @ {entry:.5}");

    // wait for the fill (the ticket's type flips from BuyLimit to Buy), or cancel on timeout
    if !sugar.wait_filled(ticket, Duration::from_secs(timeout_seconds)).await? {
        println!("timeout: not filled in {timeout_seconds}s — cancelling");
        let _ = sugar.close(ticket, None).await; // delete the resting pending order
        return Ok(ticket);
    }
    println!("filled : #{ticket}");

    sugar.modify_sl_tp_by_pips(ticket, Some(rp.sl_pips), rp.tp_pips).await?;
    let trail_from = rp.be_trigger_pips.unwrap_or(0.0);
    manage(sugar, ticket, &sp.symbol, rp, 5, trail_from).await?;

    sugar.close(ticket, None).await?;
    println!("close  : flat");
    Ok(ticket)
}
