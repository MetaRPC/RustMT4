# Symbols

> Every symbol name the terminal knows — the broker's full instrument list.

**API Information:**

* **Low-level API:** `MT4Account::symbols()` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.MarketInfo`
* **RPC:** `Symbols(SymbolsRequest) -> SymbolsReply`
* **Payload:** `SymbolsData` → `symbol_name_infos: Vec<SymbolNameInfo>`

## 💬 Just the essentials

* **What it is.** The full catalogue of symbols the broker exposes, each with its index.
* **Why you need it.** Validate a symbol before trading; build a picker; discover suffixed names (`EURUSD.m`, `EURUSDpro`).
* **When to use.** Startup / configuration. Not a hot-path call.

---

## Method Signature

```rust
pub async fn symbols(&self) -> Result<pb::SymbolsData, MT4Error>
```

**Request / Reply:**

```protobuf
message SymbolsRequest {}                                   // no arguments
message SymbolsData { repeated SymbolNameInfo symbol_name_infos = 1; }
```

## ⬆️ Output — `SymbolNameInfo` (per symbol)

| Field | Rust type | Description |
|-------|-----------|-------------|
| `symbol_name` | `String` | Exact symbol name as the broker spells it |
| `symbol_index` | `i32` | Server-side index of the symbol |

---

## 🧩 Notes & Tips

* **This is the server list, not Market Watch.** A symbol here still needs `symbol_select` before you can `quote` it.
* **Suffixes vary by broker.** Never hard-code `"EURUSD"`; look it up here and keep the exact string.

---

## 🔗 Usage Examples

### 1) List everything

```rust
let d = account.symbols().await?;
println!("{} symbols", d.symbol_name_infos.len());
for s in d.symbol_name_infos.iter().take(10) {
    println!("#{:<4} {}", s.symbol_index, s.symbol_name);
}
```

### 2) Resolve a symbol by prefix

```rust
let d = account.symbols().await?;
let eurusd = d.symbol_name_infos.iter()
    .find(|s| s.symbol_name.starts_with("EURUSD"))
    .map(|s| s.symbol_name.clone());
println!("using {eurusd:?}");
```

---

## ⚠️ Pitfalls

* **Large lists.** Some brokers expose 10,000+ symbols — filter, don't print them all.
* **Presence ≠ tradable.** Check `symbol_params_many` for `trade_mode` / stops level before sending orders.

---

## 📚 See also

* [symbol_select](symbol_select.md) · [symbol_params_many](symbol_params_many.md) · [quote](quote.md)
* **Back to:** [Symbols & Quotes — Overview](Symbols_Quotes.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
