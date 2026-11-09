# Strategies — Presets

> The two builder types that configure every orchestrator, plus the ready-made catalog. Defined in [`src/strategies/mod.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/strategies/mod.rs).

---

## `StrategyPreset`

**WHAT to trade.** A fluent builder.

| Field | Type | Meaning |
|-------|------|---------|
| `symbol` | `String` | Instrument |
| `use_market` | `bool` | `true` = market entry, `false` = pending limit |
| `entry_price` | `Option<f64>` | Limit price (when not market) |
| `lots` | `Option<f64>` | Fixed volume; `None` = size by risk % |
| `magic` | `i32` | EA magic stamped on the order |
| `comment` | `String` | Order comment |

**Builder methods:**

```rust
StrategyPreset::new("EURUSD")   // defaults: market, no fixed lots, magic 0
    .market()                   // market entry (clears entry_price)
    .limit_at(1.0750)           // ...or a BUY LIMIT at a price
    .lots(0.10)                 // fixed volume (skip risk sizing)
    .magic(777)
    .comment("my setup");
```

> Leave `.lots()` unset to let the orchestrator size by the `RiskPreset`'s risk %.

---

## `RiskPreset`

**HOW MUCH to risk and how to protect.** A fluent builder.

| Field | Type | Meaning |
|-------|------|---------|
| `risk_percent` | `f64` | % of balance to risk when sizing by risk |
| `sl_pips` | `f64` | Stop-loss distance in pips (also the risk denominator) |
| `tp_pips` | `Option<f64>` | Take-profit distance in pips |
| `trailing_pips` | `Option<f64>` | Trailing distance; `None` = no trailing |
| `be_trigger_pips` | `Option<f64>` | Profit at which breakeven arms |
| `be_plus_pips` | `f64` | Pips of profit locked at breakeven (`0` = exact BE) |

**Builder methods:**

```rust
RiskPreset::new(1.0, 20.0)      // 1% risk, 20-pip stop
    .tp(40.0)                   // 40-pip target
    .trailing(15.0)             // trail 15 pips behind
    .breakeven(10.0, 2.0);      // arm BE at +10p, lock +2p
```

---

## Preset catalog

Ready-made builders you can pass straight to an orchestrator:

### Strategy presets

| Fn | Builds |
|----|--------|
| `market_eurusd()` | Market EURUSD, magic 777 |
| `limit_eurusd(price)` | BUY LIMIT EURUSD at `price`, magic 778 |

### Risk presets

| Fn | Risk % | SL | TP | Trail | Breakeven |
|----|--------|----|----|-------|-----------|
| `conservative()` | 0.5 | 25 | 50 | — | — |
| `balanced()` | 1.0 | 20 | 40 | 15 | 10 / +2 |
| `aggressive()` | 2.0 | 15 | 45 | 10 | 8 / +1 |

```rust
use metarpc_mt4::strategies::{market_eurusd, conservative, run_market_one_shot};

let sp = market_eurusd();
let rp = conservative();
run_market_one_shot(&sugar, &sp, &rp).await?;
```

---

## How the presets drive sizing

The orchestrators call `size_lot`:

* `StrategyPreset.lots = Some(x)` → use `x` directly.
* `lots = None` → `calc_lot_by_risk(symbol, balance × risk_percent / 100, sl_pips)` — the normalized lot that risks `risk_percent` of balance over the stop.

---

**See also:** [Strategies — Master Overview](./Strategies.Master.Overview.md) · [MT4Sugar 4. Risk Sizing](../MT4Sugar/4.%20Risk_Sizing.md)
