# Getting Started with RustMT4

## Prerequisites
- **Rust 1.70+** (`rustup update stable`)
- Cargo package manager

## Minimal Example

```rust
use metarpc_mt4::{MT4Client, OrderRequest, OrderType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MT4Client::new("mt4.broker.com", 443);

    client.connect(100234, "account_pass").await?;
    println!("Connected to MT4!");

    let acc = client.get_account_info().await?;
    println!("Account Balance: {} {}", acc.balance, acc.currency);

    // Send 0.01 lot Buy
    let req = OrderRequest {
        symbol: "EURUSD".to_string(),
        order_type: OrderType::Buy,
        lots: 0.01,
        price: None,
        stop_loss: None,
        take_profit: None,
        slippage: Some(30),
        comment: Some("Rust Quickstart".to_string()),
    };

    let result = client.order_send(req).await?;
    println!("Order sent! Ticket #{}", result.ticket);

    Ok(())
}
```
