use metarpc_mt4::{MT4Client, OrderRequest, OrderType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MT4Client::new("mt4.broker.com", 443);

    println!("Connecting to MT4...");
    client.connect(100234, "password").await?;
    println!("Connected!");

    let acc = client.get_account_info().await?;
    println!("Balance: {} {}", acc.balance, acc.currency);

    let req = OrderRequest {
        symbol: "EURUSD".to_string(),
        order_type: OrderType::Buy,
        lots: 0.1,
        price: None,
        stop_loss: Some(1.0800),
        take_profit: Some(1.0950),
        slippage: Some(30),
        comment: Some("Rust MT4 Bot".to_string()),
    };

    let res = client.order_send(req).await?;
    println!("Order executed! Ticket: #{}", res.ticket);

    Ok(())
}
