/*══════════════════════════════════════════════════════════════════════════════
 FILE: examples/1_lowlevel/general_operations.rs - LOW-LEVEL MT4 GENERAL OPS DEMO
 PURPOSE:
   Every non-trading, non-streaming RPC on the raw MT4Account - account info &
   stats, orders & history, symbols & quotes. Builds each ...Request by hand and
   reads the typed ...Data. No trades here.

 🎯 WHO SHOULD USE THIS:
   • Developers learning the low-level MT4 gRPC API (read side)
   • Building monitoring / dashboard / reporting tools
   • Seeing exactly what each protobuf reply contains

 📚 WHAT THIS DEMO COVERS (3 sections):

   1. ACCOUNT INFO & STATS
      • account_summary()            - login / balance / equity / leverage
      • trade_stats() / trade_stats_equity_history()
      • equity_history() / trade_unrealized_pl()

   2. ORDERS & HISTORY
      • opened_orders() / opened_orders_tickets()
      • orders_history()             - closed orders over a window

   3. SYMBOLS & QUOTES
      • symbols() / symbol_select() / symbol_params_many()
      • tick_value_with_size() / quote() / quote_many()
      • quote_history() / server_timezone()

 🔄 API LEVELS:
   LOW-LEVEL  (MT4Account) - THIS FILE: raw gRPC/protobuf requests & replies
   MID-LEVEL  (MT4Service) - typed wrappers, less boilerplate
   HIGH-LEVEL (MT4Sugar)   - one-liner helpers with smart defaults

 USAGE:
   cargo run --example lowlevel_general
   (reads creds from examples/0_common/settings.json or the MT4_* env vars)
══════════════════════════════════════════════════════════════════════════════*/

use std::time::{SystemTime, UNIX_EPOCH};

use metarpc_mt4::mt4_term_api as pb;
use metarpc_mt4::{MT4Account, Settings};

fn ts(seconds: i64) -> prost_types::Timestamp {
    prost_types::Timestamp { seconds, nanos: 0 }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // region: CONNECT
    // ══════════════════════════════════════════════════════════════
    // CONNECT
    //   Load credentials (settings.json / MT4_* env), build the raw
    //   MT4Account and connect via the broker server name.
    // ══════════════════════════════════════════════════════════════
    let cfg = Settings::load()?;
    let acc = MT4Account::new("mt4.mrpc.pro", 443);
    acc.connect_by_server_name(cfg.user, &cfg.password, &cfg.server_name, cfg.name.as_deref(), 120).await?;
    let sym = cfg.test_symbol.clone();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
    // endregion: CONNECT

    // region: 1. ACCOUNT INFO & STATS
    // ══════════════════════════════════════════════════════════════
    // 1. ACCOUNT INFORMATION + STATS
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // account_summary()
    //   Login / balance / equity / currency / leverage in one call.
    // ──────────────────────────────────────────────────────────────
    let s = acc.account_summary().await?;
    println!("login {}  balance {:.2} {}  equity {:.2}  leverage 1:{}", s.account_login, s.account_balance, s.account_currency, s.account_equity, s.account_leverage);

    // ──────────────────────────────────────────────────────────────
    // trade_stats() / trade_stats_equity_history() / equity_history() / trade_unrealized_pl()
    //   Aggregated analytics: trade stats, the equity curve and
    //   per-order unrealized P/L extrema.
    // ──────────────────────────────────────────────────────────────
    let _ = acc.trade_stats(pb::TradeStatsRequest::default()).await?;
    let _ = acc.trade_stats_equity_history(pb::TradeStatsEquityHistoryRequest::default()).await?;
    let eq = acc.equity_history(pb::EquityHistoryRequest::default()).await?;
    let upl = acc.trade_unrealized_pl(pb::TradeUnrealizedPlRequest::default()).await?;
    println!("stats ok — equity points {}, unrealized rows {}", eq.points.len(), upl.extrema.len());
    // endregion: 1. ACCOUNT INFO & STATS

    // region: 2. ORDERS & HISTORY
    // ══════════════════════════════════════════════════════════════
    // 2. ORDERS & HISTORY
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // opened_orders() / opened_orders_tickets() / orders_history()
    //   Open orders (sorted), just their tickets, and the closed-order
    //   history over the last 7 days.
    // ──────────────────────────────────────────────────────────────
    let oo = acc
        .opened_orders(pb::OpenedOrdersRequest { sort_type: pb::EnumOpenedOrderSortType::SortByOpenTimeAsc as i32 })
        .await?;
    let tk = acc.opened_orders_tickets().await?;
    let hist = acc
        .orders_history(pb::OrdersHistoryRequest {
            input_sort_mode: pb::EnumOrderHistorySortType::HistorySortByCloseTimeDesc as i32,
            input_from: Some(ts(now - 7 * 86400)),
            input_to: Some(ts(now)),
            ..Default::default()
        })
        .await?;
    println!("open orders {} (tickets {})  history rows {}", oo.order_infos.len(), tk.tickets.len(), hist.orders_info.len());
    // endregion: 2. ORDERS & HISTORY

    // region: 3. SYMBOLS & QUOTES
    // ══════════════════════════════════════════════════════════════
    // 3. SYMBOLS & QUOTES
    // ══════════════════════════════════════════════════════════════

    // ──────────────────────────────────────────────────────────────
    // symbols() / symbol_select() / symbol_params_many() / tick_value_with_size()
    //   The symbol catalog, Market Watch select, full trading params,
    //   and tick value/size for risk sizing.
    // ──────────────────────────────────────────────────────────────
    let syms = acc.symbols().await?;
    acc.symbol_select(pb::SymbolSelectRequest { symbol: sym.clone(), select: true }).await?;
    let p = acc.symbol_params_many(pb::SymbolParamsManyRequest { symbol_name: Some(sym.clone()) }).await?;
    let tv = acc.tick_value_with_size(pb::TickValueWithSizeRequest { symbol_names: vec![sym.clone()] }).await?;

    // ──────────────────────────────────────────────────────────────
    // quote() / quote_many() / quote_history() / server_timezone()
    //   Latest Bid/Ask for one or many symbols, H1 history bars, and
    //   the trade-server timezone.
    // ──────────────────────────────────────────────────────────────
    let q = acc.quote(pb::QuoteRequest { symbol: sym.clone() }).await?;
    let qm = acc.quote_many(pb::QuoteManyRequest { symbols: vec![sym.clone(), "GBPUSD".into()] }).await?;
    let qh = acc
        .quote_history(pb::QuoteHistoryRequest {
            symbol: sym.clone(),
            timeframe: 4, // H1
            from_time: Some(ts(now - 5 * 86400)),
            to_time: Some(ts(now)),
            ..Default::default()
        })
        .await?;
    let _ = acc.server_timezone().await?;
    println!(
        "symbols {}  params {}  tick_value {}  {} bid {:.5}/{:.5}  quote_many {}  H1 bars {}",
        syms.symbol_name_infos.len(), p.symbol_infos.len(), tv.infos.len(), q.symbol, q.bid, q.ask, qm.quotes.len(), qh.historical_quotes.len()
    );
    // endregion: 3. SYMBOLS & QUOTES

    acc.disconnect(false).await?;
    Ok(())
}
