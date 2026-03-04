//! Forge application builder — fluent API for configuring and running Axum-based web applications.

pub mod app;

pub use app::{App, AuthInstallerFn, MigratorFn, SeedFn, init_tracing, shutdown_signal_future};
