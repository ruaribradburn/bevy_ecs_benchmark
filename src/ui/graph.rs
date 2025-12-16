//! Frame time graph visualization.
//!
//! # Anti-Pattern Fix: Measurement Pollution
//!
//! The graph update was previously running every frame during sampling,
//! causing layout recalculation overhead that polluted benchmark measurements.
//!
//! Now the graph only updates during non-critical phases (Idle, Adjusting, Complete)
//! to ensure accurate ECS throughput measurements during WarmUp and Sampling.

use bevy::prelude::*;

use crate::config::{colors, TARGET_FRAME_TIME_MS};
use crate::metrics::FrameMetrics;
use crate::state::BenchmarkPhase;
use crate::ui::dashboard::GraphBar;

/// Maximum frame time to display on graph (in ms)
const MAX_DISPLAY_TIME: f64 = 50.0;

/// Graph height in pixels
const GRAPH_HEIGHT: f32 = 284.0; // Container height minus padding

/// FIX: Run condition to skip graph updates during critical benchmark phases.
///
/// Modifying `Node.height` on 300 graph bars every frame triggers Bevy's UI
/// layout engine (Taffy) to recalculate the entire layout tree. This overhead
/// was polluting benchmark measurements.
///
/// By skipping updates during WarmUp and Sampling phases, we ensure the
/// frame time measurements only reflect the actual ECS workload.
pub fn should_update_graph(phase: Res<State<BenchmarkPhase>>) -> bool {
    !matches!(
        phase.get(),
        BenchmarkPhase::WarmUp | BenchmarkPhase::Sampling
    )
}

/// Update the frame time graph bars.
///
/// NOTE: This system is conditionally run via `should_update_graph` to avoid
/// polluting benchmark measurements with UI layout overhead.
pub fn update_frame_graph(
    metrics: Res<FrameMetrics>,
    mut query: Query<(&GraphBar, &mut Node, &mut BackgroundColor)>,
) {
    let frame_times = metrics.frame_times_slice();

    for (bar, mut node, mut bg_color) in &mut query {
        // Get the frame time for this bar index
        let frame_time = frame_times.get(bar.index).copied().unwrap_or(0.0);

        // Calculate bar height (normalized to max display time)
        let normalized = (frame_time / MAX_DISPLAY_TIME).clamp(0.0, 1.0);
        let height = normalized as f32 * GRAPH_HEIGHT;

        node.height = Val::Px(height);

        // Color based on relation to target
        let color = if frame_time > TARGET_FRAME_TIME_MS * 1.2 {
            colors::DANGER
        } else if frame_time > TARGET_FRAME_TIME_MS {
            colors::WARNING
        } else if frame_time > TARGET_FRAME_TIME_MS * 0.8 {
            colors::ACCENT
        } else {
            colors::GRAPH_LINE
        };

        bg_color.0 = color;
    }
}

/// Calculate graph statistics for display
pub fn calculate_graph_stats(frame_times: &[f64]) -> GraphStats {
    if frame_times.is_empty() {
        return GraphStats::default();
    }

    let min = frame_times.iter().cloned().fold(f64::MAX, f64::min);
    let max = frame_times.iter().cloned().fold(f64::MIN, f64::max);
    let avg = frame_times.iter().sum::<f64>() / frame_times.len() as f64;

    // Count frames above target
    let over_target = frame_times
        .iter()
        .filter(|&&t| t > TARGET_FRAME_TIME_MS)
        .count();

    let over_target_percent = (over_target as f64 / frame_times.len() as f64) * 100.0;

    GraphStats {
        min,
        max,
        avg,
        over_target_percent,
    }
}

/// Statistics about the displayed graph
#[derive(Default)]
pub struct GraphStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub over_target_percent: f64,
}

impl GraphStats {
    pub fn format_summary(&self) -> String {
        format!(
            "Min: {:.1}ms | Avg: {:.1}ms | Max: {:.1}ms | Over target: {:.1}%",
            self.min, self.avg, self.max, self.over_target_percent
        )
    }
}
