# Trade Stats + Equity History

> Aggregated statistics **and** the equity curve (plus the per-trade series) in one call.

**API Information:**

* **Low-level API:** `MT4Account::trade_stats_equity_history(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.TradeStats`
* **RPC:** `TradeStatsEquityHistory(TradeStatsEquityHistoryRequest) -> …Reply`
* **Payload:** `StatsWithChartsData`

## 💬 Just the essentials

* **What it is.** [`trade_stats`](trade_stats.md) + [`equity_history`](equity_history.md) combined — stats, equity points, and trade lists in one round-trip.
* **Why you need it.** Build a full report page without two calls.
* **When to use.** A stats dashboard that also draws the equity curve.

---

## Method Signature

```rust
pub async fn trade_stats_equity_history(
    &self,
    request: pb::TradeStatsEquityHistoryRequest,
) -> Result<pb::StatsWithChartsData, MT4Error>
```

**Request:**

```protobuf
message TradeStatsEquityHistoryRequest {
  optional string from = 1;
  optional bool   exclude_same_bars = 2;
  optional string from_utc = 3;
}
```

## ⬆️ Output — `StatsWithChartsData`

| Field | Type | Description |
|-------|------|-------------|
| `stats` | `Option<StatsData>` | The same aggregated stats as [`trade_stats`](trade_stats.md). |
| `charts` | — | Equity-curve points (as in [`equity_history`](equity_history.md)). |
| `trades` | — | Closed trades series. |
| `open_trades` | — | Currently open trades. |

---

## 🔗 Usage Examples

### 1) Stats + curve together

```rust
use metarpc_mt4::mt4_term_api as pb;

let d = account.trade_stats_equity_history(pb::TradeStatsEquityHistoryRequest { exclude_same_bars: Some(true), ..Default::default() }).await?;
if let Some(s) = &d.stats { println!("profit factor {:.2}, trades {}", s.profit_factor, s.trades); }
println!("equity points {}, trades {}", d.charts.len(), d.trades.len());
```

### 2) Since a date

```rust
use metarpc_mt4::mt4_term_api as pb;
let _ = account.trade_stats_equity_history(pb::TradeStatsEquityHistoryRequest { from: Some("2026-01-01T00:00:00".into()), ..Default::default() }).await?;
```

---

## ⚠️ Pitfalls

* **Heavier payload** than either single call — fetch it once and cache.
* **`stats` is `Option`** — guard before use.

---

## 📚 See also

* [trade_stats](trade_stats.md) · [equity_history](equity_history.md)
* **Back to:** [Account Information — Overview](Account_Information.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
