//! Read-only iteration workloads.
//!
//! These workloads test pure query iteration performance without mutations.
//!
//! # Optimization Patterns Demonstrated
//!
//! - **FastRng for bulk spawning**: Pre-generate random data efficiently
//! - **Pre-allocation**: Collect entities before spawn_batch

use bevy::prelude::*;
use std::hint::black_box;

use crate::benchmark::runner::SpawnEntitiesRequest;
use crate::components::{
    Acceleration, BenchmarkEntity, Counter, DataPayload, FastRng, Position, Velocity,
};

// =============================================================================
// Simple Iteration Workload
// =============================================================================

/// Spawn entities for simple iteration test.
///
/// Uses pre-allocation for efficient batch spawning.
pub fn spawn_simple_iteration_entities(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!("Spawning {} entities for simple iteration", event.count);

        // Pre-allocate entity data for efficient batch spawn
        let entities: Vec<_> = (0..event.count)
            .map(|_| (BenchmarkEntity, Counter::default()))
            .collect();

        commands.spawn_batch(entities);
    }
}

/// Simple read-only iteration over entities
///
/// This tests the raw overhead of iterating entities with a single component.
pub fn simple_iteration_system(query: Query<&Counter, With<BenchmarkEntity>>) {
    let mut sum: u64 = 0;
    for counter in &query {
        // Use black_box to prevent the compiler from optimizing away the read
        sum = sum.wrapping_add(black_box(counter.value));
    }
    // Prevent dead code elimination
    black_box(sum);
}

// =============================================================================
// Multi-Component Read Workload
// =============================================================================

/// OPTIMIZATION: Spawn with FastRng and pre-allocation.
///
/// Demonstrates efficient bulk spawning for multi-component entities.
pub fn spawn_multi_component_entities(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
    mut rng: ResMut<FastRng>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!("Spawning {} entities for multi-component read", event.count);

        // Pre-generate all entity data using FastRng
        let entities: Vec<_> = (0..event.count)
            .map(|_| {
                (
                    BenchmarkEntity,
                    Position::random_with(&mut rng.0),
                    Velocity::random_with(&mut rng.0),
                    Acceleration::random_with(&mut rng.0),
                )
            })
            .collect();

        commands.spawn_batch(entities);
    }
}

/// Read multiple components per entity
///
/// This tests cache efficiency when reading larger amounts of data per entity.
pub fn multi_component_read_system(
    query: Query<(&Position, &Velocity, &Acceleration), With<BenchmarkEntity>>,
) {
    let mut sum: f32 = 0.0;
    for (pos, vel, acc) in &query {
        // Compute something using all three components
        sum += black_box(pos.x + vel.x + acc.x);
        sum += black_box(pos.y + vel.y + acc.y);
        sum += black_box(pos.z + vel.z + acc.z);
    }
    black_box(sum);
}

// =============================================================================
// Heavy Data Read Workload (bonus)
// =============================================================================

/// Spawn entities with cache-aligned heavy data payloads.
pub fn spawn_heavy_data_entities(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
    mut rng: ResMut<FastRng>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!("Spawning {} entities with heavy data", event.count);

        let entities: Vec<_> = (0..event.count)
            .map(|_| (BenchmarkEntity, DataPayload::random_with(&mut rng.0)))
            .collect();

        commands.spawn_batch(entities);
    }
}

/// Read larger component data
pub fn heavy_data_read_system(query: Query<&DataPayload, With<BenchmarkEntity>>) {
    let mut sum: f32 = 0.0;
    for payload in &query {
        for &value in &payload.values {
            sum += black_box(value);
        }
    }
    black_box(sum);
}
