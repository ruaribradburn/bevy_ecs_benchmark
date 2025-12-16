//! Structural change workloads - testing spawn/despawn and component add/remove.
//!
//! These workloads test the performance of operations that modify the ECS structure.
//!
//! # Optimization Patterns Demonstrated
//!
//! - **Batch spawning with pre-allocation**: Collect entities before spawn_batch
//! - **FastRng for bulk operations**: Avoid thread_rng() overhead
//! - **Efficient despawning**: Process despawn commands in batches

use bevy::prelude::*;

use crate::benchmark::runner::SpawnEntitiesRequest;
use crate::components::{
    BenchmarkEntity, Counter, FastRng, Position, SecondaryToggle, ToggleComponent, Velocity,
};

// =============================================================================
// Spawn/Despawn Churn Workload
// =============================================================================

/// Resource tracking spawn/despawn state
#[derive(Resource, Default)]
pub struct SpawnDespawnState {
    pub target_count: usize,
    pub current_count: usize,
    pub churn_rate: f32,
    pub initialized: bool,
}

/// OPTIMIZATION: Initial spawn setup with FastRng and pre-allocation.
///
/// Demonstrates efficient bulk spawning for structural workloads.
pub fn spawn_despawn_setup(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
    mut state: ResMut<SpawnDespawnState>,
    mut rng: ResMut<FastRng>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!("Setting up spawn/despawn churn with {} entities", event.count);

        state.target_count = event.count;
        state.churn_rate = 0.01; // 1% churn per frame
        state.initialized = true;
        state.current_count = 0;

        // Pre-allocate all entity data before spawning
        // This allows spawn_batch to allocate exact capacity upfront
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
        state.current_count = event.count;
    }
}

/// OPTIMIZATION: Spawn/despawn churn with efficient batching and buffer reuse.
///
/// This tests the performance of the command queue and archetype management
/// when continuously creating and destroying entities.
///
/// # Anti-Pattern Fix: Allocation in Hot Loop
///
/// Previously allocated a new `Vec<Entity>` every frame to collect entities
/// for despawn. This caused heap thrashing and allocator pressure.
///
/// Now uses `Local<Vec<Entity>>` to reuse the buffer across frames:
/// - The capacity grows to accommodate the workload
/// - `clear()` resets length without deallocating
/// - `drain(..)` yields ownership while clearing
///
/// Key optimizations demonstrated:
/// - **Buffer reuse**: `Local<Vec<Entity>>` persists across frames
/// - **FastRng**: Fast random generation for replacement entities
/// - **Pre-allocation**: Collect before batch spawn
pub fn spawn_despawn_churn_system(
    mut commands: Commands,
    state: ResMut<SpawnDespawnState>,
    query: Query<Entity, With<BenchmarkEntity>>,
    mut rng: ResMut<FastRng>,
    mut despawn_buffer: Local<Vec<Entity>>,
) {
    if !state.initialized {
        return;
    }

    let churn_count = ((state.target_count as f32) * state.churn_rate) as usize;
    let churn_count = churn_count.max(10); // Minimum churn

    // Reuse buffer - clear() keeps capacity, avoids reallocation
    despawn_buffer.clear();
    despawn_buffer.extend(query.iter().take(churn_count));

    // Despawn using drain() to consume while clearing
    for entity in despawn_buffer.drain(..) {
        commands.entity(entity).despawn();
    }

    // Pre-generate replacement entities with FastRng
    let new_entities: Vec<_> = (0..churn_count)
        .map(|_| {
            (
                BenchmarkEntity,
                Position::random_with(&mut rng.0),
                Velocity::random_with(&mut rng.0),
            )
        })
        .collect();

    commands.spawn_batch(new_entities);
}

// =============================================================================
// Component Add/Remove Workload
// =============================================================================

/// Resource tracking component toggle state
#[derive(Resource, Default)]
pub struct ComponentToggleState {
    pub initialized: bool,
    pub frame_counter: usize,
}

/// OPTIMIZATION: Efficient initial spawn for toggle workload.
///
/// Spawns half entities with ToggleComponent and half without,
/// creating two distinct archetypes for testing archetype migration.
pub fn spawn_component_toggle_entities(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnEntitiesRequest>,
    mut state: ResMut<ComponentToggleState>,
) {
    if let Some(event) = spawn_events.read().last() {
        info!(
            "Spawning {} entities for component add/remove",
            event.count
        );

        // Half with ToggleComponent, half without
        // This creates two archetypes for testing migration between them
        let half = event.count / 2;

        // Pre-allocate first batch (with toggle)
        let with_toggle: Vec<_> = (0..half)
            .map(|i| {
                (
                    BenchmarkEntity,
                    Counter { value: i as u64 },
                    ToggleComponent { value: i as u32 },
                )
            })
            .collect();

        // Pre-allocate second batch (without toggle)
        let without_toggle: Vec<_> = (half..event.count)
            .map(|i| (BenchmarkEntity, Counter { value: i as u64 }))
            .collect();

        commands.spawn_batch(with_toggle);
        commands.spawn_batch(without_toggle);

        state.initialized = true;
        state.frame_counter = 0;
    }
}

/// OPTIMIZATION: Component add/remove with batched collection.
///
/// This tests archetype migration performance - moving entities between
/// different archetype tables when components are added or removed.
///
/// **About Archetype Migration:**
/// When you add/remove components, Bevy moves the entity's data from one
/// archetype table to another. This involves:
/// 1. Allocating space in the destination archetype
/// 2. Copying all component data
/// 3. Deallocating from the source archetype
///
/// **Performance Considerations:**
/// - Archetype migration is relatively expensive (memory copies)
/// - Batching helps amortize the cost
/// - Consider using EntityVariant bitflags if entities frequently change types
/// - The alternative is to use marker components only when needed for query filtering
pub fn component_add_remove_system(
    mut commands: Commands,
    mut state: ResMut<ComponentToggleState>,
    with_toggle: Query<Entity, (With<BenchmarkEntity>, With<ToggleComponent>)>,
    without_toggle: Query<Entity, (With<BenchmarkEntity>, Without<ToggleComponent>)>,
) {
    if !state.initialized {
        return;
    }

    state.frame_counter += 1;

    // Toggle every N frames to make the benchmark measurable
    // (every frame would be too aggressive and not representative)
    if state.frame_counter % 10 != 0 {
        return;
    }

    // Limit toggle count to avoid overwhelming the command queue
    let toggle_count = with_toggle.iter().count().min(1000);

    // Collect entities first, then issue commands
    // This is slightly more efficient than interleaving iteration and commands
    let to_remove: Vec<_> = with_toggle.iter().take(toggle_count).collect();
    let to_add: Vec<_> = without_toggle.iter().take(toggle_count).collect();

    // Remove ToggleComponent from entities that have it
    for entity in to_remove {
        commands.entity(entity).remove::<ToggleComponent>();
    }

    // Add ToggleComponent to entities that don't have it
    for entity in to_add {
        commands
            .entity(entity)
            .insert(ToggleComponent { value: state.frame_counter as u32 });
    }
}

// =============================================================================
// Batch Spawn Workload
// =============================================================================

/// Test batch spawning performance specifically
pub fn batch_spawn_system(
    _commands: Commands,
    query: Query<&BenchmarkEntity>,
    mut frame: Local<usize>,
) {
    *frame += 1;

    // Every 60 frames, despawn all and respawn
    if *frame % 60 == 0 {
        let _count = query.iter().count();

        // This is handled by the benchmark runner
        // Just here to show the pattern
    }
}

// =============================================================================
// Complex Structural Changes
// =============================================================================

/// Test adding multiple components at once
pub fn multi_component_insert_system(
    mut commands: Commands,
    query: Query<Entity, (With<BenchmarkEntity>, Without<SecondaryToggle>)>,
    mut frame: Local<usize>,
) {
    *frame += 1;

    if *frame % 30 != 0 {
        return;
    }

    // Add multiple components at once to subset of entities
    for entity in query.iter().take(100) {
        commands.entity(entity).insert((
            SecondaryToggle { active: true },
            ToggleComponent {
                value: *frame as u32,
            },
        ));
    }
}

/// Test removing multiple components at once
pub fn multi_component_remove_system(
    mut commands: Commands,
    query: Query<Entity, (With<BenchmarkEntity>, With<SecondaryToggle>)>,
    mut frame: Local<usize>,
) {
    *frame += 1;

    if *frame % 30 != 15 {
        return;
    }

    // Remove multiple components from subset
    for entity in query.iter().take(100) {
        commands
            .entity(entity)
            .remove::<SecondaryToggle>()
            .remove::<ToggleComponent>();
    }
}
