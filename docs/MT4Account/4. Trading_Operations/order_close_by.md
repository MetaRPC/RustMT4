# Order Close By

> Net two opposite orders on the same symbol against each other — close one *by* the other.

**API Information:**

* **Low-level API:** `MT4Account::order_close_by(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.TradingHelper`
* **RPC:** `OrderCloseBy(OrderCloseByRequest) -> OrderCloseByReply`
* **Payload:** `OrderCloseByData`

## 💬 Just the essentials

* **What it is.** MQL `OrderCloseBy()` — closes a buy against a sell (same symbol), saving one spread.
* **Why you need it.** Flatten a hedged pair in a single operation instead of closing both at market.
* **When to use.** You hold opposite positions on one symbol and want to net them.

---

## Method Signature

```rust
pub async fn order_close_by(&self, request: pb::OrderCloseByRequest) -> Result<pb::OrderCloseByData, MT4Error>
```

**Request / Reply:**

```protobuf
message OrderCloseByRequest {
  int32 ticket_to_close = 1;            // the order being closed
  int32 opposite_ticket_closing_by = 2; // the opposite order used to close it
}
message OrderCloseByData {
  double close_price = 1;
  double profit = 2;
  google.protobuf.Timestamp close_time = 3;
  google.protobuf.Timestamp close_time_utc = 4;
}
```

## ⬇️ Input — `OrderCloseByRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `ticket_to_close` | `i32` | The order you want closed |
| `opposite_ticket_closing_by` | `i32` | An **opposite-side** order on the **same symbol** used to close it |

## ⬆️ Output — `OrderCloseByData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `close_price` | `f64` | Price at which the netting happened |
| `profit` | `f64` | Realized P/L from the operation |
| `close_time` / `close_time_utc` | `Option<Timestamp>` | Close time (server / UTC) |

---

## 🧩 Notes & Tips

* **Same symbol, opposite side** — a buy closes a sell and vice versa; different symbols are rejected.
* **Unequal volumes.** If the two orders differ in size, the larger one leaves a remainder open (MT4 opens a new ticket for it).
* **Saves a spread** versus two independent market closes.
* **Not all brokers allow it** — some disable close-by; you'll get an `MT4Error::Api`.

---

## 🔗 Usage Examples

### 1) Net a hedge

```rust
use metarpc_mt4::mt4_term_api as pb;

let d = account.order_close_by(pb::OrderCloseByRequest {
    ticket_to_close: 123_456,            // the buy
    opposite_ticket_closing_by: 123_460, // the sell
}).await?;
println!("closed by @ {} profit {:+.2}", d.close_price, d.profit);
```

### 2) Guard before calling (confirm they're opposite on one symbol)

```rust
use metarpc_mt4::mt4_term_api as pb;

let orders = account.opened_orders(pb::OpenedOrdersRequest { sort_type: 0 }).await?.order_infos;
let buy  = orders.iter().find(|o| o.symbol == "EURUSD" && o.order_type == 0);
let sell = orders.iter().find(|o| o.symbol == "EURUSD" && o.order_type == 1);
if let (Some(b), Some(s)) = (buy, sell) {
    account.order_close_by(pb::OrderCloseByRequest {
        ticket_to_close: b.ticket,
        opposite_ticket_closing_by: s.ticket,
    }).await?;
}
```

---

## ⚠️ Pitfalls

* **Must be opposite + same symbol** — otherwise rejected.
* **Volume remainder** opens a fresh ticket; re-read `opened_orders` afterwards.
* **Broker support varies** — handle the "not allowed" error.

---

## 📚 See also

* [order_close_delete](order_close_delete.md) · [order_send](order_send.md)
* **Back to:** [Trading Operations — Overview](Trading_Operations.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
