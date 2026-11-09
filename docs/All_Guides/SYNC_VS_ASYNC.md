# The Async Model — How RustMT4 Executes

> RustMT4 is **async all the way down**: every call returns a future you `.await`. This page explains the model and when each pattern fits.

---

## 🎯 Everything is async

The SDK runs on the **Tokio** runtime. There is no synchronous API — every method on `MT4Account`, `MT4Service` and `MT4Sugar` is `async` and must be `.await`ed inside a Tokio context:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)));
    sugar.connect_by_server_name(12_345_678, "password", "FreshForex-Demo", None, 120).await?;
    let a = sugar.get_account_info().await?;          // await a request/response call
    println!("Balance: {:.2} {}", a.balance, a.currency);
    Ok(())
}
```

---

## Two shapes of async call

| Shape | What it is | How you consume it |
|-------|-----------|--------------------|
| **Request / response** | `account_summary()`, `quote()`, `order_send()` | `let x = call(...).await?;` |
| **Server stream** | `on_symbol_tick()`, `stream_opened_profit()` | loop on `stream.message().await`, or a Sugar watcher's `mpsc::Receiver` |

### Request / response

One round trip, one value back. Sequential by default:

```rust
let q = sugar.last_quote("EURUSD").await?;
let lots = sugar.calc_lot_by_risk("EURUSD", 50.0, 20.0).await?;
```

### Streaming

A long-lived subscription that pushes frames. Read it in a loop, or let `MT4Sugar` forward it to a channel:

```rust
let mut rx = sugar.subscribe_quotes(vec!["EURUSD".into()]).await?;
while let Some(q) = rx.recv().await {
    println!("{} {}/{}", q.symbol, q.bid, q.ask);
}
```

See **[gRPC Streaming](GRPC_STREAM_MANAGEMENT.md)** for the full patterns.

---

## 🚀 Running work concurrently

Because calls are non-blocking, independent work overlaps. Quote several symbols at once with `tokio::join!`, or spawn a management task while the main flow continues:

```rust
// concurrent, not sequential
let (a, b) = tokio::join!(
    sugar.last_quote("EURUSD"),
    sugar.last_quote("GBPUSD"),
);
let (eur, gbp) = (a?, b?);

// background trailing loop
let handle = tokio::spawn(async move {
    loop {
        let _ = sugar.trail_stop(ticket, 15.0).await;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
});
```

> For many symbols, prefer a **batched call** (`quote_many`, one `stream_ticks` with all names) over many single calls.

---

## 💡 Rules of thumb

* **Always `.await`** — a future that isn't awaited does nothing.
* **One stream, many symbols** beats many single-symbol streams.
* **Don't block the runtime** — avoid `std::thread::sleep`; use `tokio::time::sleep`.
* **Waiters poll every 500 ms** (`wait_price`, `wait_filled`) — convenience, not low latency. For hot paths, subscribe to ticks.

---

**See also:** [gRPC Streaming](GRPC_STREAM_MANAGEMENT.md) · [MT4Sugar 8. Watchers & Waiters](../MT4Sugar/8.%20Watchers_Waiters.md)
