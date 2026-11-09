# On Connect State

> Terminal connection-state events — know when the bridge to the broker goes up or down.

**API Information:**

* **Low-level API:** `MT4Account::on_connect_state(request)` ([`src/mt4/account.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/mt4/account.rs))
* **gRPC service:** `mt4_term_api.Connection` *(note: not `SubscriptionService`)*
* **RPC:** `OnConnectState(OnConnectStateRequest) -> stream OnConnectStateReply`
* **Returns:** `tonic::Streaming<OnConnectStateReply>`

## 💬 Just the essentials

* **What it is.** A stream of connection-lifecycle updates for one or more terminals.
* **Why you need it.** Pause trading while disconnected; alert on `WrongPassword` / `Expired`; drive reconnect logic.
* **When to use.** Any long-running bot that must survive broker-side drops.

---

## Method Signature

```rust
pub async fn on_connect_state(&self, request: pb::OnConnectStateRequest) -> Result<Streaming<pb::OnConnectStateReply>, MT4Error>
```

**Request / Reply:**

```protobuf
message OnConnectStateRequest { repeated string terminal_ids = 1; }  // empty = this terminal
message OnConnectStateReply { oneof response { ConnectStateData data = 1; Error error = 2; } }
message ConnectStateData {
  string id = 1;
  string state = 2;        // "Queued","Starting","Authorizing","Syncing","Connected","WrongPassword","Unreachable","Stopped","Expired"
  bool   is_connected = 3;
  bool   is_alive = 4;
  bool   api_is_alive = 5;
  string server = 6;
  uint64 account = 7;
  string error_message = 8;
  google.protobuf.Timestamp timestamp = 9;
}
```

## ⬇️ Input — `OnConnectStateRequest`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `terminal_ids` | `Vec<String>` | Terminal GUIDs to watch; **empty** = this account's terminal |

## ⬆️ Frame — `ConnectStateData`

| Field | Rust type | Description |
|-------|-----------|-------------|
| `id` | `String` | Terminal id this update is about |
| `state` | `String` | Human-readable state (see list above) |
| `is_connected` | `bool` | Broker link is up |
| `is_alive` | `bool` | Terminal process is alive |
| `api_is_alive` | `bool` | API bridge is alive |
| `server` | `String` | Broker server name |
| `account` | `u64` | Account login |
| `error_message` | `String` | Populated on failure states |
| `timestamp` | `Option<Timestamp>` | When the state was observed |

> `state` is a **free-form string**, not an enum — compare against the documented values.

---

## 🧩 Notes & Tips

* **Different service.** This RPC lives on `Connection`, not `SubscriptionService` — the layer routes it correctly.
* **Gate trading on `is_connected`.** Hold entries while it's `false`; resume on `Connected`.
* **Terminal states** worth alerting on: `WrongPassword`, `Expired`, `Unreachable`, `Stopped`.
* **`MT4Sugar`** exposes this via `MT4Service::stream_connect_state`.

---

## 🔗 Usage Examples

### 1) Trade gate

```rust
use metarpc_mt4::mt4_term_api as pb;

let mut s = account.on_connect_state(pb::OnConnectStateRequest { terminal_ids: vec![] }).await?;
while let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_connect_state_reply::Response::Data(d)) = reply.response {
        println!("{} on {}: {} (connected={})", d.account, d.server, d.state, d.is_connected);
        if !d.is_connected { /* pause trading */ }
    }
}
```

### 2) Alert on auth failure

```rust
use metarpc_mt4::mt4_term_api as pb;

let mut s = account.on_connect_state(pb::OnConnectStateRequest { terminal_ids: vec![] }).await?;
while let Ok(Some(reply)) = s.message().await {
    if let Some(pb::on_connect_state_reply::Response::Data(d)) = reply.response {
        if d.state == "WrongPassword" || d.state == "Expired" {
            eprintln!("ALERT: {} — {}", d.state, d.error_message);
            break;
        }
    }
}
```

---

## ⚠️ Pitfalls

* **`state` is a string** — don't match it as an enum; watch for typos in comparisons.
* **Empty `terminal_ids`** means "this terminal" — pass ids only when watching others.
* **Handle the `Error` arm** and reconnect per your policy.

---

## 📚 See also

* [MT4Account — Master Overview](../MT4Account.Master.Overview.md) (Connection group) · [on_trade](on_trade.md)
* **Back to:** [Streaming Methods — Overview](Streaming_Methods.Overview.md)
