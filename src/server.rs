use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::debug;

use crate::metrics::MetricsStore;

/// Application state.
#[derive(Clone)]
pub struct AppState {
    /// Metrics store (shared).
    pub metrics_store: Arc<MetricsStore>,
}

/// Health check endpoint (always returns 200 OK).
async fn healthz() -> impl IntoResponse {
    debug!("Health check");
    (StatusCode::OK, "OK")
}

/// Readiness check endpoint (returns 200 if metrics are available).
async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let metrics = state.metrics_store.get_metrics().await;
    // Consider ready if we have more than just the EOF marker
    if metrics.len() > 10 {
        debug!("Ready check: metrics available");
        (StatusCode::OK, "Ready")
    } else {
        debug!("Ready check: no metrics yet");
        (StatusCode::SERVICE_UNAVAILABLE, "Not ready")
    }
}

/// Metrics endpoint (returns current metrics in `OpenMetrics` format).
async fn metrics(State(state): State<AppState>) -> Response {
    let metrics = state.metrics_store.get_metrics().await;
    let body = metrics.as_ref().clone();
    (
        StatusCode::OK,
        [(
            "Content-Type",
            "application/openmetrics-text; version=1.0.0; charset=utf-8",
        )],
        body,
    )
        .into_response()
}

/// Build the Axum router.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_healthz_endpoint() {
        let state = AppState {
            metrics_store: Arc::new(MetricsStore::new()),
        };
        let router = build_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_endpoint_empty() {
        let state = AppState {
            metrics_store: Arc::new(MetricsStore::new()),
        };
        let router = build_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 503 if metrics not yet available
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let metrics_store = Arc::new(MetricsStore::new());
        metrics_store.update_metrics("# EOF\n".to_string()).await;

        let state = AppState { metrics_store };
        let router = build_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/openmetrics-text; version=1.0.0; charset=utf-8"
        );
    }
}
