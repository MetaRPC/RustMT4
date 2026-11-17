use thiserror::Error;

use crate::pb::mt4_term_api as pb;

/// Every failure the SDK can surface.
///
/// * `Grpc` — transport / HTTP2 level (`tonic::Status`). `Unavailable` and `DeadlineExceeded`
///   are retried by the client's `call` wrapper (reads only, never trades).
/// * `TradeOutcomeUnknown` — a trade call (`OrderSend`, `OrderModify`, `OrderCloseDelete`,
///   `OrderCloseBy`) failed with gRPC `Unavailable`, `DeadlineExceeded`, `Unknown` or `Internal`
///   after the request may have reached the terminal. **Never re-sent**: the trade may or may
///   not have been executed — check opened orders before retrying. `api_code()` is
///   `"TRADE_OUTCOME_UNKNOWN"`, `grpc_status()` keeps the original status.
/// * `Api` — the gateway answered with its `Error` oneof (`error_code`, `error_message`,
///   MQL error code / description). `TERMINAL_INSTANCE_NOT_FOUND` triggers one reconnect + retry.
/// * `InvalidLogin` — `CONNECT_EX_INVALID_LOGIN_ERROR` and friends. **Never retried**: repeated
///   bad logins are what brokers lock accounts for.
/// * `TradeError` — `OrderSend` / `OrderModify` / `OrderCloseDelete` failed on the MQL side
///   (`code` = MQL4 error such as 130 invalid stops, 132 market closed, 134 not enough money,
///   138 requote) or the gateway reported `order_was_modified == false`.
#[derive(Error, Debug)]
pub enum MT4Error {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Authentication failed for user: {0}")]
    AuthError(u64),
    #[error("Trade failed with code {0}: {1}")]
    TradeError(u32, String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("MetaRPC API error {code}: {message} (mql={mql_code} {mql_desc})")]
    Api {
        code: String,
        message: String,
        mql_code: i32,
        mql_desc: String,
        error_type: i32,
    },
    #[error("invalid login (not retried): {0}")]
    InvalidLogin(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("not connected — call connect_ex() first")]
    NotConnected,
    #[error(
        "TRADE_OUTCOME_UNKNOWN: {operation} failed with gRPC {} after the request may have reached the terminal; \
         the trade may or may not have been executed. It was NOT re-sent automatically: check open positions/orders \
         before retrying. ({})",
        grpc_code_name(.status.code()),
        .status.message()
    )]
    TradeOutcomeUnknown {
        /// The trade RPC, e.g. `"OrderSend"`.
        operation: String,
        /// The original transport status.
        #[source]
        status: tonic::Status,
    },
}

/// Error code of [`MT4Error::TradeOutcomeUnknown`].
pub const TRADE_OUTCOME_UNKNOWN: &str = "TRADE_OUTCOME_UNKNOWN";

/// Upper-case gRPC code name, e.g. `UNAVAILABLE`.
fn grpc_code_name(c: tonic::Code) -> String {
    match c {
        tonic::Code::Unavailable => "UNAVAILABLE".into(),
        tonic::Code::DeadlineExceeded => "DEADLINE_EXCEEDED".into(),
        tonic::Code::Unknown => "UNKNOWN".into(),
        tonic::Code::Internal => "INTERNAL".into(),
        other => format!("{other:?}").to_ascii_uppercase(),
    }
}

fn is_api_key_refused_text(s: &str) -> bool {
    s.contains("API_KEY_INVALID") || s.contains("API_KEY_REQUIRED")
}

impl MT4Error {
    /// Turn a transport failure of a trade call into [`MT4Error::TradeOutcomeUnknown`] when the
    /// request may already have been executed (`Unavailable`, `DeadlineExceeded`, `Unknown`,
    /// `Internal`); any other error is returned unchanged.
    pub(crate) fn into_trade_outcome(self, operation: &str) -> Self {
        match self {
            MT4Error::Grpc(status)
                if matches!(
                    status.code(),
                    tonic::Code::Unavailable
                        | tonic::Code::DeadlineExceeded
                        | tonic::Code::Unknown
                        | tonic::Code::Internal
                ) && !is_api_key_refused_text(status.message()) =>
            {
                MT4Error::TradeOutcomeUnknown { operation: operation.to_string(), status }
            }
            other => other,
        }
    }

    /// `true` for [`MT4Error::TradeOutcomeUnknown`]: the trade may or may not have been executed.
    pub fn is_trade_outcome_unknown(&self) -> bool {
        matches!(self, MT4Error::TradeOutcomeUnknown { .. })
    }

    /// The gRPC status behind a `Grpc` or `TradeOutcomeUnknown` error.
    pub fn grpc_status(&self) -> Option<&tonic::Status> {
        match self {
            MT4Error::Grpc(s) | MT4Error::TradeOutcomeUnknown { status: s, .. } => Some(s),
            _ => None,
        }
    }

    /// The API key was refused (`API_KEY_INVALID` / `API_KEY_REQUIRED`). Never retried.
    pub fn is_api_key_refused(&self) -> bool {
        match self {
            MT4Error::Api { code, message, .. } => is_api_key_refused_text(code) || is_api_key_refused_text(message),
            MT4Error::Grpc(s) => is_api_key_refused_text(s.message()),
            _ => false,
        }
    }

    pub fn from_api(e: pb::Error) -> Self {
        let code_up = e.error_code.to_ascii_uppercase();
        let msg_up = e.error_message.to_ascii_uppercase();
        if code_up.contains("INVALID_LOGIN")
            || code_up.contains("WRONG_PASSWORD")
            || code_up.contains("INVALID_PASSWORD")
            || code_up.contains("INVALID_ACCOUNT")
            || msg_up.contains("INVALID LOGIN")
            || msg_up.contains("INVALID ACCOUNT")
            || msg_up.contains("WRONG PASSWORD")
        {
            return MT4Error::InvalidLogin(format!("{}: {}", e.error_code, e.error_message));
        }
        MT4Error::Api {
            code: e.error_code,
            message: e.error_message,
            mql_code: e.mql_error_int_code,
            mql_desc: e.mql_error_description,
            error_type: e.r#type,
        }
    }

    /// Gateway lost the terminal (pod restart, idle eviction). Re-`ConnectEx` with the same id
    /// and retry the call.
    pub fn is_terminal_not_found(&self) -> bool {
        match self {
            MT4Error::Api { code, .. } => {
                code.contains("TERMINAL_INSTANCE_NOT_FOUND")
                    || code.contains("TERMINAL_REGISTRY_TERMINAL_NOT_FOUND")
            }
            _ => false,
        }
    }

    /// Transient transport failure — safe to retry a **read** after a back-off. Never `true` for
    /// [`MT4Error::TradeOutcomeUnknown`] or a refused API key.
    pub fn is_transient(&self) -> bool {
        match self {
            MT4Error::Grpc(s) => {
                matches!(
                    s.code(),
                    tonic::Code::Unavailable | tonic::Code::DeadlineExceeded | tonic::Code::Cancelled
                ) && !is_api_key_refused_text(s.message())
            }
            MT4Error::Transport(_) => true,
            _ => false,
        }
    }

    pub fn is_retryable(&self) -> bool {
        self.is_transient() || self.is_terminal_not_found()
    }

    /// The gateway's error code string (`"TRADE_OUTCOME_UNKNOWN"` for
    /// [`MT4Error::TradeOutcomeUnknown`], empty for other non-API errors).
    pub fn api_code(&self) -> &str {
        match self {
            MT4Error::Api { code, .. } => code,
            MT4Error::TradeOutcomeUnknown { .. } => TRADE_OUTCOME_UNKNOWN,
            _ => "",
        }
    }

    /// MQL4 error number if this is a trade/MQL execution error (`mql_code` of `Api`, or the
    /// code of `TradeError`), else 0.
    pub fn mql_code(&self) -> i32 {
        match self {
            MT4Error::Api { mql_code, .. } => *mql_code,
            MT4Error::TradeError(c, _) => *c as i32,
            _ => 0,
        }
    }

    /// `true` for MQL4 error 132 (market closed) or a message saying so.
    pub fn is_market_closed(&self) -> bool {
        if self.mql_code() == 132 {
            return true;
        }
        self.to_string().to_ascii_lowercase().contains("market is closed")
            || self.to_string().to_ascii_lowercase().contains("market closed")
    }
}
