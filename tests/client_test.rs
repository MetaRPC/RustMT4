//! Offline unit tests (no network). Live tests run through fleet/runners/rust-mt4/smoke.

use metarpc_mt4::mt4_term_api as pb;
use metarpc_mt4::{MT4Account, OrderRequest, OrderType, Quote, TradeMode};

#[test]
fn stable_terminal_id_is_uuid5() {
    let a = MT4Account::stable_terminal_id("fleet-audit-RustMT4");
    let b = MT4Account::stable_terminal_id("fleet-audit-RustMT4");
    assert_eq!(a, b, "same label must yield the same id");
    assert_eq!(a.len(), 36);
    assert_eq!(&a[14..15], "5", "uuid version nibble must be 5");
    assert_ne!(a, MT4Account::stable_terminal_id("S001"));
}

#[test]
fn account_construction_is_offline() {
    let id = MT4Account::stable_terminal_id("fleet-audit");
    let acc = MT4Account::new("mt4.mrpc.pro", 443).with_id(id.clone());
    assert_eq!(acc.terminal_id().as_deref(), Some(id.as_str()));
    assert!(!acc.is_connected(), "constructing never connects");
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
fn quote_history_timeframe_enum_values() {
    // EnumQuoteHistoryTimeframe is an ordinal (M1 = 0 … MN1 = 8), not MQL minutes.
    assert_eq!(pb::EnumQuoteHistoryTimeframe::QhPeriodM1 as i32, 0);
    assert_eq!(pb::EnumQuoteHistoryTimeframe::QhPeriodM5 as i32, 1);
    assert_eq!(pb::EnumQuoteHistoryTimeframe::QhPeriodH1 as i32, 4);
    assert_eq!(pb::EnumQuoteHistoryTimeframe::QhPeriodH4 as i32, 5);
    assert_eq!(pb::EnumQuoteHistoryTimeframe::QhPeriodMn1 as i32, 8);
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
