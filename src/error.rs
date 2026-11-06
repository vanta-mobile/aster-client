//! Error types shared by every request.

use std::fmt;
use std::time::Duration;

/// Result alias used throughout the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Business error returned by Aster (`{"code": -1121, "msg": "Invalid symbol."}`).
///
/// `code` is the exchange code, not the HTTP status: Aster also reports errors
/// with HTTP 200 (for example `placeStrategyOrder` returns `failureCode` in the
/// body), so always inspect this before trusting a successful status.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ApiError {
    /// Error or acknowledgement code.
    pub code: i32,
    /// Message.
    pub msg: String,
}

impl ApiError {
    /// Create a new value.
    pub fn new(code: i32, msg: impl Into<String>) -> Self {
        Self {
            code,
            msg: msg.into(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.msg, self.code)
    }
}

impl std::error::Error for ApiError {}

/// Every failure mode of the client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The exchange answered with a `code`/`msg` payload.
    #[error("api error: {0}")]
    Api(#[from] ApiError),

    /// The request may or may not have been executed (HTTP 5xx / 503 / I/O
    /// timeout). Aster documents 503 explicitly as *unknown execution state*:
    /// never blind-retry a write, re-query it with `newClientOrderId` first.
    #[error("execution state unknown (http {status}): {message}")]
    UnknownState {
        /// HTTP status.
        status: u16,
        /// Raw response body.
        message: String,
    },

    /// Rate limit (`429`) or IP ban (`418`). Honour `retry_after` before retrying.
    #[error("rate limited (http {status}, retry after {retry_after:?})")]
    RateLimited {
        /// HTTP status (`429` or `418`).
        status: u16,
        /// Server-provided `Retry-After`, when present.
        retry_after: Option<Duration>,
        /// Raw response body.
        message: String,
    },

    /// Transport-level failure: connection refused, TLS, DNS, timeout.
    #[error("transport error: {0}")]
    Transport(String),

    /// Non-2xx response that carried no `code`/`msg` payload.
    #[error("http {status}: {body}")]
    Http {
        /// HTTP status.
        status: u16,
        /// Raw response body.
        body: String,
    },

    /// A response body could not be decoded into the expected type.
    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),
    /// Decode.

    /// Signing key / EIP-712 encoding failure.
    #[error("signing error: {0}")]
    Signing(String),
    /// Signing.

    /// An authenticated endpoint was called on a client without credentials.
    #[error("this endpoint requires credentials; build the client with `Client::signer`")]
    MissingCredentials,
    /// MissingCredentials.

    /// InvalidParam.
    /// A request that would be rejected server-side, caught locally
    /// (e.g. `LIMIT` without a price). Mirrors `-1102 MANDATORY_PARAM_EMPTY`.
    #[error("invalid parameter: {0}")]
    InvalidParam(&'static str),
}

impl Error {
    /// True when the request may have reached the matching engine and its
    /// outcome is unknown. Callers must reconcile by `newClientOrderId`.
    pub fn is_execution_unknown(&self) -> bool {
        matches!(self, Error::UnknownState { .. })
    }

    /// True for `429`/`418`.
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Error::RateLimited { .. })
    }

    /// Exchange error code, when the failure came from the API itself.
    pub fn code(&self) -> Option<i32> {
        match self {
            Error::Api(e) => Some(e.code),
            _ => None,
        }
    }
}

/// Documented error codes that drive control flow.
pub mod codes {
    /// `-1000`: unknown server error.
    pub const UNKNOWN: i32 = -1000;
    /// `-1022`: invalid signature.
    pub const INVALID_SIGNATURE: i32 = -1022;
    /// `-1102`: mandatory parameter missing or malformed.
    pub const MANDATORY_PARAM_EMPTY: i32 = -1102;
    /// `-1115`: invalid time in force.
    pub const INVALID_TIF: i32 = -1115;
    /// `-2013`: order does not exist.
    pub const NO_SUCH_ORDER: i32 = -2013;
    /// `-2021`: conditional order would trigger immediately.
    pub const ORDER_WOULD_IMMEDIATELY_TRIGGER: i32 = -2021;
    /// `-4225`: nonce already used or expired.
    pub const NONCE_EXPIRED: i32 = -4225;
    /// `-4141`: symbol is closed for trading.
    pub const SYMBOL_CLOSED: i32 = -4141;
    /// `-4142`: strategy leg would trigger immediately.
    pub const STRATEGY_INVALID_TRIGGER_PRICE: i32 = -4142;
    /// `-4164`: notional below the symbol minimum.
    pub const MIN_NOTIONAL: i32 = -4164;
    /// `-4212`: account is not in multi-assets margin mode.
    pub const NOT_JOINT_MARGIN: i32 = -4212;
    /// `-5049`: builder queries are not enabled for this account.
    pub const BUILDER_QUERY_DISABLED: i32 = -5049;
    /// `-5050`: the account must deposit before using authenticated endpoints.
    pub const DEPOSIT_REQUIRED: i32 = -5050;
    /// `-5052`: privacy mode blocks the operation.
    pub const PRIVACY_MODE: i32 = -5052;
    /// `-5053`: user is not bound to this builder.
    pub const NOT_BOUND_TO_BUILDER: i32 = -5053;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_parses_body() {
        let e: ApiError =
            serde_json::from_str(r#"{"code":-1121,"msg":"Invalid symbol."}"#).unwrap();
        assert_eq!(e.code, -1121);
        assert_eq!(e.msg, "Invalid symbol.");
    }
}
