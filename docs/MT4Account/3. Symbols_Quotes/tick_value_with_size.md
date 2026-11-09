# Tick Value With Size

> Money-per-tick, tick size and contract size for one or many symbols — the numbers behind risk sizing.

**API Information:**

* **Low-level API:** `MT4Account::tick_value_with_size(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.AccountHelper`
* **RPC:** `TickValueWithSize(TickValueWithSizeRequest) -> TickValueWithSizeReply`
* **Payload:** `TickValueWithSizeData` → `infos: Vec<SymbolTickValueWithSizeInfo>`

## 💬 Just the essentials

* **What it is.** A focused slice of the contract spec: `trade_tick_value`, `trade_tick_size`, `trade_contract_size`.
* **Why you need it.** Convert a stop distance in pips into cash risk, and cash risk into a lot size.
* **When to use.** When sizing positions; lighter than pulling the full `symbol_params_many` record.

---

## Method Signature

```rust
pub async fn tick_value_with_size(&self, request: pb::TickValueWithSizeRequest) -> Result<pb::TickValueWithSizeData, MT4Error>
```

**Request / Reply:**

```protobuf
message TickValueWithSizeRequest {
  repeated string symbol_names = 1;   // exact names; EMPTY = all Market Watch symbols
}
message TickValueWithSizeData { repeated SymbolTickValueWithSizeInfo infos = 1; }
```

## ⬇️ Input — `TickValueWithSizeRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbol_names` | `Vec<String>` | Up to 1000 exact names. Empty = every symbol in Market Watch (not the whole server). Named symbols are added to Market Watch if missing. |

## ⬆️ Output — `SymbolTickValueWithSizeInfo` (per symbol)

| Field | Rust type | Description |
|-------|-----------|-------------|
| `index` | `i32` | Position in the request order |
| `symbol_name` | `String` | Symbol |
| `trade_tick_value` | `f64` | Money per tick for 1.0 lot (account currency) |
| `trade_tick_size` | `f64` | Price move equal to one tick |
| `trade_contract_size` | `f64` | Units per lot |

---

## 🧩 Notes & Tips

* **Value per pip** = `trade_tick_value × (pip_size / trade_tick_size)`. On a 5-digit FX pair `pip_size = 10 × point`.
* **Lot from risk:** `lots = cash_risk / (stop_pips × pip_value_per_lot)`. `MT4Sugar::calc_lot_by_risk` wraps exactly this.
* **Name the symbols.** Passing an empty list to "get everything" also forces every Market Watch symbol to refresh — prefer an explicit list.

---

## 🔗 Usage Examples

### 1) Tick value for a basket

```rust
use metarpc_mt4::mt4_term_api as pb;

let d = account.tick_value_with_size(pb::TickValueWithSizeRequest {
    symbol_names: vec!["EURUSD".into(), "GBPUSD".into()],
}).await?;
for i in &d.infos {
    println!("{:<8} tick_value={} tick_size={} contract={}",
        i.symbol_name, i.trade_tick_value, i.trade_tick_size, i.trade_contract_size);
}
```

### 2) Cash risk for a 20-pip stop on 0.10 lot

```rust
use metarpc_mt4::mt4_term_api as pb;

let i = account.tick_value_with_size(pb::TickValueWithSizeRequest { symbol_names: vec!["EURUSD".into()] })
    .await?.infos.into_iter().next().unwrap();
let point = 0.00001;                      // from symbol_params_many
let pip = 10.0 * point;
let pip_value_per_lot = i.trade_tick_value * (pip / i.trade_tick_size);
let risk = 20.0 * pip_value_per_lot * 0.10;
println!("risk on 0.10 lot = {risk:.2}");
```

---

## ⚠️ Pitfalls

* **Tick ≠ pip.** `trade_tick_value` is per *tick*, not per pip — scale by `pip_size / trade_tick_size`.
* **Account currency.** The value is already in your deposit currency; don't convert again.
* **≤ 1000 names** per call — a longer list is rejected with an error.

---

## 📚 See also

* [symbol_params_many](symbol_params_many.md) · [quote](quote.md)
* **Back to:** [Symbols & Quotes — Overview](Symbols_Quotes.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
