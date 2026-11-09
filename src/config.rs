//! Connection settings — load from a JSON file and/or environment variables.
//!
//! [`Settings::load`] reads a `settings.json` (if present) then overlays environment variables
//! (env wins). File path resolution: `MRPC_SETTINGS`, else `examples/0_common/settings.json`,
//! else `settings.json`. Env overlay: `MT4_LOGIN`, `MT4_PASSWORD`, `MT4_SERVER`, `MT4_SYMBOL`,
//! `MT4_NAME`, `MRPC_API_KEY`. If `api_key` is set and `MRPC_API_KEY` is not already in the
//! environment, it is exported so [`crate::MT4Account::new`] picks it up.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::MT4Error;

/// Connection settings for an MT4 account.
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// MT4 account login number.
    pub user: u64,
    /// MT4 account password.
    #[serde(default)]
    pub password: String,
    /// Broker server / cluster name (e.g. `"FreshForex-Demo"`).
    #[serde(default)]
    pub server_name: String,
    /// Optional stable terminal label (so re-runs reuse the same terminal).
    #[serde(default)]
    pub name: Option<String>,
    /// MetaRPC API key; exported as `MRPC_API_KEY` for the SDK when set.
    #[serde(default)]
    pub api_key: Option<String>,
    /// A default symbol for the examples (e.g. `"EURUSD"`).
    #[serde(default = "default_symbol")]
    pub test_symbol: String,
}

fn default_symbol() -> String {
    "EURUSD".to_string()
}

impl Settings {
    /// Load from the resolved `settings.json` (if any) and overlay environment variables.
    pub fn load() -> Result<Self, MT4Error> {
        let explicit = std::env::var("MRPC_SETTINGS").ok().map(PathBuf::from);
        Self::from_sources(explicit)
    }

    /// Load from a specific JSON file, then overlay environment variables.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, MT4Error> {
        Self::from_sources(Some(path.as_ref().to_path_buf()))
    }

    fn from_sources(explicit: Option<PathBuf>) -> Result<Self, MT4Error> {
        let candidates: Vec<PathBuf> = match explicit {
            Some(p) => vec![p],
            None => vec![PathBuf::from("examples/0_common/settings.json"), PathBuf::from("settings.json")],
        };

        let mut value = serde_json::Value::Object(Default::default());
        for p in &candidates {
            if p.exists() {
                let txt = std::fs::read_to_string(p)
                    .map_err(|e| MT4Error::Protocol(format!("reading {}: {e}", p.display())))?;
                value =
                    serde_json::from_str(&txt).map_err(|e| MT4Error::Protocol(format!("parsing {}: {e}", p.display())))?;
                break;
            }
        }

        let obj = value
            .as_object_mut()
            .ok_or_else(|| MT4Error::Protocol("settings.json must be a JSON object".to_string()))?;

        // env overlay (env wins)
        if let Ok(x) = std::env::var("MT4_LOGIN") {
            let n: u64 = x.parse().map_err(|_| MT4Error::Protocol("MT4_LOGIN must be a number".to_string()))?;
            obj.insert("user".to_string(), serde_json::json!(n));
        }
        if let Ok(x) = std::env::var("MT4_PASSWORD") {
            obj.insert("password".to_string(), serde_json::json!(x));
        }
        if let Ok(x) = std::env::var("MT4_SERVER") {
            obj.insert("server_name".to_string(), serde_json::json!(x));
        }
        if let Ok(x) = std::env::var("MT4_SYMBOL") {
            obj.insert("test_symbol".to_string(), serde_json::json!(x));
        }
        if let Ok(x) = std::env::var("MT4_NAME") {
            obj.insert("name".to_string(), serde_json::json!(x));
        }
        if let Ok(x) = std::env::var("MRPC_API_KEY") {
            obj.insert("api_key".to_string(), serde_json::json!(x));
        }

        let settings: Settings = serde_json::from_value(value).map_err(|e| {
            MT4Error::Protocol(format!("invalid settings (need at least `user`; set it in settings.json or MT4_LOGIN): {e}"))
        })?;

        if let Some(key) = &settings.api_key {
            if std::env::var("MRPC_API_KEY").is_err() {
                std::env::set_var("MRPC_API_KEY", key);
            }
        }

        Ok(settings)
    }
}
