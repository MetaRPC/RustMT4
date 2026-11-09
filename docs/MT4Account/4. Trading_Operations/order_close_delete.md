# Order Close / Delete

> Close a market order (full or partial) **or** delete a pending — one call, mode reported back.

**API Information:**

* **Low-level API:** `MT4Account::order_close_delete(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.TradingHelper`
* **RPC:** `OrderCloseDelete(OrderCloseDeleteRequest) -> OrderCloseDeleteReply`
* **Payload:** `OrderCloseDeleteData`

## 💬 Just the essentials

* **What it is.** MQL `OrderClose()` for market orders and `OrderDelete()` for pendings, merged — the server picks based on the ticket.
* **Why you need it.** Exit a position or cancel a pending.
* **When to use.** Closing out; partial close for scaling out.

---

## Method Signature

```rust
pub async fn order_close_delete(&self, request: pb::OrderCloseDeleteRequest) -> Result<pb::OrderCloseDeleteData, MT4Error>
```

**Request / Reply:**

```protobuf
message OrderCloseDeleteRequest {
  int32  order_ticket = 1;             // required
  optional double lots = 2;            // market only; omit/None = full close
  optional double closing_price = 3;   // market only; omit = current market price
  optional int32  slippage = 4;        // points
}
message OrderCloseDeleteData {
  OrderCloseDeleteMode mode = 1;             // which action the server performed
  optional string history_order_comment = 2; // broker close comment, if any
}
```

## ⬇️ Input — `OrderCloseDeleteRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `order_ticket` | `i32` | Ticket to close or delete (required) |
| `lots` | `Option<f64>` | **Market only** — partial volume; `None` = close the whole order. Ignored for pendings |
| `closing_price` | `Option<f64>` | **Market only** — explicit close price; `None` = market |
| `slippage` | `Option<i32>` | Max deviation in points |

## ⬆️ Output — `OrderCloseDeleteData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `mode` | `i32` | `OrderCloseDeleteMode` — what was done |
| `history_order_comment` | `Option<String>` | Broker's comment on the resulting history order |

### `OrderCloseDeleteMode`

| Variant | Value | Meaning |
|---------|-------|---------|
| `OcdMarketOrder` | 0 | A market order was **closed** |
| `OcdPendingOrder` | 1 | A pending order was **deleted** |

---

## 🧩 Notes & Tips

* **One call, two behaviours.** Pass a market ticket → it closes; pass a pending ticket → it deletes. `mode` tells you which happened.
* **Partial close** = set `lots` below the order volume (market orders only); the remainder stays open under a new ticket per MT4 rules.
* **Not retried** on transient errors — reconcile with `opened_orders` on failure.

---

## 🔗 Usage Examples

### 1) Full close of a position

```rust
use metarpc_mt4::mt4_term_api as pb;

let d = account.order_close_delete(pb::OrderCloseDeleteRequest {
    order_ticket: 123_456,
    ..Default::default()
}).await?;
println!("mode = {:?}", d.mode);   // OcdMarketOrder
```

### 2) Partial close (scale out half)

```rust
use metarpc_mt4::mt4_term_api as pb;

account.order_close_delete(pb::OrderCloseDeleteRequest {
    order_ticket: 123_456,
    lots: Some(0.05),
    slippage: Some(20),
    ..Default::default()
}).await?;
```

### 3) Delete a pending

```rust
use metarpc_mt4::mt4_term_api as pb;
let d = account.order_close_delete(pb::OrderCloseDeleteRequest { order_ticket: 123_457, ..Default::default() }).await?;
assert_eq!(d.mode, pb::OrderCloseDeleteMode::OcdPendingOrder as i32);
```

---

## ⚠️ Pitfalls

* **`lots` on a pending is ignored** — pendings are deleted whole.
* **Partial remainder gets a new ticket.** Track the returned state, don't assume the old ticket still holds the rest.
* **Close by price mismatch (129/138).** Let `closing_price` default to market unless you have a reason not to.

---

## 📚 See also

* [order_close_by](order_close_by.md) · [order_send](order_send.md) · [orders_history](../2.%20Orders_History/orders_history.md)
* **Back to:** [Trading Operations — Overview](Trading_Operations.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
