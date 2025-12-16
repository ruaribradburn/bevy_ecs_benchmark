//! Frame timing metrics collection and analysis.

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::config::{FRAME_HISTORY_LENGTH, SAMPLE_FRAMES};

/// Resource tracking frame timing metrics
#[derive(Resource)]
pub struct FrameMetrics {
    /// Rolling history of frame times (in milliseconds)
    pub frame_times: VecDeque<f64>,
    /// Current frame time
    pub current_frame_time: f64,
    /// Samples collected for current measurement period
    pub samples: Vec<f64>,
    /// Current throughput (entities per second)
    pub throughput: f64,
}

impl Default for FrameMetrics {
    fn default() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(FRAME_HISTORY_LENGTH),
            current_frame_time: 0.0,
            samples: Vec::with_capacity(SAMPLE_FRAMES),
            throughput: 0.0,
        }
    }
}

impl FrameMetrics {
    /// Record a new frame time
    pub fn record_frame(&mut self, delta_seconds: f64, entity_count: usize) {
        let frame_time_ms = delta_seconds * 1000.0;
        self.current_frame_time = frame_time_ms;

        // Update rolling history
        if self.frame_times.len() >= FRAME_HISTORY_LENGTH {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(frame_time_ms);

        // Calculate throughput
        if delta_seconds > 0.0 {
            self.throughput = entity_count as f64 / delta_seconds;
        }
    }

    /// Add a sample for the current measurement period
    pub fn add_sample(&mut self, delta_seconds: f64) {
        self.samples.push(delta_seconds * 1000.0);
    }

    /// Clear collected samples
    pub fn clear_samples(&mut self) {
        self.samples.clear();
    }

    /// Get statistics from collected samples
    pub fn sample_stats(&self) -> SampleStats {
        if self.samples.is_empty() {
            return SampleStats::default();
        }

        let mut sorted = self.samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = sorted.first().copied().unwrap_or(0.0);
        let max = sorted.last().copied().unwrap_or(0.0);
        let median = if sorted.len() % 2 == 0 {
            let mid = sorted.len() / 2;
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };
        let mean = self.samples.iter().sum::<f64>() / self.samples.len() as f64;

        // Calculate standard deviation
        let variance = self.samples.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / self.samples.len() as f64;
        let std_dev = variance.sqrt();

        // Percentiles
        let p95_idx = ((sorted.len() as f64) * 0.95) as usize;
        let p99_idx = ((sorted.len() as f64) * 0.99) as usize;
        let p95 = sorted.get(p95_idx.min(sorted.len() - 1)).copied().unwrap_or(0.0);
        let p99 = sorted.get(p99_idx.min(sorted.len() - 1)).copied().unwrap_or(0.0);

        SampleStats {
            min,
            max,
            median,
            mean,
            std_dev,
            p95,
            p99,
            count: self.samples.len(),
        }
    }

    /// Get the average frame time from recent history
    pub fn average_frame_time(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64
    }

    /// Get min/max from recent history
    pub fn frame_time_range(&self) -> (f64, f64) {
        let min = self.frame_times.iter().cloned().fold(f64::MAX, f64::min);
        let max = self.frame_times.iter().cloned().fold(f64::MIN, f64::max);
        (min, max)
    }

    /// Check if frame time exceeds target
    pub fn exceeds_target(&self, target_ms: f64) -> bool {
        self.average_frame_time() > target_ms
    }

    /// Get frame times as a slice for graphing
    pub fn frame_times_slice(&self) -> &VecDeque<f64> {
        &self.frame_times
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.current_frame_time = 0.0;
        self.samples.clear();
        self.throughput = 0.0;
    }
}

/// Statistics from a sample collection period
#[derive(Debug, Clone, Default)]
pub struct SampleStats {
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub p95: f64,
    pub p99: f64,
    pub count: usize,
}

impl SampleStats {
    /// Check if the median exceeds a target
    pub fn median_exceeds(&self, target_ms: f64) -> bool {
        self.median > target_ms
    }
}

/// System to update frame metrics each frame
pub fn update_frame_metrics(
    time: Res<Time>,
    mut metrics: ResMut<FrameMetrics>,
    // We need some way to know entity count - this will be passed differently per workload
) {
    metrics.record_frame(time.delta_secs_f64(), 0);
}

/// Formats a number with appropriate suffix (K, M, B)
pub fn format_count(count: usize) -> String {
    if count >= 1_000_000_000 {
        format!("{:.2}B", count as f64 / 1_000_000_000.0)
    } else if count >= 1_000_000 {
        format!("{:.2}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        format!("{}", count)
    }
}

/// Formats throughput (entities per second)
pub fn format_throughput(eps: f64) -> String {
    if eps >= 1_000_000_000.0 {
        format!("{:.2}B/s", eps / 1_000_000_000.0)
    } else if eps >= 1_000_000.0 {
        format!("{:.2}M/s", eps / 1_000_000.0)
    } else if eps >= 1_000.0 {
        format!("{:.1}K/s", eps / 1_000.0)
    } else {
        format!("{:.0}/s", eps)
    }
}
