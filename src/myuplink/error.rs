/// Errors from myUplink API interactions.
#[derive(Debug, thiserror::Error)]
pub enum MyUplinkError {
    /// HTTP request failed.
    #[error("HTTP error: {status}")]
    Http {
        /// HTTP status code.
        status: u16,
    },

    /// Authentication failed (401 Unauthorized).
    #[error("Authentication failed")]
    Unauthorized,

    /// Rate limited (429 Too Many Requests).
    #[error("Rate limited by API")]
    RateLimited {
        /// Seconds to wait before retrying (from Retry-After header).
        retry_after: Option<u64>,
    },

    /// Token refresh failed after retries.
    #[error("Failed to refresh token after retries")]
    TokenRefreshFailed,

    /// Request parsing or serialization failed.
    #[error("Request/response parsing failed: {0}")]
    ParseError(String),

    /// Configuration is invalid.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid API version (must be "v2" or "v3").
    #[error("Invalid API version: {0}. Must be 'v2' or 'v3'")]
    InvalidApiVersion(String),

    /// Device not found or unreachable.
    #[error("Device error: {0}")]
    DeviceError(String),

    /// Network or I/O error.
    #[error("Network error: {0}")]
    Network(String),
}

impl MyUplinkError {
    /// Check if error is retryable.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::Network(_) | Self::Unauthorized
        )
    }

    /// Check if error is a rate limit.
    #[must_use]
    pub const fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited { .. })
    }
}
