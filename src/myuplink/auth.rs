use crate::myuplink::error::MyUplinkError;
use crate::myuplink::models::TokenResponse;
use tokio::sync::RwLock;
use tracing::{debug, error};

/// Token state: (token, `expires_at_ms`).
#[derive(Clone, Debug)]
struct TokenState {
    token: String,
    expires_at_ms: u64,
}

impl TokenState {
    /// Check if this token is still valid (with 30s buffer).
    fn is_valid(&self) -> bool {
        #[allow(clippy::cast_possible_truncation)]
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        // Refresh 30 seconds before actual expiry
        self.expires_at_ms > now_ms + 30_000
    }
}

/// Manages `OAuth2` tokens with automatic refresh and double-check locking.
pub struct TokenManager {
    /// Shared token state with double-check lock pattern.
    inner: RwLock<Option<TokenState>>,
    /// Client ID and secret.
    client_id: String,
    client_secret: String,
    /// myUplink `OAuth2` token endpoint.
    token_url: String,
}

impl TokenManager {
    /// Create a new token manager.
    #[must_use]
    pub fn new(client_id: String, client_secret: String, token_url: String) -> Self {
        Self {
            inner: RwLock::new(None),
            client_id,
            client_secret,
            token_url,
        }
    }

    /// Get a valid token, refreshing if necessary (double-check lock pattern).
    ///
    /// # Errors
    ///
    /// Returns `MyUplinkError` if token refresh fails.
    pub async fn get_token(&self) -> Result<String, MyUplinkError> {
        // Fast path: read lock check
        {
            let state = self.inner.read().await;
            if let Some(token_state) = state.as_ref() {
                if token_state.is_valid() {
                    debug!("Using cached OAuth token (still valid)");
                    return Ok(token_state.token.clone());
                }
            }
        }

        debug!("Token not cached or expired, refreshing...");

        // Slow path: write lock + double-check
        {
            let state = self.inner.write().await;
            if let Some(token_state) = state.as_ref() {
                if token_state.is_valid() {
                    debug!("Using cached OAuth token from write lock");
                    return Ok(token_state.token.clone());
                }
            }
        }

        // Token is invalid or missing; refresh it
        debug!("Calling OAuth2 refresh endpoint: {}", &self.token_url);
        let response = self.refresh_token().await?;
        debug!("Successfully refreshed OAuth token (expires_in: {}s)", response.expires_in);

        #[allow(clippy::cast_possible_truncation)]
        let expires_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
            + (response.expires_in * 1000);

        let new_state = TokenState {
            token: response.access_token.clone(),
            expires_at_ms,
        };
        *self.inner.write().await = Some(new_state);
        Ok(response.access_token)
    }

    /// Refresh the token from the `OAuth2` endpoint.
    async fn refresh_token(&self) -> Result<TokenResponse, MyUplinkError> {
        let client = reqwest::Client::new();
        let mut params = std::collections::HashMap::new();
        params.insert("grant_type", "client_credentials".to_string());
        params.insert("client_id", self.client_id.clone());
        params.insert("client_secret", self.client_secret.clone());

        debug!("POST {} with client_id: {}", &self.token_url, &self.client_id);
        let response = client
            .post(&self.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                error!("Network error calling OAuth token endpoint {}: {}", &self.token_url, e);
                MyUplinkError::Network(e.to_string())
            })?;

        let status = response.status();
        debug!("OAuth token response status: {}", status.as_u16());

        if !status.is_success() {
            // Try to get error details from response body
            let error_body = response.text().await.unwrap_or_else(|_| "(no body)".to_string());
            error!("OAuth token endpoint returned {} from {}", status.as_u16(), &self.token_url);
            error!("Response body: {}", error_body);
            return Err(MyUplinkError::Http {
                status: status.as_u16(),
            });
        }

        let text = response.text().await.map_err(|e| {
            error!("Failed to read OAuth token response body: {}", e);
            MyUplinkError::ParseError(e.to_string())
        })?;

        debug!("OAuth response body: {}", text);

        serde_json::from_str::<TokenResponse>(&text)
            .map_err(|e| {
                error!("Failed to parse OAuth token response: {}", e);
                MyUplinkError::ParseError(e.to_string())
            })
    }

    /// Invalidate the cached token (force refresh on next request).
    pub async fn invalidate(&self) {
        *self.inner.write().await = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_state_validity() {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Valid token (expires in 1 minute)
        let valid = TokenState {
            token: "valid_token".to_string(),
            expires_at_ms: now_ms + 60_000,
        };
        assert!(valid.is_valid());

        // Expired token
        let expired = TokenState {
            token: "expired_token".to_string(),
            expires_at_ms: now_ms - 1000,
        };
        assert!(!expired.is_valid());

        // Expiring soon (within 30s buffer)
        let expiring_soon = TokenState {
            token: "expiring_soon".to_string(),
            expires_at_ms: now_ms + 20_000,
        };
        assert!(!expiring_soon.is_valid());
    }
}
