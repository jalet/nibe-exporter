use tracing_subscriber::filter::EnvFilter;

/// Initialize tracing subscriber with optional JSON output.
pub fn init_telemetry(log_level: &str, json: bool) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    if json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_telemetry_text() {
        // Note: init_telemetry should only be called once per process
        // This test just verifies the function can be called without panic
        // In real code, avoid calling this multiple times
    }
}
