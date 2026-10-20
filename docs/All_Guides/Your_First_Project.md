# Your First Project in 10 Minutes (Rust)

> **Hands-on Quick Start** - Create a working trading project with MetaTrader 4 and RustMT4 from scratch.

---

## Step 0: Obtain Your API Key

To connect to MetaRPC endpoints (`mt4.mrpc.pro:443`), obtain your API key:
1. Register for free at [https://mrpc.pro/signup](https://mrpc.pro/signup).
2. Generate your API token in your dashboard at [https://mrpc.pro/my](https://mrpc.pro/my).
3. Set your token in your environment or connection config.

---

## Step 1: Create Your Project

Create a new directory for your trading bot:

```bash
mkdir my_rustmt4_bot
cd my_rustmt4_bot
```

Get the SDK (it is not on a package registry):

```bash
git clone https://github.com/MetaRPC/RustMT4.git
```

---

## Step 2: Write Your Trading Code

Create your main application file and paste the following snippet:

```rust
use metarpc_mt4::{MT4Client, OrderRequest, OrderType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reads MRPC_API_KEY from the environment; a stable id reuses the same terminal on re-runs.
    let client = MT4Client::new("mt4.mrpc.pro", 443)
        .with_id(MT4Client::stable_terminal_id("my-first-bot"));

    // Login, password and the broker server name (e.g. "FreshForex-Demo").
    client.connect(100234, "account_pass", "FreshForex-Demo").await?;
    println!("Connected to MT4!");

    let acc = client.get_account_info().await?;
    println!("Account Balance: {} {}", acc.balance, acc.currency);

    // Send 0.01 lot Buy at market
    let req = OrderRequest::market("EURUSD", OrderType::Buy, 0.01)
        .with_slippage(30)
        .with_comment("Rust Quickstart");

    let result = client.order_send(req).await?;
    println!("Order sent! Ticket #{}", result.ticket);

    Ok(())
}
```

---

## Step 3: Run the Program

Run your application:

```bash
# from the cloned repository; arguments: <login> <password> <server name> [api_key]
cargo run --example quickstart -- 1234567 password FreshForex-Demo your_api_key_here
```

Its `examples/quickstart` does the same thing as the snippet above.

---

## 🚀 Next Steps

Congratulations! You have successfully established a direct gRPC connection to MetaTrader 4. Next:
- Explore **[gRPC Streaming](GRPC_STREAM_MANAGEMENT.md)** to listen to live ticks.
- Check the **[API Reference](../API_Reference/MT4Account.md)** for all 40+ available terminal methods.
- Learn about high-level risk management and auto-normalization in **[MT4Sugar](../API_Reference/MT4Sugar.md)**.
