# Getting Started with RustMT4

> **Quick setup & overview** — start automating MetaTrader 4 with RustMT4 (Rust + gRPC) in minutes.

---

## 🎯 What is RustMT4?

**RustMT4** is a type-safe Rust client for **MetaTrader 4** terminals over **gRPC**. No C++ DLL wrappers — direct, async, reliable programmatic trading.

### Three-layer architecture

| Layer | Type | What you get |
|-------|------|--------------|
| **[MT4Account](../MT4Account/MT4Account.Master.Overview.md)** | low | Raw gRPC — build `…Request`, get `…Data`. Full control. |
| **[MT4Service](../MT4Service/MT4Service.Overview.md)** | mid | Friendly args (`&str` symbols, typed enums), clean returns. |
| **[MT4Sugar](../MT4Sugar/MT4Sugar.Master.Overview.md)** | high | One-liners: buy/sell by side, pip math, risk sizing, trailing, watchers. |

Each layer wraps the one below, so you can drop down a level any time you need more control.

---

## 📦 Installation

Not published on crates.io yet — clone the repo:

```bash
git clone https://github.com/MetaRPC/RustMT4.git
cd RustMT4
```

Build (requires the WinLibs GNU toolchain on Windows):

```bash
cargo build
```

---

## 🔑 API key & authentication

Connecting to the MetaRPC endpoint (`mt4.mrpc.pro:443`) needs an API key:

1. **Sign up** (free): [https://mrpc.pro/signup](https://mrpc.pro/signup).
2. **Generate a key** in your dashboard: [https://mrpc.pro/my](https://mrpc.pro/my) → **API Keys**.
3. **Provide it** via the `MRPC_API_KEY` environment variable (or `api_key` in `settings.json`). The SDK sends it on every call.

> The terminal **session id** is handled for you: `Connect` returns a GUID, and the SDK attaches it to all later requests and streams automatically — no `GetId` call needed.

---

## 🔌 Minimal connection example

Compose the three layers (here, the top one) and read your balance:

```rust
use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // host + port only; MRPC_API_KEY is read from the environment.
    let sugar = MT4Sugar::new(MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443)));

    // login, password, broker server name, optional terminal label, connect timeout (s)
    sugar.connect_by_server_name(12_345_678, "password", "FreshForex-Demo", None, 120).await?;

    let a = sugar.get_account_info().await?;
    println!("Balance: {:.2} {}  Equity: {:.2}", a.balance, a.currency, a.equity);

    sugar.disconnect(false).await?;
    Ok(())
}
```

Prefer the raw layer? `MT4Account::new("mt4.mrpc.pro", 443)` then `account.account_summary().await?` (returns `AccountSummaryData` with `account_balance`, `account_equity`, …).

---

## 🗺️ Documentation road map

1. 🚀 **[Your First Project](Your_First_Project.md)** — a working bot in 10 minutes.
2. 🗺️ **[Project Map](PROJECT_MAP.md)** — the three layers and how they interact.
3. 🐣 **[MT4 for Beginners](MT4_For_Beginners.md)** — terminal + demo account setup.
4. 📖 **[Glossary](GLOSSARY.md)** — MT4 and trading terminology.
5. 📡 **[gRPC Streaming](GRPC_STREAM_MANAGEMENT.md)** — ticks, trades, profit, connection state.
6. 🔢 **[ENUMs Reference](ENUMS_USAGE_REFERENCE.md)** · 📊 **[Return Codes](RETURN_CODES_REFERENCE.md)** · ⚙️ **[Async Model](SYNC_VS_ASYNC.md)**

Then the [API Reference](../MT4Account/MT4Account.Master.Overview.md) for every method.
