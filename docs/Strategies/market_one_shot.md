# Orchestrator — Market One-Shot

> The simplest lifecycle: size → market BUY → pip bracket → manage briefly → close. [`src/strategies/market_one_shot.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/strategies/market_one_shot.rs).

```rust
pub async fn run_market_one_shot(sugar: &MT4Sugar, sp: &StrategyPreset, rp: &RiskPreset) -> Result<u64, MT4Error>
```

**Returns:** the order `ticket`.

---

## What it does

1. **Size** — `size_lot(sugar, sp, rp)` (fixed `sp.lots`, else risk-based).
2. **Enter** — `buy_market(symbol, lots, None, None)`.
3. **Bracket** — `modify_sl_tp_by_pips(ticket, Some(sl_pips), tp_pips)` from the `RiskPreset`.
4. **Manage** — the shared `manage()` worker for **~5 seconds**; trailing activates from `be_trigger_pips` (or 0 if unset), breakeven from `be_trigger_pips`.
5. **Close** — `close(ticket, None)` and return the ticket.

---

## Usage

```rust
use metarpc_mt4::strategies::{run_market_one_shot, market_eurusd, balanced};

let sp = market_eurusd();
let rp = balanced();
let ticket = run_market_one_shot(&sugar, &sp, &rp).await?;
println!("ran one-shot, ticket {ticket}");
```

---

## Notes

* **BUY-side demo.** Like all three templates it enters long; mirror the logic for shorts.
* **Fixed management window.** The ~5 s timer is for demonstration — a real strategy manages until an exit signal, not a clock.
* **Early exit.** If SL/TP fills during management, `manage()` returns as soon as the ticket is gone; the final `close` then no-ops.

---

**See also:** [Strategies — Master Overview](./Strategies.Master.Overview.md) · [bracket_trailing](./bracket_trailing.md) · [pending_bracket](./pending_bracket.md)
