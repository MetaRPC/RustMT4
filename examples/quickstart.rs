//! Quickstart — connect, read the account, get a quote, and (optionally) place + close a demo order.
//!
//! Config comes from `examples/0_common/settings.json` and/or env vars
//! (`MT4_LOGIN` / `MT4_PASSWORD` / `MT4_SERVER` / `MRPC_API_KEY`); env overrides the file.
//! Set `QUICKSTART_TRADE=1` to actually send a 0.01-lot order (demo accounts only).
//!
//!   cargo run --example quickstart

use std::time::Duration;

use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) settings (file + env, env wins) and the high-level client
    let cfg = Settings::load()?;
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)))
        .set_defaults(777, 20, "quickstart");

    // 2) connect by broker server name
    sugar.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;

    // 3) account snapshot
    let acc = sugar.get_account_info().await?;
    println!("#{}  {}  balance {:.2} {}  equity {:.2}", acc.login, acc.company, acc.balance, acc.currency, acc.equity);

    // 4) a usable quote (select + wait for a two-sided tick)
    let sym = cfg.test_symbol.clone();
    let q = sugar.ensure_symbol(&sym, Duration::from_secs(5)).await?;
    println!("{sym}  bid {:.5}  ask {:.5}  spread {:.1} pips", q.bid, q.ask, sugar.spread_pips(&sym).await?);

    // 5) optional demo trade
    if std::env::var("QUICKSTART_TRADE").ok().as_deref() == Some("1") {
        let r = sugar.buy_market(&sym, 0.01, None, None).await?;
        println!("opened ticket #{} @ {:.5}", r.ticket, r.price);
        tokio::time::sleep(Duration::from_secs(2)).await;
        sugar.close(r.ticket, None).await?;
        println!("closed #{}", r.ticket);
    } else {
        println!("(set QUICKSTART_TRADE=1 to send a 0.01-lot demo order)");
    }

    sugar.disconnect(false).await?;
    Ok(())
}
