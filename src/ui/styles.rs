//! UI styling utilities and helper functions.

use bevy::prelude::*;

use crate::config::{colors, sizes};

/// Create a standard panel node
pub fn panel_node() -> Node {
    Node {
        padding: UiRect::all(sizes::PANEL_PADDING),
        margin: UiRect::all(sizes::PANEL_MARGIN),
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

/// Create a row layout node
pub fn row_node() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(8.0),
        ..default()
    }
}

/// Create a column layout node
pub fn column_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(4.0),
        ..default()
    }
}

/// Create title text style
pub fn title_text_font() -> TextFont {
    TextFont {
        font_size: sizes::FONT_SIZE_TITLE,
        ..default()
    }
}

/// Create heading text style
pub fn heading_text_font() -> TextFont {
    TextFont {
        font_size: sizes::FONT_SIZE_HEADING,
        ..default()
    }
}

/// Create body text style
pub fn body_text_font() -> TextFont {
    TextFont {
        font_size: sizes::FONT_SIZE_BODY,
        ..default()
    }
}

/// Create small text style
pub fn small_text_font() -> TextFont {
    TextFont {
        font_size: sizes::FONT_SIZE_SMALL,
        ..default()
    }
}

/// Create large metric text style
pub fn large_metric_font() -> TextFont {
    TextFont {
        font_size: sizes::FONT_SIZE_LARGE_METRIC,
        ..default()
    }
}

/// Standard spacing between sections
pub fn section_spacing() -> Node {
    Node {
        height: Val::Px(16.0),
        ..default()
    }
}

/// Divider line
pub fn divider() -> (Node, BackgroundColor) {
    (
        Node {
            height: Val::Px(1.0),
            width: Val::Percent(100.0),
            margin: UiRect::vertical(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(colors::GRAPH_GRID),
    )
}

/// Helper to create a labeled value display
pub fn labeled_value(_label: &str) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.0),
            ..default()
        },
    )
}

/// Badge/pill style for status indicators
pub fn badge_node() -> Node {
    Node {
        padding: UiRect::new(Val::Px(8.0), Val::Px(8.0), Val::Px(4.0), Val::Px(4.0)),
        ..default()
    }
}

/// Get color based on frame time relative to target
pub fn frame_time_color(frame_time_ms: f64, target_ms: f64) -> Color {
    let ratio = frame_time_ms / target_ms;

    if ratio < 0.7 {
        colors::SUCCESS
    } else if ratio < 0.9 {
        colors::ACCENT
    } else if ratio < 1.0 {
        colors::WARNING
    } else {
        colors::DANGER
    }
}

/// Format frame time with color indicator
pub fn format_frame_time(ms: f64) -> String {
    format!("{:.2}ms", ms)
}

/// Shorthand for pixel values
pub fn px(value: f32) -> Val {
    Val::Px(value)
}

/// Shorthand for percentage values
pub fn percent(value: f32) -> Val {
    Val::Percent(value)
}
