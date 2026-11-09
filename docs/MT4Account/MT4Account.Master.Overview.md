# MT4Account — Master Overview

> One page to orient fast: the low-level async gRPC client for MetaTrader 4. Jump to any group overview or method spec from here.

`MT4Account` is **Layer 1** — direct gRPC/protobuf calls to the MT4 terminal gateway. [`MT4Service`](../MT4Service/MT4Service.Overview.md) (typed facade) and [`MT4Sugar`](../MT4Sugar/MT4Sugar.Master.Overview.md) (high-level helpers) build on top of it.

It exposes **25 feature methods** across 5 groups (plus connection helpers). Defined in [`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs).

> **MT4, not MT5.** MT4 is order-based: there are no separate "positions" — a filled market order **is** the position, and pending orders live in the same list. Symbol properties come in one bulk `symbol_params_many` call rather than per-property getters.

---

## 🚦 Start here — Section Overviews

* **[1. Account Information](./1.%20Account_Information/Account_Information.Overview.md)** — balance/equity snapshot, trade statistics, equity curve, unrealized-P/L extrema.
* **[2. Orders & History](./2.%20Orders_History/Orders_History.Overview.md)** — open orders (market + pending), their tickets, and closed-order history.
* **[3. Symbols & Quotes](./3.%20Symbols_Quotes/Symbols_Quotes.Overview.md)** — symbol list, Market Watch select, bulk params, ticks, bar history, server timezone.
* **[4. Trading Operations](./4.%20Trading_Operations/Trading_Operations.Overview.md)** — send, modify, close/delete, and close-by.
* **[5. Streaming Methods](./5.%20Streaming_Methods/Streaming_Methods.Overview.md)** — real-time server streams: ticks, trade events, ticket lists, per-order profit, connection state.

---

## 🧭 How to pick a method

| If you need… | Group | Typical methods |
|--------------|-------|-----------------|
| Account snapshot / stats | Account Information | `account_summary`, `trade_stats`, `equity_history` |
| Current open orders | Orders & History | `opened_orders`, `opened_orders_tickets` |
| Closed-order history | Orders & History | `orders_history` |
| Quotes & symbol specs | Symbols & Quotes | `quote`, `quote_many`, `symbol_params_many` |
| Bars | Symbols & Quotes | `quote_history` |
| Send / modify / close orders | Trading Operations | `order_send`, `order_modify`, `order_close_delete`, `order_close_by` |
| Real-time updates | Streaming Methods | `on_symbol_tick`, `on_trade`, `on_opened_orders_profit` |

---

## 🔌 Usage pattern (Rust, async)

`MT4Account` is **async** (tokio): each call builds the protobuf request, attaches the `id` + `apikey` metadata, and returns the reply's typed `…Data`, or an [`MT4Error`](../All_Guides/RETURN_CODES_REFERENCE.md).

```rust
use metarpc_mt4::mt4_term_api as pb;
use metarpc_mt4::MT4Account;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let account = MT4Account::new("mt4.mrpc.pro", 443); // api key from MRPC_API_KEY / "TRIAL"
    account.connect_by_server_name(12_345_678, "password", "FreshForex-Demo", None, 120).await?;

    let s = account.account_summary().await?;
    println!("{:.2} {} eq {:.2}", s.account_balance, s.account_currency, s.account_equity);

    let q = account.quote(pb::QuoteRequest { symbol: "EURUSD".into() }).await?;
    println!("bid/ask {:.5}/{:.5}", q.bid, q.ask);

    account.disconnect(false).await?;
    Ok(())
}
```

---

## 🧱 How the call contract works

Every feature method:

1. builds the protobuf `…Request` (many MT4 requests have optional fields — pass `..Default::default()`),
2. attaches the `id` + `apikey` metadata,
3. calls the right tonic service client,
4. unwraps the reply — each `…Reply` carries a `oneof { Data, Error }`, returned as `Result<…Data, MT4Error>`.

So you always get the typed `…Data` directly. Friendly argument shapes live in [`MT4Service`](../MT4Service/MT4Service.Overview.md) and [`MT4Sugar`](../MT4Sugar/MT4Sugar.Master.Overview.md).

---

## 🔐 Constructor & connection

```rust
MT4Account::new(host: impl Into<String>, port: u16) -> MT4Account
```

Connect by broker **server name** (cluster, e.g. `"FreshForex-Demo"`):

```rust
account.connect_by_server_name(login, "password", "FreshForex-Demo", Some("my-label"), 120).await?;
```

Pass a stable terminal label (`Some("…")`) so re-runs reuse the same terminal. `reconnect()` re-establishes the session with the stored parameters; `disconnect(delete)` stops it (`delete = true` removes it permanently).

---

## 🧩 Layer map

| Layer | Type | What it adds |
|-------|------|--------------|
| 1 — low-level | **`MT4Account`** | raw gRPC, auto-metadata, typed `…Data` returns |
| 2 — facade | [`MT4Service`](../MT4Service/MT4Service.Overview.md) | friendly args (`&str`, sort enums), clean values |
| 3 — high-level | [`MT4Sugar`](../MT4Sugar/MT4Sugar.Master.Overview.md) | order construction, pip math, risk sizing, management |

---

**See also:** [Getting Started](../All_Guides/GETTING_STARTED.md) · [Return Codes](../All_Guides/RETURN_CODES_REFERENCE.md) · [ENUMs Reference](../All_Guides/ENUMS_USAGE_REFERENCE.md)
