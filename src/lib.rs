#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! NIBE heat pump Prometheus exporter.
//!
//! This crate provides a Prometheus exporter for NIBE heat pumps via the myUplink REST API.
//! It periodically fetches device metrics and makes them available in `OpenMetrics` format
//! on the `/metrics` HTTP endpoint.

/// Configuration loading and validation.
pub mod config;
/// Metrics storage, encoding, and polling loop.
pub mod metrics;
/// myUplink API client and `OAuth2` token management.
pub mod myuplink;
/// HTTP server with health/readiness/metrics endpoints.
pub mod server;
/// Logging and tracing setup.
pub mod telemetry;
