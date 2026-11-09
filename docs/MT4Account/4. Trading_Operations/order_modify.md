# Order Modify

> Change an order's SL/TP (open orders) or price/SL/TP/expiry (pendings).

**API Information:**

* **Low-level API:** `MT4Account::order_modify(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.TradingHelper`
* **RPC:** `OrderModify(OrderModifyRequest) -> OrderModifyReply`
* **Payload:** `OrderModifyData` → `order_was_modified: bool`

## 💬 Just the essentials

* **What it is.** MQL `OrderModify()` — adjust protective levels or a pending's trigger.
* **Why you need it.** Move SL to breakeven, trail a stop, retarget a pending.
* **When to use.** Managing a live order.

---

## Method Signature

```rust
pub async fn order_modify(&self, request: pb::OrderModifyRequest) -> Result<pb::OrderModifyData, MT4Error>
```

**Request / Reply:**

```protobuf
message OrderModifyRequest {
  int32  order_ticket = 1;                 // required
  optional double new_price = 2;           // pendings only; ignored for open market orders
  optional double new_stop_loss = 3;
  optional double new_take_profit = 4;
  optional google.protobuf.Timestamp new_expiration = 5;      // pending expiry (server time)
  optional google.protobuf.Timestamp new_expiration_utc = 6;  // ...or UTC; not both
}
message OrderModifyData { bool order_was_modified = 1; }
```

## ⬇️ Input — `OrderModifyRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `order_ticket` | `i32` | Ticket to modify (required) |
| `new_price` | `Option<f64>` | New trigger price — **pendings only**; `None` leaves it |
| `new_stop_loss` | `Option<f64>` | New SL price; `None` leaves it |
| `new_take_profit` | `Option<f64>` | New TP price; `None` leaves it |
| `new_expiration` / `new_expiration_utc` | `Option<Timestamp>` | New pending expiry — send **one**; `None` leaves it |

## ⬆️ Output — `OrderModifyData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `order_was_modified` | `bool` | `true` if the server accepted the change |

---

## 🧩 Notes & Tips

* **`None` = unchanged.** Only the fields you set are touched — safe partial updates.
* **"No changes" (error 1).** Re-sending identical levels is rejected by MT4 as a no-op.
* **Stops distance (130).** New SL/TP must respect `trade_stops_level` from `symbol_params_many`.
* **`MT4Sugar`** exposes `modify_sl_tp_by_price` and `modify_sl_tp_by_pips`.

---

## 🔗 Usage Examples

### 1) Move SL to breakeven

```rust
use metarpc_mt4::mt4_term_api as pb;

let d = account.order_modify(pb::OrderModifyRequest {
    order_ticket: 123_456,
    new_stop_loss: Some(1.08500),   // = entry price
    ..Default::default()
}).await?;
println!("modified: {}", d.order_was_modified);
```

### 2) Retarget a pending's trigger price

```rust
use metarpc_mt4::mt4_term_api as pb;

account.order_modify(pb::OrderModifyRequest {
    order_ticket: 123_457,
    new_price: Some(1.07250),
    ..Default::default()
}).await?;
```

---

## ⚠️ Pitfalls

* **Prices, not pips.** Convert pip offsets to absolute prices first.
* **`new_price` on an open order is ignored** — only SL/TP apply there.
* **Idempotent re-sends fail.** Guard against sending the same values twice.

---

## 📚 See also

* [order_send](order_send.md) · [order_close_delete](order_close_delete.md)
* **Back to:** [Trading Operations — Overview](Trading_Operations.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
