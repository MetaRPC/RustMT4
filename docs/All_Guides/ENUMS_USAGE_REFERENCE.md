# ENUMs Usage Reference — RustMT4

> Every enumeration the RustMT4 API uses, with exact values and where it appears. All are generated into [`metarpc_mt4::mt4_term_api`](https://github.com/MetaRPC/RustMT4/blob/main/package/mt4_term_api.rs) by `prost`.

## How enums work here

`prost` generates each proto enum as a Rust `#[repr(i32)]` enum. **Request fields that are enums are plain `i32`**, so you cast when building a request:

```rust
use metarpc_mt4::mt4_term_api as pb;

let req = pb::OpenedOrdersRequest {
    sort_type: pb::EnumOpenedOrderSortType::SortByOpenTimeAsc as i32,   // enum → i32
};
```

Reply fields that are enums are also `i32`; compare with `Variant as i32`, or decode with the generated `try_from`.

> `MT4Service` casts these for you — e.g. `opened_orders(sort)` takes the typed `EnumOpenedOrderSortType` directly.

---

## 1. `EnumAccountTradeMode` — account type

**Used by:** `account_summary` → `AccountSummaryData.account_trade_mode`.

| Variant | Value | Meaning |
|---------|-------|---------|
| `AccountTradeModeDemo` | 0 | Demo account |
| `AccountTradeModeContest` | 1 | Contest account |
| `AccountTradeModeReal` | 2 | Live account |

```rust
let a = account.account_summary().await?;
if a.account_trade_mode == pb::EnumAccountTradeMode::AccountTradeModeReal as i32 {
    println!("LIVE account — be careful");
}
```

---

## 2. `EnumAccountMarginMode` — margin calculation mode

**Used by:** `account_summary` → `AccountSummaryData.account_margin_mode`.

| Variant | Value | Meaning |
|---------|-------|---------|
| `AccountMarginModeRetailNetting` | 0 | Netting (parity with MT5) |
| `AccountMarginModeExchange` | 1 | Exchange |
| `AccountMarginModeRetailHedging` | 2 | Hedging |

> **MT4 is always hedging** (several positions per symbol). The other values exist only for parity with MT5.

---

## 3. `EnumOpenedOrderSortType` — open-order sort

**Used by:** `opened_orders` → `OpenedOrdersRequest.sort_type`.

| Variant | Value |
|---------|-------|
| `SortByOpenTimeAsc` | 0 |
| `SortByOpenTimeDesc` | 1 |
| `SortByOrderTicketIdAsc` | 2 |
| `SortByOrderTicketIdDesc` | 3 |

---

## 4. `EnumOrderHistorySortType` — history sort

**Used by:** `orders_history` → `OrdersHistoryRequest.input_sort_mode`.

| Variant | Value |
|---------|-------|
| `HistorySortByOpenTimeAsc` | 0 |
| `HistorySortByOpenTimeDesc` | 1 |
| `HistorySortByCloseTimeAsc` | 2 |
| `HistorySortByCloseTimeDesc` | 3 |
| `HistorySortByOrderTicketIdAsc` | 4 |
| `HistorySortByOrderTicketIdDesc` | 5 |

---

## 5. `EnumQuoteHistoryTimeframe` — bar timeframe

**Used by:** `quote_history` → `QuoteHistoryRequest.timeframe`.

| Variant | Value | Period |
|---------|-------|--------|
| `QhPeriodM1` | 0 | 1 minute |
| `QhPeriodM5` | 1 | 5 minutes |
| `QhPeriodM15` | 2 | 15 minutes |
| `QhPeriodM30` | 3 | 30 minutes |
| `QhPeriodH1` | 4 | 1 hour |
| `QhPeriodH4` | 5 | 4 hours |
| `QhPeriodD1` | 6 | 1 day |
| `QhPeriodW1` | 7 | 1 week |
| `QhPeriodMn1` | 8 | 1 month |

> ⚠️ **Ordinal 0–8, not MQL minute counts.** `H1` is `4`, not `60`.

---

## 6. `OrderSendOperationType` — order direction/type (requests)

**Used by:** `order_send` → `OrderSendRequest.operation_type`.

| Variant | Value | Meaning |
|---------|-------|---------|
| `OcOpBuy` | 0 | Market buy |
| `OcOpSell` | 1 | Market sell |
| `OcOpBuylimit` | 2 | Buy limit |
| `OcOpSelllimit` | 3 | Sell limit |
| `OcOpBuystop` | 4 | Buy stop |
| `OcOpSellstop` | 5 | Sell stop |

> Same numbering as MQL4 `OP_BUY`..`OP_SELLSTOP`.

---

## 7. `OrderCloseDeleteMode` — what a close/delete did (reply)

**Used by:** `order_close_delete` → `OrderCloseDeleteData.mode`.

| Variant | Value | Meaning |
|---------|-------|---------|
| `OcdMarketOrder` | 0 | A market order was closed |
| `OcdPendingOrder` | 1 | A pending order was deleted |

---

## 8. `ServerTimezoneSource` — where the UTC offset came from (reply)

**Used by:** `server_timezone` → `ServerTimezoneData.source`.

| Variant | Value | Meaning |
|---------|-------|---------|
| `Unknown` | 0 | Not read yet; `utc_offset_seconds` stays unset |
| `Live` | 1 | Read by this terminal |
| `LastValidated` | 2 | Last validated offset for this broker, used until this terminal reads its own |

---

## 9. `Mt4SubEnumEventGroupType` — streaming event group (reply)

**Used by:** `on_trade` → `OnTradeData.type`; `on_opened_orders_profit` → `OnOpenedOrdersProfitData.type` (access as `d.r#type`).

| Variant | Value |
|---------|-------|
| `OrderProfit` | 0 |
| `OrderUpdate` | 1 |

---

## 10. `SubOrderOperationType` — order type in stream frames (reply)

**Used by:** `on_trade` / `on_opened_orders_profit` order rows → `.type` (`OnTradeOrderInfo`, `OnOpenedOrdersProfitOrderInfo`).

| Variant | Value | Meaning |
|---------|-------|---------|
| `SubOpBuy` | 0 | Buy |
| `SubOpSell` | 1 | Sell |
| `SubOpBuylimit` | 2 | Buy limit |
| `SubOpSelllimit` | 3 | Sell limit |
| `SubOpBuystop` | 4 | Buy stop |
| `SubOpSellstop` | 5 | Sell stop |

---

## Not an enum, but you'll decode it: `order_type`

`OpenedOrderInfo.order_type` and `HistoryOrderInfo.order_type` are plain **`i32`**, not a named proto enum — but they follow the same **0 Buy, 1 Sell, 2 BuyLimit, 3 SellLimit, 4 BuyStop, 5 SellStop** numbering. `MT4Sugar` decodes them via [`OrderType::from_i32`](https://github.com/MetaRPC/RustMT4/blob/main/src/models.rs), which also exposes `.is_market()` / `.is_buy()`.

```rust
use metarpc_mt4::OrderType;
let positions = orders.iter().filter(|o| matches!(OrderType::from_i32(o.order_type), Some(t) if t.is_market()));
```

---

**See also:** [MT4Account — Master Overview](../MT4Account/MT4Account.Master.Overview.md) · [Return Codes](RETURN_CODES_REFERENCE.md) · [MT4Service API](../MT4Service/MT4Service.Overview.md)
