# Orders History

> Closed / cancelled orders within a time window — paged, with sort.

**API Information:**

* **Low-level API:** `MT4Account::orders_history(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.AccountHelper`
* **RPC:** `OrdersHistory(OrdersHistoryRequest) -> OrdersHistoryReply`
* **Payload:** `OrdersHistoryData` → `orders_info: Vec<HistoryOrderInfo>`, `total_count`, `next_page_token`

## 💬 Just the essentials

* **What it is.** The account's closed-order history over `[from, to]`.
* **Why you need it.** Trade journals, realized-P/L reports.
* **When to use.** Statements and performance review.

---

## Method Signature

```rust
pub async fn orders_history(&self, request: pb::OrdersHistoryRequest) -> Result<pb::OrdersHistoryData, MT4Error>
```

**Request (key fields):**

```protobuf
message OrdersHistoryRequest {
  EnumOrderHistorySortType input_sort_mode = 1;
  google.protobuf.Timestamp input_from = 2;      // server time (or input_from_utc)
  google.protobuf.Timestamp input_to = 4;        // server time (or input_to_utc)
  optional int32 page_number = 6;
  optional int32 items_per_page = 7;
  string page_token = 8;  int32 page_size = 9;
}
```

### `EnumOrderHistorySortType`

| Variant | Value | | Variant | Value |
|---------|-------|---|---------|-------|
| `HistorySortByOpenTimeAsc` | 0 | | `HistorySortByCloseTimeDesc` | 3 |
| `HistorySortByOpenTimeDesc` | 1 | | `HistorySortByOrderTicketIdAsc` | 4 |
| `HistorySortByCloseTimeAsc` | 2 | | `HistorySortByOrderTicketIdDesc` | 5 |

## ⬆️ Output — `HistoryOrderInfo` (per closed order)

| Field | Rust type | Description |
|-------|-----------|-------------|
| `ticket` | `i32` | Order ticket |
| `symbol` | `String` | Symbol |
| `order_type` | `i32` | 0 Buy … 5 SellStop |
| `lots` | `f64` | Volume |
| `open_price` / `close_price` | `f64` | Open / close price |
| `open_time` / `close_time` (+ `_utc`) | `Option<Timestamp>` | Times |
| `stop_loss` / `take_profit` | `f64` | SL / TP |
| `profit` / `swap` / `commision` | `f64` | Result + costs |
| `magic_number` | `i32` | EA magic |
| `comment` | `String` | Comment |

`total_count` and `next_page_token` support paging.

---

## 🔗 Usage Examples

### 1) Last 7 days, newest first

```rust
use metarpc_mt4::mt4_term_api as pb;
fn ts(s: i64) -> prost_types::Timestamp { prost_types::Timestamp { seconds: s, nanos: 0 } }

let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
let d = account.orders_history(pb::OrdersHistoryRequest {
    input_sort_mode: pb::EnumOrderHistorySortType::HistorySortByCloseTimeDesc as i32,
    input_from: Some(ts(now - 7 * 86_400)),
    input_to: Some(ts(now)),
    ..Default::default()
}).await?;
println!("{} rows (total {})", d.orders_info.len(), d.total_count);
```

### 2) Realized P/L over the window

```rust
let net: f64 = d.orders_info.iter().map(|o| o.profit + o.swap + o.commision).sum();
println!("net realized {net:+.2}");
```

### 3) Win rate

```rust
let wins = d.orders_info.iter().filter(|o| o.profit > 0.0).count();
if !d.orders_info.is_empty() { println!("win rate {:.1}%", wins as f64 / d.orders_info.len() as f64 * 100.0); }
```

---

## ⚠️ Pitfalls

* **Server time.** `input_from`/`input_to` are broker server time — mind the UTC offset (or use the `_utc` fields).
* **Paging.** Large windows return `total_count` > rows; page with `page_number`/`items_per_page` (or `page_token`).
* **Costs.** `profit` excludes swap/commission — sum them for net.

---

## 📚 See also

* [opened_orders](opened_orders.md) · [trade_stats](../1.%20Account_Information/trade_stats.md)
* **Back to:** [Orders & History — Overview](Orders_History.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
