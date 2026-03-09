use crate::myuplink::auth::TokenManager;
use crate::myuplink::error::MyUplinkError;
use crate::myuplink::models::DeviceInfo;
use std::sync::Arc;
use std::time::Duration;

/// API client for myUplink REST API.
pub struct MyUplinkClient {
    /// HTTP client (reused across requests).
    http_client: reqwest::Client,
    /// Token manager for `OAuth2`.
    token_manager: Arc<TokenManager>,
    /// Base API URL (e.g., `<https://api.myuplink.com/v2>`).
    base_url: String,
    /// API version ("v2" or "v3").
    api_version: String,
}

impl MyUplinkClient {
    /// Create a new myUplink API client.
    ///
    /// # Errors
    /// Returns `MyUplinkError::InvalidApiVersion` if version is not "v2" or "v3".
    pub fn new(
        client_id: String,
        client_secret: String,
        api_version: String,
    ) -> Result<Self, MyUplinkError> {
        // Validate API version at parse time
        if api_version != "v2" && api_version != "v3" {
            return Err(MyUplinkError::InvalidApiVersion(api_version));
        }

        let base_url = format!("https://api.myuplink.com/{api_version}");
        let token_url = format!("{base_url}/oauth/token");

        let token_manager = Arc::new(TokenManager::new(client_id, client_secret, token_url));

        Ok(Self {
            http_client: reqwest::Client::new(),
            token_manager,
            base_url,
            api_version,
        })
    }

    /// Fetch devices and parameters from myUplink API.
    ///
    /// Makes authenticated request to `/api/v2/devices` or `/api/v3/devices`.
    /// Handles 401 by invalidating token and retrying once.
    /// Handles 429 by returning error with `retry_after` if present.
    ///
    /// # Errors
    ///
    /// Returns `MyUplinkError` for network errors, authentication failures, or API errors.
    pub async fn fetch_devices(&self) -> Result<Vec<DeviceInfo>, MyUplinkError> {
        self.fetch_devices_internal(false).await
    }

    /// Internal fetch with retry handling.
    fn fetch_devices_internal(
        &self,
        retry: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<DeviceInfo>, MyUplinkError>> + Send + '_>,
    > {
        Box::pin(async move {
            let token = self.token_manager.get_token().await?;
            let url = format!("{}/devices", self.base_url);

            let response = self
                .http_client
                .get(&url)
                .header("Authorization", format!("Bearer {token}"))
                .timeout(Duration::from_secs(30))
                .send()
                .await
                .map_err(|e| MyUplinkError::Network(e.to_string()))?;

            let status = response.status();

            // Handle 401: invalidate token and retry once
            if status.as_u16() == 401 {
                if !retry {
                    self.token_manager.invalidate().await;
                    return self.fetch_devices_internal(true).await;
                }
                return Err(MyUplinkError::Unauthorized);
            }

            // Handle 429: extract Retry-After if present
            if status.as_u16() == 429 {
                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());
                return Err(MyUplinkError::RateLimited { retry_after });
            }

            // Handle other errors
            if !status.is_success() {
                return Err(MyUplinkError::Http {
                    status: status.as_u16(),
                });
            }

            let devices: Vec<DeviceInfo> = response
                .json()
                .await
                .map_err(|e| MyUplinkError::ParseError(e.to_string()))?;

            Ok(devices)
        })
    }

    /// Get API version.
    #[must_use]
    pub fn api_version(&self) -> &str {
        &self.api_version
    }

    /// Get base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_api_version() {
        let result = MyUplinkClient::new("id".to_string(), "secret".to_string(), "v1".to_string());
        assert!(matches!(result, Err(MyUplinkError::InvalidApiVersion(_))));
    }

    #[test]
    fn test_valid_api_versions() {
        let v2 = MyUplinkClient::new("id".to_string(), "secret".to_string(), "v2".to_string());
        assert!(v2.is_ok());

        let v3 = MyUplinkClient::new("id".to_string(), "secret".to_string(), "v3".to_string());
        assert!(v3.is_ok());
    }

    #[test]
    fn test_base_url_construction() {
        let client =
            MyUplinkClient::new("id".to_string(), "secret".to_string(), "v2".to_string()).unwrap();
        assert_eq!(client.base_url(), "https://api.myuplink.com/v2");
        assert_eq!(client.api_version(), "v2");
    }
}
