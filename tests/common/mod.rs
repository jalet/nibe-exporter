use nibe_exporter::metrics::MetricsStore;
use nibe_exporter::server::AppState;
use std::sync::Arc;

/// Build a test application state with an empty metrics store.
pub fn build_test_state() -> AppState {
    AppState {
        metrics_store: Arc::new(MetricsStore::new()),
    }
}

/// Helper to create a test metrics store with initial data.
#[allow(dead_code)]
pub fn build_test_metrics_store_with_data(initial_metrics: &str) -> Arc<MetricsStore> {
    let store = Arc::new(MetricsStore::new());
    let store_clone = store.clone();

    // Use tokio::block_on in test context
    tokio::runtime::Handle::current().block_on(async {
        store_clone
            .update_metrics(initial_metrics.to_string())
            .await;
    });

    store
}
