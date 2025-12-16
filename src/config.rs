//! Configuration constants for the benchmark suite.
//!
//! Modify these values to adjust benchmark behavior.

/// Target frame time in milliseconds (16.6ms = 60 FPS)
pub const TARGET_FRAME_TIME_MS: f64 = 16.666;

/// Alternative target for 30 FPS testing
pub const TARGET_FRAME_TIME_30FPS_MS: f64 = 33.333;

/// Number of warm-up frames to skip before measuring
pub const WARMUP_FRAMES: usize = 60;

/// Number of frames to sample for each measurement
pub const SAMPLE_FRAMES: usize = 120;

/// Initial entity count when starting a benchmark
pub const INITIAL_ENTITY_COUNT: usize = 10_000;

/// Minimum entity count for binary search
pub const MIN_ENTITY_COUNT: usize = 100;

/// Maximum entity count to test (effectively unlimited)
pub const MAX_ENTITY_COUNT: usize = usize::MAX / 2;

/// Multiplier for exponential growth phase
pub const GROWTH_MULTIPLIER: f64 = 2.0;

/// Manual adjustment step size
pub const MANUAL_STEP_SIZE: usize = 1_000;

/// Large manual adjustment step (with shift held)
pub const MANUAL_STEP_SIZE_LARGE: usize = 10_000;

/// Minimum gap for binary search convergence (finer granularity)
pub const MIN_CONVERGENCE_GAP: usize = 100;

/// Frame history length for graph display
pub const FRAME_HISTORY_LENGTH: usize = 300;

/// Results output directory
pub const RESULTS_DIR: &str = "benchmark_results";

/// UI Colors
pub mod colors {
    use bevy::prelude::*;

    pub const BACKGROUND: Color = Color::srgb(0.1, 0.1, 0.12);
    pub const PANEL_BG: Color = Color::srgb(0.15, 0.15, 0.18);
    pub const TEXT_PRIMARY: Color = Color::srgb(0.95, 0.95, 0.95);
    pub const TEXT_SECONDARY: Color = Color::srgb(0.6, 0.6, 0.65);
    pub const ACCENT: Color = Color::srgb(0.3, 0.7, 0.9);
    pub const SUCCESS: Color = Color::srgb(0.3, 0.8, 0.4);
    pub const WARNING: Color = Color::srgb(0.9, 0.7, 0.2);
    pub const DANGER: Color = Color::srgb(0.9, 0.3, 0.3);
    pub const GRAPH_LINE: Color = Color::srgb(0.4, 0.8, 0.95);
    pub const GRAPH_TARGET: Color = Color::srgb(0.9, 0.4, 0.4);
    pub const GRAPH_GRID: Color = Color::srgba(0.4, 0.4, 0.45, 0.3);
}

/// UI Sizing
pub mod sizes {
    use bevy::prelude::*;

    pub const SIDEBAR_WIDTH: Val = Val::Px(320.0);
    pub const PANEL_PADDING: Val = Val::Px(16.0);
    pub const PANEL_MARGIN: Val = Val::Px(8.0);
    pub const BORDER_RADIUS: Val = Val::Px(8.0);

    pub const FONT_SIZE_TITLE: f32 = 28.0;
    pub const FONT_SIZE_HEADING: f32 = 20.0;
    pub const FONT_SIZE_BODY: f32 = 16.0;
    pub const FONT_SIZE_SMALL: f32 = 13.0;
    pub const FONT_SIZE_LARGE_METRIC: f32 = 48.0;
}
