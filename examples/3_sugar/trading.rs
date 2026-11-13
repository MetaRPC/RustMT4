/*══════════════════════════════════════════════════════════════════════════════
 FILE: examples/3_sugar/trading.rs - HIGH-LEVEL (MT4Sugar) TRADING DEMO
 PURPOSE:
   Exercise the full MT4Sugar trading surface: order entry, management,
   trailing / breakeven, pending orders, close-by and the waiters.

 🎯 WHO SHOULD USE THIS:
   • Developers writing order-execution logic with MT4Sugar
   • Learning the one-liner entry / modify / close helpers
   • Seeing the pending-order and close-by flows end to end

 📚 WHAT THIS DEMO COVERS (4 sections):

   1. GUARD
      • SUGAR_TRADE=1 gate (does nothing unless explicitly enabled)

   2. MANAGED MARKET BUY
      • buy_market() / modify_sl_tp_by_pips() / modify_sl_tp_by_price()
      • auto_breakeven() / trail_stop() / close_partial() / close() / wait_closed()

   3. CONSTRUCTORS & PENDINGS
      • place_order() / sell_market()
      • buy_limit() / sell_limit() / buy_stop() / sell_stop()
      • wait_filled() / cancel_pendings()

   4. CLOSE-BY & FLATTEN
      • close_by() (opposite pair) / wait_price() / close_all()

 🔄 API LEVELS:
   HIGH-LEVEL (MT4Sugar)   - THIS FILE: one-liner trading with smart defaults
   MID-LEVEL  (MT4Service) - typed wrappers
   LOW-LEVEL  (MT4Account) - raw gRPC/protobuf

 ⚠️  WARNING: opens and closes real 0.01-lot orders on a DEMO account
     (gated behind SUGAR_TRADE=1; self-cleaning — flattens everything it opens).

 USAGE:
   SUGAR_TRADE=1 cargo run --example sugar_trading
══════════════════════════════════════════════════════════════════════════════*/

use std::time::Duration;

use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar, OrderRequest, OrderType, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // region: CONNECT & GUARD
    // ══════════════════════════════════════════════════════════════
    // 1. CONNECT & GUARD (SUGAR_TRADE=1)
    // ══════════════════════════════════════════════════════════════
    let cfg = Settings::load()?;
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)))
        .set_defaults(1001, 20, "sugar_trading");
    sugar.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    let sym = cfg.test_symbol.as_str();
    let q = sugar.ensure_symbol(sym, Duration::from_secs(5)).await?;

    // Safety gate: do nothing (and disconnect) unless SUGAR_TRADE=1 is set.
    if std::env::var("SUGAR_TRADE").ok().as_deref() != Some("1") {
        println!("(set SUGAR_TRADE=1 to run the trading flow — doing nothing)");
        sugar.disconnect(false).await?;
        return Ok(());
    }
    // endregion: CONNECT & GUARD

    // region: 2. MANAGED MARKET BUY
    // ══════════════════════════════════════════════════════════════
    // 2. MANAGED MARKET BUY (enter, SL/TP, breakeven + trail, close)
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // buy_market() + modify_sl_tp_by_pips/price() + auto_breakeven() + trail_stop()
    //   Open a risk-sized BUY, attach SL/TP (by pips, then by price),
    //   arm auto break-even and a trailing stop.
    // ──────────────────────────────────────────────────────────────
    let lot = sugar.calc_lot_by_risk(sym, sugar.get_account_info().await?.balance * 0.01, 20.0).await?;
    let r = sugar.buy_market(sym, lot, None, None).await?;
    println!("opened #{} {:.2} lot @ {:.5}", r.ticket, r.lots, r.price);
    sugar.modify_sl_tp_by_pips(r.ticket, Some(20.0), Some(40.0)).await?;
    sugar.modify_sl_tp_by_price(r.ticket, Some(r.price - 0.0020), None).await?; // explicit-price variant
    let _ = sugar.auto_breakeven(r.ticket, 10.0, 2.0).await?;
    let _ = sugar.trail_stop(r.ticket, 15.0).await?;

    // ──────────────────────────────────────────────────────────────
    // close_partial() / close() / wait_closed()
    //   Close half, then the rest, and wait for confirmation.
    // ──────────────────────────────────────────────────────────────
    sugar.close_partial(r.ticket, lot / 2.0).await.ok();
    sugar.close(r.ticket, None).await?;
    let _ = sugar.wait_closed(r.ticket, Duration::from_secs(5)).await?;
    println!("managed buy closed");
    // endregion: 2. MANAGED MARKET BUY

    // region: 3. CONSTRUCTORS & PENDINGS
    // ══════════════════════════════════════════════════════════════
    // 3. CONSTRUCTORS & PENDING ORDERS
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // place_order() / sell_market()
    //   The generic builder and the SELL shortcut (opened then closed).
    // ──────────────────────────────────────────────────────────────
    let s = sugar.place_order(OrderRequest::market(sym, OrderType::Sell, lot).with_comment("place_order")).await?;
    sugar.close(s.ticket, None).await?;
    let sm = sugar.sell_market(sym, lot, None, None).await?;
    sugar.close(sm.ticket, None).await?;

    // ──────────────────────────────────────────────────────────────
    // buy_limit/sell_limit/buy_stop/sell_stop() + wait_filled() + cancel_pendings()
    //   Rest four pendings far from market, wait briefly (won't fill),
    //   then cancel them all for the symbol.
    // ──────────────────────────────────────────────────────────────
    let p = sugar.pips_to_price(sym, 50.0).await?;
    let _ = sugar.buy_limit(sym, lot, q.bid - p, None, None).await?;
    let _ = sugar.sell_limit(sym, lot, q.ask + p, None, None).await?;
    let _ = sugar.buy_stop(sym, lot, q.ask + p, None, None).await?;
    let pend = sugar.sell_stop(sym, lot, q.bid - p, None, None).await?;
    let _ = sugar.wait_filled(pend.ticket, Duration::from_secs(2)).await?; // won't fill (far)
    println!("cancelled {} pendings", sugar.cancel_pendings(Some(sym)).await?);
    // endregion: 3. CONSTRUCTORS & PENDINGS

    // region: 4. CLOSE-BY & FLATTEN
    // ══════════════════════════════════════════════════════════════
    // 4. CLOSE-BY & FLATTEN
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // close_by()
    //   Open an opposite BUY+SELL pair and close one by the other.
    // ──────────────────────────────────────────────────────────────
    let b = sugar.buy_market(sym, lot, None, None).await?;
    let a = sugar.sell_market(sym, lot, None, None).await?;
    sugar.close_by(b.ticket, a.ticket).await?;
    println!("closed #{} by #{}", b.ticket, a.ticket);

    // ──────────────────────────────────────────────────────────────
    // wait_price() / close_all()
    //   Demonstrate the price waiter (short timeout), then flatten any
    //   remaining positions on the symbol.
    // ──────────────────────────────────────────────────────────────
    let _ = sugar.wait_price(sym, q.ask + 1.0, true, Duration::from_secs(1)).await;
    println!("closed {} remaining positions", sugar.close_all(Some(sym)).await?);
    // endregion: 4. CLOSE-BY & FLATTEN

    sugar.disconnect(false).await?;
    Ok(())
}
