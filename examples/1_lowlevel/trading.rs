//! Low-level `MT4Account` trading: build an `OrderSendRequest` by hand, then modify and close by
//! ticket. Guarded by `LOWLEVEL_TRADE=1` so it does nothing by default (demo accounts only).
//!
//!   LOWLEVEL_TRADE=1 cargo run --example lowlevel_trading

use metarpc_mt4::mt4_term_api as pb;
use metarpc_mt4::{MT4Account, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Settings::load()?;
    let acc = MT4Account::new("mt4.mrpc.pro", 443);
    acc.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    acc.symbol_select(pb::SymbolSelectRequest { symbol: cfg.test_symbol.clone(), select: true }).await?;

    if std::env::var("LOWLEVEL_TRADE").ok().as_deref() != Some("1") {
        println!("(set LOWLEVEL_TRADE=1 to actually trade — doing nothing)");
        acc.disconnect(false).await?;
        return Ok(());
    }

    // ── OrderSend: a 0.01-lot market BUY ──
    let send = acc
        .order_send(pb::OrderSendRequest {
            symbol: cfg.test_symbol.clone(),
            operation_type: pb::OrderSendOperationType::OcOpBuy as i32,
            volume: 0.01,
            slippage: Some(20),
            comment: Some("lowlevel".into()),
            ..Default::default()
        })
        .await?;
    let ticket = send.ticket as u64;
    println!("opened #{ticket} @ {:.5}", send.price);

    // ── OrderModify: attach a take-profit 50 points above the fill ──
    let _ = acc
        .order_modify(pb::OrderModifyRequest {
            order_ticket: ticket as i32,
            new_take_profit: Some(send.price + 0.0050),
            ..Default::default()
        })
        .await;

    // ── OrderCloseDelete: close the whole order ──
    let close = acc
        .order_close_delete(pb::OrderCloseDeleteRequest { order_ticket: ticket as i32, lots: None, closing_price: None, slippage: Some(20) })
        .await?;
    println!("closed #{ticket} (mode {})", close.mode);

    // ── OrderCloseBy: open an opposite pair and net them out ──
    let mkt = |op: pb::OrderSendOperationType| pb::OrderSendRequest {
        symbol: cfg.test_symbol.clone(),
        operation_type: op as i32,
        volume: 0.01,
        slippage: Some(20),
        ..Default::default()
    };
    let buy = acc.order_send(mkt(pb::OrderSendOperationType::OcOpBuy)).await?;
    let sell = acc.order_send(mkt(pb::OrderSendOperationType::OcOpSell)).await?;
    let _by = acc
        .order_close_by(pb::OrderCloseByRequest { ticket_to_close: buy.ticket, opposite_ticket_closing_by: sell.ticket })
        .await?;
    println!("closed #{} by #{} (OrderCloseBy)", buy.ticket, sell.ticket);

    acc.disconnect(false).await?;
    Ok(())
}
