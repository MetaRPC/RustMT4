# On Symbol Tick (RECOMMENDED)

> Live server-push ticks for one or more symbols — the real-time price feed.

**API Information:**

* **Low-level API:** `MT4Account::on_symbol_tick(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.SubscriptionService`
* **RPC:** `OnSymbolTick(OnSymbolTickRequest) -> stream OnSymbolTickReply`
* **Returns:** `tonic::Streaming<OnSymbolTickReply>`

## 💬 Just the essentials

* **What it is.** A server stream that pushes a frame on every tick of the subscribed symbols.
* **Why you need it.** Live prices without polling `quote` — lower latency, less load.
* **When to use.** Dashboards, trailing logic, anything price-reactive. Drop the stream to unsubscribe.

---

## Method Signature

```rust
pub async fn on_symbol_tick(&self, request: pb::OnSymbolTickRequest) -> Result<Streaming<pb::OnSymbolTickReply>, MT4Error>
```

**Request / Reply:**

```protobuf
message OnSymbolTickRequest { repeated string symbol_names = 1; }
message OnSymbolTickReply { oneof response { OnSymbolTickData data = 1; Error error = 2; } }
message OnSymbolTickData {
  OnSymbolMqlTickInfo symbol_tick = 1;
  string terminal_instance_guid_id = 2;
}
```

## ⬇️ Input — `OnSymbolTickRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbol_names` | `Vec<String>` | Symbols to stream; each must be in Market Watch |

## ⬆️ Frame — `OnSymbolTickData` → `symbol_tick: OnSymbolMqlTickInfo`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbol` | `String` | Symbol this tick is for |
| `bid` / `ask` | `f64` | Current bid / ask |
| `last` | `f64` | Last deal price (where provided) |
| `volume` | `u64` | Volume for the last price |
| `time` / `time_utc` | `Option<Timestamp>` | Tick time (server / UTC) |
| `time_msc` / `time_msc_utc` | `i64` | Tick time in milliseconds (server / UTC) |

> `OnSymbolTickData` also carries `terminal_instance_guid_id` — the terminal that emitted the tick.

---

## 🧩 Notes & Tips

* **Reading frames:** loop on `stream.message().await`, match the `oneof`.
* **Unsubscribe** = drop the `Streaming` value.
* **`MT4Sugar::subscribe_quotes`** wraps this into an `mpsc::Receiver<Quote>` of friendly values.
* **Multiple symbols, one stream** — subscribe once with the whole list.

---

## 🔗 Usage Examples

### 1) Print ticks

```rust
use metarpc_mt4::mt4_term_api as pb;

let mut s = account.on_symbol_tick(pb::OnSymbolTickRequest {
    symbol_names: vec!["EURUSD".into(), "GBPUSD".into()],
}).await?;

while let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_symbol_tick_reply::Response::Data(d)) = reply.response {
        if let Some(t) = d.symbol_tick {
            println!("{:<8} {} / {}", t.symbol, t.bid, t.ask);
        }
    }
}
```

### 2) Stop after N ticks

```rust
use metarpc_mt4::mt4_term_api as pb;

let mut s = account.on_symbol_tick(pb::OnSymbolTickRequest { symbol_names: vec!["EURUSD".into()] }).await?;
let mut n = 0;
while let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_symbol_tick_reply::Response::Data(_)) = reply.response {
        n += 1;
        if n >= 10 { break; }   // drop(s) on scope exit = unsubscribe
    }
}
```

---

## ⚠️ Pitfalls

* **Select first.** A symbol not in Market Watch produces no ticks — `symbol_select` / `ensure_symbol`.
* **Handle the `Error` arm** — the server can push an error frame.
* **Quiet markets.** No ticks ≠ broken stream; it's just no price change.

---

## 📚 See also

* [quote](../3.%20Symbols_Quotes/quote.md) · [on_trade](on_trade.md) · [on_opened_orders_profit](on_opened_orders_profit.md)
* **Back to:** [Streaming Methods — Overview](Streaming_Methods.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
