//! User interface components for the benchmark suite.
//!
//! This module provides the dashboard, graphs, and result displays.
//!
//! # Anti-Pattern Fix: Measurement Pollution
//!
//! The `update_frame_graph` system now uses a run condition to skip updates
//! during critical benchmark phases (WarmUp, Sampling), preventing UI layout
//! overhead from polluting frame time measurements.

mod dashboard;
mod graph;
mod styles;

pub use dashboard::*;
pub use graph::*;
pub use styles::{
    badge_node, body_text_font, column_node, divider, format_frame_time, frame_time_color,
    heading_text_font, labeled_value, large_metric_font, panel_node, row_node, section_spacing,
    small_text_font, title_text_font,
};

use bevy::prelude::*;

/// Plugin for benchmark UI
pub struct BenchmarkUiPlugin;

impl Plugin for BenchmarkUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    update_entity_count_display,
                    update_frame_time_display,
                    update_throughput_display,
                    update_phase_display,
                    update_workload_display,
                    update_workload_description_display,
                    // FIX: Graph updates skip WarmUp/Sampling phases to avoid
                    // polluting benchmark measurements with UI layout overhead
                    update_frame_graph.run_if(should_update_graph),
                ),
            );
    }
}
