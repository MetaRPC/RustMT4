# Your First Project in 10 Minutes

> **Hands-on quick start** — connect to MetaTrader 4 and place a demo order with RustMT4, from scratch.

---

## Step 0 — API key

Connecting to `mt4.mrpc.pro:443` needs a key:

1. Register (free): [https://mrpc.pro/signup](https://mrpc.pro/signup).
2. Generate a token: [https://mrpc.pro/my](https://mrpc.pro/my).
3. Export it: `set MRPC_API_KEY=...` (Windows) / `export MRPC_API_KEY=...` (bash).

---

## Step 1 — Get the SDK

```bash
git clone https://github.com/MetaRPC/RustMT4.git
cd RustMT4
```

It already builds as a library + a set of runnable examples — the fastest path is to run one.

---

## Step 2 — The code

The three-layer client. `MT4Sugar` gives you the shortest path to a trade:

```rust
use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // host + port; MRPC_API_KEY comes from the environment.
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)))
        .set_defaults(1001, 20, "my-first-bot");   // magic, slippage (points), comment

    // login, password, broker server name, optional label, timeout (s)
    sugar.connect_by_server_name(12_345_678, "password", "FreshForex-Demo", None, 120).await?;
    println!("Connected to MT4!");

    let a = sugar.get_account_info().await?;
    println!("Balance: {:.2} {}", a.balance, a.currency);

    // make sure the symbol is live, then buy 0.01 lot at market
    sugar.ensure_symbol("EURUSD", std::time::Duration::from_secs(10)).await?;
    let r = sugar.buy_market("EURUSD", 0.01, None, None).await?;
    println!("Order sent! Ticket #{} @ {}", r.ticket, r.price);

    // 20-pip stop, 40-pip target (direction-aware, by pips from entry)
    sugar.modify_sl_tp_by_pips(r.ticket, Some(20.0), Some(40.0)).await?;

    sugar.disconnect(false).await?;
    Ok(())
}
```

> **Demo accounts only** while you learn — `buy_market` sends a real order on whatever account you connect.

---

## Step 3 — Run the bundled example

The repo ships the same flow as [`examples/quickstart.rs`](https://github.com/MetaRPC/RustMT4/blob/main/examples/quickstart.rs):

```bash
# arguments: <login> <password> <server name> [api_key]
cargo run --example quickstart -- 12345678 password FreshForex-Demo your_api_key_here
```

Or launch the **interactive menu** that lists every example:

```bash
cargo run --example menu
```

Credentials can also come from `examples/0_common/settings.json` (or `MRPC_*` env vars) — see the [Settings loader](PROJECT_MAP.md).

---

## 🚀 Next steps

* Stream live prices → **[gRPC Streaming](GRPC_STREAM_MANAGEMENT.md)**.
* Every terminal method → **[MT4Account API Reference](../MT4Account/MT4Account.Master.Overview.md)**.
* Risk sizing, trailing, one-liners → **[MT4Sugar](../MT4Sugar/MT4Sugar.Master.Overview.md)**.
* Full lifecycle templates → **[Strategies](../Strategies/Strategies.Master.Overview.md)**.
