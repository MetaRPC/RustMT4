# RustMT4 SDK

> Asynchronous, memory-safe Rust SDK for MetaTrader 4 algorithmic trading automation.

[![Docs](https://img.shields.io/badge/docs-RustMT4-blue.svg)](https://metarpc.github.io/RustMT4/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

📄 **Full Documentation**: [https://metarpc.github.io/RustMT4/](https://metarpc.github.io/RustMT4/)

---

## 📦 Installation

The crate is **not** published on crates.io. Clone the repository and add it as a git or path
dependency:

```bash
git clone https://github.com/MetaRPC/RustMT4.git
```

```toml
[dependencies]
metarpc-mt4 = { git = "https://github.com/MetaRPC/RustMT4" }   # or: path = "../RustMT4"
tokio = { version = "1", features = ["full"] }
```

The client talks gRPC over TLS (tonic 0.12 / prost 0.13, generated code vendored in `package/`), so
no `protoc` is needed at build time.

---

## 🏃 How to Run Examples

Clone the repository and run the quickstart example out-of-the-box:

```bash
git clone https://github.com/MetaRPC/RustMT4.git
cd RustMT4

# arguments: <login> <password> <server name> [api_key]
cargo run --example quickstart -- 1234567 password FreshForex-Demo your_api_key_here

# or via the environment (MRPC_API_KEY, MT4_LOGIN, MT4_PASSWORD, MT4_SERVER):
export MRPC_API_KEY="your_api_key_here"        # Windows PowerShell: $env:MRPC_API_KEY="your_api_key_here"
cargo run --example quickstart
```

Minimal usage (three-layer client — `MT4Sugar` is the shortest path):

```rust
use metarpc_mt4::{MT4Account, MT4Service, MT4Sugar};
use metarpc_mt4::mt4_term_api as pb;

// host + port; MRPC_API_KEY is read from the environment.
// A stable id reuses the same terminal across runs.
let sugar = MT4Sugar::new(MT4Service::new(
        MT4Account::new("mt4.mrpc.pro", 443)
            .with_id(MT4Account::stable_terminal_id("my-strategy-1")),
    ))
    .set_defaults(1, 20, "my-strategy-1");          // magic, slippage (points), comment

sugar.connect_by_server_name(login, &password, "FreshForex-Demo", Some("my-strategy-1"), 120).await?;

let acc = sugar.get_account_info().await?;          // acc.balance, acc.currency, acc.trade_mode.is_demo()
let q   = sugar.last_quote("EURUSD").await?;         // q.bid / q.ask
let bars = sugar.bars("EURUSD", pb::EnumQuoteHistoryTimeframe::QhPeriodM1 as i32, 50).await?;

let res = sugar.buy_market("EURUSD", 0.01, None, None).await?;
sugar.modify_sl_tp_by_pips(res.ticket, Some(20.0), Some(40.0)).await?;   // 20-pip SL, 40-pip TP
sugar.close(res.ticket, None).await?;
```

Layers: **MT4Account** (raw gRPC) → **MT4Service** (friendly args) → **MT4Sugar** (one-liners). Drop down a level any time with `sugar.service()` / `service.account()`.

---

## 🔑 API Key & Authentication

Connecting to MetaRPC production endpoints (`mt4.mrpc.pro:443`) requires an API key:

1. **Sign Up**: Create an account for free at [https://mrpc.pro/signup](https://mrpc.pro/signup).
2. **Generate API Key**: In your MetaRPC Portal dashboard at [https://mrpc.pro/my](https://mrpc.pro/my), go to **API Keys** to generate and copy your personal API token.
3. **Configure Connection**: Pass your API key / token along with the server address (`mt4.mrpc.pro:443`) in your connection settings.

---

## 🆔 Automatic Account ID & Authentication

MetaRPC endpoints route calls using a terminal session identifier (`id`):
- When connecting via `Connect` / `ConnectEx`, the server automatically generates and returns a session GUID (`terminalInstanceGuid`).
- The SDK automatically captures this session ID and attaches both the `id` and `APIKey` headers on all subsequent calls (`AccountSummary`, `OrderSend`, streaming, etc.) — no manual `GetId` or `curl` calls required.
- Provide your API key via the `MRPC_API_KEY` environment variable (or `api_key` in `settings.json`); the SDK sends it on every call.

## 🌐 Production Endpoints

| Environment | Host | Port | Protocol |
| :--- | :--- | :--- | :--- |
| **Production** | `mt4.mrpc.pro` | `443` | TLS / gRPC |
| **Direct API UI (Swagger)** | `https://mt4.mrpc.pro/apiui` | `443` | HTTPS |
| **Portal Dashboard** | `https://mrpc.pro/my` | `443` | HTTPS |
| **Registration / API Key** | `https://mrpc.pro/signup` | `443` | HTTPS |

---

## 📄 Documentation & Guides

Explore comprehensive documentation at [https://metarpc.github.io/RustMT4/](https://metarpc.github.io/RustMT4/):
- 🚀 **Quick Start & First Project**
- 🔑 **Authentication & API Keys**
- 📡 **Live Market Data & gRPC Streaming**
- 💼 **Account Management & Order Execution**
- 📊 **Return Codes & Error Handling Reference**
