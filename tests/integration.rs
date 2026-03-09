mod common;

use axum::body::Body;
use axum::http::Request;
use nibe_exporter::config::Config;
use nibe_exporter::metrics::{MetricSample, MetricType};
use nibe_exporter::myuplink::error::MyUplinkError;
use nibe_exporter::myuplink::models::{DeviceInfo, Parameter, ParameterValue, Product};
use nibe_exporter::server::build_router;
use std::collections::HashMap;
use tower::ServiceExt;

/// Test 1: MyUplinkClient validates API version at parse time.
#[test]
fn test_client_rejects_invalid_api_version() {
    let result = nibe_exporter::myuplink::MyUplinkClient::new(
        "id".to_string(),
        "secret".to_string(),
        "v99".to_string(),
    );
    assert!(matches!(result, Err(MyUplinkError::InvalidApiVersion(_))));
}

/// Test 2: MyUplinkClient accepts valid API versions.
#[test]
fn test_client_accepts_valid_v2() {
    let result = nibe_exporter::myuplink::MyUplinkClient::new(
        "id".to_string(),
        "secret".to_string(),
        "v2".to_string(),
    );
    assert!(result.is_ok());
}

/// Test 3: MyUplinkClient accepts v3.
#[test]
fn test_client_accepts_valid_v3() {
    let result = nibe_exporter::myuplink::MyUplinkClient::new(
        "id".to_string(),
        "secret".to_string(),
        "v3".to_string(),
    );
    assert!(result.is_ok());
}

/// Test 4: ParameterValue numeric conversion.
#[test]
fn test_parameter_value_numeric_conversion() {
    let numeric = ParameterValue::Numeric(serde_json::Number::from_f64(42.5).unwrap());
    assert_eq!(numeric.as_numeric(), Some(42.5));

    let string = ParameterValue::String("32.1".to_string());
    assert_eq!(string.as_numeric(), Some(32.1));

    let non_numeric = ParameterValue::String("OFF".to_string());
    assert_eq!(non_numeric.as_numeric(), None);
}

/// Test 5: ParameterValue scaling.
#[test]
fn test_parameter_value_scaling() {
    let value = ParameterValue::Numeric(serde_json::Number::from_f64(100.0).unwrap());
    assert_eq!(value.as_numeric_scaled(0), Some(100.0));
    assert_eq!(value.as_numeric_scaled(-2), Some(1.0));
    assert_eq!(value.as_numeric_scaled(1), Some(1000.0));
}

/// Test 6: MetricsStore initializes with empty cache.
#[tokio::test]
async fn test_metrics_store_initialization() {
    let store = nibe_exporter::metrics::MetricsStore::new();
    assert_eq!(store.polls_total(), 0);
    assert_eq!(store.scrape_errors_total(), 0);
    assert_eq!(store.auth_failures_total(), 0);
    assert_eq!(store.rate_limited_total(), 0);
}

/// Test 7: MetricsStore caches metrics efficiently.
#[tokio::test]
async fn test_metrics_store_caching() {
    let store = nibe_exporter::metrics::MetricsStore::new();
    let test_metrics = "# HELP test Test\n# TYPE test gauge\ntest 42\n# EOF\n";
    store.update_metrics(test_metrics.to_string()).await;

    let metrics1 = store.get_metrics().await;
    let metrics2 = store.get_metrics().await;

    // Both should be Arc-equal (same allocation)
    assert!(std::ptr::eq(metrics1.as_ref(), metrics2.as_ref()));
}

/// Test 8: HTTP /healthz endpoint always returns 200.
#[tokio::test]
async fn test_healthz_endpoint() {
    let state = common::build_test_state();
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

    assert_eq!(response.status(), 200);
}

/// Test 9: HTTP /ready endpoint returns 503 when metrics empty.
#[tokio::test]
async fn test_ready_endpoint_not_ready() {
    let state = common::build_test_state();
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

    assert_eq!(response.status(), 503);
}

/// Test 10: HTTP /ready endpoint returns 200 when metrics available.
#[tokio::test]
async fn test_ready_endpoint_ready() {
    let store = nibe_exporter::metrics::MetricsStore::new();
    let metrics = "# HELP test Test\n# TYPE test gauge\ntest{device_id=\"dev1\"} 42\n# EOF\n";
    store.update_metrics(metrics.to_string()).await;

    let state = nibe_exporter::server::AppState {
        metrics_store: std::sync::Arc::new(store),
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

    assert_eq!(response.status(), 200);
}

/// Test 11: HTTP /metrics endpoint returns OpenMetrics format.
#[tokio::test]
async fn test_metrics_endpoint() {
    let store = nibe_exporter::metrics::MetricsStore::new();
    let metrics = "# HELP nibe_temp Temperature\n# TYPE nibe_temp gauge\nnibe_temp 45.5\n# EOF\n";
    store.update_metrics(metrics.to_string()).await;

    let state = nibe_exporter::server::AppState {
        metrics_store: std::sync::Arc::new(store),
    };
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

    assert_eq!(response.status(), 200);
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("openmetrics-text"));
}

/// Test 12: Config validation requires client_id.
#[test]
fn test_config_validation_missing_client_id() {
    let mut config = Config {
        client_id: None,
        client_secret: Some("secret".to_string()),
        api_version: "v2".to_string(),
        device_id: None,
        poll_interval: 60,
        listen_addr: "0.0.0.0:9090".to_string(),
        log_level: "info".to_string(),
        log_json: false,
        metrics_mapping_file: None,
        client_secret_file: None,
        client_id_file: None,
    };

    assert!(config.validate().is_err());
}

/// Test 13: Config validation requires client_secret.
#[test]
fn test_config_validation_missing_client_secret() {
    let mut config = Config {
        client_id: Some("id".to_string()),
        client_secret: None,
        api_version: "v2".to_string(),
        device_id: None,
        poll_interval: 60,
        listen_addr: "0.0.0.0:9090".to_string(),
        log_level: "info".to_string(),
        log_json: false,
        metrics_mapping_file: None,
        client_secret_file: None,
        client_id_file: None,
    };

    assert!(config.validate().is_err());
}

/// Test 14: Config validation rejects invalid API version.
#[test]
fn test_config_validation_invalid_api_version() {
    let mut config = Config {
        client_id: Some("id".to_string()),
        client_secret: Some("secret".to_string()),
        api_version: "v1".to_string(),
        device_id: None,
        poll_interval: 60,
        listen_addr: "0.0.0.0:9090".to_string(),
        log_level: "info".to_string(),
        log_json: false,
        metrics_mapping_file: None,
        client_secret_file: None,
        client_id_file: None,
    };

    assert!(config.validate().is_err());
}

/// Test 15: Metrics encoding produces valid OpenMetrics format.
#[test]
fn test_metrics_encoding() {
    let mut labels = HashMap::new();
    labels.insert("device_id".to_string(), "dev1".to_string());
    labels.insert("parameter_id".to_string(), "40083".to_string());

    let sample = MetricSample {
        name: "nibe_temperature".to_string(),
        metric_type: MetricType::Gauge,
        help: "Temperature reading".to_string(),
        value: 45.5,
        labels,
    };

    let encoded = nibe_exporter::metrics::encode_metrics(&[sample]);
    assert!(encoded.contains("# HELP nibe_temperature Temperature reading"));
    assert!(encoded.contains("# TYPE nibe_temperature gauge"));
    assert!(encoded.contains("nibe_temperature{"));
    assert!(encoded.contains("} 45.5"));
    assert!(encoded.contains("# EOF"));
}

/// Test 16: Metric type string representation.
#[test]
fn test_metric_type_encoding() {
    let gauge_sample = MetricSample {
        name: "test_gauge".to_string(),
        metric_type: MetricType::Gauge,
        help: "Test gauge".to_string(),
        value: 10.0,
        labels: HashMap::new(),
    };

    let counter_sample = MetricSample {
        name: "test_counter".to_string(),
        metric_type: MetricType::Counter,
        help: "Test counter".to_string(),
        value: 20.0,
        labels: HashMap::new(),
    };

    let encoded = nibe_exporter::metrics::encode_metrics(&[gauge_sample, counter_sample]);
    assert!(encoded.contains("# TYPE test_gauge gauge"));
    assert!(encoded.contains("# TYPE test_counter counter"));
}

/// Test 17: Parameter mapping preserves device_id label.
#[test]
fn test_parameter_mapping_device_id() {
    let samples =
        nibe_exporter::metrics::map_parameter_to_samples("40083", Some("BT3"), 45.5, "my-device");
    assert_eq!(samples.len(), 1);
    assert_eq!(
        samples[0].labels.get("device_id"),
        Some(&"my-device".to_string())
    );
}

/// Test 18: Unknown parameters are mapped generically.
#[test]
fn test_unknown_parameter_mapping() {
    let samples = nibe_exporter::metrics::map_parameter_to_samples("99999", None, 100.0, "device1");
    assert_eq!(samples.len(), 1);
    assert!(samples[0].name.contains("99999"));
    assert_eq!(samples[0].metric_type, MetricType::Gauge);
}
