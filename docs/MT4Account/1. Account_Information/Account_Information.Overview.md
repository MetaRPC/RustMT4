# 1. Account Information — Overview

> The account's money picture and trading statistics: a one-call snapshot, aggregated stats, the equity curve, and per-order unrealized-P/L extrema.

These methods are **read-only**. `account_summary` lives on `mt4_term_api.AccountHelper`; the rest on `mt4_term_api.TradeStats`. Each returns a typed `…Data` payload.

---

## Methods in this group

| Method | Returns | One-liner |
|--------|---------|-----------|
| [`account_summary()`](./account_summary.md) | `AccountSummaryData` | Balance, equity, margin, leverage, currency, trade mode — one round-trip. |
| [`trade_stats(req)`](./trade_stats.md) | `StatsData` | Aggregated trade statistics (drawdown, profit factor, Sharpe, …). |
| [`trade_stats_equity_history(req)`](./trade_stats_equity_history.md) | `StatsWithChartsData` | Stats **plus** equity points and per-trade series. |
| [`equity_history(req)`](./equity_history.md) | `EquityHistoryData` | Just the equity-curve points. |
| [`trade_unrealized_pl(req)`](./trade_unrealized_pl.md) | `TradeUnrealizedPlData` | Max/min unrealized P/L reached per open order. |

---

## When to use which

* **Startup banner / pre-trade snapshot** → `account_summary()`.
* **Performance report** (win rate, drawdown, profit factor) → `trade_stats()`.
* **Draw an equity curve** → `equity_history()` (or `trade_stats_equity_history()` for stats + curve in one call).
* **How far each open trade swung** → `trade_unrealized_pl()`.

> Unlike MT5, this `AccountSummaryData` **does** carry margin / free-margin / margin-level directly.

The stats requests all accept an optional `from` (server time, `yyyy-MM-ddTHH:mm:ss`) or `from_utc` — send one, not both; omit for the full account history.

---

**Back to:** [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
