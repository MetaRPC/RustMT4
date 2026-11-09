# Orchestrator — Bracket + Trailing

> Enter with a bracket, then **start trailing only after the trade earns a cushion**. [`src/strategies/bracket_trailing.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/strategies/bracket_trailing.rs).

```rust
pub async fn run_bracket_trailing(sugar: &MT4Sugar, sp: &StrategyPreset, rp: &RiskPreset, activate_at_pips: f64) -> Result<u64, MT4Error>
```

**Extra arg:** `activate_at_pips` — trailing stays off until the position is this many pips in profit.

**Returns:** the order `ticket`.

---

## What it does

1. **Size** — `size_lot(sugar, sp, rp)`.
2. **Enter** — `buy_market(symbol, lots, None, None)`.
3. **Bracket** — `modify_sl_tp_by_pips(ticket, Some(sl_pips), tp_pips)`.
4. **Manage** — `manage()` for **~10 seconds**, passing `activate_at_pips` as the trail activation gate. Auto-breakeven (from `be_trigger_pips`) still applies **earlier**; trailing waits for the cushion.
5. **Close** — `close(ticket, None)`.

---

## Why the activation gate

Trailing immediately from entry often gets stopped out by normal spread/noise before the trade develops. Gating it behind `activate_at_pips` lets the position breathe, then protects profit once it's real — while breakeven still arms as soon as `be_trigger_pips` is hit.

---

## Usage

```rust
use metarpc_mt4::strategies::{run_bracket_trailing, market_eurusd, balanced};

let sp = market_eurusd();
let rp = balanced();                 // trail 15, BE 10/+2
let ticket = run_bracket_trailing(&sugar, &sp, &rp, 12.0).await?;  // trail arms at +12 pips
```

---

## Notes

* **Needs `trailing_pips` set** in the `RiskPreset` for the gate to matter (`balanced()` / `aggressive()` set it; `conservative()` does not).
* **Longer window (~10 s)** than Market One-Shot, to give the cushion time to form in the demo.

---

**See also:** [Strategies — Master Overview](./Strategies.Master.Overview.md) · [market_one_shot](./market_one_shot.md) · [pending_bracket](./pending_bracket.md)
