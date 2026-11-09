//! `MT4Sugar` read-only one-liners: account, prices, bars, symbol math and risk sizing. Every
//! non-mutating helper is exercised here.
//!
//!   cargo run --example sugar_basics

use std::time::Duration;

use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Settings::load()?;
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)));
    sugar.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    let sym = cfg.test_symbol.as_str();

    // ── connection & account ──
    println!("alive: {}", sugar.ping().await);
    let a = sugar.get_account_info().await?;
    println!("#{}  {:.2} {}  equity {:.2}", a.login, a.balance, a.currency, a.equity);

    // ── prices ──
    sugar.ensure_symbol(sym, Duration::from_secs(5)).await?;
    let q = sugar.last_quote(sym).await?;
    let qs = sugar.quotes(&[sym, "GBPUSD"]).await?;
    println!(
        "{} bid {:.5} ask {:.5}  mid {:.5}  spread {:.1}p  (quotes {})",
        sym, q.bid, q.ask, sugar.mid_price(sym).await?, sugar.spread_pips(sym).await?, qs.len()
    );
    let bars = sugar.bars(sym, 4 /* H1 */, 20).await?;
    println!("last {} H1 bars; latest close {:.5}", bars.len(), bars.last().map(|b| b.close).unwrap_or(0.0));

    // ── symbol math ──
    let pip = sugar.pip_size(sym).await?;
    println!(
        "digits {}  point {}  pip {}  tick_value {}  20pips={:.5}  (0.0020 -> {:.1} pips)",
        sugar.digits(sym).await?,
        sugar.point(sym).await?,
        pip,
        sugar.tick_value(sym).await?,
        sugar.pips_to_price(sym, 20.0).await?,
        sugar.price_to_pips(sym, 0.0020).await?
    );
    println!("normalize: 0.137 lot -> {:.2}  price 1.234567 -> {:.5}", sugar.normalize_lot(sym, 0.137).await?, sugar.normalize_price(sym, 1.234567).await?);

    // ── risk sizing ──
    let risk_cash = a.balance * 0.01;
    let lot = sugar.calc_lot_by_risk(sym, risk_cash, 20.0).await?;
    let back = sugar.calc_cash_risk(sym, lot, 20.0).await?;
    println!("risk {:.2} {} over 20 pips -> {:.2} lot (≈ {:.2} back)", risk_cash, a.currency, lot, back);

    sugar.disconnect(false).await?;
    Ok(())
}
