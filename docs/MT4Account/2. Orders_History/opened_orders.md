# Opened Orders (RECOMMENDED)

> Everything currently live: open market orders **and** pending orders, with prices, SL/TP, profit and magic.

**API Information:**

* **Low-level API:** `MT4Account::opened_orders(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.AccountHelper`
* **RPC:** `OpenedOrders(OpenedOrdersRequest) -> OpenedOrdersReply`
* **Payload:** `OpenedOrdersData` → `order_infos: Vec<OpenedOrderInfo>`

## 💬 Just the essentials

* **What it is.** The full snapshot of your open book — market orders and pendings together.
* **Why you need it.** Live P/L, SL/TP and volume per order — everything to manage trades.
* **When to use.** Any position/order dashboard or management loop. This is what `MT4Sugar`'s position helpers wrap.

---

## Method Signature

```rust
pub async fn opened_orders(&self, request: pb::OpenedOrdersRequest) -> Result<pb::OpenedOrdersData, MT4Error>
```

**Request / Reply:**

```protobuf
message OpenedOrdersRequest { EnumOpenedOrderSortType sort_type = 1; }
message OpenedOrdersData { repeated OpenedOrderInfo order_infos = 1; }
```

### `EnumOpenedOrderSortType`

| Variant | Value |
|---------|-------|
| `SortByOpenTimeAsc` | 0 |
| `SortByOpenTimeDesc` | 1 |
| `SortByOrderTicketIdAsc` | 2 |
| `SortByOrderTicketIdDesc` | 3 |

## ⬆️ Output — `OpenedOrderInfo` (per order)

| Field | Rust type | Description |
|-------|-----------|-------------|
| `ticket` | `i32` | Order ticket |
| `symbol` | `String` | Symbol |
| `order_type` | `i32` | 0 Buy, 1 Sell, 2 BuyLimit, 3 SellLimit, 4 BuyStop, 5 SellStop |
| `lots` | `f64` | Volume |
| `open_price` | `f64` | Open / trigger price |
| `stop_loss` / `take_profit` | `f64` | SL / TP (0 if unset) |
| `profit` | `f64` | Floating P/L |
| `swap` | `f64` | Accrued swap |
| `commision` | `f64` | Commission (proto spelling) |
| `magic_number` | `i32` | EA magic |
| `comment` | `String` | Comment |
| `open_time` / `open_time_utc` | `Option<Timestamp>` | Open time (server / UTC) |
| `expiration_time` | `Option<Timestamp>` | Pending-order expiry |

---

## 🧩 Notes & Tips

* **One list, two kinds.** Market orders (`order_type` 0/1) are the "positions"; 2–5 are pendings.
* **Sum `profit + swap + commision`** for the true net floating result.
* **Snapshot.** For live updates, stream `on_opened_orders_profit`.

---

## 🔗 Usage Examples

### 1) Print open orders with P/L

```rust
use metarpc_mt4::mt4_term_api as pb;

let d = account.opened_orders(pb::OpenedOrdersRequest { sort_type: pb::EnumOpenedOrderSortType::SortByOpenTimeAsc as i32 }).await?;
for o in &d.order_infos {
    let side = match o.order_type { 0 => "BUY", 1 => "SELL", _ => "PENDING" };
    println!("#{} {side} {} {:.2} lot  P/L {:+.2}", o.ticket, o.symbol, o.lots, o.profit);
}
```

### 2) Market positions only

```rust
use metarpc_mt4::mt4_term_api as pb;
let d = account.opened_orders(pb::OpenedOrdersRequest { sort_type: 0 }).await?;
let positions: Vec<_> = d.order_infos.iter().filter(|o| o.order_type <= 1).collect();
println!("{} open positions", positions.len());
```

### 3) Total floating P/L

```rust
use metarpc_mt4::mt4_term_api as pb;
let d = account.opened_orders(pb::OpenedOrdersRequest { sort_type: 0 }).await?;
let total: f64 = d.order_infos.iter().map(|o| o.profit + o.swap + o.commision).sum();
println!("floating P/L {total:+.2}");
```

---

## ⚠️ Pitfalls

* **`sort_type` is required** — pass the enum as `i32`.
* **Pendings mixed in** — filter by `order_type` when you mean "positions".
* **Snapshot.** Re-query for fresh P/L.

---

## 📚 See also

* [opened_orders_tickets](opened_orders_tickets.md) · [orders_history](orders_history.md) · [on_opened_orders_profit](../5.%20Streaming_Methods/on_opened_orders_profit.md)
* **Back to:** [Orders & History — Overview](Orders_History.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
