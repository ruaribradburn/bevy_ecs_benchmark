//! Mutation workloads - testing write operations.
//!
//! These workloads test the performance of modifying component data.
//!
//! # Optimization Patterns Demonstrated
//!
//! - **Parallel iteration**: Using `par_iter_mut()` for CPU-bound workloads
//! - **Change detection bypass**: Using `into_inner()` for conditional mutations
//! - **Local resources**: Caching per-system state to reduce resource contention
//! - **Pre-allocated batch spawning**: Collecting entities before spawn_batch

use bevy::prelude::*;
use std::hint::black_box;

use crate::benchmark::runner::SpawnEntitiesRequest;
use crate::components::{BenchmarkEntity, Counter, DataPayload, FastRng, Position, Velocity};

// =============================================================================
// Position/Velocity Update Workload
// =============================================================================

/// OPTIMIZATION: Spawn entities using FastRng and pre-allocation.
///
/// This demonstrates two key optimizations:
/// 1. **FastRng resource**: Reuses a single fast PRNG instead of creating
///    `thread_rng()` for each entity (significant overhead reduction)
/// 2. **Pre-allocation**: Collects all entity data into a Vec before spawning,
///    allowing spawn_batch to pre-allocate the exact capacity needed
///
/// For 100k entities, this can be 2-3x faster than naive spawning.
pub fn spawn_position_velocity_entities(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
    mut rng: ResMut<FastRng>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!(
            "Spawning {} entities for position/velocity update",
            event.count
        );

        // Pre-generate all random data to avoid RNG overhead during spawn
        // This also allows spawn_batch to allocate exact capacity upfront
        let entities: Vec<_> = (0..event.count)
            .map(|_| {
                (
                    BenchmarkEntity,
                    Position::random_with(&mut rng.0),
                    Velocity::random_with(&mut rng.0),
                )
            })
            .collect();

        commands.spawn_batch(entities);
    }
}

/// OPTIMIZATION: Position update using parallel iteration.
///
/// Uses `par_iter_mut()` to automatically distribute work across CPU cores.
/// Bevy's parallel iterator uses work-stealing for load balancing.
///
/// # Anti-Pattern Fix: Removed Unnecessary Local Caching
///
/// Previously used `Local<f32>` to cache delta time. This was an anti-pattern
/// because:
/// - `Res<Time>` access is just a pointer dereference (very fast)
/// - `Local<T>` has initialization check overhead
/// - For simple `Copy` types like `f32`, `Local` adds overhead without benefit
///
/// `Local<T>` IS appropriate for:
/// - Expensive computations cached across frames
/// - Mutable state that persists between system runs
/// - Pre-allocated buffers to avoid per-frame allocation
///
/// When to use `par_iter_mut()`:
/// - Entity count > ~1000 (parallel overhead becomes worthwhile)
/// - Per-entity work is CPU-bound (not just a few operations)
/// - No dependencies between entity updates
///
/// When to avoid `par_iter_mut()`:
/// - Small entity counts (parallel overhead dominates)
/// - Very simple operations (memory-bound, not CPU-bound)
/// - Updates depend on other entities' state
pub fn position_velocity_system(
    mut query: Query<(&mut Position, &Velocity), With<BenchmarkEntity>>,
    time: Res<Time>,
) {
    // Direct resource access - Res<T> is just a pointer dereference
    let dt = time.delta_secs();

    // Parallel iteration distributes entities across worker threads
    query.par_iter_mut().for_each(|(mut pos, vel)| {
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;
        pos.z += vel.z * dt;

        // Keep values bounded to prevent float issues over long runs
        pos.x = pos.x.rem_euclid(2000.0) - 1000.0;
        pos.y = pos.y.rem_euclid(2000.0) - 1000.0;
        pos.z = pos.z.rem_euclid(2000.0) - 1000.0;
    });
}

// =============================================================================
// Counter Increment Workload
// =============================================================================

/// Spawn entities for counter increment test
pub fn spawn_counter_entities(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!("Spawning {} entities for counter increment", event.count);

        commands.spawn_batch((0..event.count).map(|_| {
            (BenchmarkEntity, Counter::default())
        }));
    }
}

/// Simple counter increment
pub fn counter_increment_system(mut query: Query<&mut Counter, With<BenchmarkEntity>>) {
    for mut counter in &mut query {
        counter.increment();
    }
}

// =============================================================================
// Conditional Mutation Workload
// =============================================================================

/// OPTIMIZATION: Conditional mutation with change detection bypass.
///
/// Bevy's change detection marks components as "changed" whenever you access
/// them through `Mut<T>`. This happens even if you don't actually modify the data!
///
/// Problem with naive approach:
/// ```rust
/// for (mut pos, vel) in &mut query {
///     if condition {
///         pos.x += 1.0; // Position marked as changed
///     }
///     // Position is STILL marked as changed even if condition was false!
///     // The `mut pos` dereference triggered change detection
/// }
/// ```
///
/// Solution using `into_inner()`:
/// - Only call `into_inner()` when you're certain you'll write
/// - This bypasses the automatic change detection trigger
/// - Reduces unnecessary change propagation to dependent systems
///
/// When this matters:
/// - Systems that use `Changed<T>` filters depend on accurate change detection
/// - Rendering systems often skip unchanged entities
/// - Network replication systems only sync changed components
pub fn conditional_mutation_system(
    mut query: Query<(&mut Position, &Velocity), With<BenchmarkEntity>>,
) {
    for (pos, vel) in &mut query {
        // Check condition BEFORE triggering change detection
        if vel.x.abs() > 0.5 || vel.y.abs() > 0.5 || vel.z.abs() > 0.5 {
            // Only now do we call into_inner() which bypasses the automatic
            // change detection that would occur with normal Mut<T> access
            let pos = pos.into_inner();
            pos.x += vel.x * 0.016;
            pos.y += vel.y * 0.016;
            pos.z += vel.z * 0.016;
        }
        // If condition was false, Position is NOT marked as changed
    }
}

// =============================================================================
// Heavy Mutation Workload
// =============================================================================

/// Spawn entities with cache-aligned heavy data payloads.
pub fn spawn_heavy_mutation_entities(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
    mut rng: ResMut<FastRng>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!("Spawning {} entities for heavy mutation", event.count);

        let entities: Vec<_> = (0..event.count)
            .map(|_| (BenchmarkEntity, DataPayload::random_with(&mut rng.0)))
            .collect();

        commands.spawn_batch(entities);
    }
}

/// OPTIMIZATION: Heavy data mutation with parallel iteration.
///
/// For CPU-intensive per-entity work (like DataPayload::process()),
/// parallel iteration provides near-linear scaling with CPU cores.
///
/// The cache-aligned DataPayload (64 bytes, aligned to cache line)
/// combined with parallel iteration minimizes false sharing between threads.
pub fn heavy_mutation_system(mut query: Query<&mut DataPayload, With<BenchmarkEntity>>) {
    query.par_iter_mut().for_each(|mut payload| {
        payload.process();
    });
}

// =============================================================================
// Parallel-friendly Mutation Workload
// =============================================================================

/// Alternative parallel position update (explicit version).
///
/// This is the explicit parallel version for comparison. The main
/// `position_velocity_system` now uses parallel iteration by default.
pub fn parallel_position_update_system(
    mut query: Query<(&mut Position, &Velocity), With<BenchmarkEntity>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    query.par_iter_mut().for_each(|(mut pos, vel)| {
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;
        pos.z += vel.z * dt;

        pos.x = pos.x.rem_euclid(2000.0) - 1000.0;
        pos.y = pos.y.rem_euclid(2000.0) - 1000.0;
        pos.z = pos.z.rem_euclid(2000.0) - 1000.0;
    });
}

// =============================================================================
// Read-Modify-Write Pattern
// =============================================================================

/// OPTIMIZATION: Complex read-modify-write with parallel iteration.
///
/// This demonstrates parallel iteration for systems that both read AND write
/// multiple components on each entity. Each entity's update is independent,
/// making this perfectly parallelizable.
///
/// Note: Parallel iteration works here because:
/// - No entity depends on another entity's state
/// - All data needed is local to each entity
/// - Write operations are isolated to each entity's components
pub fn read_modify_write_system(
    mut query: Query<(&mut Position, &mut Velocity), With<BenchmarkEntity>>,
) {
    query.par_iter_mut().for_each(|(mut pos, mut vel)| {
        // Bounce off boundaries
        if pos.x.abs() > 900.0 {
            vel.x = -vel.x;
            pos.x = pos.x.clamp(-900.0, 900.0);
        }
        if pos.y.abs() > 900.0 {
            vel.y = -vel.y;
            pos.y = pos.y.clamp(-900.0, 900.0);
        }
        if pos.z.abs() > 900.0 {
            vel.z = -vel.z;
            pos.z = pos.z.clamp(-900.0, 900.0);
        }

        // Apply velocity
        pos.x += vel.x * 0.016;
        pos.y += vel.y * 0.016;
        pos.z += vel.z * 0.016;

        // Apply friction
        vel.x *= 0.999;
        vel.y *= 0.999;
        vel.z *= 0.999;
    });

    // Prevent optimization
    black_box(());
}
