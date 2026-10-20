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

Minimal usage:

```rust
use metarpc_mt4::{MT4Client, OrderRequest, OrderType, timeframe};

let client = MT4Client::with_api_key("mt4.mrpc.pro", 443, api_key)
    .with_id(MT4Client::stable_terminal_id("my-strategy-1"));   // stable id => terminal is reused
client.connect_ex(login, &password, "FreshForex-Demo", Some("my-strategy-1"), 120).await?;
let acc = client.get_account_info().await?;           // acc.trade_mode.is_demo()
let q = client.quote("EURUSD").await?;                 // bid / ask
let bars = client.bars("EURUSD", timeframe::M1, 50).await?;
let res = client.order_send(OrderRequest::market("EURUSD", OrderType::Buy, 0.01).with_magic(1)).await?;
client.order_modify(res.ticket, Some(sl), Some(tp)).await?;
client.order_close(res.ticket, None, 20).await?;
```

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
- Pass your API key directly to the Account/Client constructor or via the `MRPC_API_KEY` environment variable.

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
