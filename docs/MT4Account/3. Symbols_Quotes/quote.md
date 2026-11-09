# Quote (RECOMMENDED)

> The latest tick for one symbol — bid, ask, session high/low and the quote time.

**API Information:**

* **Low-level API:** `MT4Account::quote(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.MarketInfo`
* **RPC:** `Quote(QuoteRequest) -> QuoteReply`
* **Payload:** `QuoteData`

## 💬 Just the essentials

* **What it is.** A one-shot snapshot of the current price for a symbol.
* **Why you need it.** Spread checks, mid-price, "is the symbol live?", a price reference before sending an order.
* **When to use.** On demand. For a continuous feed use the `on_symbol_tick` stream instead of polling this.

---

## Method Signature

```rust
pub async fn quote(&self, request: pb::QuoteRequest) -> Result<pb::QuoteData, MT4Error>
```

**Request / Reply:**

```protobuf
message QuoteRequest { string symbol = 1; }
message QuoteData {
  string symbol = 1;
  double bid = 2;
  double ask = 3;
  google.protobuf.Timestamp date_time = 4;      // broker server time
  google.protobuf.Timestamp date_time_utc = 7;
  double high = 5;                              // session high
  double low  = 6;                              // session low
}
```

## ⬇️ Input — `QuoteRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbol` | `String` | Exact symbol name; must be in Market Watch (`symbol_select` first) |

## ⬆️ Output — `QuoteData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbol` | `String` | Echoed symbol |
| `bid` / `ask` | `f64` | Current bid / ask |
| `high` / `low` | `f64` | Session high / low |
| `date_time` | `Option<Timestamp>` | Quote time (server) |
| `date_time_utc` | `Option<Timestamp>` | Quote time (UTC) |

---

## 🧩 Notes & Tips

* **Spread** = `ask − bid`; divide by `pip_size` for spread in pips (`MT4Sugar::spread_pips`).
* **Mid price** = `(bid + ask) / 2` (`MT4Sugar::mid_price`).
* **Not selected → error.** If the symbol isn't in Market Watch, select it (or use `ensure_symbol`) first.

---

## 🔗 Usage Examples

### 1) Bid / ask / spread

```rust
use metarpc_mt4::mt4_term_api as pb;

let q = account.quote(pb::QuoteRequest { symbol: "EURUSD".into() }).await?;
println!("bid {} ask {} spread {:.5}", q.bid, q.ask, q.ask - q.bid);
```

### 2) Guard: skip trading on a wide spread

```rust
use metarpc_mt4::mt4_term_api as pb;

let q = account.quote(pb::QuoteRequest { symbol: "EURUSD".into() }).await?;
let spread_pips = (q.ask - q.bid) / (10.0 * 0.00001);
if spread_pips > 2.0 { println!("spread {spread_pips:.1} pips — skipping"); }
```

---

## ⚠️ Pitfalls

* **Snapshot, not a stream.** Re-call for a fresh price, or subscribe with `on_symbol_tick`.
* **Market closed.** Over the weekend `date_time` may be stale — check it before acting.

---

## 📚 See also

* [quote_many](quote_many.md) · [on_symbol_tick](../5.%20Streaming_Methods/on_symbol_tick.md) · [symbol_select](symbol_select.md)
* **Back to:** [Symbols & Quotes — Overview](Symbols_Quotes.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
