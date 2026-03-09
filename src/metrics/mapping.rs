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
    /// Metric name (e.g., `` `nibe_supply_temperature_celsius` ``).
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

/// Map a parameter to metric samples based on configuration.
#[must_use]
pub fn map_parameter_to_samples(
    parameter_id: &str,
    parameter_name: Option<&str>,
    value: f64,
    device_id: &str,
) -> Vec<MetricSample> {
    // Default mapping for known NIBE parameters
    match parameter_id {
        "40083" => vec![MetricSample {
            name: "nibe_return_temperature_celsius".to_string(),
            metric_type: MetricType::Gauge,
            help: "Return temperature (BT3)".to_string(),
            value,
            labels: {
                let mut m = HashMap::new();
                m.insert("device_id".to_string(), device_id.to_string());
                m.insert("parameter_id".to_string(), parameter_id.to_string());
                if let Some(name) = parameter_name {
                    m.insert("name".to_string(), name.to_string());
                }
                m
            },
        }],
        "40008" => vec![MetricSample {
            name: "nibe_supply_temperature_celsius".to_string(),
            metric_type: MetricType::Gauge,
            help: "Supply temperature (BT1)".to_string(),
            value,
            labels: {
                let mut m = HashMap::new();
                m.insert("device_id".to_string(), device_id.to_string());
                m.insert("parameter_id".to_string(), parameter_id.to_string());
                if let Some(name) = parameter_name {
                    m.insert("name".to_string(), name.to_string());
                }
                m
            },
        }],
        "40045" => vec![MetricSample {
            name: "nibe_external_temperature_celsius".to_string(),
            metric_type: MetricType::Gauge,
            help: "External temperature (BT20)".to_string(),
            value,
            labels: {
                let mut m = HashMap::new();
                m.insert("device_id".to_string(), device_id.to_string());
                m.insert("parameter_id".to_string(), parameter_id.to_string());
                if let Some(name) = parameter_name {
                    m.insert("name".to_string(), name.to_string());
                }
                m
            },
        }],
        "40057" => vec![MetricSample {
            name: "nibe_compressor_frequency_hz".to_string(),
            metric_type: MetricType::Gauge,
            help: "Compressor frequency".to_string(),
            value,
            labels: {
                let mut m = HashMap::new();
                m.insert("device_id".to_string(), device_id.to_string());
                m.insert("parameter_id".to_string(), parameter_id.to_string());
                if let Some(name) = parameter_name {
                    m.insert("name".to_string(), name.to_string());
                }
                m
            },
        }],
        "43005" => vec![MetricSample {
            name: "nibe_total_power_consumption_watts".to_string(),
            metric_type: MetricType::Gauge,
            help: "Total power consumption".to_string(),
            value,
            labels: {
                let mut m = HashMap::new();
                m.insert("device_id".to_string(), device_id.to_string());
                m.insert("parameter_id".to_string(), parameter_id.to_string());
                if let Some(name) = parameter_name {
                    m.insert("name".to_string(), name.to_string());
                }
                m
            },
        }],
        // Generic parameter mapping for unknown parameters
        _ => vec![MetricSample {
            name: format!("nibe_parameter_{parameter_id}"),
            metric_type: MetricType::Gauge,
            help: format!("Parameter {parameter_id}"),
            value,
            labels: {
                let mut m = HashMap::new();
                m.insert("device_id".to_string(), device_id.to_string());
                m.insert("parameter_id".to_string(), parameter_id.to_string());
                if let Some(name) = parameter_name {
                    m.insert("name".to_string(), name.to_string());
                }
                m
            },
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_known_parameter() {
        let samples = map_parameter_to_samples("40083", Some("BT3 Return temp"), 45.5, "device1");
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].name, "nibe_return_temperature_celsius");
        assert_eq!(samples[0].value, 45.5);
    }

    #[test]
    fn test_map_unknown_parameter() {
        let samples = map_parameter_to_samples("99999", Some("Custom param"), 123.45, "device1");
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].name, "nibe_parameter_99999");
        assert_eq!(samples[0].metric_type, MetricType::Gauge);
    }

    #[test]
    fn test_metric_sample_labels() {
        let samples = map_parameter_to_samples("40083", Some("BT3 Return temp"), 45.5, "device1");
        let labels = &samples[0].labels;
        assert_eq!(labels.get("device_id"), Some(&"device1".to_string()));
        assert_eq!(labels.get("parameter_id"), Some(&"40083".to_string()));
        assert_eq!(labels.get("name"), Some(&"BT3 Return temp".to_string()));
    }

    #[test]
    fn test_metric_sample_labels_no_name() {
        let samples = map_parameter_to_samples("40083", None, 45.5, "device1");
        let labels = &samples[0].labels;
        assert_eq!(labels.get("device_id"), Some(&"device1".to_string()));
        assert_eq!(labels.get("parameter_id"), Some(&"40083".to_string()));
        assert!(!labels.contains_key("name"));
    }
}
