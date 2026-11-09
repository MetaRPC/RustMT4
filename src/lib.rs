//! MetaRPC MT4 Rust SDK.
//!
//! A three-layer async client talking gRPC+TLS to `mt4.mrpc.pro:443`:
//!
//! * [`MT4Account`] — low-level: raw protobuf `…Request` in, `…Data` out; owns the channel,
//!   TLS, request metadata, retry and reconnect.
//! * [`MT4Service`] — mid-level: friendly arguments and clean return values over `MT4Account`.
//! * [`MT4Sugar`] — high-level: one-liners, pip math, risk sizing, trailing, watchers.
//!
//! [`strategies`] bundles ready-made orchestrators and presets. [`pb`] exposes the raw
//! prost/tonic generated types (`pb::mt4_term_api::*`) for anything the layers do not cover.

pub mod config;
pub mod error;
pub mod models;
pub mod mt4;
pub mod pb;
pub mod strategies;

pub use config::Settings;
pub use error::MT4Error;
pub use models::*;
pub use mt4::account::MT4Account;
pub use mt4::service::MT4Service;
pub use mt4::sugar::MT4Sugar;
pub use pb::mt4_term_api;
