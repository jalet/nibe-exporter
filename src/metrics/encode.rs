use crate::metrics::mapping::{MetricSample, MetricType};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

/// Encode metrics to `OpenMetrics` 1.0 text format.
#[must_use]
pub fn encode_metrics(samples: &[MetricSample]) -> String {
    let mut output = String::new();
    let mut metric_metadata: BTreeMap<&str, (&MetricType, &str)> = BTreeMap::new();

    // First pass: collect metadata and build index
    for sample in samples {
        metric_metadata
            .entry(&sample.name)
            .or_insert((&sample.metric_type, &sample.help));
    }

    // Second pass: output metadata
    for (name, (metric_type, help)) in &metric_metadata {
        let _ = writeln!(output, "# HELP {name} {}", escape_help(help));
        let type_str = match metric_type {
            MetricType::Gauge => "gauge",
            MetricType::Counter | MetricType::CounterTotal => "counter",
        };
        let _ = writeln!(output, "# TYPE {name} {type_str}");
    }

    // Third pass: output samples (sorted by metric name then labels)
    let mut sorted_samples = samples.to_vec();
    sorted_samples.sort_by(|a, b| {
        a.name.cmp(&b.name).then_with(|| {
            let a_labels = format_labels(&a.labels);
            let b_labels = format_labels(&b.labels);
            a_labels.cmp(&b_labels)
        })
    });

    for sample in sorted_samples {
        let _ = writeln!(
            output,
            "{}{{{}}} {}",
            sample.name,
            format_labels(&sample.labels),
            sample.value
        );
    }

    // EOF marker for `OpenMetrics`
    output.push_str("# EOF\n");

    output
}

/// Format labels as `OpenMetrics` label list.
///
/// Labels are sorted by key and formatted as: `key1="value1",key2="value2"`
fn format_labels(labels: &HashMap<String, String>) -> String {
    if labels.is_empty() {
        return String::new();
    }

    let mut items: Vec<_> = labels.iter().collect();
    items.sort_by_key(|(k, _)| k.as_str());

    let formatted: Vec<String> = items
        .iter()
        .map(|(k, v)| format!("{k}=\"{}\"", escape_label_value(v)))
        .collect();

    formatted.join(",")
}

/// Escape help text for `OpenMetrics`.
fn escape_help(help: &str) -> String {
    help.replace('\\', "\\\\").replace('\n', "\\n")
}

/// Escape label value for `OpenMetrics`.
fn escape_label_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_empty() {
        let samples: Vec<MetricSample> = vec![];
        let encoded = encode_metrics(&samples);
        assert!(encoded.contains("# EOF"));
    }

    #[test]
    fn test_encode_single_sample() {
        let mut labels = HashMap::new();
        labels.insert("device_id".to_string(), "dev1".to_string());

        let sample = MetricSample {
            name: "test_metric".to_string(),
            metric_type: MetricType::Gauge,
            help: "Test metric".to_string(),
            value: 42.5,
            labels,
        };

        let encoded = encode_metrics(&[sample]);
        assert!(encoded.contains("# HELP test_metric Test metric"));
        assert!(encoded.contains("# TYPE test_metric gauge"));
        assert!(encoded.contains(r#"test_metric{device_id="dev1"} 42.5"#));
        assert!(encoded.contains("# EOF"));
    }

    #[test]
    fn test_escape_label_value() {
        assert_eq!(escape_label_value("normal"), "normal");
        assert_eq!(escape_label_value("with\\backslash"), "with\\\\backslash");
        assert_eq!(escape_label_value(r#"with"quote"#), r#"with\"quote"#);
        assert_eq!(escape_label_value("with\nnewline"), "with\\nnewline");
    }

    #[test]
    fn test_escape_help() {
        assert_eq!(escape_help("simple"), "simple");
        assert_eq!(escape_help("with\\backslash"), "with\\\\backslash");
        assert_eq!(escape_help("with\nnewline"), "with\\nnewline");
    }

    #[test]
    fn test_label_sorting() {
        let mut labels = HashMap::new();
        labels.insert("z_last".to_string(), "value_z".to_string());
        labels.insert("a_first".to_string(), "value_a".to_string());
        labels.insert("m_middle".to_string(), "value_m".to_string());

        let formatted = format_labels(&labels);
        // Check order: a_first should come before m_middle, which should come before z_last
        let a_pos = formatted.find("a_first").unwrap();
        let m_pos = formatted.find("m_middle").unwrap();
        let z_pos = formatted.find("z_last").unwrap();
        assert!(a_pos < m_pos && m_pos < z_pos);
    }

    #[test]
    fn test_encode_multiple_samples() {
        let mut labels1 = HashMap::new();
        labels1.insert("device_id".to_string(), "dev1".to_string());

        let mut labels2 = HashMap::new();
        labels2.insert("device_id".to_string(), "dev2".to_string());

        let samples = vec![
            MetricSample {
                name: "metric1".to_string(),
                metric_type: MetricType::Gauge,
                help: "Metric 1".to_string(),
                value: 10.0,
                labels: labels1,
            },
            MetricSample {
                name: "metric1".to_string(),
                metric_type: MetricType::Gauge,
                help: "Metric 1".to_string(),
                value: 20.0,
                labels: labels2,
            },
        ];

        let encoded = encode_metrics(&samples);
        assert!(encoded.contains("# HELP metric1 Metric 1"));
        assert!(encoded.contains("# TYPE metric1 gauge"));
        assert!(encoded.contains(r#"metric1{device_id="dev1"} 10"#));
        assert!(encoded.contains(r#"metric1{device_id="dev2"} 20"#));
    }
}
