# Return Codes & Errors Reference — RustMT4

> How failures reach you in RustMT4, and the MQL4 trade-error codes behind rejected orders.

> ⚠️ MT4 uses **MQL4 error codes** (0–150 for the trade server, 4000+ for MQL runtime) — **not** the MT5 `ENUM_TRADE_RETCODE` numbers (10004, 10018, 10019, …). If you've seen those, they belong to MT5.

---

## How errors surface

RustMT4 doesn't return a status code you have to inspect — **a failed call returns `Err(MT4Error)`**. Everything is one error enum ([`src/error.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/error.rs)):

```rust
pub enum MT4Error {
    ConnectionError(String),
    AuthError(u64),
    TradeError(u32, String),     // MQL trade failure: code + message
    Io(std::io::Error),
    Grpc(tonic::Status),         // transport / HTTP2
    Transport(String),
    Api { code: String, message: String, mql_code: i32, mql_desc: String, error_type: i32 },
    InvalidLogin(String),        // never retried
    Protocol(String),
    NotConnected,
}
```

* **`Grpc`** — transport level. `Unavailable` / `DeadlineExceeded` / `Cancelled` are retried automatically.
* **`Api`** — the gateway answered with its `Error` oneof (a gateway `code` string **plus** an MQL code/description). `TERMINAL_INSTANCE_NOT_FOUND` triggers one reconnect + retry.
* **`InvalidLogin`** — wrong login/password. **Never retried** — repeated bad logins get accounts locked.
* **`TradeError`** — an `order_send` / `order_modify` / `order_close_delete` was rejected on the MQL side. `MT4Sugar::place_order` maps a non-zero `mql_code` into this variant.

> **No error ≠ nothing to check** only for *reads*. For *trades*, a returned `Ok` means it executed; a rejection comes back as `Err(TradeError|Api)`, so handle the `Result`.

---

## The gateway `Error` payload

When a call fails server-side, the reply's `Error` oneof carries (proto `Error`):

| Field | Rust | Meaning |
|-------|------|---------|
| `error_code` | `String` | Gateway code, e.g. `TERMINAL_INSTANCE_NOT_FOUND`, `CONNECT_EX_INVALID_LOGIN_ERROR` |
| `error_message` | `String` | Human-readable message |
| `mql_error_int_code` | `i32` | The **MQL4 error number** (see tables below) |
| `mql_error_description` | `String` | MQL4 error text |
| `type` | `i32` | `ErrorType` category |

`MT4Error::from_api` turns this into `InvalidLogin` (for login/password failures) or `Api { .. }` otherwise.

---

## MQL4 trade-server error codes

These are the ones you'll actually hit from trading calls (`mql_code`). Source: the generated `MqlErrorCode` enum.

| Code | Name | Meaning / fix |
|------|------|---------------|
| 0 | `ERR_NO_ERROR` | Success |
| 1 | `ERR_NO_RESULT` | No error, but nothing changed (e.g. modify to identical values) |
| 2 | `ERR_COMMON_ERROR` | Generic error — retry / check connection |
| 3 | `ERR_INVALID_TRADE_PARAMETERS` | Bad request fields |
| 4 | `ERR_SERVER_BUSY` | Trade server busy — back off and retry |
| 6 | `ERR_NO_CONNECTION` | No link to the trade server |
| 8 | `ERR_TOO_FREQUENT_REQUESTS` | Throttle your calls |
| 128 | `ERR_TRADE_TIMEOUT` | Trade timed out — **may still have executed**; reconcile |
| 129 | `ERR_INVALID_PRICE` | Price off/stale — normalize price, re-quote |
| **130** | `ERR_INVALID_STOPS` | SL/TP too close — respect `trade_stops_level` |
| 131 | `ERR_INVALID_TRADE_VOLUME` | Lot invalid — normalize to `volume_step`/min/max |
| **132** | `ERR_MARKET_CLOSED` | Market closed — wait for session |
| 133 | `ERR_TRADE_DISABLED` | Trading disabled for the account/symbol |
| **134** | `ERR_NOT_ENOUGH_MONEY` | Insufficient margin — smaller lot |
| 135 | `ERR_PRICE_CHANGED` | Price moved — re-quote and resend |
| 136 | `ERR_OFF_QUOTES` | No prices — symbol not live / illiquid |
| 137 | `ERR_BROKER_BUSY` | Broker busy — retry |
| **138** | `ERR_REQUOTE` | Requote — resend at the new price, or raise slippage |
| 139 | `ERR_ORDER_LOCKED` | Order already being processed |
| 140 | `ERR_LONG_POSITIONS_ONLY_ALLOWED` | Only buys allowed on this symbol |
| 141 | `ERR_TOO_MANY_REQUESTS` | Too many requests — throttle |
| 145 | `ERR_TRADE_MODIFY_DENIED` | Modify denied (too close to market / frozen) |
| 146 | `ERR_TRADE_CONTEXT_BUSY` | Trade context busy — serialize trade calls |
| 147 | `ERR_TRADE_EXPIRATION_DENIED` | Expiration not allowed by broker |
| 148 | `ERR_TRADE_TOO_MANY_ORDERS` | Order count limit reached |
| 149 | `ERR_TRADE_HEDGE_PROHIBITED` | Hedging not allowed |
| 150 | `ERR_TRADE_PROHIBITED_BY_FIFO` | FIFO rule violated |

> Codes **4000+** (`ERR_NO_MQLERROR`, `ERR_ARRAY_INDEX_OUT_OF_RANGE`, …) are MQL **runtime** errors from the terminal side — rare for API trading. The full set is in the `MqlErrorCode` enum.

---

## Handling errors in practice

### Inspect the code

`MT4Error` has helpers so you don't parse strings:

```rust
use metarpc_mt4::MT4Error;

match sugar.buy_market("EURUSD", 0.10, None, None).await {
    Ok(r)  => println!("opened #{}", r.ticket),
    Err(e) => {
        if e.is_market_closed() {                 // mql_code == 132
            println!("market closed — try later");
        } else if e.mql_code() == 134 {
            println!("not enough money — size down");
        } else if e.is_retryable() {              // transient transport / terminal-not-found
            println!("transient — safe to retry: {e}");
        } else {
            eprintln!("failed: {e}");
        }
    }
}
```

| Helper | True when |
|--------|-----------|
| `mql_code()` | returns the MQL4 number (`Api.mql_code` or `TradeError` code), else 0 |
| `is_market_closed()` | MQL 132 or a "market closed" message |
| `is_retryable()` | transient transport **or** terminal-not-found |
| `is_terminal_not_found()` | gateway lost the terminal (reconnect + retry) |
| `api_code()` | the gateway `code` string (empty for non-API errors) |

### What's retried for you

The client's `call` wrapper **auto-retries**: transient gRPC (`Unavailable`/`DeadlineExceeded`/`Cancelled`) and `TERMINAL_INSTANCE_NOT_FOUND` (one reconnect). **Trade calls are not retried blindly** — a timed-out `order_send` may have filled, so on error you reconcile with [`opened_orders`](../MT4Account/2.%20Orders_History/opened_orders.md) / `positions` before resending.

---

**See also:** [ENUMs Reference](ENUMS_USAGE_REFERENCE.md) · [order_send](../MT4Account/4.%20Trading_Operations/order_send.md) · [MT4Sugar 5. Order Entry](../MT4Sugar/5.%20Order_Entry.md)
