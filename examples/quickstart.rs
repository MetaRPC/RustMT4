/*══════════════════════════════════════════════════════════════════════════════
 FILE: examples/quickstart.rs - MT4 QUICKSTART
 PURPOSE:
   The shortest useful round-trip: connect, read the account, get a usable quote,
   and (optionally) place + close a small demo order. The place to start.

 🎯 WHO SHOULD USE THIS:
   • First-time users getting a connection working end to end
   • A copy-paste starting point for your own program
   • Verifying credentials / server reachability

 📚 WHAT THIS DEMO COVERS:

   1. SETUP & CONNECT
      • Settings::load() + MT4Sugar client + connect_by_server_name()

   2. READ (account & quote)
      • get_account_info()           - login / company / balance / equity
      • ensure_symbol() / spread_pips() - select a symbol, wait for a tick

   3. OPTIONAL TRADE (QUICKSTART_TRADE=1)
      • buy_market() / close()       - open a 0.01-lot order, then close it

 🔄 API LEVELS:
   HIGH-LEVEL (MT4Sugar) - THIS FILE: one-liner helpers with smart defaults

 ℹ️  SAFE by default (read-only). Set QUICKSTART_TRADE=1 to send a real
   0.01-lot order (opened then closed). Demo accounts only.

 USAGE:
   cargo run --example quickstart
   (config from examples/0_common/settings.json and/or MT4_* env vars)
══════════════════════════════════════════════════════════════════════════════*/

use std::time::Duration;

use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // region: 1. SETUP & CONNECT
    // ══════════════════════════════════════════════════════════════
    // 1. SETUP & CONNECT
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // Settings::load() + MT4Sugar client
    //   Settings (file + env, env wins) and the high-level client with
    //   shared defaults (magic 777, 20-pip deviation, "quickstart" comment).
    // ──────────────────────────────────────────────────────────────
    let cfg = Settings::load()?;
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)))
        .set_defaults(777, 20, "quickstart");

    // ──────────────────────────────────────────────────────────────
    // connect_by_server_name()
    //   Connect by the broker server name.
    // ──────────────────────────────────────────────────────────────
    sugar.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    // endregion: 1. SETUP & CONNECT

    // region: 2. READ (account & quote)
    // ══════════════════════════════════════════════════════════════
    // 2. READ — account snapshot & a usable quote
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // get_account_info()
    //   Account snapshot: login, company, balance, currency, equity.
    // ──────────────────────────────────────────────────────────────
    let acc = sugar.get_account_info().await?;
    println!("#{}  {}  balance {:.2} {}  equity {:.2}", acc.login, acc.company, acc.balance, acc.currency, acc.equity);

    // ──────────────────────────────────────────────────────────────
    // ensure_symbol() / spread_pips()
    //   Select the symbol and wait for a two-sided tick, then read the spread.
    // ──────────────────────────────────────────────────────────────
    let sym = cfg.test_symbol.clone();
    let q = sugar.ensure_symbol(&sym, Duration::from_secs(5)).await?;
    println!("{sym}  bid {:.5}  ask {:.5}  spread {:.1} pips", q.bid, q.ask, sugar.spread_pips(&sym).await?);
    // endregion: 2. READ (account & quote)

    // region: 3. OPTIONAL TRADE
    // ══════════════════════════════════════════════════════════════
    // 3. OPTIONAL TRADE (QUICKSTART_TRADE=1 ⚠️ real order)
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // buy_market() / close()
    //   Only when QUICKSTART_TRADE=1: open a 0.01-lot BUY, hold 2s, close.
    // ──────────────────────────────────────────────────────────────
    if std::env::var("QUICKSTART_TRADE").ok().as_deref() == Some("1") {
        let r = sugar.buy_market(&sym, 0.01, None, None).await?;
        println!("opened ticket #{} @ {:.5}", r.ticket, r.price);
        tokio::time::sleep(Duration::from_secs(2)).await;
        sugar.close(r.ticket, None).await?;
        println!("closed #{}", r.ticket);
    } else {
        println!("(set QUICKSTART_TRADE=1 to send a 0.01-lot demo order)");
    }
    // endregion: 3. OPTIONAL TRADE

    sugar.disconnect(false).await?;
    Ok(())
}
