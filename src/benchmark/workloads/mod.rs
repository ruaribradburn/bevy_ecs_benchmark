//! Benchmark workload definitions.
//!
//! Each workload tests a different aspect of ECS performance.
//!
//! # System Ordering and Parallelism
//!
//! Bevy automatically runs systems in parallel when they don't conflict
//! (i.e., they don't access the same resources/components mutably).
//!
//! SystemSets help organize systems for:
//! - **Explicit ordering**: Ensure spawn happens before iteration
//! - **Grouping**: Apply run conditions to multiple systems at once
//! - **Parallel optimization**: Systems in different sets can run in parallel
//!
//! The benchmark uses sets to ensure:
//! 1. Spawn systems run before workload systems
//! 2. Multiple independent workload systems could theoretically run in parallel

mod fragmentation;
mod iteration;
mod mutation;
mod structural;

pub use fragmentation::*;
pub use iteration::*;
pub use mutation::*;
pub use structural::*;

use bevy::prelude::*;
use bevy::ecs::schedule::SystemSet;

use crate::benchmark::runner::SpawnEntitiesRequest;
use crate::components::FastRng;
use crate::state::{AppState, SelectedWorkload};

// =============================================================================
// System Sets for Organized Execution
// =============================================================================

/// OPTIMIZATION: SystemSets for explicit ordering and parallel execution.
///
/// Using SystemSets provides several benefits:
/// - **Clear execution order**: Spawn → Process → Cleanup
/// - **Parallel-friendly**: Systems in the same phase can run in parallel
/// - **Grouped run conditions**: Apply `run_if` to entire phases
///
/// Without explicit sets, Bevy still parallelizes automatically, but sets
/// make the execution order explicit and easier to reason about.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BenchmarkSet {
    /// Systems that spawn entities (run first)
    Spawn,
    /// Systems that process/iterate entities (run after spawn)
    Process,
}

/// Plugin that registers all workload systems
pub struct WorkloadsPlugin;

impl Plugin for WorkloadsPlugin {
    fn build(&self, app: &mut App) {
        // Initialize FastRng resource for optimized random number generation
        app.init_resource::<FastRng>();

        // Configure system set ordering: Spawn → Process
        // This ensures entities exist before systems try to iterate them
        app.configure_sets(
            Update,
            (BenchmarkSet::Spawn, BenchmarkSet::Process).chain(),
        );

        app
            // =================================================================
            // Iteration workloads
            // =================================================================
            .add_systems(
                Update,
                spawn_simple_iteration_entities
                    .in_set(BenchmarkSet::Spawn)
                    .run_if(resource_equals(SelectedWorkload::SimpleIteration))
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(
                Update,
                simple_iteration_system
                    .in_set(BenchmarkSet::Process)
                    .run_if(resource_equals(SelectedWorkload::SimpleIteration))
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(
                Update,
                spawn_multi_component_entities
                    .in_set(BenchmarkSet::Spawn)
                    .run_if(resource_equals(SelectedWorkload::MultiComponentRead))
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(
                Update,
                multi_component_read_system
                    .in_set(BenchmarkSet::Process)
                    .run_if(resource_equals(SelectedWorkload::MultiComponentRead))
                    .run_if(in_state(AppState::Running)),
            )
            // =================================================================
            // Mutation workloads (now with parallel iteration)
            // =================================================================
            .add_systems(
                Update,
                spawn_position_velocity_entities
                    .in_set(BenchmarkSet::Spawn)
                    .run_if(resource_equals(SelectedWorkload::PositionVelocity))
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(
                Update,
                position_velocity_system
                    .in_set(BenchmarkSet::Process)
                    .run_if(resource_equals(SelectedWorkload::PositionVelocity))
                    .run_if(in_state(AppState::Running)),
            )
            // =================================================================
            // Structural workloads
            // =================================================================
            .add_systems(
                Update,
                spawn_despawn_setup
                    .in_set(BenchmarkSet::Spawn)
                    .run_if(resource_equals(SelectedWorkload::SpawnDespawn))
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(
                Update,
                spawn_despawn_churn_system
                    .in_set(BenchmarkSet::Process)
                    .run_if(resource_equals(SelectedWorkload::SpawnDespawn))
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(
                Update,
                spawn_component_toggle_entities
                    .in_set(BenchmarkSet::Spawn)
                    .run_if(resource_equals(SelectedWorkload::ComponentAddRemove))
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(
                Update,
                component_add_remove_system
                    .in_set(BenchmarkSet::Process)
                    .run_if(resource_equals(SelectedWorkload::ComponentAddRemove))
                    .run_if(in_state(AppState::Running)),
            )
            // =================================================================
            // Fragmentation workloads
            // =================================================================
            .add_systems(
                Update,
                spawn_fragmented_entities
                    .in_set(BenchmarkSet::Spawn)
                    .run_if(resource_equals(SelectedWorkload::FragmentedArchetypes))
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(
                Update,
                fragmented_iteration_system
                    .in_set(BenchmarkSet::Process)
                    .run_if(resource_equals(SelectedWorkload::FragmentedArchetypes))
                    .run_if(in_state(AppState::Running)),
            );
    }
}

/// Helper to check which entities need spawning
fn needs_spawn(
    spawn_events: &mut MessageReader<SpawnEntitiesRequest>,
) -> Option<usize> {
    spawn_events.read().last().map(|e| e.count)
}
