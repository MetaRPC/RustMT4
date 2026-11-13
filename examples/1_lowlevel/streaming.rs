/*══════════════════════════════════════════════════════════════════════════════
 FILE: examples/1_lowlevel/streaming.rs - LOW-LEVEL MT4 STREAMING DEMO
 PURPOSE:
   Subscribe to the raw MT4Account streams: read a few OnSymbolTick frames off
   the tonic::Streaming (drop it to unsubscribe), then open the other four
   streams just to prove the subscription.

 🎯 WHO SHOULD USE THIS:
   • Developers learning the low-level MT4 streaming RPCs
   • Building live price / order / connection monitors
   • Seeing how a tonic::Streaming is consumed and dropped

 📚 WHAT THIS DEMO COVERS (2 sections):

   1. MARKET DATA STREAM
      • on_symbol_tick()            - live Bid/Ask ticks (read ~10 frames)

   2. OTHER STREAMS (open then drop)
      • on_trade()                  - trade events
      • on_opened_orders_tickets()  - periodic open-order ticket snapshots
      • on_opened_orders_profit()   - periodic open-order P/L
      • on_connect_state()          - terminal connection state

 🔄 API LEVELS:
   LOW-LEVEL  (MT4Account) - THIS FILE: raw tonic::Streaming
   MID-LEVEL  (MT4Service) - typed streaming wrappers
   HIGH-LEVEL (MT4Sugar)   - callback-style watch helpers

 ℹ️  SAFE: read-only streaming, opens no position and places no orders.

 USAGE:
   cargo run --example lowlevel_streaming
   (reads creds from examples/0_common/settings.json or the MT4_* env vars)
══════════════════════════════════════════════════════════════════════════════*/

use metarpc_mt4::mt4_term_api as pb;
use metarpc_mt4::{MT4Account, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // region: CONNECT
    // ══════════════════════════════════════════════════════════════
    // CONNECT
    //   Load credentials, build the raw MT4Account, connect and add
    //   the test symbol to Market Watch so it streams prices.
    // ══════════════════════════════════════════════════════════════
    let cfg = Settings::load()?;
    let acc = MT4Account::new("mt4.mrpc.pro", 443);
    acc.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    acc.symbol_select(pb::SymbolSelectRequest { symbol: cfg.test_symbol.clone(), select: true }).await?;
    // endregion: CONNECT

    // region: 1. MARKET DATA STREAM
    // ══════════════════════════════════════════════════════════════
    // 1. MARKET DATA STREAM
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // on_symbol_tick()
    //   Subscribe to live Bid/Ask ticks and read up to 10 frames off
    //   the raw tonic::Streaming; dropping the stream unsubscribes.
    // ──────────────────────────────────────────────────────────────
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
    // endregion: 1. MARKET DATA STREAM

    // region: 2. OTHER STREAMS
    // ══════════════════════════════════════════════════════════════
    // 2. OTHER STREAMS (open then drop)
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // on_trade() / on_opened_orders_tickets() / on_opened_orders_profit() / on_connect_state()
    //   Open each of the remaining streams (proves the subscription),
    //   then drop them to unsubscribe.
    // ──────────────────────────────────────────────────────────────
    let _ = acc.on_trade(pb::OnTradeRequest {}).await?;
    let _ = acc.on_opened_orders_tickets(pb::OnOpenedOrdersTicketsRequest { pull_interval_milliseconds: 1000 }).await?;
    let _ = acc.on_opened_orders_profit(pb::OnOpenedOrdersProfitRequest { timer_period_milliseconds: 1000 }).await?;
    let _ = acc.on_connect_state(pb::OnConnectStateRequest { terminal_ids: vec![] }).await?;
    println!("opened on_trade / on_opened_orders_tickets / on_opened_orders_profit / on_connect_state (then dropped)");
    // endregion: 2. OTHER STREAMS

    acc.disconnect(false).await?;
    Ok(())
}
