# Strategies — Master Overview

> Ready-made **orchestrators** that chain [`MT4Sugar`](../MT4Sugar/MT4Sugar.Master.Overview.md) calls into a full trade lifecycle — size → enter → manage → close — driven by two small preset types.

Defined in [`src/strategies/`](https://github.com/MetaRPC/RustMT4/blob/main/src/strategies/mod.rs). Each orchestrator is a plain `async fn run_*(sugar, …)`.

---

## ⚠️ Demonstration templates only

**These orchestrators are teaching examples — run them on DEMO accounts.** They show how to compose the SDK into a lifecycle; they are **not** production trading systems. There is no market analysis, no portfolio risk control, no drawdown guard, and the management window is a short fixed timer. Use them as a skeleton and add your own edge, risk rules and supervision before anything live.

---

## The two presets

A strategy is **what** to trade + **how much** to risk:

* **[`StrategyPreset`](./Presets.md#strategypreset)** — WHAT: symbol, market vs limit entry, optional fixed lots, magic, comment.
* **[`RiskPreset`](./Presets.md#riskpreset)** — HOW MUCH: risk %, SL/TP in pips, optional trailing and breakeven.

Catalog of ready builders: `market_eurusd()`, `limit_eurusd(price)`, `conservative()`, `balanced()`, `aggressive()` — see **[Presets](./Presets.md)**.

---

## The three orchestrators

| Orchestrator | Entry | Management | Page |
|--------------|-------|------------|------|
| **Market One-Shot** | market BUY | breakeven + trail (from preset), ~5 s | [market_one_shot](./market_one_shot.md) |
| **Bracket + Trailing** | market BUY + bracket | trail starts only after a profit cushion, ~10 s | [bracket_trailing](./bracket_trailing.md) |
| **Pending Bracket** | BUY LIMIT, wait for fill (or cancel) | breakeven + trail after fill | [pending_bracket](./pending_bracket.md) |

All three: size the lot (fixed or by risk %), enter, apply a **pip-based SL/TP bracket**, run the shared `manage()` worker, then flatten.

---

## Shared building blocks

* **`size_lot(sugar, sp, rp)`** — fixed `sp.lots` if set, else `calc_lot_by_risk(balance × risk% , sl_pips)`.
* **`manage(sugar, ticket, symbol, rp, secs, trail_activate_pips)`** — for `secs`, repeatedly: apply [`auto_breakeven`](../MT4Sugar/7.%20Trailing_Breakeven.md) (from `be_trigger_pips`) and, once at least `trail_activate_pips` in profit, [`trail_stop`](../MT4Sugar/7.%20Trailing_Breakeven.md). Returns early if the order closed (SL/TP hit). Polls every 500 ms.

---

## Minimal usage

```rust
use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar};
use metarpc_mt4::strategies::{run_market_one_shot, market_eurusd, balanced};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)))
        .set_defaults(777, 20, "demo");
    sugar.connect_by_server_name(12_345_678, "password", "FreshForex-Demo", None, 120).await?;

    let sp = market_eurusd();   // WHAT: market EURUSD, magic 777
    let rp = balanced();        // HOW: 1% risk, SL 20 / TP 40, trail 15, BE 10/+2
    let ticket = run_market_one_shot(&sugar, &sp, &rp).await?;
    println!("done, ticket {ticket}");

    sugar.disconnect(false).await?;
    Ok(())
}
```

> The runnable version is [`examples/4_strategies/strategies_demo.rs`](https://github.com/MetaRPC/RustMT4/blob/main/examples/4_strategies/strategies_demo.rs) — gated behind `STRATEGY_RUN=1`.

---

**See also:** [Presets](./Presets.md) · [MT4Sugar — Master Overview](../MT4Sugar/MT4Sugar.Master.Overview.md)
