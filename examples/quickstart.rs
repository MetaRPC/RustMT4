//! Quickstart: connect to a demo account on the MetaRPC MT4 gateway, read the account, get a
//! quote and (optionally) place + close a market order.
//!
//! cargo run --example quickstart -- <login> <password> <server> [api_key]
//! or env: MRPC_API_KEY, MT4_LOGIN, MT4_PASSWORD, MT4_SERVER
//! Set QUICKSTART_TRADE=1 to actually send the 0.01-lot order (demo accounts only).

use std::time::Duration;

use metarpc_mt4::{MT4Client, OrderRequest, OrderType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let login: u64 = args
        .get(1)
        .cloned()
        .or_else(|| std::env::var("MT4_LOGIN").ok())
        .unwrap_or_default()
        .parse()
        .unwrap_or(0);
    let password = args.get(2).cloned().or_else(|| std::env::var("MT4_PASSWORD").ok()).unwrap_or_default();
    let server = args.get(3).cloned().or_else(|| std::env::var("MT4_SERVER").ok()).unwrap_or_default();
    let api_key = args
        .get(4)
        .cloned()
        .or_else(|| std::env::var("MRPC_API_KEY").ok())
        .unwrap_or_else(|| "TRIAL".to_string());
    if login == 0 || password.is_empty() || server.is_empty() {
        eprintln!("usage: quickstart <login> <password> <server e.g. FreshForex-Demo> [api_key]");
        std::process::exit(2);
    }

    // A stable terminal id per account: re-runs reuse the same terminal instead of creating one.
    let terminal_id = MT4Client::stable_terminal_id(&format!("quickstart-{login}"));
    let client = MT4Client::with_api_key("mt4.mrpc.pro", 443, api_key).with_id(terminal_id);

    println!("Connecting to {server} as {login} via mt4.mrpc.pro:443 ...");
    let conn = client.connect_ex(login, &password, &server, Some("quickstart"), 120).await?;
    println!("Connected. terminalInstanceGuid={}", conn.terminal_instance_guid);

    let acc = client.get_account_info().await?;
    println!(
        "Account {} ({}) balance={:.2} equity={:.2} {} leverage=1:{} mode={:?}",
        acc.login, acc.name, acc.balance, acc.equity, acc.currency, acc.leverage, acc.trade_mode
    );
    if !acc.trade_mode.is_demo() {
        eprintln!("not a demo account - refusing to trade");
        return Ok(());
    }

    let q = client.ensure_symbol("EURUSD", Duration::from_secs(15)).await?;
    println!("EURUSD bid={} ask={} spread={:.5}", q.bid, q.ask, q.spread());

    let bars = client.bars("EURUSD", metarpc_mt4::timeframe::M1, 5).await?;
    for b in &bars {
        println!("  M1 t={} o={} h={} l={} c={}", b.time, b.open, b.high, b.low, b.close);
    }

    if std::env::var("QUICKSTART_TRADE").map(|v| v == "1").unwrap_or(false) {
        let req = OrderRequest::market("EURUSD", OrderType::Buy, 0.01).with_magic(123456).with_comment("quickstart");
        let res = client.order_send(req).await?;
        println!("Order placed: ticket #{} at {} lots={}", res.ticket, res.price, res.lots);
        let c = client.order_close(res.ticket, None, 20).await?;
        println!("Closed: mode={} comment={:?}", c.mode, c.history_order_comment);
    }

    // Not disconnecting: the gateway keeps the terminal and the stable id reuses it next run.
    Ok(())
}
