//! Core benchmarking infrastructure.
//!
//! This module contains the benchmark runner, results handling,
//! and workload definitions.

pub mod results;
pub mod runner;
pub mod workloads;

pub use results::*;
pub use runner::*;
