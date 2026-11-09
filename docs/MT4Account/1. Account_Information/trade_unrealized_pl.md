# Trade Unrealized P/L

> The maximum and minimum unrealized P/L each open order reached, with the times — the MFE / MAE per position.

**API Information:**

* **Low-level API:** `MT4Account::trade_unrealized_pl(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.TradeStats`
* **RPC:** `TradeUnrealizedPL(TradeUnrealizedPlRequest) -> …Reply`
* **Payload:** `TradeUnrealizedPlData` → `extrema: Vec<TradeUnrealizedExtremaData>`

## 💬 Just the essentials

* **What it is.** Per open order, the best (MFE) and worst (MAE) floating P/L it hit, with timestamps.
* **Why you need it.** Trade-quality analysis: how far trades ran in your favour / against you.
* **When to use.** Reviewing open positions, tuning SL/TP.

---

## Method Signature

```rust
pub async fn trade_unrealized_pl(&self, request: pb::TradeUnrealizedPlRequest) -> Result<pb::TradeUnrealizedPlData, MT4Error>
```

**Request:**

```protobuf
message TradeUnrealizedPlRequest { optional string from = 1; optional string from_utc = 2; }
```

## ⬆️ Output — `TradeUnrealizedExtremaData` (per order)

| Field | Type | Description |
|-------|------|-------------|
| `ticket` | `i64` | Order ticket |
| `symbol` | `String` | Symbol |
| `max_unrealized_pl` | `f64` | Best floating P/L reached (MFE) |
| `max_unrealized_time` / `_utc` | `Option<Timestamp>` | When the max was hit |
| `min_unrealized_pl` | `f64` | Worst floating P/L reached (MAE) |
| `min_unrealized_time` / `_utc` | `Option<Timestamp>` | When the min was hit |

---

## 🔗 Usage Examples

### 1) MFE / MAE per open order

```rust
use metarpc_mt4::mt4_term_api as pb;

let d = account.trade_unrealized_pl(pb::TradeUnrealizedPlRequest::default()).await?;
for e in &d.extrema {
    println!("#{} {}  MFE {:+.2}  MAE {:+.2}", e.ticket, e.symbol, e.max_unrealized_pl, e.min_unrealized_pl);
}
```

### 2) Orders that gave back a big winner

```rust
use metarpc_mt4::mt4_term_api as pb;
let d = account.trade_unrealized_pl(pb::TradeUnrealizedPlRequest::default()).await?;
for e in d.extrema.iter().filter(|e| e.max_unrealized_pl > 50.0) {
    println!("#{} peaked at {:+.2}", e.ticket, e.max_unrealized_pl);
}
```

---

## ⚠️ Pitfalls

* **Open orders only.** This is about currently open positions' extrema.
* **Server time** in the `*_time` fields; `*_utc` for UTC.

---

## 📚 See also

* [trade_stats](trade_stats.md) · [opened_orders](../2.%20Orders_History/opened_orders.md)
* **Back to:** [Account Information — Overview](Account_Information.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
