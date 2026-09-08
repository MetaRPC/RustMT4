use metarpc_mt4::{MT4Client, OrderRequest, OrderType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MT4Client::new("mt4.mrpc.pro", 443);

    println!("Step 1: Generating Account ID (GetId)...");
    let account_id = client.get_id(100234, "demo_password").await?;
    println!("Generated Account ID: {}", account_id);

    println!("\nStep 2: Connecting to MT4 (mt4.mrpc.pro:443)...");
    client.connect(100234, "demo_password").await?;
    println!("Connected successfully!");

    println!("\nStep 3: Querying Account Balance...");
    let acc = client.get_account_info().await?;
    println!("Account: {} ({})", acc.login, acc.name);
    println!("Balance: {} {}", acc.balance, acc.currency);

    println!("\nStep 4: Executing Market Order...");
    let req = OrderRequest {
        symbol: "EURUSD".to_string(),
        order_type: OrderType::Buy,
        lots: 0.1,
        price: Some(1.0850),
        stop_loss: Some(1.0800),
        take_profit: Some(1.0900),
        slippage: Some(5),
        comment: Some("Rust MT4 Bot".to_string()),
    };

    let res = client.order_send(req).await?;
    println!("Order placed! Ticket: #{} (Code: {})", res.ticket, res.error_code);

    client.disconnect().await;
    println!("\nDisconnected.");
    Ok(())
}
