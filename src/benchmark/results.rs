//! Benchmark results collection, storage, and export.

use bevy::ecs::message::Message;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::config::RESULTS_DIR;
use crate::metrics::SampleStats;
use crate::state::SelectedWorkload;

/// A single workload's benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadResult {
    pub workload_name: String,
    pub workload_description: String,
    pub breakdown_point: usize,
    pub throughput_at_breakdown: f64,
    pub frame_time_stats: FrameTimeStats,
}

/// Frame time statistics for a result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameTimeStats {
    pub min_ms: f64,
    pub max_ms: f64,
    pub median_ms: f64,
    pub mean_ms: f64,
    pub std_dev_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

impl From<SampleStats> for FrameTimeStats {
    fn from(stats: SampleStats) -> Self {
        Self {
            min_ms: stats.min,
            max_ms: stats.max,
            median_ms: stats.median,
            mean_ms: stats.mean,
            std_dev_ms: stats.std_dev,
            p95_ms: stats.p95,
            p99_ms: stats.p99,
        }
    }
}

/// System information for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub cpu_cores: usize,
    pub bevy_version: String,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            cpu_cores: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(1),
            bevy_version: "0.17.3".to_string(),
        }
    }
}

/// Complete benchmark report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub timestamp: String,
    pub target_frame_time_ms: f64,
    pub system_info: SystemInfo,
    pub results: Vec<WorkloadResult>,
}

impl BenchmarkReport {
    pub fn new(target_ms: f64) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            target_frame_time_ms: target_ms,
            system_info: SystemInfo::default(),
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: WorkloadResult) {
        self.results.push(result);
    }

    /// Save report to JSON file
    pub fn save(&self) -> Result<String, String> {
        // Ensure directory exists
        let dir = Path::new(RESULTS_DIR);
        if !dir.exists() {
            fs::create_dir_all(dir).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Generate filename with timestamp
        let filename = format!(
            "{}/benchmark_{}.json",
            RESULTS_DIR,
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );

        // Serialize and write
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        fs::write(&filename, json).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(filename)
    }
}

/// Resource holding collected results
#[derive(Resource, Default)]
pub struct BenchmarkResults {
    pub report: Option<BenchmarkReport>,
    pub current_workload_result: Option<WorkloadResult>,
}

impl BenchmarkResults {
    pub fn start_new_report(&mut self, target_ms: f64) {
        self.report = Some(BenchmarkReport::new(target_ms));
        self.current_workload_result = None;
    }

    pub fn record_workload_result(
        &mut self,
        workload: SelectedWorkload,
        breakdown_point: usize,
        throughput: f64,
        stats: SampleStats,
    ) {
        let result = WorkloadResult {
            workload_name: workload.name().to_string(),
            workload_description: workload.description().to_string(),
            breakdown_point,
            throughput_at_breakdown: throughput,
            frame_time_stats: stats.into(),
        };

        self.current_workload_result = Some(result.clone());

        if let Some(ref mut report) = self.report {
            report.add_result(result);
        }
    }

    pub fn save_report(&self) -> Result<String, String> {
        match &self.report {
            Some(report) => report.save(),
            None => Err("No report to save".to_string()),
        }
    }

    pub fn has_results(&self) -> bool {
        self.report.as_ref().map(|r| !r.results.is_empty()).unwrap_or(false)
    }

    pub fn result_count(&self) -> usize {
        self.report.as_ref().map(|r| r.results.len()).unwrap_or(0)
    }
}

/// Event signaling that a benchmark completed
#[derive(Event, Message)]
pub struct BenchmarkComplete {
    pub workload: SelectedWorkload,
    pub breakdown_point: usize,
    pub throughput: f64,
}

/// Event requesting results to be saved
#[derive(Event, Message)]
pub struct SaveResultsRequest;
