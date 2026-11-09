//! The three orchestrators run with a preset. Guarded by `STRATEGY_RUN=1` (demo accounts only) —
//! each opens and closes a small managed order.
//!
//!   STRATEGY_RUN=1 cargo run --example strategies_demo

use metarpc_mt4::strategies::{balanced, run_bracket_trailing, run_market_one_shot, run_pending_bracket, StrategyPreset};
use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Settings::load()?;
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)));
    sugar.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;

    if std::env::var("STRATEGY_RUN").ok().as_deref() != Some("1") {
        println!("(set STRATEGY_RUN=1 to run the orchestrators — doing nothing)");
        sugar.disconnect(false).await?;
        return Ok(());
    }

    let sp = StrategyPreset::new(&cfg.test_symbol).magic(777).comment("demo");
    let rp = balanced(); // 1% risk, SL 20, TP 40, trail 15, breakeven 10/+2

    let t1 = run_market_one_shot(&sugar, &sp, &rp).await?;
    println!("market one-shot done, ticket #{t1}\n");

    let t2 = run_bracket_trailing(&sugar, &sp, &rp, 25.0).await?;
    println!("bracket + trailing done, ticket #{t2}\n");

    let t3 = run_pending_bracket(&sugar, &sp, &rp, 30).await?;
    println!("pending bracket done, ticket #{t3}");

    sugar.disconnect(false).await?;
    Ok(())
}
