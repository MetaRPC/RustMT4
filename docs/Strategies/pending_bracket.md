# Orchestrator — Pending Bracket

> Place a BUY LIMIT, **wait for the fill (or cancel on timeout)**, then bracket and manage. [`src/strategies/pending_bracket.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/strategies/pending_bracket.rs).

```rust
pub async fn run_pending_bracket(sugar: &MT4Sugar, sp: &StrategyPreset, rp: &RiskPreset, timeout_seconds: u64) -> Result<u64, MT4Error>
```

**Extra arg:** `timeout_seconds` — how long to wait for the pending to fill before cancelling it.

**Returns:** the order `ticket` (filled-and-managed, or the cancelled pending).

---

## What it does

1. **Entry price** — `sp.entry_price` if set, else **5 pips below current bid** (demo default).
2. **Size** — `size_lot(sugar, sp, rp)`.
3. **Place** — `buy_limit(symbol, lots, entry, None, None)`.
4. **Wait** — `wait_filled(ticket, timeout_seconds)`:
   * **Not filled in time** → `close(ticket, None)` deletes the resting pending, return the ticket.
   * **Filled** → continue.
5. **Bracket** — `modify_sl_tp_by_pips(ticket, Some(sl_pips), tp_pips)`.
6. **Manage** — `manage()` for **~5 seconds** (trail from `be_trigger_pips`).
7. **Close** — `close(ticket, None)`.

---

## Usage

```rust
use metarpc_mt4::strategies::{run_pending_bracket, limit_eurusd, conservative};

// Explicit limit price via the preset
let sp = limit_eurusd(1.0750);
let rp = conservative();
let ticket = run_pending_bracket(&sugar, &sp, &rp, 60).await?;   // wait up to 60s for the fill
```

Or let it pick the demo entry (5 pips below bid):

```rust
use metarpc_mt4::strategies::{run_pending_bracket, market_eurusd, balanced};
let sp = market_eurusd();   // no entry_price → 5 pips below bid
run_pending_bracket(&sugar, &sp, &balanced(), 30).await?;
```

---

## Notes

* **Fill detection** = the ticket's `order_type` flipping from `BuyLimit` (2) to `Buy` (0); `wait_filled` polls every 500 ms.
* **Cancel is a `close`.** Deleting a resting pending uses the same `order_close_delete` path (mode = pending deleted).
* **Timeout returns the ticket anyway** — check whether it filled via your own state if you need to branch.

---

**See also:** [Strategies — Master Overview](./Strategies.Master.Overview.md) · [market_one_shot](./market_one_shot.md) · [bracket_trailing](./bracket_trailing.md)
