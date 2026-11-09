//! `MT4Sugar` trading — exercises order entry, management, trailing/breakeven and the waiters.
//! Guarded by `SUGAR_TRADE=1` (demo accounts only); it opens and closes small 0.01-lot orders.
//!
//!   SUGAR_TRADE=1 cargo run --example sugar_trading

use std::time::Duration;

use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar, OrderRequest, OrderType, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Settings::load()?;
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)))
        .set_defaults(1001, 20, "sugar_trading");
    sugar.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    let sym = cfg.test_symbol.as_str();
    let q = sugar.ensure_symbol(sym, Duration::from_secs(5)).await?;

    if std::env::var("SUGAR_TRADE").ok().as_deref() != Some("1") {
        println!("(set SUGAR_TRADE=1 to run the trading flow — doing nothing)");
        sugar.disconnect(false).await?;
        return Ok(());
    }

    // ── a managed market BUY: enter, set pip SL/TP, breakeven + trail, then close ──
    let lot = sugar.calc_lot_by_risk(sym, sugar.get_account_info().await?.balance * 0.01, 20.0).await?;
    let r = sugar.buy_market(sym, lot, None, None).await?;
    println!("opened #{} {:.2} lot @ {:.5}", r.ticket, r.lots, r.price);
    sugar.modify_sl_tp_by_pips(r.ticket, Some(20.0), Some(40.0)).await?;
    sugar.modify_sl_tp_by_price(r.ticket, Some(r.price - 0.0020), None).await?; // explicit-price variant
    let _ = sugar.auto_breakeven(r.ticket, 10.0, 2.0).await?;
    let _ = sugar.trail_stop(r.ticket, 15.0).await?;
    sugar.close_partial(r.ticket, lot / 2.0).await.ok();
    sugar.close(r.ticket, None).await?;
    let _ = sugar.wait_closed(r.ticket, Duration::from_secs(5)).await?;
    println!("managed buy closed");

    // ── place_order + the other market/pending constructors ──
    let s = sugar.place_order(OrderRequest::market(sym, OrderType::Sell, lot).with_comment("place_order")).await?;
    sugar.close(s.ticket, None).await?;
    let sm = sugar.sell_market(sym, lot, None, None).await?;
    sugar.close(sm.ticket, None).await?;

    // pendings far from market so they rest; then wait briefly and cancel them
    let p = sugar.pips_to_price(sym, 50.0).await?;
    let _ = sugar.buy_limit(sym, lot, q.bid - p, None, None).await?;
    let _ = sugar.sell_limit(sym, lot, q.ask + p, None, None).await?;
    let _ = sugar.buy_stop(sym, lot, q.ask + p, None, None).await?;
    let pend = sugar.sell_stop(sym, lot, q.bid - p, None, None).await?;
    let _ = sugar.wait_filled(pend.ticket, Duration::from_secs(2)).await?; // won't fill (far)
    println!("cancelled {} pendings", sugar.cancel_pendings(Some(sym)).await?);

    // ── OrderCloseBy on an opposite pair ──
    let b = sugar.buy_market(sym, lot, None, None).await?;
    let a = sugar.sell_market(sym, lot, None, None).await?;
    sugar.close_by(b.ticket, a.ticket).await?;
    println!("closed #{} by #{}", b.ticket, a.ticket);

    // ── wait_price (short timeout — just demonstrates the call) ──
    let _ = sugar.wait_price(sym, q.ask + 1.0, true, Duration::from_secs(1)).await;

    // ── flatten anything left ──
    println!("closed {} remaining positions", sugar.close_all(Some(sym)).await?);

    sugar.disconnect(false).await?;
    Ok(())
}
