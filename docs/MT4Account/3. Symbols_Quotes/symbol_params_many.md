# Symbol Params Many (RECOMMENDED)

> The full contract specification for one symbol, or all of Market Watch — digits, point, volume rules, tick value, margin, swap, session data.

**API Information:**

* **Low-level API:** `MT4Account::symbol_params_many(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.AccountHelper`
* **RPC:** `SymbolParamsMany(SymbolParamsManyRequest) -> SymbolParamsManyReply`
* **Payload:** `SymbolParamsManyData` → `symbol_infos: Vec<SymbolParamsManyInfo>`

## 💬 Just the essentials

* **What it is.** MT4's one-shot replacement for MT5's per-property getters — the whole `SymbolParamsManyInfo` record at once.
* **Why you need it.** `digits` / `point` for price math, `volume_min/max/step` for lot normalization, `trade_tick_value` for risk sizing, `trade_stops_level` for valid SL/TP distances.
* **When to use.** Once per symbol at startup; cache the result.

---

## Method Signature

```rust
pub async fn symbol_params_many(&self, request: pb::SymbolParamsManyRequest) -> Result<pb::SymbolParamsManyData, MT4Error>
```

**Request / Reply:**

```protobuf
message SymbolParamsManyRequest {
  optional string symbol_name = 1;   // one symbol; omit/None = all Market Watch symbols
}
message SymbolParamsManyData { repeated SymbolParamsManyInfo symbol_infos = 1; }
```

## ⬇️ Input — `SymbolParamsManyRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbol_name` | `Option<String>` | `Some("EURUSD")` for one symbol; `None` returns every symbol in Market Watch |

## ⬆️ Output — `SymbolParamsManyInfo` (the fields you'll actually use)

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbol_name` | `String` | Symbol this record describes |
| `digits` | `i32` | Price decimal places (5 for EURUSD, 3 for USDJPY) |
| `point` | `f64` | Smallest price step (`0.00001` on a 5-digit symbol) |
| `volume_min` / `volume_max` / `volume_step` | `f64` | Lot bounds and increment |
| `volume_limit` | `f64` | Max total volume per symbol (0 = no limit) |
| `trade_tick_value` | `f64` | Money per tick for 1.0 lot (account currency) |
| `trade_tick_size` | `f64` | Price change that equals one tick |
| `trade_contract_size` | `f64` | Units per lot (100000 for FX majors) |
| `trade_stops_level` | `i32` | Min SL/TP distance from price, in points |
| `trade_freeze_level` | `i32` | Freeze distance where modify/close is blocked |
| `trade_mode` | `i32` | `SpEnumSymbolTradeMode` — disabled / long-only / short-only / close-only / full |
| `trade_calc_mode` | `i32` | `SpEnumTradeCalcMode` — 0 Forex, 1 CFD, 2 Futures, 3 CFD-index |
| `swap_long` / `swap_short` | `f64` | Swap charged per lot held overnight |
| `bid` / `ask` | `f64` | Last known bid/ask at the time of the call |
| `currency_base` / `currency_profit` / `currency_margin` | `String` | The three symbol currencies |
| `sym_description` / `path` | `String` | Human label and Market-Watch tree path |

> The record has ~60 fields (session stats, margin rates, ISIN, timestamps). The table lists the ones used in day-to-day automation.

---

## 🧩 Notes & Tips

* **Pips vs points.** A *pip* is `point × 10` on 3/5-digit symbols. `MT4Sugar::pip_size` does this for you.
* **Lot normalization.** Round to `volume_step` and clamp to `[volume_min, volume_max]` — see `MT4Sugar::normalize_lot`.
* **Not selected?** If a named symbol isn't in Market Watch the call adds it, then returns its record.

---

## 🔗 Usage Examples

### 1) One symbol's trading rules

```rust
use metarpc_mt4::mt4_term_api as pb;

let d = account.symbol_params_many(pb::SymbolParamsManyRequest {
    symbol_name: Some("EURUSD".into()),
}).await?;
let s = &d.symbol_infos[0];
println!("digits={} point={} step={} tick_value={}",
    s.digits, s.point, s.volume_step, s.trade_tick_value);
```

### 2) Normalize a lot to the symbol's step

```rust
use metarpc_mt4::mt4_term_api as pb;

let s = account.symbol_params_many(pb::SymbolParamsManyRequest { symbol_name: Some("EURUSD".into()) })
    .await?.symbol_infos.into_iter().next().unwrap();
let want = 0.137_f64;
let lots = (want / s.volume_step).round() * s.volume_step;
let lots = lots.clamp(s.volume_min, s.volume_max);
println!("normalized lots = {lots}");
```

### 3) Dump every Market Watch symbol's digits

```rust
use metarpc_mt4::mt4_term_api as pb;
let d = account.symbol_params_many(pb::SymbolParamsManyRequest { symbol_name: None }).await?;
for s in &d.symbol_infos { println!("{:<12} digits={}", s.symbol_name, s.digits); }
```

---

## ⚠️ Pitfalls

* **Empty result for a bad name.** A misspelled symbol yields no `symbol_infos` — check `.first()` before indexing.
* **`trade_stops_level` is in points.** Multiply by `point` to get a price distance for SL/TP validation.
* **Cache it.** These values rarely change intraday; don't call per tick.

---

## 📚 See also

* [tick_value_with_size](tick_value_with_size.md) · [symbols](symbols.md) · [quote](quote.md)
* **Back to:** [Symbols & Quotes — Overview](Symbols_Quotes.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
