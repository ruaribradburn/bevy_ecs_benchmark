//! Bevy ECS Benchmark Suite
//!
//! A comprehensive stress testing tool for measuring Bevy ECS performance.
//!
//! Run with: `cargo run --release`

use bevy::prelude::*;
use bevy_ecs_benchmark::BenchmarkPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy ECS Benchmark Suite".into(),
                resolution: (1280u32, 800u32).into(),
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BenchmarkPlugin)
        .run();
}
