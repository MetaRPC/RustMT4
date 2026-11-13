/*══════════════════════════════════════════════════════════════════════════════
 FILE: examples/4_strategies/strategies_demo.rs - MT4 STRATEGY ORCHESTRATORS DEMO
 PURPOSE:
   Run the three strategy orchestrators with one shared preset: a market one-shot,
   a bracket + trailing entry, and a pending bracket. Each opens and closes a small
   managed order. Guarded by STRATEGY_RUN=1 so it does nothing unless you opt in.

 🎯 WHO SHOULD USE THIS:
   • Developers learning the ready-made strategy orchestrators
   • Seeing how a StrategyPreset + risk preset drive a managed trade
   • A starting point for your own automated entries

 📚 WHAT THIS DEMO COVERS:

   PRESETS
      • StrategyPreset::new() / balanced()  - shared strategy & risk settings

   ORCHESTRATORS
      • run_market_one_shot()   - instant managed entry, then exit
      • run_bracket_trailing()  - entry + breakeven / trailing management
      • run_pending_bracket()   - pending entry + bracket

 🔄 API LEVELS:
   Orchestrators sit ABOVE MT4Sugar - they combine Sugar one-liners into a
   complete entry -> manage -> exit lifecycle.

 ⚠️  WARNING: with STRATEGY_RUN=1 this OPENS AND CLOSES REAL managed orders
   (minimal lot) on your account. Demo accounts only.

 USAGE:
   STRATEGY_RUN=1 cargo run --example strategies_demo
══════════════════════════════════════════════════════════════════════════════*/

use metarpc_mt4::strategies::{balanced, run_bracket_trailing, run_market_one_shot, run_pending_bracket, StrategyPreset};
use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // region: CONNECT
    // ══════════════════════════════════════════════════════════════
    // CONNECT
    //   Load credentials and build the high-level MT4Sugar client
    //   (Sugar -> Service -> Account), then connect by server name.
    // ══════════════════════════════════════════════════════════════
    let cfg = Settings::load()?;
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)));
    sugar.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    // endregion: CONNECT

    // region: RUN GUARD
    // ══════════════════════════════════════════════════════════════
    // RUN GUARD
    //   Safety switch: without STRATEGY_RUN=1 the demo connects and
    //   exits without placing any order.
    // ══════════════════════════════════════════════════════════════
    if std::env::var("STRATEGY_RUN").ok().as_deref() != Some("1") {
        println!("(set STRATEGY_RUN=1 to run the orchestrators — doing nothing)");
        sugar.disconnect(false).await?;
        return Ok(());
    }
    // endregion: RUN GUARD

    // region: PRESETS
    // ══════════════════════════════════════════════════════════════
    // PRESETS
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // StrategyPreset::new() / balanced()
    //   The shared strategy preset (symbol, magic, comment) and a
    //   "balanced" risk preset: 1% risk, SL 20, TP 40, trail 15, BE 10/+2.
    // ──────────────────────────────────────────────────────────────
    let sp = StrategyPreset::new(&cfg.test_symbol).magic(777).comment("demo");
    let rp = balanced(); // 1% risk, SL 20, TP 40, trail 15, breakeven 10/+2
    // endregion: PRESETS

    // region: ORCHESTRATORS
    // ══════════════════════════════════════════════════════════════
    // ORCHESTRATORS (⚠️ real managed orders)
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // run_market_one_shot()
    //   Instant market entry with managed SL/TP, then exit.
    // ──────────────────────────────────────────────────────────────
    let t1 = run_market_one_shot(&sugar, &sp, &rp).await?;
    println!("market one-shot done, ticket #{t1}\n");

    // ──────────────────────────────────────────────────────────────
    // run_bracket_trailing()
    //   Entry with a bracket, then breakeven + trailing management.
    // ──────────────────────────────────────────────────────────────
    let t2 = run_bracket_trailing(&sugar, &sp, &rp, 25.0).await?;
    println!("bracket + trailing done, ticket #{t2}\n");

    // ──────────────────────────────────────────────────────────────
    // run_pending_bracket()
    //   Place a pending entry with a bracket around the current price.
    // ──────────────────────────────────────────────────────────────
    let t3 = run_pending_bracket(&sugar, &sp, &rp, 30).await?;
    println!("pending bracket done, ticket #{t3}");
    // endregion: ORCHESTRATORS

    sugar.disconnect(false).await?;
    Ok(())
}
