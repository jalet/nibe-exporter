# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.8] - 2026-03-11

### Added
- Prometheus metric relabeling for NIBE parameters (configuration-driven naming)
- ServiceMonitor support for metricRelabeling in Helm chart
- 109 parameter relabeling rules for NIBE heat pump devices
- Documentation for metric relabeling configuration

### Fixed
- OAuth token endpoint now uses unversioned URL (https://api.myuplink.com/oauth/token)
- OAuth form parameters now use HashMap for proper serialization
- Authorization header now uses actual token variable instead of hardcoded "***redacted***"
- Grafana sidecar now scans all namespaces for dashboard discovery

### Changed
- Simplified metric output to use generic nibe_parameter_<id> format
- Removed static parameter mapping logic in favor of Prometheus relabeling

[0.0.8]: https://github.com/jalet/nibe-exporter/releases/tag/v0.0.8
