# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0](https://github.com/jalet/nibe-exporter/compare/v0.5.0...v0.6.0) (2026-04-04)


### Features

* trigger release ([#13](https://github.com/jalet/nibe-exporter/issues/13)) ([1a3b7a9](https://github.com/jalet/nibe-exporter/commit/1a3b7a9ecedd8bd3b52fdd8422d82a0b91da3828))

## [0.5.0](https://github.com/jalet/nibe-exporter/compare/v0.4.0...v0.5.0) (2026-03-13)


### Features

* **chart:** add additionalRules to PrometheusRule ([#10](https://github.com/jalet/nibe-exporter/issues/10)) ([6d8f5e2](https://github.com/jalet/nibe-exporter/commit/6d8f5e2cbd25997b294bac39993234c02ae33b03))

## [0.4.0](https://github.com/jalet/nibe-exporter/compare/v0.3.2...v0.4.0) (2026-03-13)


### Features

* add info logging to metrics poll loop ([04d03aa](https://github.com/jalet/nibe-exporter/commit/04d03aaa4c9715746f23a098a6ceede098cd6996))


### Bug Fixes

* replace unwrap() with expect() in tests and fix clippy warnings ([73975cc](https://github.com/jalet/nibe-exporter/commit/73975cc80c80325bc848d42937427d2bb5e9a61f))

## [0.3.2](https://github.com/jalet/nibe-exporter/compare/v0.3.1...v0.3.2) (2026-03-12)


### Bug Fixes

* apply clippy and fmt fixes ([f05914d](https://github.com/jalet/nibe-exporter/commit/f05914d6b2e73042814ccb79d16f2da2e268c0a5))

## [0.3.1](https://github.com/jalet/nibe-exporter/compare/v0.3.0...v0.3.1) (2026-03-11)


### Bug Fixes

* trigger release ([7bfc031](https://github.com/jalet/nibe-exporter/commit/7bfc03157688895d95ae1e69248b19ef54c03ecc))

## [0.3.0](https://github.com/jalet/nibe-exporter/compare/v0.2.0...v0.3.0) (2026-03-11)


### Features

* trigger release ([5305836](https://github.com/jalet/nibe-exporter/commit/5305836d137f3bf1a5b16c7d6f8b28716c874320))

## [0.2.0](https://github.com/jalet/nibe-exporter/compare/v0.1.1...v0.2.0) (2026-03-11)


### Features

* trigger release ([6713d56](https://github.com/jalet/nibe-exporter/commit/6713d56bfc776b4b1dc9bcf4769487b6fe43af3e))

## [0.1.1](https://github.com/jalet/nibe-exporter/compare/v0.1.0...v0.1.1) (2026-03-11)


### Bug Fixes

* **clippy:** resolve doc-markdown and formatting warnings ([feb67bf](https://github.com/jalet/nibe-exporter/commit/feb67bf1119fde48d8bba7d32a8d0d21a9ee089d))

## [0.1.0](https://github.com/jalet/nibe-exporter/compare/v0.0.8...v0.1.0) (2026-03-11)


### Features

* **logging:** add detailed tracing for API calls and debugging ([934f8a3](https://github.com/jalet/nibe-exporter/commit/934f8a3b1d12f72e91511d31166237d3f2f3dd8f))
* **logging:** add OAuth error response body logging ([8971214](https://github.com/jalet/nibe-exporter/commit/8971214cb628b7067e3fa653739cc54bbabdab78))
* **logging:** add OAuth token refresh tracing ([d2525b2](https://github.com/jalet/nibe-exporter/commit/d2525b2be2c5560965e02a592b2f99b6f201a52c))
* **metrics:** use parameter ID based names with Prometheus relabeling ([97f52f7](https://github.com/jalet/nibe-exporter/commit/97f52f7fd4f42e57c7c97f624142c5495c51fc99))


### Bug Fixes

* **api:** pass actual OAuth token in Authorization header ([bc228a7](https://github.com/jalet/nibe-exporter/commit/bc228a7d24b704803df1906c728fb1f0f7588e34))
* **myuplink:** fetch device points separately from systems endpoint ([a6ce018](https://github.com/jalet/nibe-exporter/commit/a6ce0180e108e3e3401e778949988675a99911bb))

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
