# NIBE Exporter for Prometheus

A high-performance Prometheus exporter for NIBE heat pumps via the myUplink REST API. Written in Rust with careful attention to security, reliability, and operational excellence.

## Features

- **`OAuth2` Authentication**: Secure token management with double-check locking pattern
- **Automatic Token Refresh**: Tokens are refreshed automatically with 30-second safety buffer
- **Rate Limit Handling**: Graceful handling of API rate limits with retry-after support
- **Multi-Version API Support**: Compatible with myUplink API v2 and v3
- **Manual `OpenMetrics` Encoding**: Direct to `OpenMetrics` 1.0.0 format without external dependencies
- **Metrics Caching**: Efficient Arc<String> caching for reader efficiency
- **Healthz & Ready Probes**: Kubernetes-ready health check endpoints
- **JSON Logging**: Structured logging with optional JSON output
- **Distroless Container**: Minimal attack surface with Google distroless images
- **Multi-Architecture Builds**: Docker images for amd64 and arm64
- **Helm Chart**: Production-ready Kubernetes deployment
- **SBOM & Code Signing**: Container images signed with Cosign, SBOM in `CycloneDX` format

## Getting Started

See **[Getting Started Guide](docs/getting-started.md)** for step-by-step setup:
- Register `OAuth2` credentials on myUplink
- Discover your device ID
- Run the exporter with Docker
- Verify metrics are being exposed

For Kubernetes deployment with Helm, see **[Helm Production Deployment](docs/helm-deployment.md)**.

For development setup and building from source, see [Building from Source](#building-from-source) below.

## Endpoints

- `GET /healthz` - Health check (always 200 OK)
- `GET /ready` - Readiness check (200 OK when metrics available)
- `GET /metrics` - Prometheus metrics in `OpenMetrics` format

## Metrics

All metrics are gauges unless otherwise noted. Label format follows Prometheus conventions.

### Common Metrics

```
nibe_supply_temperature_celsius{device_id="...",parameter_id="40008",name="BT1 Supply temp"}
nibe_return_temperature_celsius{device_id="...",parameter_id="40083",name="BT3 Return temp"}
nibe_external_temperature_celsius{device_id="...",parameter_id="40045",name="BT20 External temp"}
nibe_compressor_frequency_hz{device_id="...",parameter_id="40057"}
nibe_total_power_consumption_watts{device_id="...",parameter_id="43005"}
```

### Status Metrics

```
nibe_polls_total{device_id="..."}                      # Total poll attempts
nibe_scrape_errors_total{device_id="..."}             # Scrape errors
nibe_auth_failures_total{device_id="..."}             # Authentication failures (401)
nibe_rate_limited_total{device_id="..."}              # Rate limit hits (429)
```

## API Version

Specify API version via `NIBE_API_VERSION` (default: v2):

- `v2`: Stable, well-documented myUplink API
- `v3`: Latest features and improvements

Invalid versions are rejected at parse time.

## Metrics Mapping

Parameter IDs from myUplink are mapped to Prometheus-friendly metric names:

- `40083` → `nibe_return_temperature_celsius`
- `40008` → `nibe_supply_temperature_celsius`
- `40045` → `nibe_external_temperature_celsius`
- `40057` → `nibe_compressor_frequency_hz`
- `43005` → `nibe_total_power_consumption_watts`
- Other parameters → `nibe_parameter_{id}` (generic mapping)

## Security

### Encryption

- All network communication uses TLS 1.2+
- Secrets Manager support for credential storage
- Support for mounted secret files (Kubernetes)

### Access Control

- `OAuth2` with client credentials flow
- No hardcoded credentials
- Least-privilege IAM roles in Kubernetes

### Container Security

- Distroless base image (Google's minimal runtime)
- Non-root user execution (UID 65532)
- Read-only root filesystem
- No shell or package manager
- Code signing with Cosign

## Configuration

See **[Helm Production Deployment](docs/helm-deployment.md)** for comprehensive Helm configuration examples including:
- Secret management with Kubernetes Secrets
- myUplink `OAuth2` settings
- Prometheus `ServiceMonitor` and `PrometheusRule`
- Network policies (standard Kubernetes and Cilium)
- Grafana dashboard integration

For API version and metrics mapping reference, see [API Version](#api-version) and [Metrics Mapping](#metrics-mapping) sections below.

## Building from Source

### Prerequisites

- Rust 1.85.0+ (check with `rustc --version`)
- Linux/macOS/Windows with standard build tools

### Build

```bash
# Development
cargo build

# Release (optimized)
cargo build --release

# Specific features
cargo build --release --all-features
```

### Testing

```bash
# All tests
cargo test --all

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test '*'

# With output
cargo test -- --nocapture

# Specific test
cargo test test_name
```

### Linting

```bash
# Check formatting
cargo fmt --all -- --check

# Format code
cargo fmt --all

# Run clippy
cargo clippy --all -- -D warnings
```

## Docker Build

Build Docker image locally:

```bash
# Using Makefile
make docker-build

# Manual
docker build -t nibe-exporter:dev .

# Multi-architecture
docker buildx build --platform linux/amd64,linux/arm64 -t nibe-exporter .
```

## Development

### Project Structure

```
src/
  lib.rs              # Library root
  main.rs             # Binary entrypoint
  config.rs           # Configuration management
  telemetry.rs        # Observability setup
  server.rs           # HTTP server
  myuplink/           # myUplink API client
    mod.rs
    error.rs          # Error types
    models.rs         # API data models
    auth.rs           # OAuth2 token manager
    client.rs         # HTTP client
  metrics/            # Metrics handling
    mod.rs
    mapping.rs        # Parameter to metric mapping
    encode.rs         # OpenMetrics encoder
    handler.rs        # Metrics store and polling

tests/
  common/             # Shared test utilities
  integration.rs      # Integration tests
  snapshot_metrics.rs # Snapshot tests with insta

charts/
  nibe-exporter/      # Helm chart
```

### Key Design Decisions

1. **Manual `OpenMetrics` Encoding**: No dependency on `prometheus-client`, direct text encoding for control and simplicity
2. **Double-Check Locking**: `TokenManager` uses `RwLock` for efficient token caching with write-once semantics
3. **Arc<String> for Metrics**: Readers clone Arc, not the entire string, for efficiency
4. **`MissedTickBehavior::Delay`**: Polling uses delay behavior to prevent thundering herd
5. **Distroless Container**: Minimal runtime reduces attack surface and image size

### Testing Strategy

- Unit tests for individual components
- Integration tests with wiremock for API mocking
- Snapshot tests with insta for metrics encoding
- No unsafe code (forbidden at linter level)

## Deployment

### Kubernetes (Helm)

See **[Helm Production Deployment](docs/helm-deployment.md)** for complete Helm installation, upgrade, and management procedures.

### Docker Compose

```yaml
services:
  nibe-exporter:
    image: ghcr.io/jalet/nibe-exporter:latest
    environment:
      NIBE_CLIENT_ID: ${NIBE_CLIENT_ID}
      NIBE_CLIENT_SECRET: ${NIBE_CLIENT_SECRET}
      NIBE_API_VERSION: v2
    ports:
      - "9090:9090"
    restart: unless-stopped
```

## Monitoring the Exporter

### Prometheus Configuration

```yaml
scrape_configs:
  - job_name: nibe-exporter
    static_configs:
      - targets: [localhost:9090]
    scrape_interval: 30s
    scrape_timeout: 10s
```

### Alerting Rules

The Helm chart includes `PrometheusRule` with alerts for:

- Exporter down (5+ minutes)
- Authentication failures
- API rate limiting
- Scrape errors

## Troubleshooting

For deployment and configuration troubleshooting, see:
- **[Getting Started Troubleshooting](docs/getting-started.md#troubleshooting)** - setup and credential issues
- **[Helm Deployment Troubleshooting](docs/helm-deployment.md#troubleshooting)** - Kubernetes, `ServiceMonitor`, rate limiting, and secret issues

### General Issues

**Metric cardinality too high**: Filter metrics by device ID if not needed, or review polling interval.

**Rate limiting**: Increase `NIBE_POLL_INTERVAL` or check myUplink API rate limit quotas.

**Container not starting**: Check logs with `docker logs nibe-exporter` or `kubectl logs -l app.kubernetes.io/name=nibe-exporter`.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass: `cargo test`
6. Ensure formatting: `cargo fmt --all`
7. Ensure clippy passes: `cargo clippy -- -D warnings`
8. Submit a pull request

## License

MIT License - see LICENSE file

## Support

For issues, questions, or feature requests:
- Open an issue on GitHub
- Check existing issues first
- Include exporter version, Kubernetes version (if applicable)
- Include relevant logs and configuration (without credentials)

## Roadmap

- [ ] Configuration file support (TOML/YAML)
- [ ] Custom metrics mapping
- [ ] Multi-device dashboard templates
- [ ] Historical data retention
- [ ] Webhook notifications
- [ ] Performance optimizations
