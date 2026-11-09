# Quote Many

> Latest ticks for several symbols in one round-trip.

**API Information:**

* **Low-level API:** `MT4Account::quote_many(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.MarketInfo`
* **RPC:** `QuoteMany(QuoteManyRequest) -> QuoteManyReply`
* **Payload:** `QuoteManyData` → `quotes: Vec<QuoteData>`

## 💬 Just the essentials

* **What it is.** `quote`, batched — one call for a basket of symbols.
* **Why you need it.** A watchlist, a correlation dashboard, or pricing a multi-symbol strategy without N round-trips.
* **When to use.** Whenever you need more than one price at once.

---

## Method Signature

```rust
pub async fn quote_many(&self, request: pb::QuoteManyRequest) -> Result<pb::QuoteManyData, MT4Error>
```

**Request / Reply:**

```protobuf
message QuoteManyRequest { repeated string symbols = 1; }
message QuoteManyData    { repeated QuoteData quotes = 1; }
```

## ⬇️ Input — `QuoteManyRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbols` | `Vec<String>` | Exact symbol names; each should be in Market Watch |

## ⬆️ Output — `QuoteManyData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `quotes` | `Vec<QuoteData>` | One `QuoteData` per symbol (same fields as [`quote`](quote.md)) |

Each `QuoteData`: `symbol`, `bid`, `ask`, `high`, `low`, `date_time`, `date_time_utc`.

---

## 🧩 Notes & Tips

* **Order isn't guaranteed.** Match results by `q.symbol`, don't assume request order.
* **Missing symbols.** A symbol not in Market Watch may be absent from the reply — select it first.

---

## 🔗 Usage Examples

### 1) Price a watchlist

```rust
use metarpc_mt4::mt4_term_api as pb;

let d = account.quote_many(pb::QuoteManyRequest {
    symbols: vec!["EURUSD".into(), "GBPUSD".into(), "USDJPY".into()],
}).await?;
for q in &d.quotes {
    println!("{:<8} {} / {}", q.symbol, q.bid, q.ask);
}
```

### 2) Build a symbol → mid-price map

```rust
use std::collections::HashMap;
use metarpc_mt4::mt4_term_api as pb;

let d = account.quote_many(pb::QuoteManyRequest { symbols: vec!["EURUSD".into(), "GBPUSD".into()] }).await?;
let mids: HashMap<_, _> = d.quotes.iter().map(|q| (q.symbol.clone(), (q.bid + q.ask) / 2.0)).collect();
println!("{mids:?}");
```

---

## ⚠️ Pitfalls

* **Match by name, not index.**
* **Snapshot.** For live multi-symbol prices, open one `on_symbol_tick` stream with all the names.

---

## 📚 See also

* [quote](quote.md) · [on_symbol_tick](../5.%20Streaming_Methods/on_symbol_tick.md)
* **Back to:** [Symbols & Quotes — Overview](Symbols_Quotes.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
