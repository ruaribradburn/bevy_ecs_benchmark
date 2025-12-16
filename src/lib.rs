//! # Bevy ECS Benchmark Suite
//!
//! A modular, extensible benchmarking framework for Bevy's Entity Component System.
//!
//! ## Modules
//!
//! - [`benchmark`]: Core benchmarking logic and workload definitions
//! - [`components`]: Test components used in benchmarks
//! - [`ui`]: Dashboard and visualization
//! - [`metrics`]: Performance measurement utilities
//! - [`config`]: Configuration constants
//! - [`state`]: Application state management

pub mod benchmark;
pub mod components;
pub mod config;
pub mod metrics;
pub mod plugin;
pub mod state;
pub mod ui;

pub use plugin::BenchmarkPlugin;
