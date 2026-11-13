/*══════════════════════════════════════════════════════════════════════════════
 FILE: examples/3_sugar/positions.rs - HIGH-LEVEL (MT4Sugar) POSITIONS & WATCHERS
 PURPOSE:
   Order/position info and live watchers via MT4Sugar: open orders, filters,
   floating P/L, exposure, and the streaming channels. No orders are placed.

 🎯 WHO SHOULD USE THIS:
   • Developers building position monitors / dashboards
   • Learning the read + filter + exposure helpers
   • Seeing the live channels (quotes / profit / tickets)

 📚 WHAT THIS DEMO COVERS (3 sections):

   1. INFO
      • orders() / positions() / orders_for_symbol() / orders_by_magic()
      • find_ticket()

   2. MONEY
      • total_profit() / positions_value() / exposure_summary()

   3. WATCHERS
      • subscribe_quotes() / watch_profit() / watch_tickets()

 🔄 API LEVELS:
   HIGH-LEVEL (MT4Sugar)   - THIS FILE: one-liner helpers with smart defaults
   MID-LEVEL  (MT4Service) - typed wrappers
   LOW-LEVEL  (MT4Account) - raw gRPC/protobuf

 ℹ️  SAFE: read-only — queries and opens watcher channels, never sends an order.

 USAGE:
   cargo run --example sugar_positions
══════════════════════════════════════════════════════════════════════════════*/

use std::time::Duration;

use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // region: CONNECT
    // ══════════════════════════════════════════════════════════════
    // CONNECT
    // ══════════════════════════════════════════════════════════════
    let cfg = Settings::load()?;
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)));
    sugar.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    let sym = cfg.test_symbol.as_str();
    // endregion: CONNECT

    // region: 1. INFO
    // ══════════════════════════════════════════════════════════════
    // 1. INFO
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // orders() / positions() / orders_for_symbol() / orders_by_magic() / find_ticket()
    //   List open orders and market positions, filter by symbol / magic,
    //   and look a ticket up.
    // ──────────────────────────────────────────────────────────────
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
    // endregion: 1. INFO

    // region: 2. MONEY
    // ══════════════════════════════════════════════════════════════
    // 2. MONEY
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // total_profit() / positions_value() / exposure_summary()
    //   Floating P/L, position value for a symbol, and net exposure
    //   per symbol.
    // ──────────────────────────────────────────────────────────────
    println!("total floating P/L: {:+.2}", sugar.total_profit(None).await?);
    println!("positions_value {}: {:+.2}", sym, sugar.positions_value(Some(sym), None).await?);
    for (s, net) in sugar.exposure_summary().await? {
        println!("  exposure {s}: {net:+.2} lot");
    }
    // endregion: 2. MONEY

    // region: 3. WATCHERS
    // ══════════════════════════════════════════════════════════════
    // 3. WATCHERS (open each channel, read one item or time out, drop)
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // subscribe_quotes()
    //   Live tick channel for the symbol; read one tick (5s cap).
    // ──────────────────────────────────────────────────────────────
    sugar.service().account().symbol_select(metarpc_mt4::mt4_term_api::SymbolSelectRequest { symbol: sym.to_string(), select: true }).await?;
    let mut ticks = sugar.subscribe_quotes(vec![sym.to_string()]).await?;
    if let Ok(Some(q)) = tokio::time::timeout(Duration::from_secs(5), ticks.recv()).await {
        println!("tick: {} bid {:.5} ask {:.5}", q.symbol, q.bid, q.ask);
    }
    drop(ticks);

    // ──────────────────────────────────────────────────────────────
    // watch_profit() / watch_tickets()
    //   Periodic P/L and ticket channels; opened then dropped.
    // ──────────────────────────────────────────────────────────────
    let mut profit = sugar.watch_profit(1000).await?;
    let _ = tokio::time::timeout(Duration::from_secs(2), profit.recv()).await;
    let mut tkts = sugar.watch_tickets(1000).await?;
    let _ = tokio::time::timeout(Duration::from_secs(2), tkts.recv()).await;
    println!("watch_profit / watch_tickets channels opened (then dropped)");
    // endregion: 3. WATCHERS

    sugar.disconnect(false).await?;
    Ok(())
}
