# Order Send (RECOMMENDED)

> Open a market or pending order — the one write call that creates positions and pendings.

**API Information:**

* **Low-level API:** `MT4Account::order_send(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.TradingHelper`
* **RPC:** `OrderSend(OrderSendRequest) -> OrderSendReply`
* **Payload:** `OrderSendData`

## 💬 Just the essentials

* **What it is.** MQL `OrderSend()` — market buy/sell or any of the four pendings.
* **Why you need it.** It's how you enter the market.
* **When to use.** Entry. SL/TP/comment/magic go in the same request.

---

## Method Signature

```rust
pub async fn order_send(&self, request: pb::OrderSendRequest) -> Result<pb::OrderSendData, MT4Error>
```

**Request / Reply:**

```protobuf
message OrderSendRequest {
  string symbol = 1;                      // required, non-empty
  OrderSendOperationType operation_type = 2;  // required
  double volume = 3;                      // required, lots
  optional double price = 4;              // required for pendings; ignored for market
  optional int32  slippage = 5;           // points, default 0
  optional double stoploss = 6;           // price, default 0 = none
  optional double takeprofit = 7;         // price, default 0 = none
  optional string comment = 8;
  optional int32  magic_number = 9;
  optional google.protobuf.Timestamp expiration = 10;      // pending expiry (server time)
  optional google.protobuf.Timestamp expiration_utc = 11;  // ...or UTC; not both
}
message OrderSendData {
  int32  ticket = 1;
  double volume = 2;
  double price = 3;                       // actual fill / placement price
  google.protobuf.Timestamp open_time = 4;
  google.protobuf.Timestamp open_time_utc = 5;
}
```

## ⬇️ Input — `OrderSendRequest`

| Field | Rust type | Required | Description |
|-------|-----------|----------|-------------|
| `symbol` | `String` | ✅ | Exact name, in Market Watch |
| `operation_type` | `i32` | ✅ | `OrderSendOperationType` as `i32` (below) |
| `volume` | `f64` | ✅ | Lots (normalize to `volume_step`) |
| `price` | `Option<f64>` | pendings | Trigger price; `None` for market (fills at current bid/ask) |
| `slippage` | `Option<i32>` | — | Max deviation in points |
| `stoploss` / `takeprofit` | `Option<f64>` | — | SL / TP as prices; `0`/`None` = none |
| `comment` | `Option<String>` | — | Free text |
| `magic_number` | `Option<i32>` | — | EA id for later filtering |
| `expiration` / `expiration_utc` | `Option<Timestamp>` | — | Pending expiry — send **one** |

### `OrderSendOperationType`

| Variant | Value | Meaning |
|---------|-------|---------|
| `OcOpBuy` | 0 | Market buy |
| `OcOpSell` | 1 | Market sell |
| `OcOpBuylimit` | 2 | Buy limit (below price) |
| `OcOpSelllimit` | 3 | Sell limit (above price) |
| `OcOpBuystop` | 4 | Buy stop (above price) |
| `OcOpSellstop` | 5 | Sell stop (below price) |

## ⬆️ Output — `OrderSendData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `ticket` | `i32` | New order ticket |
| `volume` | `f64` | Filled / placed volume |
| `price` | `f64` | Actual fill or placement price |
| `open_time` / `open_time_utc` | `Option<Timestamp>` | Open time (server / UTC) |

---

## 🧩 Notes & Tips

* **Not retried.** A timed-out `order_send` may already have filled — the layer never retries it blindly. Reconcile via `opened_orders` on error.
* **MQL failures** (130 invalid stops, 132 market closed, 134 not enough money, 138 requote) come back in the reply's `Error` → `MT4Error::Api { mql_code, .. }`.
* **Respect `trade_stops_level`.** SL/TP closer than the broker's minimum distance is rejected with 130.
* **`MT4Sugar`** has `buy_market` / `sell_limit` / … that build this request for you.

---

## 🔗 Usage Examples

### 1) Market buy with SL/TP

```rust
use metarpc_mt4::mt4_term_api as pb;

let r = account.order_send(pb::OrderSendRequest {
    symbol: "EURUSD".into(),
    operation_type: pb::OrderSendOperationType::OcOpBuy as i32,
    volume: 0.10,
    stoploss: Some(1.0800),
    takeprofit: Some(1.0900),
    slippage: Some(20),
    magic_number: Some(1001),
    comment: Some("demo".into()),
    ..Default::default()
}).await?;
println!("opened #{} @ {}", r.ticket, r.price);
```

### 2) Buy limit (pending) below the market

```rust
use metarpc_mt4::mt4_term_api as pb;

let r = account.order_send(pb::OrderSendRequest {
    symbol: "EURUSD".into(),
    operation_type: pb::OrderSendOperationType::OcOpBuylimit as i32,
    volume: 0.10,
    price: Some(1.0750),
    ..Default::default()
}).await?;
println!("pending #{}", r.ticket);
```

---

## ⚠️ Pitfalls

* **Pending needs `price`.** Without it a pending request is invalid.
* **Prices, not pips.** SL/TP are absolute prices — convert pip distances with `pip_size` first.
* **Don't double-send on timeout.** Check `opened_orders` before retrying.

---

## 📚 See also

* [order_modify](order_modify.md) · [order_close_delete](order_close_delete.md) · [opened_orders](../2.%20Orders_History/opened_orders.md)
* **Back to:** [Trading Operations — Overview](Trading_Operations.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
