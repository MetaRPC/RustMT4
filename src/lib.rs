//! MetaRPC MT4 Rust SDK.
//!
//! * [`MT4Client`] — high-level async client (ConnectEx, account, symbols, quotes, bars, tick
//!   stream, OrderSend / OrderModify / OrderCloseDelete, history) talking gRPC+TLS to
//!   `mt4.mrpc.pro:443`.
//! * [`pb`] — the raw prost/tonic generated types (`pb::mt4_term_api::*`) for anything the
//!   high-level wrappers do not cover.

pub mod client;
pub mod error;
pub mod models;
pub mod pb;

pub use client::{timeframe, ConnectParams, MT4Client};
pub use error::MT4Error;
pub use models::*;
pub use pb::mt4_term_api;
