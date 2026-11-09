# 🗺️ RustMT4 Project Map

> The whole project as one annotated tree, with links. **Links point to section *overviews*, never to individual method pages** — open an overview to drill down into its methods.

---

## 🧱 The three layers

RustMT4 is built in three decoupled layers (composition, not inheritance). Your code can enter at any level.

| Layer | Source | What it does | Overview |
| ----- | ------ | ------------ | -------- |
| **3 · MT4Sugar** | `src/mt4/sugar.rs` | Convenience on top of Service: risk-based sizing, pip-based SL/TP, price/volume helpers, trailing & breakeven, watchers, batch close, order queries. | [MT4Sugar overview](../MT4Sugar/MT4Sugar.Master.Overview.md) |
| **2 · MT4Service** | `src/mt4/service.rs` | Typed facade: `&str` symbols and typed-enum args; simple getters return clean scalars / `Vec`. What most code uses. | [MT4Service overview](../MT4Service/MT4Service.Overview.md) |
| **1 · MT4Account** | `src/mt4/account.rs` | Low-level gRPC: builds each proto `…Request`, attaches auth metadata, unwraps replies to `Result<…Data, MT4Error>`; connection & streaming. Everything builds on it. | [MT4Account overview](../MT4Account/MT4Account.Master.Overview.md) |

```text
Your code → MT4Sugar → MT4Service → MT4Account → gRPC → MetaRPC gateway → MT4 terminal
```

Mix freely: `sugar.service()` borrows the Service, `service.account()` borrows the Account.

---

## 🌳 Repository tree

- **`RustMT4/`** — crate root (`metarpc-mt4`)
  - **`src/`** — the library
    - `lib.rs` — crate root; re-exports `MT4Account` / `MT4Service` / `MT4Sugar` / `Settings` / `MT4Error`, the DTOs, and the `strategies` module
    - `pb.rs` — `include!`s the vendored stubs as `mt4_term_api`
    - `config.rs` — the `Settings` loader (`settings.json` + env overlay)
    - `error.rs` — the `MT4Error` enum + `From` impls → [Return Codes](RETURN_CODES_REFERENCE.md)
    - `models.rs` — plain DTOs reused by Sugar (`Quote`, `Bar`, `OrderRequest`, `OrderResult`, `CloseResult`, `AccountInfo`, `OrderType`) → [Glossary](GLOSSARY.md)
    - **`mt4/`** — the three SDK layers
      - `account.rs` — **Layer 1** source → [MT4Account overview](../MT4Account/MT4Account.Master.Overview.md)
      - `service.rs` — **Layer 2** source → [MT4Service overview](../MT4Service/MT4Service.Overview.md)
      - `sugar.rs` — **Layer 3** source → [MT4Sugar overview](../MT4Sugar/MT4Sugar.Master.Overview.md)
      - `mod.rs` — `pub mod account; service; sugar;`
    - **`strategies/`** — orchestrators + presets → [Strategies overview](../Strategies/Strategies.Master.Overview.md)
      - `mod.rs` — `StrategyPreset`, `RiskPreset`, `size_lot`, `manage`, presets catalog
      - `market_one_shot.rs` · `bracket_trailing.rs` · `pending_bracket.rs`
  - **`package/`** — the raw gRPC contract
    - `mt4_term_api.rs` — vendored prost/tonic generated stubs (no `build.rs`)
  - **`examples/`** — runnable demos (`cargo run --example …`), numbered by layer
    - `quickstart.rs` — the 10-minute path · `menu.rs` — interactive launcher
    - `0_common/` — `settings.json` loaded by the examples
    - **`1_lowlevel/`** → [MT4Account overview](../MT4Account/MT4Account.Master.Overview.md)
    - **`2_service/`** → [MT4Service overview](../MT4Service/MT4Service.Overview.md)
    - **`3_sugar/`** → [MT4Sugar overview](../MT4Sugar/MT4Sugar.Master.Overview.md)
    - **`4_strategies/`** → [Strategies overview](../Strategies/Strategies.Master.Overview.md)
  - **`tests/`** — offline unit tests (construction, DTO mapping, no network)
  - `Cargo.toml` — crate manifest · `mkdocs.yml` — docs site config

---

## 📚 Documentation tree

- **`docs/`**
  - [Home](../index.md) — landing page
  - **`All_Guides/`** — the guides ([Getting Started](GETTING_STARTED.md), [Your First Project](Your_First_Project.md), [MT4 for Beginners](MT4_For_Beginners.md), [Glossary](GLOSSARY.md), [Async Model](SYNC_VS_ASYNC.md), [gRPC Streaming](GRPC_STREAM_MANAGEMENT.md), [ENUMs](ENUMS_USAGE_REFERENCE.md), [Return Codes](RETURN_CODES_REFERENCE.md))
  - **`MT4Account/`** — low-level gRPC docs → [overview](../MT4Account/MT4Account.Master.Overview.md) *(per-method pages inside)*
  - **`MT4Service/`** — typed facade docs → [overview](../MT4Service/MT4Service.Overview.md) *(per-group pages inside)*
  - **`MT4Sugar/`** — convenience API docs → [overview](../MT4Sugar/MT4Sugar.Master.Overview.md) *(per-group pages inside)*
  - **`Strategies/`** — orchestrators + presets → [overview](../Strategies/Strategies.Master.Overview.md) *(per-strategy pages inside)*
  - `styles/` · `javascripts/` · `includes/` · `Guide_Images/` — assets & theme

> The `MT4Account/`, `MT4Service/`, `MT4Sugar/` and `Strategies/` folders hold per-method / per-group pages. By design this map links only their **overviews** — each overview is the index into its own detail pages.

---

## 🎯 Which layer should you use?

- **Most trading apps → [MT4Sugar](../MT4Sugar/MT4Sugar.Master.Overview.md).** Risk sizing, pip math and bulk ops out of the box.
- **Custom execution → [MT4Service](../MT4Service/MT4Service.Overview.md).** Typed calls when you manage your own request shapes.
- **Protocol-level control → [MT4Account](../MT4Account/MT4Account.Master.Overview.md).** Raw proto structures and streams.
