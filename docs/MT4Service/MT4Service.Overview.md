# MT4Service — Mid-Level API Overview

> The friendly facade over [`MT4Account`](../MT4Account/MT4Account.Master.Overview.md): the same operations, but you pass **plain arguments** (`&str` symbols, tickets, typed sort enums) and get back **clean values** instead of hand-built `…Request` structs.

Defined in [`src/mt4/service.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/service.rs). It wraps one `MT4Account` (field `acc`); borrow it with `service.account()` / `service.account_mut()`.

```text
MT4Sugar (HIGH)   order building, pip math, risk sizing, one-liners
   │ uses
MT4Service (MID)  ← you are here — friendly args, clean returns
   │ uses
MT4Account (LOW)  raw gRPC, …Data payloads
```

The value-add is two conveniences: **friendly arguments** (build the `…Request` and cast enums for you) and **clean returns** (simple getters unwrap `…Data` to a scalar/`Vec`). Streaming methods are renamed to `stream_*` and return the raw `tonic::Streaming<…Reply>`.

---

## Section pages

* **[1. Account Information](./1.%20Account_Information.md)** — summary + trade stats / equity / unrealized P/L.
* **[2. Orders & History](./2.%20Orders_History.md)** — open orders, tickets, closed history.
* **[3. Symbols & Quotes](./3.%20Symbols_Quotes.md)** — symbols, select, params, ticks, bars, server timezone.
* **[4. Trading Operations](./4.%20Trading_Operations.md)** — send / modify / close-delete / close-by.
* **[5. Streaming Methods](./5.%20Streaming_Methods.md)** — ticks, trades, ticket lists, per-order profit, connect state.

---

## Complete method index (25)

### 1. Account Information
| Method | Returns |
|--------|---------|
| `account_summary()` | `AccountSummaryData` |
| `trade_stats(from)` | `StatsData` |
| `trade_stats_equity_history(from, exclude_same_bars)` | `StatsWithChartsData` |
| `equity_history(from, exclude_same_bars)` | `EquityHistoryData` |
| `trade_unrealized_pl(from)` | `TradeUnrealizedPlData` |

### 2. Orders & History
| Method | Returns |
|--------|---------|
| `opened_orders(sort)` | `Vec<OpenedOrderInfo>` |
| `opened_orders_tickets()` | `Vec<i32>` |
| `orders_history(sort, from, to)` | `OrdersHistoryData` |

### 3. Symbols & Quotes
| Method | Returns |
|--------|---------|
| `symbols()` | `Vec<SymbolNameInfo>` |
| `symbol_select(symbol, select)` | `bool` |
| `symbol_params_many(filter)` | `SymbolParamsManyData` |
| `tick_value_with_size(symbols)` | `TickValueWithSizeData` |
| `quote(symbol)` | `QuoteData` |
| `quote_many(symbols)` | `QuoteManyData` |
| `quote_history(symbol, tf, from, to)` | `QuoteHistoryData` |
| `server_timezone()` | `ServerTimezoneData` |

### 4. Trading Operations
| Method | Returns |
|--------|---------|
| `order_send(request)` | `OrderSendData` |
| `order_modify(ticket, sl, tp)` | `OrderModifyData` |
| `order_close_delete(ticket, lots, slippage)` | `OrderCloseDeleteData` |
| `order_close_by(ticket, opposite)` | `OrderCloseByData` |

### 5. Streaming Methods
| Method | Item |
|--------|------|
| `stream_ticks(symbols)` | `OnSymbolTickReply` |
| `stream_trades()` | `OnTradeReply` |
| `stream_opened_tickets(interval_ms)` | `OnOpenedOrdersTicketsReply` |
| `stream_opened_profit(interval_ms)` | `OnOpenedOrdersProfitReply` |
| `stream_connect_state()` | `OnConnectStateReply` |

Plus connection delegates: `connect_by_server_name`, `reconnect`, `disconnect`.

---

## Usage pattern

```rust
use metarpc_mt4::{MT4Account, MT4Service};
use metarpc_mt4::mt4_term_api as pb;

let svc = MT4Service::new(MT4Account::new("mt4.mrpc.pro", 443));
svc.connect_by_server_name(12_345_678, "password", "FreshForex-Demo", None, 120).await?;

let balance = svc.account_summary().await?.account_balance;      // field off the typed data
let selected = svc.symbol_select("EURUSD", true).await?;         // bool
let q = svc.quote("EURUSD").await?;                              // QuoteData
let open = svc.opened_orders(pb::EnumOpenedOrderSortType::SortByOpenTimeAsc).await?; // Vec
```

> Order **construction** (buy/sell by side, pip SL/TP, risk sizing) lives in [`MT4Sugar`](../MT4Sugar/MT4Sugar.Master.Overview.md); at this layer `order_send` takes the raw `OrderSendRequest`.

---

**See also:** [MT4Account — Master Overview](../MT4Account/MT4Account.Master.Overview.md) · [MT4Sugar — Master Overview](../MT4Sugar/MT4Sugar.Master.Overview.md)
