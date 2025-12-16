//! Archetype fragmentation workload.
//!
//! This tests how performance degrades when entities are spread across
//! many different archetypes (component combinations).
//!
//! # Two Approaches Demonstrated
//!
//! 1. **Marker Component Approach** (Fragmented):
//!    - Uses separate VariantA-H marker components
//!    - Creates up to 256 different archetypes
//!    - Allows query filtering with `With<VariantA>`, etc.
//!    - Results in cache-unfriendly iteration patterns
//!
//! 2. **EntityVariant Bitflags Approach** (Unified):
//!    - Uses single `EntityVariant(u8)` component with bitflags
//!    - All entities stay in ONE archetype
//!    - Runtime checks instead of query filters
//!    - Optimal cache utilization during iteration
//!
//! Use marker components when query-level filtering is essential.
//! Use EntityVariant when iteration performance is critical.

use bevy::prelude::*;
use rand::Rng;
use std::hint::black_box;

use crate::benchmark::runner::SpawnEntitiesRequest;
use crate::components::{
    BenchmarkEntity, EntityVariant, FastRng, Position, Velocity,
    VariantA, VariantB, VariantC, VariantD, VariantE, VariantF, VariantG, VariantH,
};

/// Number of different archetype variants to create
pub const ARCHETYPE_VARIANTS: usize = 8;

/// Spawn entities distributed across many archetypes (FRAGMENTED approach).
///
/// This deliberately creates cache-unfriendly access patterns for benchmarking.
/// In production code, prefer the EntityVariant approach below for better performance.
///
/// **Why this is slow:**
/// - Each variant combination creates a separate archetype
/// - Iterating over all entities jumps between archetype tables
/// - CPU cache is constantly evicted as we hop between memory regions
pub fn spawn_fragmented_entities(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
    mut rng: ResMut<FastRng>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!(
            "Spawning {} entities across {} archetypes (FRAGMENTED)",
            event.count, ARCHETYPE_VARIANTS
        );

        // NOTE: We're deliberately NOT using batch spawning here because
        // entities go into different archetypes. For fragmented spawns,
        // individual spawns are necessary.

        for i in 0..event.count {
            let variant = i % ARCHETYPE_VARIANTS;
            let pos = Position::random_with(&mut rng.0);
            let vel = Velocity::random_with(&mut rng.0);

            // Each match arm creates entities in a different archetype
            match variant {
                0 => { commands.spawn((BenchmarkEntity, pos, vel, VariantA)); }
                1 => { commands.spawn((BenchmarkEntity, pos, vel, VariantB)); }
                2 => { commands.spawn((BenchmarkEntity, pos, vel, VariantC)); }
                3 => { commands.spawn((BenchmarkEntity, pos, vel, VariantD)); }
                4 => { commands.spawn((BenchmarkEntity, pos, vel, VariantA, VariantB)); }
                5 => { commands.spawn((BenchmarkEntity, pos, vel, VariantC, VariantD)); }
                6 => { commands.spawn((BenchmarkEntity, pos, vel, VariantE, VariantF)); }
                7 => { commands.spawn((BenchmarkEntity, pos, vel, VariantG, VariantH)); }
                _ => { commands.spawn((BenchmarkEntity, pos, vel)); }
            }
        }
    }
}

/// OPTIMIZATION: Spawn entities with EntityVariant bitflags (UNIFIED approach).
///
/// All entities share the same archetype, with variant information stored
/// as bitflags in a single component. This provides:
/// - Optimal cache utilization during iteration
/// - No archetype fragmentation
/// - Ability to batch spawn all entities
///
/// Trade-off: Cannot use query filters like `With<VariantA>` - must check
/// the bitflags at runtime instead.
pub fn spawn_unified_variant_entities(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
    mut rng: ResMut<FastRng>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!(
            "Spawning {} entities with unified EntityVariant (OPTIMIZED)",
            event.count
        );

        // All entities go into the SAME archetype - batch spawn is efficient
        let entities: Vec<_> = (0..event.count)
            .map(|_| {
                (
                    BenchmarkEntity,
                    Position::random_with(&mut rng.0),
                    Velocity::random_with(&mut rng.0),
                    // Random variant flags - but all in same archetype!
                    EntityVariant::random_with(&mut rng.0),
                )
            })
            .collect();

        commands.spawn_batch(entities);
    }
}

/// Iterate over fragmented entities (FRAGMENTED version).
///
/// This tests query performance when matching entities are spread across
/// many archetype tables (cache unfriendly access patterns).
///
/// **Why this can be slow:**
/// - Query must iterate through multiple archetype tables
/// - Each archetype table is in different memory location
/// - CPU prefetcher struggles to predict memory access pattern
pub fn fragmented_iteration_system(
    query: Query<(&Position, &Velocity), With<BenchmarkEntity>>,
) {
    let mut sum: f32 = 0.0;

    for (pos, vel) in &query {
        sum += black_box(pos.x * vel.x + pos.y * vel.y + pos.z * vel.z);
    }

    black_box(sum);
}

/// OPTIMIZATION: Iterate with EntityVariant (UNIFIED version).
///
/// All entities are in a single archetype, enabling:
/// - Sequential memory access (cache-friendly)
/// - CPU prefetcher can predict access pattern
/// - Near-optimal iteration performance
///
/// Trade-off: Must check variant flags at runtime if variant-specific
/// logic is needed, but for simple iteration this is much faster.
pub fn unified_variant_iteration_system(
    query: Query<(&Position, &Velocity, &EntityVariant), With<BenchmarkEntity>>,
) {
    let mut sum: f32 = 0.0;

    // All entities in ONE archetype = optimal cache utilization
    for (pos, vel, variant) in &query {
        // Can still do variant-specific logic with runtime checks
        if variant.has(EntityVariant::A) {
            sum += black_box(pos.x * vel.x);
        }
        if variant.has(EntityVariant::B) {
            sum += black_box(pos.y * vel.y);
        }
        // Even without variant checks, iteration is faster due to cache locality
        sum += black_box(pos.z * vel.z);
    }

    black_box(sum);
}

/// Alternative: More heavily fragmented spawning (EXTREME fragmentation).
///
/// Creates up to 256 different archetypes by randomly assigning variants.
/// This is worst-case for iteration performance.
pub fn spawn_heavily_fragmented_entities(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
    mut rng: ResMut<FastRng>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!("Spawning {} entities across up to 256 archetypes (EXTREME)", event.count);

        for _ in 0..event.count {
            let pos = Position::random_with(&mut rng.0);
            let vel = Velocity::random_with(&mut rng.0);

            // Randomly assign variants creating up to 256 archetypes
            let mut entity_commands = commands.spawn((BenchmarkEntity, pos, vel));

            // Each if statement potentially adds to a different archetype
            if rng.0.gen_bool(0.5) { entity_commands.insert(VariantA); }
            if rng.0.gen_bool(0.5) { entity_commands.insert(VariantB); }
            if rng.0.gen_bool(0.5) { entity_commands.insert(VariantC); }
            if rng.0.gen_bool(0.5) { entity_commands.insert(VariantD); }
            if rng.0.gen_bool(0.5) { entity_commands.insert(VariantE); }
            if rng.0.gen_bool(0.5) { entity_commands.insert(VariantF); }
            if rng.0.gen_bool(0.5) { entity_commands.insert(VariantG); }
            if rng.0.gen_bool(0.5) { entity_commands.insert(VariantH); }
        }
    }
}

/// Mutation on fragmented data.
///
/// Even parallel iteration can't fully compensate for archetype fragmentation
/// because each archetype table is in different memory.
pub fn fragmented_mutation_system(
    mut query: Query<(&mut Position, &Velocity), With<BenchmarkEntity>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // Using par_iter_mut helps, but fragmentation still hurts cache performance
    query.par_iter_mut().for_each(|(mut pos, vel)| {
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;
        pos.z += vel.z * dt;

        pos.x = pos.x.rem_euclid(2000.0) - 1000.0;
        pos.y = pos.y.rem_euclid(2000.0) - 1000.0;
        pos.z = pos.z.rem_euclid(2000.0) - 1000.0;
    });
}

/// OPTIMIZATION: Unified variant mutation using parallel iteration.
///
/// Demonstrates that with EntityVariant, mutation is cache-efficient
/// even with variant-specific logic.
pub fn unified_variant_mutation_system(
    mut query: Query<(&mut Position, &Velocity, &EntityVariant), With<BenchmarkEntity>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // Single archetype = optimal parallel iteration performance
    query.par_iter_mut().for_each(|(mut pos, vel, variant)| {
        // Can still have variant-specific behavior
        let multiplier = if variant.has(EntityVariant::A) { 1.5 } else { 1.0 };

        pos.x += vel.x * dt * multiplier;
        pos.y += vel.y * dt * multiplier;
        pos.z += vel.z * dt * multiplier;

        pos.x = pos.x.rem_euclid(2000.0) - 1000.0;
        pos.y = pos.y.rem_euclid(2000.0) - 1000.0;
        pos.z = pos.z.rem_euclid(2000.0) - 1000.0;
    });
}

/// Query with filter on fragmented data.
///
/// This demonstrates when marker components ARE useful: when you need
/// to process only a subset of entities and the filtering benefit
/// outweighs the fragmentation cost.
pub fn fragmented_filtered_query_system(
    query_a: Query<&Position, (With<BenchmarkEntity>, With<VariantA>)>,
    query_b: Query<&Position, (With<BenchmarkEntity>, With<VariantB>)>,
) {
    let mut sum: f32 = 0.0;

    // These queries only iterate entities with the specific variants
    // Useful when you have few entities of each type
    for pos in &query_a {
        sum += black_box(pos.x + pos.y + pos.z);
    }

    for pos in &query_b {
        sum += black_box(pos.x + pos.y + pos.z);
    }

    black_box(sum);
}

/// OPTIMIZATION: Filter at runtime with EntityVariant.
///
/// For comparison: processes all entities with runtime variant check.
/// More efficient when most entities need processing anyway.
pub fn unified_filtered_query_system(
    query: Query<(&Position, &EntityVariant), With<BenchmarkEntity>>,
) {
    let mut sum: f32 = 0.0;

    // Single iteration over unified archetype
    for (pos, variant) in &query {
        // Runtime variant filtering - still faster than fragmented queries
        // for large entity counts due to cache efficiency
        if variant.has(EntityVariant::A) || variant.has(EntityVariant::B) {
            sum += black_box(pos.x + pos.y + pos.z);
        }
    }

    black_box(sum);
}
