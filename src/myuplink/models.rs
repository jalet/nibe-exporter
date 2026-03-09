use serde::{Deserialize, Serialize};

/// Token response from myUplink `OAuth2` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// `OAuth2` access token.
    pub access_token: String,
    /// Token type (typically "Bearer").
    pub token_type: String,
    /// Token expiration time in seconds.
    pub expires_in: u64,
}

/// Error response from myUplink API.
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    /// Error code.
    pub error: Option<String>,
    /// Error description.
    pub error_description: Option<String>,
}

/// Device information from myUplink API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier.
    #[serde(rename = "deviceId")]
    pub device_id: String,
    /// Human-readable device name.
    pub name: Option<String>,
    /// Product information for this device.
    pub product: Option<Product>,
    /// Device parameters and measurements.
    pub parameters: Option<Vec<Parameter>>,
}

/// Product metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    /// Product name (e.g., "NIBE F2120").
    #[serde(rename = "productName")]
    pub product_name: String,
    /// Product series identifier.
    #[serde(rename = "productSeries")]
    pub product_series: Option<String>,
}

/// Single parameter from myUplink device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Unique parameter identifier (e.g., "40083").
    #[serde(rename = "parameterId")]
    pub parameter_id: String,
    /// Human-readable parameter name (e.g., "BT3 Return temp").
    pub name: Option<String>,
    /// Unit of measurement (e.g., "°C").
    pub unit: Option<String>,
    /// Current parameter value.
    pub value: Option<ParameterValue>,
    /// Parameter type classification.
    #[serde(rename = "parameterType")]
    pub parameter_type: Option<String>,
}

/// Parameter value can be numeric, enum, or string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ParameterValue {
    /// Numeric value (integer or float).
    Numeric(serde_json::Number),
    /// String-based value (enum choice or text).
    String(String),
}

impl ParameterValue {
    /// Convert to numeric representation if possible.
    ///
    /// For numeric values, returns the number as f64.
    /// For string values, attempts to parse as f64.
    /// Returns None if conversion is not possible.
    #[must_use]
    pub fn as_numeric(&self) -> Option<f64> {
        match self {
            Self::Numeric(n) => n.as_f64(),
            Self::String(s) => s.parse::<f64>().ok(),
        }
    }

    /// Convert to numeric representation, applying optional scaling.
    ///
    /// Applies scale factor: `value * 10^scale`
    /// For example, scale=-2 means divide by 100 (0.01x multiplier).
    #[must_use]
    pub fn as_numeric_scaled(&self, scale: i32) -> Option<f64> {
        let base = self.as_numeric()?;
        if scale == 0 {
            Some(base)
        } else {
            Some(base * 10_f64.powi(scale))
        }
    }
}

/// Multi-device status response.
#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    /// List of systems and their devices.
    pub systems: Option<Vec<SystemInfo>>,
}

/// System information.
#[derive(Debug, Deserialize)]
pub struct SystemInfo {
    /// Unique system identifier.
    #[serde(rename = "systemId")]
    pub system_id: String,
    /// Human-readable system name.
    pub name: Option<String>,
    /// Devices in this system.
    pub devices: Option<Vec<DeviceInfo>>,
}

/// Metrics mapping configuration (parameter -> metric).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsMapping {
    /// Parameter ID (e.g., `"40083"`)
    pub parameter_id: String,
    /// Metric name (e.g., `` `nibe_parameter_40083` ``)
    pub metric_name: String,
    /// Description for HELP line
    pub description: String,
    /// Optional scale factor (-2 = /100, 1 = *10, etc.)
    pub scale: Option<i32>,
    /// Optional: parameter name to use as label value
    pub parameter_name: Option<String>,
}

/// Complete metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Configuration version.
    pub version: String,
    /// Parameter-to-metric mappings.
    pub mappings: Vec<MetricsMapping>,
}

/// Default metrics mappings for common NIBE parameters.
#[must_use]
pub fn default_metrics_mappings() -> Vec<MetricsMapping> {
    vec![
        MetricsMapping {
            parameter_id: "40083".to_string(),
            metric_name: "nibe_return_temperature_celsius".to_string(),
            description: "Return temperature (BT3)".to_string(),
            scale: None,
            parameter_name: Some("BT3 Return temp".to_string()),
        },
        MetricsMapping {
            parameter_id: "40008".to_string(),
            metric_name: "nibe_supply_temperature_celsius".to_string(),
            description: "Supply temperature (BT1)".to_string(),
            scale: None,
            parameter_name: Some("BT1 Supply temp".to_string()),
        },
        MetricsMapping {
            parameter_id: "40045".to_string(),
            metric_name: "nibe_external_temperature_celsius".to_string(),
            description: "External temperature (BT20)".to_string(),
            scale: None,
            parameter_name: Some("BT20 External temp".to_string()),
        },
        MetricsMapping {
            parameter_id: "40057".to_string(),
            metric_name: "nibe_compressor_frequency_hz".to_string(),
            description: "Compressor frequency".to_string(),
            scale: None,
            parameter_name: None,
        },
        MetricsMapping {
            parameter_id: "43005".to_string(),
            metric_name: "nibe_total_power_consumption_watts".to_string(),
            description: "Total power consumption".to_string(),
            scale: None,
            parameter_name: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_value_as_numeric() {
        let value = ParameterValue::Numeric(serde_json::Number::from_f64(45.5).unwrap());
        assert_eq!(value.as_numeric(), Some(45.5));
    }

    #[test]
    fn test_string_value_as_numeric() {
        let value = ParameterValue::String("32.1".to_string());
        assert_eq!(value.as_numeric(), Some(32.1));
    }

    #[test]
    fn test_as_numeric_scaled() {
        let value = ParameterValue::Numeric(serde_json::Number::from_f64(100.0).unwrap());
        assert_eq!(value.as_numeric_scaled(-2), Some(1.0));
        assert_eq!(value.as_numeric_scaled(0), Some(100.0));
    }

    #[test]
    fn test_string_non_numeric() {
        let value = ParameterValue::String("OFF".to_string());
        assert_eq!(value.as_numeric(), None);
    }
}
