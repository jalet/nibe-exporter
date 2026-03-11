use std::collections::HashMap;

/// Metric type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Gauge metric (can go up or down).
    Gauge,
    /// Counter metric (only increases).
    Counter,
    /// Cumulative counter.
    CounterTotal,
}

/// A single metric sample with value and labels.
#[derive(Debug, Clone)]
pub struct MetricSample {
    /// Metric name (e.g., `` `nibe_parameter_40004` ``).
    pub name: String,
    /// Metric type.
    pub metric_type: MetricType,
    /// Human-readable help text.
    pub help: String,
    /// Sample value (numeric).
    pub value: f64,
    /// Labels as key-value pairs.
    pub labels: HashMap<String, String>,
}

/// Map a parameter to a metric sample with the parameter ID in the name.
///
/// Metric names follow the pattern `nibe_parameter_{id}` and can be renamed
/// using Prometheus relabel_configs in the ServiceMonitor configuration.
#[must_use]
pub fn map_parameter_to_samples(
    parameter_id: &str,
    parameter_name: Option<&str>,
    value: f64,
    device_id: &str,
) -> Vec<MetricSample> {
    vec![MetricSample {
        name: format!("nibe_parameter_{parameter_id}"),
        metric_type: MetricType::Gauge,
        help: parameter_name.unwrap_or(&format!("Parameter {parameter_id}")).to_string(),
        value,
        labels: {
            let mut m = HashMap::new();
            m.insert("device_id".to_string(), device_id.to_string());
            m.insert("parameter_id".to_string(), parameter_id.to_string());
            if let Some(name) = parameter_name {
                m.insert("parameter_name".to_string(), name.to_string());
            }
            m
        },
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_parameter() {
        let samples = map_parameter_to_samples("40012", Some("Return line (BT3)"), 29.6, "device1");
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].name, "nibe_parameter_40012");
        assert_eq!(samples[0].value, 29.6);
        assert_eq!(samples[0].metric_type, MetricType::Gauge);
    }

    #[test]
    fn test_metric_labels() {
        let samples = map_parameter_to_samples("40012", Some("Return line (BT3)"), 29.6, "device1");
        let labels = &samples[0].labels;
        assert_eq!(labels.get("device_id"), Some(&"device1".to_string()));
        assert_eq!(labels.get("parameter_id"), Some(&"40012".to_string()));
        assert_eq!(labels.get("parameter_name"), Some(&"Return line (BT3)".to_string()));
    }

    #[test]
    fn test_parameter_without_name() {
        let samples = map_parameter_to_samples("40012", None, 30.0, "device1");
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].name, "nibe_parameter_40012");
        assert_eq!(samples[0].labels.get("parameter_name"), None);
    }
}
