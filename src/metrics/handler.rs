use crate::metrics::encode::encode_metrics;
use crate::metrics::mapping::map_parameter_to_samples;
use crate::myuplink::client::MyUplinkClient;
use crate::myuplink::error::MyUplinkError;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use tokio::time::{MissedTickBehavior, interval};
use tracing::{debug, error, warn};

/// Metrics store with caching and counters.
pub struct MetricsStore {
    /// Cached metrics (Arc for cheap cloning by readers).
    body: RwLock<Arc<String>>,
    /// Total poll attempts.
    polls_total: AtomicU64,
    /// Total scrape errors.
    scrape_errors_total: AtomicU64,
    /// Total auth failures (401s).
    auth_failures_total: AtomicU64,
    /// Total rate limit hits (429s).
    rate_limited_total: AtomicU64,
}

impl MetricsStore {
    /// Create a new metrics store with empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            body: RwLock::new(Arc::new("# EOF\n".to_string())),
            polls_total: AtomicU64::new(0),
            scrape_errors_total: AtomicU64::new(0),
            auth_failures_total: AtomicU64::new(0),
            rate_limited_total: AtomicU64::new(0),
        }
    }

    /// Get current metrics (cheap clone of Arc<String>).
    pub async fn get_metrics(&self) -> Arc<String> {
        self.body.read().await.clone()
    }

    /// Update cached metrics and increment poll counter.
    pub async fn update_metrics(&self, body: String) {
        *self.body.write().await = Arc::new(body);
        self.polls_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment error counter.
    fn increment_errors(&self) {
        self.scrape_errors_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment auth failure counter.
    fn increment_auth_failures(&self) {
        self.auth_failures_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment rate limit counter.
    fn increment_rate_limited(&self) {
        self.rate_limited_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Get poll counter value.
    pub fn polls_total(&self) -> u64 {
        self.polls_total.load(Ordering::Relaxed)
    }

    /// Get error counter value.
    pub fn scrape_errors_total(&self) -> u64 {
        self.scrape_errors_total.load(Ordering::Relaxed)
    }

    /// Get auth failure counter value.
    pub fn auth_failures_total(&self) -> u64 {
        self.auth_failures_total.load(Ordering::Relaxed)
    }

    /// Get rate limit counter value.
    pub fn rate_limited_total(&self) -> u64 {
        self.rate_limited_total.load(Ordering::Relaxed)
    }

    /// Fetch metrics from myUplink and encode to `OpenMetrics` text format.
    ///
    /// # Errors
    ///
    /// Returns `MyUplinkError` if device fetch fails.
    pub async fn fetch_and_encode(
        &self,
        client: &MyUplinkClient,
        device_id: Option<&str>,
    ) -> Result<(), MyUplinkError> {
        let devices = client.fetch_devices().await?;

        let mut samples = vec![];

        for device in devices {
            // Skip device if filtering by ID
            if let Some(filter_id) = device_id {
                if device.device_id != filter_id {
                    continue;
                }
            }

            if let Some(params) = device.parameters {
                for param in params {
                    // Skip parameters without value
                    if let Some(value) = param.value {
                        // Try to convert to numeric
                        if let Some(numeric) = value.as_numeric() {
                            let samples_for_param = map_parameter_to_samples(
                                &param.parameter_id,
                                param.name.as_deref(),
                                numeric,
                                &device.device_id,
                            );
                            samples.extend(samples_for_param);
                        }
                    }
                }
            }
        }

        let encoded = encode_metrics(&samples);
        self.update_metrics(encoded).await;
        Ok(())
    }

    /// Poll once (for testing purposes).
    pub async fn poll_once_for_test(
        &self,
        client: &MyUplinkClient,
        device_id: Option<&str>,
    ) -> bool {
        match self.fetch_and_encode(client, device_id).await {
            Ok(()) => {
                debug!("Poll succeeded");
                true
            }
            Err(MyUplinkError::Unauthorized) => {
                warn!("Authentication failed");
                self.increment_auth_failures();
                false
            }
            Err(MyUplinkError::RateLimited { retry_after }) => {
                warn!("Rate limited: {:?}", retry_after);
                self.increment_rate_limited();
                false
            }
            Err(e) => {
                error!("Poll failed: {}", e);
                self.increment_errors();
                false
            }
        }
    }
}

impl Default for MetricsStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn the metrics polling loop.
///
/// Runs indefinitely, polling the myUplink API at the specified interval.
/// Uses `MissedTickBehavior::Delay` to prevent thundering herd.
pub fn spawn_poll_loop(
    store: Arc<MetricsStore>,
    client: Arc<MyUplinkClient>,
    device_id: Option<String>,
    poll_interval_secs: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = interval(std::time::Duration::from_secs(poll_interval_secs));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            ticker.tick().await;
            match store.fetch_and_encode(&client, device_id.as_deref()).await {
                Ok(()) => {
                    debug!("Metrics poll succeeded");
                }
                Err(MyUplinkError::Unauthorized) => {
                    store.increment_auth_failures();
                    warn!("Authentication failed during poll");
                }
                Err(MyUplinkError::RateLimited { retry_after }) => {
                    store.increment_rate_limited();
                    warn!("Rate limited (retry after {:?}s)", retry_after);
                }
                Err(e) => {
                    store.increment_errors();
                    error!("Poll failed: {}", e);
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_store_new() {
        let store = MetricsStore::new();
        assert_eq!(store.polls_total(), 0);
        assert_eq!(store.scrape_errors_total(), 0);
        assert_eq!(store.auth_failures_total(), 0);
        assert_eq!(store.rate_limited_total(), 0);
    }

    #[tokio::test]
    async fn test_metrics_store_update() {
        let store = MetricsStore::new();
        store.update_metrics("test metrics".to_string()).await;
        assert_eq!(store.polls_total(), 1);
        let metrics = store.get_metrics().await;
        assert_eq!(metrics.as_str(), "test metrics");
    }

    #[test]
    fn test_metrics_store_increment_errors() {
        let store = MetricsStore::new();
        store.increment_errors();
        store.increment_errors();
        assert_eq!(store.scrape_errors_total(), 2);
    }

    #[test]
    fn test_metrics_store_increment_auth_failures() {
        let store = MetricsStore::new();
        store.increment_auth_failures();
        assert_eq!(store.auth_failures_total(), 1);
    }

    #[test]
    fn test_metrics_store_increment_rate_limited() {
        let store = MetricsStore::new();
        store.increment_rate_limited();
        store.increment_rate_limited();
        store.increment_rate_limited();
        assert_eq!(store.rate_limited_total(), 3);
    }
}
