# gRPC Streaming & Subscription Management

> How to subscribe to live data in RustMT4, consume it cleanly, and shut it down — the Rust/Tokio way.

---

## The model in one line

A streaming call returns a **`tonic::Streaming<…Reply>`**. You read frames with `stream.message().await` in a loop. **To unsubscribe, drop the stream** — there's no cancellation token or `finally` cleanup to remember. When the value goes out of scope, the subscription ends.

```rust
let mut stream = account.on_symbol_tick(pb::OnSymbolTickRequest {
    symbol_names: vec!["EURUSD".into()],
}).await?;

while let Ok(Some(reply)) = stream.message().await {
    if let Some(pb::on_symbol_tick_reply::Response::Data(d)) = reply.response {
        if let Some(t) = d.symbol_tick { println!("{} {}/{}", t.symbol, t.bid, t.ask); }
    }
}
// stream dropped here → unsubscribed
```

---

## The five streams

On **[`MT4Account`](../MT4Account/5.%20Streaming_Methods/Streaming_Methods.Overview.md)** (raw `Streaming<…Reply>`):

| Method | Pushes |
|--------|--------|
| `on_symbol_tick(req)` | live ticks for the given symbols |
| `on_trade(req)` | order/trade events (new / updated / removed) |
| `on_opened_orders_tickets(req)` | the open-order ticket list on a timer |
| `on_opened_orders_profit(req)` | per-order profit + account snapshot on a timer |
| `on_connect_state(req)` | terminal connection-state events |

On **[`MT4Service`](../MT4Service/5.%20Streaming_Methods.md)** the same streams are renamed `stream_ticks`, `stream_trades`, `stream_opened_tickets`, `stream_opened_profit`, `stream_connect_state` (friendly args, same `Streaming<…Reply>`).

On **[`MT4Sugar`](../MT4Sugar/8.%20Watchers_Waiters.md)** three of them are wrapped into `mpsc` channels of friendly values — see below.

---

## Two ways to consume

### A) Raw loop (full control)

Match the reply's `oneof { Data, Error }` yourself:

```rust
use metarpc_mt4::mt4_term_api as pb;

let mut stream = account.on_symbol_tick(pb::OnSymbolTickRequest { symbol_names: vec!["EURUSD".into()] }).await?;
while let Ok(Some(reply)) = stream.message().await {
    match reply.response {
        Some(pb::on_symbol_tick_reply::Response::Data(d)) => { /* handle d */ }
        Some(pb::on_symbol_tick_reply::Response::Error(e)) => { eprintln!("stream error: {e:?}"); break; }
        None => break,
    }
}
```

### B) Sugar channel (simplest)

`MT4Sugar` spawns the loop for you and forwards values onto a `tokio::mpsc` channel. The background task ends on the first stream error **or when you drop the receiver**:

```rust
let mut rx = sugar.subscribe_quotes(vec!["EURUSD".into(), "GBPUSD".into()]).await?;
while let Some(q) = rx.recv().await {
    println!("{} {}/{}", q.symbol, q.bid, q.ask);
}
```

Also: `sugar.watch_profit(1000)` → `OnOpenedOrdersProfitData`, `sugar.watch_tickets(1000)` → `OnOpenedOrdersTicketsData`.

---

## Bounded consumption

Streams run until you stop them. Common ways to bound them:

### Stop after N frames

```rust
let mut stream = account.on_symbol_tick(req).await?;
let mut n = 0;
while let Ok(Some(reply)) = stream.message().await {
    if let Some(pb::on_symbol_tick_reply::Response::Data(_)) = reply.response {
        n += 1;
        if n >= 10 { break; }   // drop(stream) on scope exit = unsubscribe
    }
}
```

### Stop after a timeout

Wrap the whole loop in `tokio::time::timeout`:

```rust
use std::time::Duration;

let _ = tokio::time::timeout(Duration::from_secs(10), async {
    while let Ok(Some(reply)) = stream.message().await {
        // handle frames
    }
}).await;   // elapsed → future dropped → stream dropped → unsubscribed
```

### Stop on a condition

```rust
while let Ok(Some(reply)) = stream.message().await {
    if let Some(pb::on_symbol_tick_reply::Response::Data(d)) = reply.response {
        if let Some(t) = d.symbol_tick {
            if t.bid >= 1.0950 { println!("target hit"); break; }
        }
    }
}
```

> For "wait until price X" or "wait until filled", `MT4Sugar` already has `wait_price` / `wait_filled` / `wait_closed`.

---

## Filtering by enum

Stream frames carry the same enums as the rest of the API (see [ENUMs Reference](ENUMS_USAGE_REFERENCE.md)). Example — react only to closes on `on_trade`:

```rust
use metarpc_mt4::mt4_term_api as pb;

let mut s = account.on_trade(pb::OnTradeRequest {}).await?;
while let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_trade_reply::Response::Data(d)) = reply.response {
        if let Some(ev) = d.event_data {
            for o in &ev.removed_orders {           // order closed/deleted
                println!("CLOSE #{} {} P/L {:+.2}", o.ticket, o.symbol, o.order_profit);
            }
        }
    }
}
```

> `OnTradeData.type` / `OnOpenedOrdersProfitData.type` are the raw identifier `r#type` in Rust (`Mt4SubEnumEventGroupType`); order rows carry `SubOrderOperationType` (0 Buy … 5 SellStop).

---

## Multiple concurrent streams

Each stream is independent. Run several at once by spawning a task per stream, and coordinate shutdown with a channel or `tokio::select!`:

```rust
use std::time::Duration;

// ticks in the background
let mut rx = sugar.subscribe_quotes(vec!["EURUSD".into()]).await?;
let ticks = tokio::spawn(async move {
    while let Some(q) = rx.recv().await { println!("tick {} {}", q.symbol, q.bid); }
});

// profit in the foreground, with a deadline
let mut prof = sugar.watch_profit(1000).await?;
let _ = tokio::time::timeout(Duration::from_secs(30), async {
    while let Some(d) = prof.recv().await {
        if let Some(a) = d.account_info { println!("equity {:.2}", a.equity); }
    }
}).await;

ticks.abort();   // stop the background task (drops its receiver → unsubscribes)
```

`tokio::select!` lets one task react to whichever stream fires first:

```rust
tokio::select! {
    Some(q) = rx.recv()   => println!("tick {}", q.bid),
    Some(d) = prof.recv() => println!("profit frame"),
}
```

---

## Reconnection

Pair streams with `on_connect_state` to know when the broker link drops, and re-subscribe after a reconnect:

```rust
let mut cs = account.on_connect_state(pb::OnConnectStateRequest { terminal_ids: vec![] }).await?;
while let Ok(Some(reply)) = cs.message().await {
    if let Some(pb::on_connect_state_reply::Response::Data(d)) = reply.response {
        if !d.is_connected { /* pause trading; re-open data streams after reconnect */ }
    }
}
```

The unary `call` wrapper already auto-retries transient transport errors and terminal-not-found; streams, being long-lived, are re-opened by your own logic.

---

## ✅ Do / ❌ Don't

**✅ Do**
- Drop the stream (or receiver) to unsubscribe — it's deterministic.
- Bound streams (`break` on N / condition, or `tokio::time::timeout`).
- Handle the `Error` arm of the `oneof`.
- Prefer **one stream with many symbols** over many single-symbol streams.
- Use Sugar watchers (`subscribe_quotes` / `watch_*`) for the common cases.

**❌ Don't**
- Don't `std::thread::sleep` inside an async task — use `tokio::time::sleep`.
- Don't poll `quote` in a tight loop when a tick stream exists.
- Don't assume "no frames" means "broken" — a quiet market simply sends nothing.

---

**See also:** [MT4Account 5. Streaming Methods](../MT4Account/5.%20Streaming_Methods/Streaming_Methods.Overview.md) · [MT4Sugar 8. Watchers & Waiters](../MT4Sugar/8.%20Watchers_Waiters.md) · [The Async Model](SYNC_VS_ASYNC.md)
