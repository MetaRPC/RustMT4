# Connection & Authentication

Managing connections with `MT4Client`.

```rust
use metarpc_mt4::MT4Client;

let mut client = MT4Client::new("mt4.mrpc.pro", 443);
client.connect(100234, "password").await?;

if client.is_connected() {
    println!("Connected and authenticated!");
}

client.disconnect().await;
```


> **Authentication Note**: Connecting to `mt4.mrpc.pro:443` requires a valid MetaRPC API key. Register for free at [https://mrpc.pro/signup](https://mrpc.pro/signup) and generate your token in [https://mrpc.pro/my](https://mrpc.pro/my).

