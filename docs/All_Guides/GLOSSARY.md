# RustMT4 Glossary

> The terms you'll meet across these docs — architecture, MT4 trading, Rust/async, and gRPC — each with a one-paragraph definition and where it shows up in the SDK.

---

## 🏗️ Architecture

### Three-layer architecture
RustMT4 is split into three stacked layers, each wrapping the one below:

| Layer | Type | Role |
|-------|------|------|
| [`MT4Account`](../MT4Account/MT4Account.Master.Overview.md) | low | Raw gRPC — build `…Request`, get `…Data`. |
| [`MT4Service`](../MT4Service/MT4Service.Overview.md) | mid | Friendly args, clean returns. |
| [`MT4Sugar`](../MT4Sugar/MT4Sugar.Master.Overview.md) | high | One-liners, pip math, risk sizing. |

### MT4Account
Layer 1. Owns the gRPC channel, TLS, request metadata (session id + API key), retry and reconnect. Methods take raw protobuf requests and return raw `…Data`. Use it for full control.

### MT4Service
Layer 2. The same 25 operations with **friendly arguments** (`&str` symbols, typed sort enums) and **clean returns** (a `Vec` or scalar instead of a wrapper struct). Streams are renamed `stream_*`.

### MT4Sugar
Layer 3. ~45 convenience methods: buy/sell by side, pip conversions, risk-based lot sizing, trailing/breakeven, watchers, order info. Reuses the plain DTOs in `src/models.rs`.

### Orchestrator / Preset
An **orchestrator** (`run_*` in `src/strategies/`) chains Sugar calls into a full lifecycle (size → enter → manage → close). A **preset** configures it: `StrategyPreset` (what to trade) + `RiskPreset` (how much to risk). See [Strategies](../Strategies/Strategies.Master.Overview.md).

---

## 📈 MT4 trading terms

### Order vs Position
In MT4 there's no separate "position" object: an **open market order** *is* the position. All open orders — market and pending — live in one list, told apart by `order_type`. `MT4Sugar::positions()` filters to the market ones; `orders()` returns everything.

### Order type
An integer **0 Buy, 1 Sell, 2 BuyLimit, 3 SellLimit, 4 BuyStop, 5 SellStop** (MQL4 `OP_*` numbering). Decoded by `OrderType::from_i32` with `.is_market()` / `.is_buy()`.

### Pending order
An order that triggers when price reaches a level, rather than filling now: **limit** (buy below / sell above the market) and **stop** (buy above / sell below). Placed with `buy_limit`/`sell_limit`/`buy_stop`/`sell_stop`; deleted with `close`/`cancel_pendings`.

### Lot / Volume
Trade size. 1.0 lot = `trade_contract_size` units (100,000 for FX majors). Brokers constrain it with `volume_min`, `volume_max`, `volume_step` — always pass sizes through [`normalize_lot`](../MT4Sugar/3.%20Symbol_Math.md).

### Point vs Pip
A **point** is the smallest price step (`SYMBOL_POINT`, e.g. `0.00001` on a 5-digit pair). A **pip** is the conventional unit: `point × 10` on 3/5-digit symbols, else `point`. `MT4Sugar::pip_size` computes it; `pips_to_price` / `price_to_pips` convert.

### Digits
Price decimal places (`5` for EURUSD, `3` for USDJPY). Round prices to this with [`normalize_price`](../MT4Sugar/3.%20Symbol_Math.md) to avoid "invalid price" (129).

### Stop Loss (SL) / Take Profit (TP)
Protective exit prices attached to an order — SL caps loss, TP locks profit. Set them as absolute prices (`modify_sl_tp_by_price`) or as pip distances from entry (`modify_sl_tp_by_pips`). Must respect `trade_stops_level` or the broker rejects with 130.

### Spread
`ask − bid`. In pips via `MT4Sugar::spread_pips`. Wide spreads (news, low liquidity) are a common reason to skip an entry.

### Swap
Interest credited/debited for holding a position overnight (`swap_long` / `swap_short` per lot). Part of an order's net result: `profit + swap + commision`.

### Margin / Equity / Free margin / Margin level
**Margin** is collateral reserved by open positions; **equity** = balance + floating P/L; **free margin** = equity − margin; **margin level** = equity / margin × 100. All on `AccountSummaryData` (`account_summary`). Unlike MT5, MT4's summary carries these directly.

### Magic number
An integer tag you stamp on orders (`set_defaults(magic, …)`) to identify which bot/strategy owns them. Filter with `orders_by_magic`.

### Hedging
Holding opposite positions on the same symbol at once. **MT4 is always hedging** (`account_margin_mode` = RetailHedging). Net two opposite orders with `close_by` to save a spread.

### Trailing stop / Breakeven
**Trailing** ratchets the SL behind price as it moves in your favor (`trail_stop`, only ever tightening). **Breakeven** moves the SL to entry (±a few pips) once in profit (`auto_breakeven`, never loosening). Both are one-shot — call on a timer.

---

## ⚙️ Rust & async terms

### Future / `async` / `.await`
Every SDK method is `async` and returns a **Future** — a value that completes later. Nothing runs until you `.await` it inside a Tokio runtime. See [The Async Model](SYNC_VS_ASYNC.md).

### Tokio
The async runtime RustMT4 runs on. Start it with `#[tokio::main]`; sleep with `tokio::time::sleep` (never `std::thread::sleep`, which blocks the runtime).

### `Result<T, MT4Error>`
Every call returns a `Result`. `Ok(T)` on success, `Err(MT4Error)` on failure — see [Return Codes](RETURN_CODES_REFERENCE.md). Propagate with `?`.

### `tonic::Streaming<…Reply>`
A server stream — a long-lived subscription that pushes frames. Read with `stream.message().await` in a loop, matching the reply's `oneof { Data, Error }`. `MT4Account` returns these directly; `MT4Sugar` forwards them to channels.

### `mpsc::Receiver`
A Tokio multi-producer/single-consumer channel. Sugar's watchers (`subscribe_quotes`, `watch_profit`, `watch_tickets`) spawn a task that forwards stream frames onto one; you `rx.recv().await`. Drop the receiver to unsubscribe.

### DTO (`Quote`, `Bar`, `OrderResult`, …)
Plain Rust structs in `src/models.rs` that `MT4Sugar` returns instead of raw protobuf — friendlier fields, `Serialize`/`Deserialize` derived.

---

## 🔌 gRPC & protocol terms

### gRPC
The high-performance RPC framework the SDK speaks over HTTP/2 + TLS to `mt4.mrpc.pro:443`. Supports both unary (request/response) and server-streaming calls.

### Protobuf (protocol buffers)
The binary message format for requests/replies. The generated Rust types live in [`metarpc_mt4::mt4_term_api`](https://github.com/MetaRPC/RustMT4/blob/main/package/mt4_term_api.rs) (via `prost`). `MT4Account` works with these directly.

### `…Request` / `…Reply` / `…Data`
The call shape: you send a `…Request`, the server answers a `…Reply` whose `response` is a `oneof { Data, Error }`. `MT4Account` unwraps the `Data` payload (`…Data`) or turns `Error` into `MT4Error`.

### Session id (terminal GUID)
A per-terminal identifier returned by `Connect`/`ConnectEx`. The SDK captures it and attaches it (with your API key) to every later request and stream — you never call `GetId`.

### API key (`MRPC_API_KEY`)
Your MetaRPC authentication token, sent on every call. Provide it via the `MRPC_API_KEY` env var or `api_key` in `settings.json`. Get one at [mrpc.pro/my](https://mrpc.pro/my).

### `Settings`
The config loader ([`src/config.rs`](https://github.com/MetaRPC/RustMT4/blob/main/src/config.rs)): reads `settings.json` and overlays env vars (`MT4_LOGIN`, `MT4_PASSWORD`, `MT4_SERVER`, `MT4_SYMBOL`, `MT4_NAME`, `MRPC_API_KEY` — env wins). Fields: `user`, `password`, `server_name`, `name`, `api_key`, `test_symbol`.

---

## 🧰 SDK conveniences

| Term | What it is |
|------|-----------|
| **Auto-normalization** | `normalize_lot` / `normalize_price` round to the symbol's step/digits so the broker accepts the order. |
| **Risk-based sizing** | `calc_lot_by_risk(symbol, cash, sl_pips)` → the normalized lot that risks `cash` over the stop. |
| **Pip helpers** | `pip_size`, `pips_to_price`, `price_to_pips`, `spread_pips`. |
| **Watchers / Waiters** | Watchers stream values onto a channel; waiters poll until a condition holds ([8. Watchers & Waiters](../MT4Sugar/8.%20Watchers_Waiters.md)). |
| **`ensure_symbol`** | Select a symbol and wait for a usable two-sided quote before trading it. |

---

**See also:** [Getting Started](GETTING_STARTED.md) · [Project Map](PROJECT_MAP.md) · [ENUMs Reference](ENUMS_USAGE_REFERENCE.md) · [Return Codes](RETURN_CODES_REFERENCE.md)
