//! Low-level `MT4Account` streaming: subscribe to `OnSymbolTick` and read a few frames off the raw
//! `tonic::Streaming` (drop it to unsubscribe).
//!
//!   cargo run --example lowlevel_streaming

use metarpc_mt4::mt4_term_api as pb;
use metarpc_mt4::{MT4Account, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Settings::load()?;
    let acc = MT4Account::new("mt4.mrpc.pro", 443);
    acc.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    acc.symbol_select(pb::SymbolSelectRequest { symbol: cfg.test_symbol.clone(), select: true }).await?;

    let mut stream = acc
        .on_symbol_tick(pb::OnSymbolTickRequest { symbol_names: vec![cfg.test_symbol.clone()] })
        .await?;

    println!("reading up to 10 ticks for {} …", cfg.test_symbol);
    let mut n = 0;
    while n < 10 {
        match stream.message().await {
            Ok(Some(reply)) => {
                if let Some(pb::on_symbol_tick_reply::Response::Data(d)) = reply.response {
                    if let Some(t) = d.symbol_tick {
                        println!("  {} bid {:.5} ask {:.5}", t.symbol, t.bid, t.ask);
                        n += 1;
                    }
                }
            }
            Ok(None) => break,
            Err(e) => {
                eprintln!("stream error: {e}");
                break;
            }
        }
    }

    // the other four streams: open each (proves the subscription), then drop to unsubscribe
    let _ = acc.on_trade(pb::OnTradeRequest {}).await?;
    let _ = acc.on_opened_orders_tickets(pb::OnOpenedOrdersTicketsRequest { pull_interval_milliseconds: 1000 }).await?;
    let _ = acc.on_opened_orders_profit(pb::OnOpenedOrdersProfitRequest { timer_period_milliseconds: 1000 }).await?;
    let _ = acc.on_connect_state(pb::OnConnectStateRequest { terminal_ids: vec![] }).await?;
    println!("opened on_trade / on_opened_orders_tickets / on_opened_orders_profit / on_connect_state (then dropped)");

    acc.disconnect(false).await?;
    Ok(())
}
