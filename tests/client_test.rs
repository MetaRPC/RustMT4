//! Offline unit tests (no network). Live tests run through fleet/runners/rust-mt4/smoke.

use metarpc_mt4::{timeframe, MT4Client, OrderRequest, OrderType, Quote, TradeMode};

#[test]
fn deterministic_id_is_stable() {
    let id = MT4Client::compute_deterministic_id(12345678, "demo_password");
    assert_eq!(id, "6ce74465-7aa9-3a79-2bd8-e40241b50c43");
}

#[test]
fn stable_terminal_id_is_uuid5() {
    let a = MT4Client::stable_terminal_id("fleet-audit-RustMT4");
    let b = MT4Client::stable_terminal_id("fleet-audit-RustMT4");
    assert_eq!(a, b);
    assert_eq!(a.len(), 36);
    assert_eq!(&a[14..15], "5", "uuid version nibble must be 5");
    assert_ne!(a, MT4Client::stable_terminal_id("S001"));
}

#[tokio::test]
async fn get_id_sets_client_id() {
    let mut client = MT4Client::new("mt4.mrpc.pro", 443);
    let id = client.get_id(100234, "demo_pass").await.unwrap();
    assert!(!id.is_empty());
    assert_eq!(client.id.as_deref(), Some(id.as_str()));
    assert!(!client.is_connected());
}

#[test]
fn order_request_maps_to_proto() {
    let req = OrderRequest::market("EURUSD", OrderType::Buy, 0.01)
        .with_magic(599999)
        .with_comment("fleet-audit")
        .with_sl_tp(Some(1.0), Some(1.2));
    let p = req.to_proto();
    assert_eq!(p.symbol, "EURUSD");
    assert_eq!(p.operation_type, 0);
    assert_eq!(p.volume, 0.01);
    assert_eq!(p.price, None);
    assert_eq!(p.stoploss, Some(1.0));
    assert_eq!(p.takeprofit, Some(1.2));
    assert_eq!(p.magic_number, Some(599999));
    assert_eq!(p.comment.as_deref(), Some("fleet-audit"));
    assert_eq!(p.slippage, Some(20));
    assert_eq!(OrderType::SellStop.to_operation() as i32, 5);
    assert_eq!(OrderType::from_i32(3), Some(OrderType::SellLimit));
}

#[test]
fn timeframes_are_quote_history_enum_values() {
    assert_eq!(timeframe::M1, 0);
    assert_eq!(timeframe::H1, 4);
    assert_eq!(timeframe::parse("m5"), Some(1));
    assert_eq!(timeframe::parse("X"), None);
    assert_eq!(timeframe::seconds(timeframe::H4), 14400);
}

#[test]
fn quote_usability() {
    let q = Quote { symbol: "EURUSD".into(), bid: 1.1, ask: 1.1, time: 0, time_msc: 0, high: 0.0, low: 0.0 };
    assert!(!q.is_usable(), "bid == ask is unusable");
    let q = Quote { bid: 1.1, ask: 1.1002, ..q };
    assert!(q.is_usable());
}

#[test]
fn trade_mode_decode() {
    assert!(TradeMode::from_i32(0).is_demo());
    assert!(!TradeMode::from_i32(2).is_demo());
}
