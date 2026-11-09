# 3. Symbols & Quotes — Overview

> Everything about instruments and prices: the symbol list, Market Watch select, bulk contract params, live ticks, bar history, and the server timezone.

Mostly read-only (except `symbol_select`). Live on `mt4_term_api.MarketInfo`, `AccountHelper` (params / tick value) and `ServerTimezone`.

---

## Methods in this group

| Method | Returns | One-liner |
|--------|---------|-----------|
| [`symbols()`](./symbols.md) | `SymbolsData` | Every symbol name the terminal knows. |
| [`symbol_select(req)`](./symbol_select.md) | `SymbolSelectData` | Add / remove a symbol in Market Watch. |
| [`symbol_params_many(req)`](./symbol_params_many.md) | `SymbolParamsManyData` | Full contract params for one symbol or all. |
| [`tick_value_with_size(req)`](./tick_value_with_size.md) | `TickValueWithSizeData` | Tick value / size / contract size per symbol. |
| [`quote(req)`](./quote.md) | `QuoteData` | The latest tick (bid/ask + session high/low). |
| [`quote_many(req)`](./quote_many.md) | `QuoteManyData` | Latest ticks for several symbols. |
| [`quote_history(req)`](./quote_history.md) | `QuoteHistoryData` | OHLC bars for a symbol/timeframe range. |
| [`server_timezone()`](./server_timezone.md) | `ServerTimezoneData` | The broker server's UTC offset. |

---

## When to use which

* **"Is this symbol tradable?"** → `symbols()` / `symbol_select()` → `quote()`.
* **Contract specs** (digits, point, volume rules, tick value) → `symbol_params_many()`.
* **Money per tick** → `tick_value_with_size()`.
* **Candles / backtest input** → `quote_history()` (MT4 timeframe enum: M1 = 0 … MN1 = 8).

> MT4 has **no per-property getters** like MT5's `symbol_info_double/integer/string` — `symbol_params_many` returns the whole `SymbolParamsManyInfo` record at once.

---

**Back to:** [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
