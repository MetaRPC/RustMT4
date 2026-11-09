# On Opened Orders Profit

> Account equity plus per-order floating profit, pushed on a timer — a live P/L feed.

**API Information:**

* **Low-level API:** `MT4Account::on_opened_orders_profit(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.SubscriptionService`
* **RPC:** `OnOpenedOrdersProfit(OnOpenedOrdersProfitRequest) -> stream OnOpenedOrdersProfitReply`
* **Returns:** `tonic::Streaming<OnOpenedOrdersProfitReply>`

## 💬 Just the essentials

* **What it is.** Every `timer_period_milliseconds`, the server pushes each open order's profit plus an account snapshot.
* **Why you need it.** A live P/L dashboard, equity-based risk guards, trailing on floating profit.
* **When to use.** Monitoring open exposure in real time.

---

## Method Signature

```rust
pub async fn on_opened_orders_profit(&self, request: pb::OnOpenedOrdersProfitRequest) -> Result<Streaming<pb::OnOpenedOrdersProfitReply>, MT4Error>
```

**Request / Reply:**

```protobuf
message OnOpenedOrdersProfitRequest { int32 timer_period_milliseconds = 1; }
message OnOpenedOrdersProfitReply { oneof response { OnOpenedOrdersProfitData data = 1; Error error = 2; } }
message OnOpenedOrdersProfitData {
  Mt4SubEnumEventGroupType type = 1;
  repeated OnOpenedOrdersProfitOrderInfo opened_orders_with_profit_updated = 2;
  OnEventAccountInfo account_info = 3;
  string terminal_instance_guid_id = 4;
}
```

## ⬇️ Input — `OnOpenedOrdersProfitRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `timer_period_milliseconds` | `i32` | Push cadence in ms |

## ⬆️ Frame — `OnOpenedOrdersProfitData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `type` | `i32` | `Mt4SubEnumEventGroupType` (`OrderProfit` 0 / `OrderUpdate` 1) |
| `opened_orders_with_profit_updated` | `Vec<OnOpenedOrdersProfitOrderInfo>` | Per-order profit rows (below) |
| `account_info` | `Option<OnEventAccountInfo>` | Equity snapshot (below) |
| `terminal_instance_guid_id` | `String` | Emitting terminal |

### `OnOpenedOrdersProfitOrderInfo` (per order)

`ticket` · `symbol` · `type` (`SubOrderOperationType`) · `lots` · `open_price` · `stop_loss` · `take_profit` · `order_profit` · `swap` · `commission` · `magic_number` · `comment` · `open_time` (+ `_utc`).

### `OnEventAccountInfo`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `balance` | `f64` | Account balance |
| `equity` | `f64` | Balance + floating P/L |
| `margin` / `free_margin` | `f64` | Used / free margin |
| `margin_level` | `f64` | Equity / margin × 100 |
| `profit` | `f64` | Total floating P/L |
| `credit` | `f64` | Credit |
| `login` | `i64` | Account login |

---

## 🧩 Notes & Tips

* **One frame = whole picture** — account snapshot + every order's profit together.
* **Net per order** = `order_profit + swap + commission`.
* **Equity-based guards:** watch `account_info.equity` / `margin_level` and act on thresholds.
* **`MT4Sugar::watch_profit`** wraps this into an mpsc channel.

---

## 🔗 Usage Examples

### 1) Live equity + total profit

```rust
use metarpc_mt4::mt4_term_api as pb;

let mut s = account.on_opened_orders_profit(pb::OnOpenedOrdersProfitRequest {
    timer_period_milliseconds: 1000,
}).await?;

while let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_opened_orders_profit_reply::Response::Data(d)) = reply.response {
        if let Some(a) = d.account_info {
            println!("equity {:.2}  floating {:+.2}  margin_level {:.1}%",
                a.equity, a.profit, a.margin_level);
        }
    }
}
```

### 2) Close-all on a drawdown limit

```rust
use metarpc_mt4::mt4_term_api as pb;

let mut s = account.on_opened_orders_profit(pb::OnOpenedOrdersProfitRequest { timer_period_milliseconds: 1000 }).await?;
while let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_opened_orders_profit_reply::Response::Data(d)) = reply.response {
        if let Some(a) = d.account_info {
            if a.profit < -100.0 {
                println!("drawdown hit {:+.2} — flatten now", a.profit);
                break;   // then close positions via order_close_delete
            }
        }
    }
}
```

---

## ⚠️ Pitfalls

* **`type` → `d.r#type`** in Rust (raw identifier).
* **Cadence vs load** — 1000 ms is a good default; sub-200 ms is wasteful.
* **Handle the `Error` arm.**

---

## 📚 See also

* [account_summary](../1.%20Account_Information/account_summary.md) · [on_trade](on_trade.md) · [opened_orders](../2.%20Orders_History/opened_orders.md)
* **Back to:** [Streaming Methods — Overview](Streaming_Methods.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
