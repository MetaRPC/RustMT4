//! `MT4Sugar` order/position info and watchers: open orders, filters, floating P/L, exposure, and
//! the live channels (`subscribe_quotes`, `watch_profit`, `watch_tickets`).
//!
//!   cargo run --example sugar_positions

use std::time::Duration;

use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Settings::load()?;
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)));
    sugar.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    let sym = cfg.test_symbol.as_str();

    // ── info ──
    let orders = sugar.orders().await?;
    let positions = sugar.positions().await?;
    println!("orders {}  (market positions {})", orders.len(), positions.len());
    for p in &positions {
        let side = if p.order_type == 0 { "BUY" } else if p.order_type == 1 { "SELL" } else { "PEND" };
        println!("  #{} {} {} {:.2} lot  P/L {:+.2}", p.ticket, side, p.symbol, p.lots, p.profit);
    }
    println!("on {}: {}  by magic 1001: {}", sym, sugar.orders_for_symbol(sym).await?.len(), sugar.orders_by_magic(1001).await?.len());
    if let Some(first) = orders.first() {
        let found = sugar.find_ticket(first.ticket as u64).await?.is_some();
        println!("find_ticket #{}: {}", first.ticket, found);
    }

    // ── money ──
    println!("total floating P/L: {:+.2}", sugar.total_profit(None).await?);
    println!("positions_value {}: {:+.2}", sym, sugar.positions_value(Some(sym), None).await?);
    for (s, net) in sugar.exposure_summary().await? {
        println!("  exposure {s}: {net:+.2} lot");
    }

    // ── watchers: open each channel, read one item (or time out), then drop ──
    sugar.service().account().symbol_select(metarpc_mt4::mt4_term_api::SymbolSelectRequest { symbol: sym.to_string(), select: true }).await?;
    let mut ticks = sugar.subscribe_quotes(vec![sym.to_string()]).await?;
    if let Ok(Some(q)) = tokio::time::timeout(Duration::from_secs(5), ticks.recv()).await {
        println!("tick: {} bid {:.5} ask {:.5}", q.symbol, q.bid, q.ask);
    }
    drop(ticks);

    let mut profit = sugar.watch_profit(1000).await?;
    let _ = tokio::time::timeout(Duration::from_secs(2), profit.recv()).await;
    let mut tkts = sugar.watch_tickets(1000).await?;
    let _ = tokio::time::timeout(Duration::from_secs(2), tkts.recv()).await;
    println!("watch_profit / watch_tickets channels opened (then dropped)");

    sugar.disconnect(false).await?;
    Ok(())
}
