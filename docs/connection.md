# Connection & Authentication

Managing connections with `MT4Client`.

```rust
use metarpc_mt4::MT4Client;

let mut client = MT4Client::new("demo.broker.com", 443);
client.connect(100234, "password").await?;

if client.is_connected() {
    println!("Connected and authenticated!");
}

client.disconnect().await;
```
