# On Opened Orders Tickets

> The list of open position and pending tickets, pushed on a timer.

**API Information:**

* **Low-level API:** `MT4Account::on_opened_orders_tickets(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.SubscriptionService`
* **RPC:** `OnOpenedOrdersTickets(OnOpenedOrdersTicketsRequest) -> stream OnOpenedOrdersTicketsReply`
* **Returns:** `tonic::Streaming<OnOpenedOrdersTicketsReply>`

## 💬 Just the essentials

* **What it is.** A lightweight heartbeat of *which* tickets are open — positions and pendings, split into two lists.
* **Why you need it.** Cheaply detect that the open set changed, then fetch detail only when it does.
* **When to use.** A watchdog loop; a "did anything open/close?" trigger.

---

## Method Signature

```rust
pub async fn on_opened_orders_tickets(&self, request: pb::OnOpenedOrdersTicketsRequest) -> Result<Streaming<pb::OnOpenedOrdersTicketsReply>, MT4Error>
```

**Request / Reply:**

```protobuf
message OnOpenedOrdersTicketsRequest { int32 pull_interval_milliseconds = 1; }
message OnOpenedOrdersTicketsReply { oneof response { OnOpenedOrdersTicketsData data = 1; Error error = 2; } }
message OnOpenedOrdersTicketsData {
  repeated int32 position_tickets = 1;
  repeated int32 pending_order_tickets = 2;
  google.protobuf.Timestamp server_time = 3;
  google.protobuf.Timestamp server_time_utc = 6;
  string terminal_instance_guid_id = 5;
}
```

## ⬇️ Input — `OnOpenedOrdersTicketsRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `pull_interval_milliseconds` | `i32` | How often (ms) the server re-samples and pushes the list |

## ⬆️ Frame — `OnOpenedOrdersTicketsData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `position_tickets` | `Vec<i32>` | Tickets of open **market** orders (positions) |
| `pending_order_tickets` | `Vec<i32>` | Tickets of **pending** orders |
| `server_time` / `server_time_utc` | `Option<Timestamp>` | Sample time (server / UTC) |
| `terminal_instance_guid_id` | `String` | Emitting terminal |

---

## 🧩 Notes & Tips

* **Positions vs pendings are pre-split** here (unlike `opened_orders`, where you filter by `order_type`).
* **Diff against the previous frame** to spot opens/closes, then call `opened_orders` for the details of changed tickets.
* **`pull_interval_milliseconds`** trades freshness for load — 500–2000 ms is typical.
* **`MT4Sugar::watch_tickets`** wraps this into an mpsc channel.

---

## 🔗 Usage Examples

### 1) React when the open set changes

```rust
use std::collections::BTreeSet;
use metarpc_mt4::mt4_term_api as pb;

let mut s = account.on_opened_orders_tickets(pb::OnOpenedOrdersTicketsRequest {
    pull_interval_milliseconds: 1000,
}).await?;

let mut prev: BTreeSet<i32> = BTreeSet::new();
while let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_opened_orders_tickets_reply::Response::Data(d)) = reply.response {
        let now: BTreeSet<i32> = d.position_tickets.iter().copied().collect();
        if now != prev {
            println!("positions changed: {:?}", now);
            prev = now;
        }
    }
}
```

### 2) Count open positions and pendings

```rust
use metarpc_mt4::mt4_term_api as pb;
let mut s = account.on_opened_orders_tickets(pb::OnOpenedOrdersTicketsRequest { pull_interval_milliseconds: 2000 }).await?;
if let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_opened_orders_tickets_reply::Response::Data(d)) = reply.response {
        println!("{} positions, {} pendings", d.position_tickets.len(), d.pending_order_tickets.len());
    }
}
```

---

## ⚠️ Pitfalls

* **Tickets only.** No prices/P/L here — use `on_opened_orders_profit` or `opened_orders` for detail.
* **Interval too small** = needless load; pick a sane cadence.
* **Handle the `Error` arm.**

---

## 📚 See also

* [opened_orders_tickets](../2.%20Orders_History/opened_orders_tickets.md) · [on_opened_orders_profit](on_opened_orders_profit.md)
* **Back to:** [Streaming Methods — Overview](Streaming_Methods.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
