use nibe_exporter::metrics::{MetricSample, MetricType, encode_metrics};
use std::collections::HashMap;

#[test]
fn snapshot_single_temperature_metric() {
    let mut labels = HashMap::new();
    labels.insert("device_id".to_string(), "nibe-123".to_string());
    labels.insert("parameter_id".to_string(), "40083".to_string());
    labels.insert("name".to_string(), "BT3 Return temp".to_string());

    let sample = MetricSample {
        name: "nibe_return_temperature_celsius".to_string(),
        metric_type: MetricType::Gauge,
        help: "Return temperature (BT3)".to_string(),
        value: 45.5,
        labels,
    };

    let encoded = encode_metrics(&[sample]);
    insta::assert_snapshot!(encoded);
}

#[test]
fn snapshot_multiple_temperature_metrics() {
    let mut labels1 = HashMap::new();
    labels1.insert("device_id".to_string(), "nibe-123".to_string());
    labels1.insert("parameter_id".to_string(), "40083".to_string());

    let mut labels2 = HashMap::new();
    labels2.insert("device_id".to_string(), "nibe-123".to_string());
    labels2.insert("parameter_id".to_string(), "40008".to_string());

    let mut labels3 = HashMap::new();
    labels3.insert("device_id".to_string(), "nibe-456".to_string());
    labels3.insert("parameter_id".to_string(), "40083".to_string());

    let samples = vec![
        MetricSample {
            name: "nibe_return_temperature_celsius".to_string(),
            metric_type: MetricType::Gauge,
            help: "Return temperature (BT3)".to_string(),
            value: 45.5,
            labels: labels1,
        },
        MetricSample {
            name: "nibe_supply_temperature_celsius".to_string(),
            metric_type: MetricType::Gauge,
            help: "Supply temperature (BT1)".to_string(),
            value: 50.2,
            labels: labels2,
        },
        MetricSample {
            name: "nibe_return_temperature_celsius".to_string(),
            metric_type: MetricType::Gauge,
            help: "Return temperature (BT3)".to_string(),
            value: 42.1,
            labels: labels3,
        },
    ];

    let encoded = encode_metrics(&samples);
    insta::assert_snapshot!(encoded);
}

#[test]
fn snapshot_escaping_special_characters() {
    let mut labels = HashMap::new();
    labels.insert(
        "device_id".to_string(),
        "device\\with\\backslash".to_string(),
    );
    labels.insert("description".to_string(), r#"Quote"in"value"#.to_string());
    labels.insert("multiline".to_string(), "Line1\nLine2".to_string());

    let sample = MetricSample {
        name: "test_metric".to_string(),
        metric_type: MetricType::Gauge,
        help: "Help text with\\backslash and\nnewline".to_string(),
        value: 99.9,
        labels,
    };

    let encoded = encode_metrics(&[sample]);
    insta::assert_snapshot!(encoded);
}

#[test]
fn snapshot_counter_metric() {
    let mut labels = HashMap::new();
    labels.insert("device_id".to_string(), "nibe-789".to_string());

    let sample = MetricSample {
        name: "nibe_polls_total".to_string(),
        metric_type: MetricType::Counter,
        help: "Total number of poll attempts".to_string(),
        value: 1234.0,
        labels,
    };

    let encoded = encode_metrics(&[sample]);
    insta::assert_snapshot!(encoded);
}

#[test]
fn snapshot_mixed_metrics() {
    let mut labels1 = HashMap::new();
    labels1.insert("device_id".to_string(), "nibe-123".to_string());

    let mut labels2 = HashMap::new();
    labels2.insert("device_id".to_string(), "nibe-123".to_string());

    let mut labels3 = HashMap::new();
    labels3.insert("device_id".to_string(), "nibe-456".to_string());

    let samples = vec![
        MetricSample {
            name: "nibe_temperature_celsius".to_string(),
            metric_type: MetricType::Gauge,
            help: "Current temperature".to_string(),
            value: 45.5,
            labels: labels1,
        },
        MetricSample {
            name: "nibe_polls_total".to_string(),
            metric_type: MetricType::Counter,
            help: "Total poll count".to_string(),
            value: 5000.0,
            labels: labels2,
        },
        MetricSample {
            name: "nibe_temperature_celsius".to_string(),
            metric_type: MetricType::Gauge,
            help: "Current temperature".to_string(),
            value: 38.2,
            labels: labels3,
        },
    ];

    let encoded = encode_metrics(&samples);
    insta::assert_snapshot!(encoded);
}

#[test]
fn snapshot_empty_metrics() {
    let samples: Vec<MetricSample> = vec![];
    let encoded = encode_metrics(&samples);
    insta::assert_snapshot!(encoded);
}

#[test]
fn snapshot_metric_with_zero_value() {
    let mut labels = HashMap::new();
    labels.insert("device_id".to_string(), "test".to_string());

    let sample = MetricSample {
        name: "nibe_counter".to_string(),
        metric_type: MetricType::Counter,
        help: "Counter at zero".to_string(),
        value: 0.0,
        labels,
    };

    let encoded = encode_metrics(&[sample]);
    insta::assert_snapshot!(encoded);
}

#[test]
fn snapshot_metric_with_negative_value() {
    let mut labels = HashMap::new();
    labels.insert("device_id".to_string(), "test".to_string());

    let sample = MetricSample {
        name: "nibe_differential".to_string(),
        metric_type: MetricType::Gauge,
        help: "Differential temperature".to_string(),
        value: -5.3,
        labels,
    };

    let encoded = encode_metrics(&[sample]);
    insta::assert_snapshot!(encoded);
}

#[test]
fn snapshot_metric_with_scientific_notation() {
    let mut labels = HashMap::new();
    labels.insert("device_id".to_string(), "test".to_string());

    let sample = MetricSample {
        name: "nibe_small_value".to_string(),
        metric_type: MetricType::Gauge,
        help: "Very small measurement".to_string(),
        value: 0.00001234,
        labels,
    };

    let encoded = encode_metrics(&[sample]);
    insta::assert_snapshot!(encoded);
}
