# Equity History

> The equity-curve points over the account's history — the data for an equity/balance chart.

**API Information:**

* **Low-level API:** `MT4Account::equity_history(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.TradeStats`
* **RPC:** `EquityHistory(EquityHistoryRequest) -> EquityHistoryReply`
* **Payload:** `EquityHistoryData` → `points: Vec<EquityPointData>`

## 💬 Just the essentials

* **What it is.** A time series of balance / equity (and drawdown) points.
* **Why you need it.** Plot the equity curve; measure drawdown over time.
* **When to use.** Reporting / charting.

---

## Method Signature

```rust
pub async fn equity_history(&self, request: pb::EquityHistoryRequest) -> Result<pb::EquityHistoryData, MT4Error>
```

**Request:**

```protobuf
message EquityHistoryRequest {
  optional string from = 1;              // server time; or from_utc
  optional bool   exclude_same_bars = 2; // skip unchanged points (shorter series)
  optional string from_utc = 3;
}
```

## ⬆️ Output — `EquityPointData` (per point)

| Field | Type | Description |
|-------|------|-------------|
| `time` / `time_utc` | `Option<Timestamp>` | Point time (server / UTC) |
| `balance` | `f64` | Balance at the point |
| `equity` | `f64` | Equity at the point |
| `realized_pl` / `unrealized_pl` | `f64` | Realized / floating P/L |
| `balance_drawdown_raw` / `..._relative` | `f64` | Balance drawdown (money / %) |
| `equity_drawdown_raw` / `..._relative` | `f64` | Equity drawdown (money / %) |
| `cash_flow` | `f64` | Deposit/withdrawal at the point |

---

## 🔗 Usage Examples

### 1) Last points of the curve

```rust
use metarpc_mt4::mt4_term_api as pb;

let eq = account.equity_history(pb::EquityHistoryRequest { exclude_same_bars: Some(true), ..Default::default() }).await?;
println!("{} equity points", eq.points.len());
if let Some(p) = eq.points.last() { println!("latest equity {:.2} (balance {:.2})", p.equity, p.balance); }
```

### 2) Peak drawdown from the series

```rust
use metarpc_mt4::mt4_term_api as pb;
let eq = account.equity_history(pb::EquityHistoryRequest::default()).await?;
let max_dd = eq.points.iter().map(|p| p.equity_drawdown_relative).fold(0.0, f64::max);
println!("max equity drawdown {:.1}%", max_dd);
```

### 3) Since a date

```rust
use metarpc_mt4::mt4_term_api as pb;
let eq = account.equity_history(pb::EquityHistoryRequest { from: Some("2026-01-01T00:00:00".into()), ..Default::default() }).await?;
```

---

## ⚠️ Pitfalls

* **`exclude_same_bars`** shortens the series by dropping unchanged points — good for charts, but gaps are implicit.
* **Server time** in `time`; use `time_utc` for UTC.

---

## 📚 See also

* [trade_stats](trade_stats.md) · [trade_stats_equity_history](trade_stats_equity_history.md)
* **Back to:** [Account Information — Overview](Account_Information.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
