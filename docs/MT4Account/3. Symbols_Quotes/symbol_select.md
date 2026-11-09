# Symbol Select

> Add a symbol to Market Watch (or remove it) so the terminal starts streaming its quotes.

**API Information:**

* **Low-level API:** `MT4Account::symbol_select(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.MarketInfo`
* **RPC:** `SymbolSelect(SymbolSelectRequest) -> SymbolSelectReply`
* **Payload:** `SymbolSelectData` → `success: bool`

## 💬 Just the essentials

* **What it is.** The MQL `SymbolSelect()` — toggles a symbol in Market Watch.
* **Why you need it.** A symbol not in Market Watch has no live quote; `quote` / `order_send` will fail until it's selected.
* **When to use.** Before trading or quoting a symbol you haven't touched this session.

---

## Method Signature

```rust
pub async fn symbol_select(&self, request: pb::SymbolSelectRequest) -> Result<pb::SymbolSelectData, MT4Error>
```

**Request / Reply:**

```protobuf
message SymbolSelectRequest {
  string symbol = 1;   // exact symbol name
  bool   select = 2;   // true = add to Market Watch, false = remove
}
message SymbolSelectData { bool success = 1; }
```

## ⬇️ Input — `SymbolSelectRequest`

| Field | Rust type | Required | Description |
|-------|-----------|----------|-------------|
| `symbol` | `String` | ✅ | Exact symbol name (from `symbols()`) |
| `select` | `bool` | ✅ | `true` to add, `false` to remove |

## ⬆️ Output — `SymbolSelectData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `success` | `bool` | `false` on failure (e.g. unknown symbol, or removal blocked) |

---

## 🧩 Notes & Tips

* **Removal can be refused.** A symbol can't be removed while its chart is open or it has open positions — `success` comes back `false`.
* **`MT4Sugar::ensure_symbol`** wraps "select + wait for first quote" so you don't poll yourself.

---

## 🔗 Usage Examples

### 1) Make a symbol tradable

```rust
use metarpc_mt4::mt4_term_api as pb;

let d = account.symbol_select(pb::SymbolSelectRequest {
    symbol: "EURUSD".into(),
    select: true,
}).await?;
assert!(d.success, "could not select EURUSD");
```

### 2) Clean up (remove from Market Watch)

```rust
use metarpc_mt4::mt4_term_api as pb;
let _ = account.symbol_select(pb::SymbolSelectRequest { symbol: "GBPJPY".into(), select: false }).await?;
```

---

## ⚠️ Pitfalls

* **Quote isn't instant.** After `select = true` the first tick may take a moment — retry `quote` or use `ensure_symbol`.
* **Exact name only.** `"eurusd"` or a wrong suffix returns `success = false`.

---

## 📚 See also

* [symbols](symbols.md) · [quote](quote.md) · [symbol_params_many](symbol_params_many.md)
* **Back to:** [Symbols & Quotes — Overview](Symbols_Quotes.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
