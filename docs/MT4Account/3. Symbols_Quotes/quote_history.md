# Quote History

> OHLC bars for a symbol / timeframe over a time range — the input for backtests and indicators.

**API Information:**

* **Low-level API:** `MT4Account::quote_history(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.MarketInfo`
* **RPC:** `QuoteHistory(QuoteHistoryRequest) -> QuoteHistoryReply`
* **Payload:** `QuoteHistoryData` → `historical_quotes: Vec<HistoryQuote>`

## 💬 Just the essentials

* **What it is.** Historical candles (bars), not ticks.
* **Why you need it.** Compute indicators, backtest, draw charts, seed a strategy's warm-up window.
* **When to use.** On demand; range-limited by what the broker keeps on the server.

---

## Method Signature

```rust
pub async fn quote_history(&self, request: pb::QuoteHistoryRequest) -> Result<pb::QuoteHistoryData, MT4Error>
```

**Request / Reply:**

```protobuf
message QuoteHistoryRequest {
  string symbol = 1;
  EnumQuoteHistoryTimeframe timeframe = 2;
  google.protobuf.Timestamp from_time = 3;      // server time  — send from_time OR from_time_utc
  google.protobuf.Timestamp from_time_utc = 5;
  google.protobuf.Timestamp to_time = 4;        // server time  — send to_time   OR to_time_utc
  google.protobuf.Timestamp to_time_utc = 6;
}
message QuoteHistoryData { repeated HistoryQuote historical_quotes = 1; }
```

## ⬇️ Input — `QuoteHistoryRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbol` | `String` | Exact symbol name |
| `timeframe` | `i32` | `EnumQuoteHistoryTimeframe` as `i32` (see below) |
| `from_time` / `from_time_utc` | `Option<Timestamp>` | Range start — send **one** of the two |
| `to_time` / `to_time_utc` | `Option<Timestamp>` | Range end — send **one** of the two |

### `EnumQuoteHistoryTimeframe`

| Variant | Value | Period |
|---------|-------|--------|
| `QhPeriodM1` | 0 | 1 minute |
| `QhPeriodM5` | 1 | 5 minutes |
| `QhPeriodM15` | 2 | 15 minutes |
| `QhPeriodM30` | 3 | 30 minutes |
| `QhPeriodH1` | 4 | 1 hour |
| `QhPeriodH4` | 5 | 4 hours |
| `QhPeriodD1` | 6 | 1 day |
| `QhPeriodW1` | 7 | 1 week |
| `QhPeriodMn1` | 8 | 1 month |

> ⚠️ This is an **ordinal enum (0–8), not MQL minute counts.** `H1` is `4`, not `60`.

## ⬆️ Output — `HistoryQuote` (per bar)

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbol` | `String` | Symbol |
| `index` | `i32` | Bar index in the returned series |
| `time` / `time_utc` | `Option<Timestamp>` | Bar open time (server / UTC) |
| `open` / `high` / `low` / `close` | `f64` | OHLC |
| `tick_volume` | `i64` | Tick count in the bar |
| `real_volume` | `i64` | Real volume (0 on most FX feeds) |
| `spread` | `i32` | Recorded spread, in points |

---

## 🧩 Notes & Tips

* **Pick UTC or server, not both** for each bound — sending both is rejected.
* **Broker retention limits the range.** Very old bars may simply not come back.
* **`MT4Sugar::bars`** returns friendly `Bar` structs built on this call.

---

## 🔗 Usage Examples

### 1) Last 100 H1 bars

```rust
use metarpc_mt4::mt4_term_api as pb;
use prost_types::Timestamp;

let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
let d = account.quote_history(pb::QuoteHistoryRequest {
    symbol: "EURUSD".into(),
    timeframe: pb::EnumQuoteHistoryTimeframe::QhPeriodH1 as i32,
    from_time_utc: Some(Timestamp { seconds: now - 100 * 3600, nanos: 0 }),
    to_time_utc:   Some(Timestamp { seconds: now, nanos: 0 }),
    ..Default::default()
}).await?;
println!("{} bars", d.historical_quotes.len());
```

### 2) Highest high in the window

```rust
let hi = d.historical_quotes.iter().map(|b| b.high).fold(f64::MIN, f64::max);
println!("range high = {hi}");
```

---

## ⚠️ Pitfalls

* **Enum, not minutes** — `timeframe: 60` is wrong (that's out of range); use `QhPeriodH1 as i32` = 4.
* **Both bounds set in the same zone pair** — use `_utc` fields together, or the server fields together.
* **Empty series** = range outside broker retention or market never open there.

---

## 📚 See also

* [quote](quote.md) · [server_timezone](server_timezone.md)
* **Back to:** [Symbols & Quotes — Overview](Symbols_Quotes.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
