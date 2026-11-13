/*══════════════════════════════════════════════════════════════════════════════
 FILE: examples/3_sugar/basics.rs - HIGH-LEVEL (MT4Sugar) BASICS DEMO
 PURPOSE:
   Read-only MT4Sugar one-liners: account, prices, bars, symbol math and risk
   sizing. Every non-mutating helper is exercised here - no trades.

 🎯 WHO SHOULD USE THIS:
   • Developers who want the short, ergonomic MT4Sugar API
   • Quick price/symbol/risk lookups without raw protobuf
   • Learning the read-only helpers before trading

 📚 WHAT THIS DEMO COVERS (4 sections):

   1. CONNECTION & ACCOUNT
      • ping() / get_account_info()  - liveness + balance/equity/currency

   2. PRICES
      • ensure_symbol() / last_quote() / quotes()
      • mid_price() / spread_pips() / bars()

   3. SYMBOL MATH
      • digits() / point() / pip_size() / tick_value()
      • pips_to_price() / price_to_pips() / normalize_lot() / normalize_price()

   4. RISK SIZING
      • calc_lot_by_risk() / calc_cash_risk()

 🔄 API LEVELS:
   HIGH-LEVEL (MT4Sugar)   - THIS FILE: one-liner helpers with smart defaults
   MID-LEVEL  (MT4Service) - typed wrappers
   LOW-LEVEL  (MT4Account) - raw gRPC/protobuf

 ℹ️  SAFE: read-only — never sends an order.

 USAGE:
   cargo run --example sugar_basics
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
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new(cfg.host.clone(), cfg.port)));
    sugar.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    let sym = cfg.test_symbol.as_str();
    // endregion: CONNECT

    // region: 1. CONNECTION & ACCOUNT
    // ══════════════════════════════════════════════════════════════
    // 1. CONNECTION & ACCOUNT
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // ping() / get_account_info()
    //   Liveness check + balance / equity / currency snapshot.
    // ──────────────────────────────────────────────────────────────
    println!("alive: {}", sugar.ping().await);
    let a = sugar.get_account_info().await?;
    println!("#{}  {:.2} {}  equity {:.2}", a.login, a.balance, a.currency, a.equity);
    // endregion: 1. CONNECTION & ACCOUNT

    // region: 2. PRICES
    // ══════════════════════════════════════════════════════════════
    // 2. PRICES
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // ensure_symbol() / last_quote() / quotes() / mid_price() / spread_pips()
    //   Select the symbol, read its latest Bid/Ask (one or many), mid
    //   price and spread in pips.
    // ──────────────────────────────────────────────────────────────
    sugar.ensure_symbol(sym, Duration::from_secs(5)).await?;
    let q = sugar.last_quote(sym).await?;
    let qs = sugar.quotes(&[sym, "GBPUSD"]).await?;
    println!(
        "{} bid {:.5} ask {:.5}  mid {:.5}  spread {:.1}p  (quotes {})",
        sym, q.bid, q.ask, sugar.mid_price(sym).await?, sugar.spread_pips(sym).await?, qs.len()
    );

    // ──────────────────────────────────────────────────────────────
    // bars()
    //   Recent H1 candles (here the last 20) for the symbol.
    // ──────────────────────────────────────────────────────────────
    let bars = sugar.bars(sym, 4 /* H1 */, 20).await?;
    println!("last {} H1 bars; latest close {:.5}", bars.len(), bars.last().map(|b| b.close).unwrap_or(0.0));
    // endregion: 2. PRICES

    // region: 3. SYMBOL MATH
    // ══════════════════════════════════════════════════════════════
    // 3. SYMBOL MATH
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // digits/point/pip_size/tick_value + pips<->price + normalize_lot/price
    //   Precision metrics and the conversions used for order math.
    // ──────────────────────────────────────────────────────────────
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
    // endregion: 3. SYMBOL MATH

    // region: 4. RISK SIZING
    // ══════════════════════════════════════════════════════════════
    // 4. RISK SIZING
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // calc_lot_by_risk() / calc_cash_risk()
    //   Lot size for a cash risk over an SL distance, and the inverse.
    // ──────────────────────────────────────────────────────────────
    let risk_cash = a.balance * 0.01;
    let lot = sugar.calc_lot_by_risk(sym, risk_cash, 20.0).await?;
    let back = sugar.calc_cash_risk(sym, lot, 20.0).await?;
    println!("risk {:.2} {} over 20 pips -> {:.2} lot (≈ {:.2} back)", risk_cash, a.currency, lot, back);
    // endregion: 4. RISK SIZING

    sugar.disconnect(false).await?;
    Ok(())
}
