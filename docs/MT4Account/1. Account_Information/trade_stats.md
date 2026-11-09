# Trade Stats

> Aggregated trading statistics for the account: drawdowns, profit factor, Sharpe, win rates, expectancy and more.

**API Information:**

* **Low-level API:** `MT4Account::trade_stats(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.TradeStats`
* **RPC:** `TradeStats(TradeStatsRequest) -> TradeStatsReply`
* **Payload:** `StatsData`

## 💬 Just the essentials

* **What it is.** A full performance report computed server-side from the account's closed trades.
* **Why you need it.** Win rate, profit factor, drawdown and risk-adjusted metrics without crunching history yourself.
* **When to use.** Dashboards and periodic performance checks.

---

## Method Signature

```rust
pub async fn trade_stats(&self, request: pb::TradeStatsRequest) -> Result<pb::StatsData, MT4Error>
```

**Request:**

```protobuf
message TradeStatsRequest {
  optional string from = 1;      // server time "yyyy-MM-ddTHH:mm:ss"
  optional string from_utc = 2;  // or UTC — send one, not both
}
```

Build it with `pb::TradeStatsRequest { from: Some("2026-01-01T00:00:00".into()), ..Default::default() }`, or `::default()` for the full account history.

---

## ⬆️ Output — `StatsData` (selected fields)

| Field | Type | Description |
|-------|------|-------------|
| `trades` | `i64` | Number of trades counted |
| `profit_factor` | `f64` | Gross profit / gross loss |
| `pips` | `f64` | Net pips |
| `lots` | `f64` | Total volume |
| `comissions` | `f64` | Total commissions |
| `ghpr` | `f64` | Geometric holding-period return |
| `growth_percent` | `f64` | Account growth % |
| `sharpe_ratio` | `f64` | Risk-adjusted return |
| `standard_deviation` | `f64` | Of trade results |
| `max_balance_drawdown_raw` / `..._relative` | `f64` | Balance drawdown (money / %) |
| `max_equity_drawdown_raw` / `..._relative` | `f64` | Equity drawdown (money / %) |
| `summary` | `Option<TradeSummaryData>` | Totals (wins/losses/…) |
| `profitability` | `Option<ProfitabilityData>` | Win/loss breakdown |
| `best_trade` / `worst_trade` | `Option<ProfitData>` | Extremes by money |
| `longs_won` / `shorts_won` | `Option<WonData>` | Direction win rates |
| `expectancy` | `Option<ExpectancyData>` | Expected value per trade |
| `z_score` | `Option<ZScoreData>` | Streak randomness |
| `deposits` / `withdrawals` | `f64` | Cash flows |

---

## 🔗 Usage Examples

### 1) Headline numbers

```rust
use metarpc_mt4::mt4_term_api as pb;

let st = account.trade_stats(pb::TradeStatsRequest::default()).await?;
println!("trades {}  profit factor {:.2}  sharpe {:.2}", st.trades, st.profit_factor, st.sharpe_ratio);
println!("max equity drawdown {:.2} ({:.1}%)", st.max_equity_drawdown_raw, st.max_equity_drawdown_relative);
```

### 2) Since a date

```rust
use metarpc_mt4::mt4_term_api as pb;

let st = account.trade_stats(pb::TradeStatsRequest { from: Some("2026-01-01T00:00:00".into()), ..Default::default() }).await?;
println!("growth since Jan: {:.1}%", st.growth_percent);
```

### 3) Direction edge

```rust
use metarpc_mt4::mt4_term_api as pb;
let st = account.trade_stats(pb::TradeStatsRequest::default()).await?;
println!("longs won: {:?}", st.longs_won);
```

---

## ⚠️ Pitfalls

* **`from` vs `from_utc`** — send only one.
* **Server time.** `from` is broker server time; convert from UTC using the account's shift if needed.
* **Many `Option` sub-messages** — guard before unwrapping.

---

## 📚 See also

* [trade_stats_equity_history](trade_stats_equity_history.md) · [equity_history](equity_history.md)
* **Back to:** [Account Information — Overview](Account_Information.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
