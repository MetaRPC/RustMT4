# Server Timezone

> The broker server's UTC offset — so you can line up server-time fields with wall-clock UTC.

**API Information:**

* **Low-level API:** `MT4Account::server_timezone()` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.ServerTimezone`
* **RPC:** `ServerTimezone(ServerTimezoneRequest) -> ServerTimezoneReply`
* **Payload:** `ServerTimezoneData`

## 💬 Just the essentials

* **What it is.** How many seconds east of UTC the broker server clock runs.
* **Why you need it.** MT4 trade/quote timestamps are in **server time**; this offset converts them to UTC (most responses already carry a `*_utc` twin, but this lets you do it yourself).
* **When to use.** Once after connecting; cache it.

---

## Method Signature

```rust
pub async fn server_timezone(&self) -> Result<pb::ServerTimezoneData, MT4Error>
```

**Request / Reply:**

```protobuf
message ServerTimezoneRequest {}   // no arguments
message ServerTimezoneData {
  string server = 1;
  optional int32 utc_offset_seconds = 2;        // server time = UTC + this; multiple of 15 min
  ServerTimezoneSource source = 3;
  google.protobuf.Timestamp validated_utc = 4;
}
```

## ⬆️ Output — `ServerTimezoneData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `server` | `String` | Broker server name |
| `utc_offset_seconds` | `Option<i32>` | Seconds east of UTC (`server = UTC + offset`). `None` when unknown; a present `0` means a genuine UTC+0 broker |
| `source` | `i32` | `ServerTimezoneSource` — where the offset came from |
| `validated_utc` | `Option<Timestamp>` | When the offset was last validated from a fresh quote |

### `ServerTimezoneSource`

| Variant | Value | Meaning |
|---------|-------|---------|
| `Unknown` | 0 | No terminal has read the offset yet (e.g. MT4 with the market closed); `utc_offset_seconds` / `validated_utc` stay unset |
| `Live` | 1 | Read by this terminal itself |
| `LastValidated` | 2 | Last offset validated for this broker, used until this terminal reads its own |

---

## 🧩 Notes & Tips

* **`None` offset is real.** When `source = Unknown` the broker's clock hasn't been read — prefer the `*_utc` fields on other responses, which stay correct.
* **Multiple of 15 minutes.** Brokers sit on quarter-hour offsets (e.g. +10800 = UTC+3).
* **`MT4Account` caches `server_utc_shift`** internally from this; you rarely convert by hand.

---

## 🔗 Usage Examples

### 1) Print the offset

```rust
let tz = account.server_timezone().await?;
match tz.utc_offset_seconds {
    Some(s) => println!("{} is UTC{:+}", tz.server, s / 3600),
    None    => println!("{}: offset unknown (source {:?})", tz.server, tz.source),
}
```

### 2) Convert a server-time unix second to UTC

```rust
let tz = account.server_timezone().await?;
let offset = tz.utc_offset_seconds.unwrap_or(0) as i64;
let server_secs = 1_700_000_000_i64;        // some server-time stamp
let utc_secs = server_secs - offset;
println!("utc unix = {utc_secs}");
```

---

## ⚠️ Pitfalls

* **Don't assume 0.** Many brokers run UTC+2/+3 (with DST) — always read it.
* **Unknown ≠ zero.** Treat `None` as "use the `_utc` fields", not "UTC+0".

---

## 📚 See also

* [quote_history](quote_history.md) · [quote](quote.md)
* **Back to:** [Symbols & Quotes — Overview](Symbols_Quotes.Overview.md) · [MT4Account — Master Overview](../MT4Account.Master.Overview.md)
