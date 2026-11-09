# MT4Sugar — Master Overview

> High-level, one-line convenience methods for MetaTrader 4 automation in Rust.

**`MT4Sugar`** wraps an `MT4Service` (composition) and adds ergonomic business logic: quick prices, symbol math, risk-based lot sizing, order construction from side/lots, order management, trailing/breakeven, live watchers and account info. Defined in [`src/mt4/sugar.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/sugar.rs).

It reuses the plain DTOs in [`crate::models`](https://github.com/MetaRPC/RustMT4/blob/main/src/models.rs): `Quote`, `Bar`, `OrderRequest`, `OrderResult`, `CloseResult`, `AccountInfo`, `OrderType`.

```text
MT4Sugar (HIGH)  ← you are here
   │ uses  MT4Service (MID) → MT4Account (LOW) → gRPC → MT4 terminal
```

---

## Conventions

* **Pips.** A pip is `point × 10` on 3/5-digit symbols, else `point`.
* **Defaults.** `MT4Sugar::new(svc).set_defaults(magic, slippage, comment)` sets the magic / slippage / comment used by the order-entry helpers.
* **Orders, not positions.** MT4's open market orders are the "positions"; `orders()` returns everything, `positions()` the market ones.
* **Async.** Every method is `async` — `.await` it.

---

## Navigation by category

### 🔌 [1] Connection & Defaults
`connect_by_server_name` · `reconnect` · `disconnect` · `ping` · `set_defaults` → [Docs](./1.%20Connection_Defaults.md)

### 💹 [2] Prices & Quotes
`last_quote` · `quotes` · `mid_price` · `spread_pips` · `ensure_symbol` · `bars` → [Docs](./2.%20Prices_Quotes.md)

### 📐 [3] Symbol Math
`point` · `digits` · `pip_size` · `pips_to_price` · `price_to_pips` · `normalize_lot` · `normalize_price` · `tick_value` → [Docs](./3.%20Symbol_Math.md)

### 🛡️ [4] Risk Sizing
`calc_cash_risk` · `calc_lot_by_risk` → [Docs](./4.%20Risk_Sizing.md)

### 🟢 [5] Order Entry
`place_order` · `buy_market` · `sell_market` · `buy_limit` · `sell_limit` · `buy_stop` · `sell_stop` → [Docs](./5.%20Order_Entry.md)

### 🛠️ [6] Order Management
`close` · `close_partial` · `close_by` · `close_all` · `cancel_pendings` · `modify_sl_tp_by_price` · `modify_sl_tp_by_pips` → [Docs](./6.%20Order_Management.md)

### 🪜 [7] Trailing & Breakeven
`auto_breakeven` · `trail_stop` → [Docs](./7.%20Trailing_Breakeven.md)

### 👀 [8] Watchers & Waiters
`subscribe_quotes` · `watch_profit` · `watch_tickets` · `wait_price` · `wait_filled` · `wait_closed` → [Docs](./8.%20Watchers_Waiters.md)

### 📋 [9] Order Info
`orders` · `positions` · `orders_for_symbol` · `orders_by_magic` · `find_ticket` · `total_profit` · `positions_value` · `exposure_summary` · `get_account_info` → [Docs](./9.%20Order_Info.md)

---

## Minimal usage

```rust
use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)))
        .set_defaults(1001, 20, "demo");
    sugar.connect_by_server_name(12_345_678, "password", "FreshForex-Demo", None, 120).await?;

    let lots = sugar.calc_lot_by_risk("EURUSD", sugar.get_account_info().await?.balance * 0.01, 20.0).await?;
    let r = sugar.buy_market("EURUSD", lots, None, None).await?;
    sugar.modify_sl_tp_by_pips(r.ticket, Some(20.0), Some(40.0)).await?;
    println!("opened #{}", r.ticket);

    sugar.disconnect(false).await?;
    Ok(())
}
```

---

**See also:** [MT4Service — Overview](../MT4Service/MT4Service.Overview.md) · [MT4Account — Master Overview](../MT4Account/MT4Account.Master.Overview.md) · [Strategies](../Strategies/Strategies.Master.Overview.md)
