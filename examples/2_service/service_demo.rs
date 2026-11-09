//! `MT4Service` — the typed facade. This demo exercises **every** Service method across the five
//! groups (friendly args, clean returns). Trading is guarded by `SERVICE_TRADE=1` (demo only).
//!
//!   cargo run --example service_demo

use std::time::{SystemTime, UNIX_EPOCH};

use metarpc_mt4::mt4_term_api as pb;
use metarpc_mt4::{MT4Account, MT4Service, Settings};

fn ts(seconds: i64) -> prost_types::Timestamp {
    prost_types::Timestamp { seconds, nanos: 0 }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Settings::load()?;
    let svc = MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443));
    svc.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    let sym = cfg.test_symbol.as_str();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    // ── 1. Account information ──
    let s = svc.account_summary().await?;
    println!("balance {:.2} {}  equity {:.2}", s.account_balance, s.account_currency, s.account_equity);
    let st = svc.trade_stats(None).await?;
    let _ = svc.trade_stats_equity_history(None, Some(true)).await?;
    let eq = svc.equity_history(None, Some(true)).await?;
    let upl = svc.trade_unrealized_pl(None).await?;
    println!("trade_stats ok (equity points {}, unrealized rows {})", eq.points.len(), upl.extrema.len());
    let _ = st;

    // ── 2. Orders & history ──
    let oo = svc.opened_orders(pb::EnumOpenedOrderSortType::SortByOpenTimeAsc).await?;
    let tk = svc.opened_orders_tickets().await?;
    let hist = svc
        .orders_history(pb::EnumOrderHistorySortType::HistorySortByCloseTimeDesc, Some(ts(now - 7 * 86400)), Some(ts(now)))
        .await?;
    println!("open orders {} (tickets {})  history rows {}", oo.len(), tk.len(), hist.orders_info.len());

    // ── 3. Symbols & quotes ──
    println!("symbols {}", svc.symbols().await?.len());
    println!("selected {}", svc.symbol_select(sym, true).await?);
    let _ = svc.symbol_params_many(Some(sym)).await?;
    let tv = svc.tick_value_with_size(&[sym]).await?;
    let q = svc.quote(sym).await?;
    let qm = svc.quote_many(&[sym, "GBPUSD"]).await?;
    let qh = svc.quote_history(sym, 4 /* H1 */, Some(ts(now - 5 * 86400)), Some(ts(now))).await?;
    let _ = svc.server_timezone().await?;
    println!(
        "{sym} bid {:.5} ask {:.5}  quote_many {}  tick_value rows {}  H1 bars {}",
        q.bid, q.ask, qm.quotes.len(), tv.infos.len(), qh.historical_quotes.len()
    );

    // ── 4. Trading (guarded) ──
    if std::env::var("SERVICE_TRADE").ok().as_deref() == Some("1") {
        let send = svc
            .order_send(pb::OrderSendRequest {
                symbol: sym.to_string(),
                operation_type: pb::OrderSendOperationType::OcOpBuy as i32,
                volume: 0.01,
                slippage: Some(20),
                comment: Some("service_demo".into()),
                ..Default::default()
            })
            .await?;
        let ticket = send.ticket as u64;
        println!("opened #{ticket} @ {:.5}", send.price);
        let _ = svc.order_modify(ticket, None, Some(send.price + 0.0050)).await;
        let close = svc.order_close_delete(ticket, None, 20).await?;
        println!("closed #{ticket} (mode {})", close.mode);

        // order_close_by: open an opposite pair and net them out
        let buy = svc
            .order_send(pb::OrderSendRequest {
                symbol: sym.to_string(),
                operation_type: pb::OrderSendOperationType::OcOpBuy as i32,
                volume: 0.01,
                slippage: Some(20),
                ..Default::default()
            })
            .await?;
        let sell = svc
            .order_send(pb::OrderSendRequest {
                symbol: sym.to_string(),
                operation_type: pb::OrderSendOperationType::OcOpSell as i32,
                volume: 0.01,
                slippage: Some(20),
                ..Default::default()
            })
            .await?;
        let _ = svc.order_close_by(buy.ticket as u64, sell.ticket as u64).await?;
        println!("closed #{} by #{} (OrderCloseBy)", buy.ticket, sell.ticket);
    } else {
        println!("(set SERVICE_TRADE=1 to exercise order_send/modify/close)");
    }

    // ── 5. Streaming: read a couple of tick frames, then drop the stream ──
    let mut stream = svc.stream_ticks(&[sym]).await?;
    let mut n = 0;
    while n < 3 {
        match stream.message().await {
            Ok(Some(reply)) => {
                if let Some(pb::on_symbol_tick_reply::Response::Data(d)) = reply.response {
                    if let Some(t) = d.symbol_tick {
                        println!("  tick {} bid {:.5} ask {:.5}", t.symbol, t.bid, t.ask);
                        n += 1;
                    }
                }
            }
            _ => break,
        }
    }
    drop(stream);

    // the remaining streams: open each (proves the subscription), then drop to unsubscribe
    let _ = svc.stream_trades().await?;
    let _ = svc.stream_opened_tickets(1000).await?;
    let _ = svc.stream_opened_profit(1000).await?;
    let _ = svc.stream_connect_state().await?;
    println!("opened trades / opened_tickets / opened_profit / connect_state streams (then dropped)");

    svc.disconnect(false).await?;
    Ok(())
}
