# On Trade

> Order/trade events — new, changed and closed orders pushed as they happen.

**API Information:**

* **Low-level API:** `MT4Account::on_trade(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.SubscriptionService`
* **RPC:** `OnTrade(OnTradeRequest) -> stream OnTradeReply`
* **Returns:** `tonic::Streaming<OnTradeReply>`

## 💬 Just the essentials

* **What it is.** A stream of trade-book change events for the account.
* **Why you need it.** React to fills, modifications and closes without re-polling `opened_orders`.
* **When to use.** Event-driven bots, trade copiers, audit logs.

---

## Method Signature

```rust
pub async fn on_trade(&self, request: pb::OnTradeRequest) -> Result<Streaming<pb::OnTradeReply>, MT4Error>
```

**Request / Reply:**

```protobuf
message OnTradeRequest {}   // no arguments — subscribes to this account's events
message OnTradeReply { oneof response { OnTradeData data = 1; Error error = 2; } }
message OnTradeData {
  Mt4SubEnumEventGroupType type = 1;
  OnTadeEventData event_data = 2;
  OnEventAccountInfo account_info = 3;
  string terminal_instance_guid_id = 4;
}
```

## ⬆️ Frame — `OnTradeData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `type` | `i32` | `Mt4SubEnumEventGroupType` — `OrderProfit` (0) or `OrderUpdate` (1) |
| `event_data` | `Option<OnTadeEventData>` | The changed orders (below) |
| `account_info` | `Option<OnEventAccountInfo>` | Balance / equity / margin snapshot at the event |
| `terminal_instance_guid_id` | `String` | Emitting terminal |

### `OnTadeEventData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `new_orders` | `Vec<OnTradeOrderInfo>` | Orders that just appeared |
| `updated_orders` | `Vec<OnTradeUpdatedOrderInfo>` | Each has `previous` + `current` snapshots |
| `removed_orders` | `Vec<OnTradeOrderInfo>` | Orders that were closed/deleted |
| `new_history_orders` | `Vec<OnTradeOrderInfo>` | Newly archived history orders |

### `OnTradeOrderInfo` (key fields)

`ticket` · `symbol` · `type` (`SubOrderOperationType` 0–5) · `lots` · `open_price` · `close_price` · `stop_loss` · `take_profit` · `order_profit` · `swap` · `commission` · `magic_number` · `comment` · `open_time`/`close_time` (+ `_utc`) · `is_history` · `account_login`.

### `Mt4SubEnumEventGroupType`

| Variant | Value |
|---------|-------|
| `OrderProfit` | 0 |
| `OrderUpdate` | 1 |

---

## 🧩 Notes & Tips

* **`updated_orders` carries both sides** — diff `previous` vs `current` to see what changed (SL moved, partially closed, …).
* **`removed_orders`** is your "position closed" signal.
* **No request fields** — the subscription is for the whole account.

---

## 🔗 Usage Examples

### 1) Log fills and closes

```rust
use metarpc_mt4::mt4_term_api as pb;

let mut s = account.on_trade(pb::OnTradeRequest {}).await?;
while let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_trade_reply::Response::Data(d)) = reply.response {
        if let Some(ev) = d.event_data {
            for o in &ev.new_orders     { println!("NEW   #{} {} {:.2}", o.ticket, o.symbol, o.lots); }
            for o in &ev.removed_orders { println!("CLOSE #{} P/L {:+.2}", o.ticket, o.order_profit); }
        }
    }
}
```

### 2) Detect SL moves

```rust
use metarpc_mt4::mt4_term_api as pb;

let mut s = account.on_trade(pb::OnTradeRequest {}).await?;
while let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_trade_reply::Response::Data(d)) = reply.response {
        if let Some(ev) = d.event_data {
            for u in &ev.updated_orders {
                if let (Some(p), Some(c)) = (&u.previous, &u.current) {
                    if p.stop_loss != c.stop_loss {
                        println!("#{} SL {} -> {}", c.ticket, p.stop_loss, c.stop_loss);
                    }
                }
            }
        }
    }
}
```

---

## ⚠️ Pitfalls

* **Handle the `Error` arm.**
* **Field is `type` →** access as `o.r#type` in Rust (raw identifier).
* **Snapshot semantics.** A single event may touch several lists — scan all four.

---

## 📚 See also

* [on_opened_orders_profit](on_opened_orders_profit.md) · [opened_orders](../2.%20Orders_History/opened_orders.md)
* **Back to:** [Streaming Methods — Overview](Streaming_Methods.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
