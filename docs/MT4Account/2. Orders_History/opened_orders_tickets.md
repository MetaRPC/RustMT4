# Opened Orders Tickets

> Just the ticket numbers of all open orders — a lightweight list for iteration or diffing.

**API Information:**

* **Low-level API:** `MT4Account::opened_orders_tickets()` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.AccountHelper`
* **RPC:** `OpenedOrdersTickets(OpenedOrdersTicketsRequest) -> …Reply`
* **Payload:** `OpenedOrdersTicketsData` → `tickets: Vec<i32>`

## 💬 Just the essentials

* **What it is.** The ticket ids only — no prices, no P/L.
* **Why you need it.** Cheap iteration, or diffing "what's open now vs before" without the full payload.
* **When to use.** When you only need ids (to modify/close by ticket). For details use [`opened_orders`](./opened_orders.md).

---

## Method Signature

```rust
pub async fn opened_orders_tickets(&self) -> Result<pb::OpenedOrdersTicketsData, MT4Error>
```

Takes no argument (the request is empty).

**Reply:**

```protobuf
message OpenedOrdersTicketsData { repeated int32 tickets = 1; }
```

---

## 🔗 Usage Examples

### 1) List the tickets

```rust
let t = account.opened_orders_tickets().await?;
println!("{} open: {:?}", t.tickets.len(), t.tickets);
```

### 2) Detect new orders across two polls

```rust
use std::collections::HashSet;
let before: HashSet<i32> = account.opened_orders_tickets().await?.tickets.into_iter().collect();
// ... later ...
let after: HashSet<i32> = account.opened_orders_tickets().await?.tickets.into_iter().collect();
for t in after.difference(&before) { println!("new order {t}"); }
```

### 3) Iterate and act

```rust
for ticket in account.opened_orders_tickets().await?.tickets {
    // e.g. look it up in opened_orders, or modify/close by ticket
    let _ = ticket;
}
```

---

## ⚠️ Pitfalls

* **Ids are `i32`.** Cast when a method wants `u64` (`ticket as u64`).
* **No details.** For symbol/lots/P/L call [`opened_orders`](opened_orders.md).

---

## 📚 See also

* [opened_orders](opened_orders.md) · [on_opened_orders_tickets](../5.%20Streaming_Methods/on_opened_orders_tickets.md)
* **Back to:** [Orders & History — Overview](Orders_History.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
