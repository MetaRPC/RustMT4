/*══════════════════════════════════════════════════════════════════════════════
 FILE: examples/1_lowlevel/trading.rs - LOW-LEVEL MT4 TRADING DEMO
 PURPOSE:
   Build an OrderSendRequest by hand on the raw MT4Account, then modify and close
   it by ticket, and finally net an opposite pair with OrderCloseBy.

 🎯 WHO SHOULD USE THIS:
   • Developers who need full control over order construction
   • Building a custom trading layer on top of MT4Account
   • Learning the raw order_send / modify / close flow

 📚 WHAT THIS DEMO COVERS (2 sections):

   1. SEND · MODIFY · CLOSE
      • order_send()         - open a 0.01-lot market BUY
      • order_modify()       - attach a take-profit above the fill
      • order_close_delete() - close the whole order

   2. CLOSE BY OPPOSITE
      • order_send() x2      - open an opposite BUY + SELL pair
      • order_close_by()     - net the two tickets out

 🔄 API LEVELS:
   LOW-LEVEL  (MT4Account) - THIS FILE: raw order_send / modify / close
   MID-LEVEL  (MT4Service) - typed helpers, cleaner error handling
   HIGH-LEVEL (MT4Sugar)   - one-liner buy/sell/close with smart defaults

 ⚠️  WARNING: places/modifies/closes REAL orders (0.01 lot, demo account only).
     Guarded by LOWLEVEL_TRADE=1 — does nothing unless that env var is set.

 USAGE:
   LOWLEVEL_TRADE=1 cargo run --example lowlevel_trading
   (reads creds from examples/0_common/settings.json or the MT4_* env vars)
══════════════════════════════════════════════════════════════════════════════*/

use metarpc_mt4::mt4_term_api as pb;
use metarpc_mt4::{MT4Account, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // region: CONNECT & GUARD
    // ══════════════════════════════════════════════════════════════
    // CONNECT & GUARD
    //   Connect, select the symbol, then bail out unless LOWLEVEL_TRADE=1
    //   is set (so the demo is a no-op by default).
    // ══════════════════════════════════════════════════════════════
    let cfg = Settings::load()?;
    let acc = MT4Account::new("mt4.mrpc.pro", 443);
    acc.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    acc.symbol_select(pb::SymbolSelectRequest { symbol: cfg.test_symbol.clone(), select: true }).await?;

    if std::env::var("LOWLEVEL_TRADE").ok().as_deref() != Some("1") {
        println!("(set LOWLEVEL_TRADE=1 to actually trade — doing nothing)");
        acc.disconnect(false).await?;
        return Ok(());
    }
    // endregion: CONNECT & GUARD

    // region: 1. SEND · MODIFY · CLOSE
    // ══════════════════════════════════════════════════════════════
    // 1. SEND · MODIFY · CLOSE (⚠️ real orders)
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // order_send()
    //   Open a 0.01-lot market BUY; returns the ticket and fill price.
    // ──────────────────────────────────────────────────────────────
    let send = acc
        .order_send(pb::OrderSendRequest {
            symbol: cfg.test_symbol.clone(),
            operation_type: pb::OrderSendOperationType::OcOpBuy as i32,
            volume: 0.01,
            slippage: Some(20),
            comment: Some("lowlevel".into()),
            ..Default::default()
        })
        .await?;
    let ticket = send.ticket as u64;
    println!("opened #{ticket} @ {:.5}", send.price);

    // ──────────────────────────────────────────────────────────────
    // order_modify()
    //   Attach a take-profit 50 points above the fill price.
    // ──────────────────────────────────────────────────────────────
    let _ = acc
        .order_modify(pb::OrderModifyRequest {
            order_ticket: ticket as i32,
            new_take_profit: Some(send.price + 0.0050),
            ..Default::default()
        })
        .await;

    // ──────────────────────────────────────────────────────────────
    // order_close_delete()
    //   Close the whole order by ticket.
    // ──────────────────────────────────────────────────────────────
    let close = acc
        .order_close_delete(pb::OrderCloseDeleteRequest { order_ticket: ticket as i32, lots: None, closing_price: None, slippage: Some(20) })
        .await?;
    println!("closed #{ticket} (mode {})", close.mode);
    // endregion: 1. SEND · MODIFY · CLOSE

    // region: 2. CLOSE BY OPPOSITE
    // ══════════════════════════════════════════════════════════════
    // 2. CLOSE BY OPPOSITE (⚠️ real orders)
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // order_send() x2 + order_close_by()
    //   Open an opposite BUY + SELL pair, then net them out against
    //   each other by ticket (MT4 "close by").
    // ──────────────────────────────────────────────────────────────
    let mkt = |op: pb::OrderSendOperationType| pb::OrderSendRequest {
        symbol: cfg.test_symbol.clone(),
        operation_type: op as i32,
        volume: 0.01,
        slippage: Some(20),
        ..Default::default()
    };
    let buy = acc.order_send(mkt(pb::OrderSendOperationType::OcOpBuy)).await?;
    let sell = acc.order_send(mkt(pb::OrderSendOperationType::OcOpSell)).await?;
    let _by = acc
        .order_close_by(pb::OrderCloseByRequest { ticket_to_close: buy.ticket, opposite_ticket_closing_by: sell.ticket })
        .await?;
    println!("closed #{} by #{} (OrderCloseBy)", buy.ticket, sell.ticket);
    // endregion: 2. CLOSE BY OPPOSITE

    acc.disconnect(false).await?;
    Ok(())
}
