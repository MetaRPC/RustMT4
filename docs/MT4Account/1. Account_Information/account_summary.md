# Account Summary (RECOMMENDED)

> Complete account snapshot — balance, equity, margin, leverage, currency, trade mode — in a single call.

**API Information:**

* **Low-level API:** `MT4Account::account_summary()` (defined in [`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.AccountHelper`
* **RPC:** `AccountSummary(AccountSummaryRequest) -> AccountSummaryReply`
* **Payload:** `AccountSummaryData`

## 💬 Just the essentials

* **What it is.** One call that returns every essential account property.
* **Why you need it.** The most efficient way to read balance/equity/margin at once — one round-trip.
* **When to use.** At session start (banner / sanity check), and before any risk decision or order.

As a side effect it caches the server→UTC shift (`account.server_utc_shift_secs()`), used to convert gateway timestamps.

---

## Method Signature

```rust
pub async fn account_summary(&self) -> Result<pb::AccountSummaryData, MT4Error>
```

No arguments. Returns the typed payload, or an [`MT4Error`](../../All_Guides/RETURN_CODES_REFERENCE.md).

**Request / Reply:**

```protobuf
message AccountSummaryRequest {}
message AccountSummaryReply { oneof response { AccountSummaryData data = 1; Error error = 2; } }
```

---

## ⬆️ Output — `AccountSummaryData` (key fields)

| Field | Rust type | Description |
|-------|-----------|-------------|
| `account_login` | `i64` | Account number |
| `account_balance` | `f64` | Balance (excludes floating P/L) |
| `account_equity` | `f64` | Equity = balance + floating P/L |
| `account_credit` | `f64` | Credit |
| `account_margin` | `f64` | Margin used |
| `account_margin_free` | `f64` | Free margin |
| `account_margin_level` | `f64` | Margin level, percent |
| `account_leverage` | `i64` | Leverage (e.g. 100 = 1:100) |
| `account_currency` | `String` | Deposit currency |
| `account_user_name` | `String` | Account owner |
| `account_company_name` | `String` | Broker |
| `account_trade_mode` | `i32` | 0 demo / 1 contest / 2 real |
| `account_trade_allowed` | `bool` | Trading allowed for the account |
| `account_trade_expert` | `bool` | EA trading allowed |
| `is_investor` | `bool` | Connected with the investor (read-only) password |
| `server_time` | `Option<Timestamp>` | Broker server time |
| `utc_server_time_shift_minutes` | `i32` | Server offset from UTC, minutes |

---

## 🧩 Notes & Tips

* **One call beats many.** Balance, equity, margin and leverage come together.
* **`balance` vs `equity`.** `equity − balance` is your current floating P/L.
* **Margin is here.** Unlike MT5, you don't need a separate call for margin / free-margin / level.
* **Snapshot, not a stream.** Re-query before a risk check.

---

## 🔗 Usage Examples

### 1) Session-start banner

```rust
let s = account.account_summary().await?;
println!("#{}  {}  balance {:.2} {}  equity {:.2}  (floating {:+.2})",
    s.account_login, s.account_company_name, s.account_balance, s.account_currency,
    s.account_equity, s.account_equity - s.account_balance);
println!("leverage 1:{}  free margin {:.2}  level {:.1}%", s.account_leverage, s.account_margin_free, s.account_margin_level);
```

### 2) Trade-mode guard

```rust
let s = account.account_summary().await?;
if s.account_trade_mode == 2 { println!("REAL account — be careful"); }
if !s.account_trade_allowed { println!("trading is disabled on this account"); }
```

### 3) Margin-level alert

```rust
let s = account.account_summary().await?;
if s.account_margin > 0.0 && s.account_margin_level < 200.0 {
    println!("[ALERT] margin level {:.0}%", s.account_margin_level);
}
```

---

## ⚠️ Pitfalls

* **Snapshot.** Equity moves with price; don't cache it across time.
* **Currency.** Every money field is in `account_currency`.
* **`account_trade_mode` is an `i32`** (0/1/2) — compare numerically.

---

## 🧪 Testing Suggestions

* **Demo.** `account_trade_mode == 0`; `account_currency` non-empty.
* **Flat account.** `account_equity == account_balance` and `account_margin == 0`.

---

## 📚 See also

* [trade_stats](trade_stats.md) · [equity_history](equity_history.md)
* **Back to:** [Account Information — Overview](Account_Information.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
