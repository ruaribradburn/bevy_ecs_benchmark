//! Benchmark execution and control logic.

use bevy::ecs::message::Message;
use bevy::prelude::*;

use crate::benchmark::results::{BenchmarkComplete, BenchmarkResults};
use crate::components::BenchmarkEntity;
use crate::config::{
    GROWTH_MULTIPLIER, MAX_ENTITY_COUNT, MIN_CONVERGENCE_GAP, MIN_ENTITY_COUNT, SAMPLE_FRAMES,
    TARGET_FRAME_TIME_MS, WARMUP_FRAMES,
};
use crate::metrics::FrameMetrics;
use crate::state::{AppState, BenchmarkPhase, BenchmarkState, SelectedWorkload};

/// Plugin for benchmark execution systems
pub struct BenchmarkRunnerPlugin;

impl Plugin for BenchmarkRunnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BenchmarkComplete>()
            .add_message::<SpawnEntitiesRequest>()
            .add_message::<DespawnAllRequest>()
            .add_systems(
                Update,
                (
                    manage_benchmark_phase,
                    handle_phase_transitions,
                    collect_samples.run_if(in_state(BenchmarkPhase::Sampling)),
                    adjust_entity_count.run_if(in_state(BenchmarkPhase::Adjusting)),
                )
                    .chain()
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(
                Update,
                (handle_spawn_requests, handle_despawn_requests).chain(),
            );
    }
}

/// Event requesting entities to be spawned
#[derive(Event, Message)]
pub struct SpawnEntitiesRequest {
    pub count: usize,
}

/// Event requesting all benchmark entities to be despawned
#[derive(Event, Message)]
pub struct DespawnAllRequest;

/// Manages the benchmark phase state machine
fn manage_benchmark_phase(
    phase: Res<State<BenchmarkPhase>>,
    mut next_phase: ResMut<NextState<BenchmarkPhase>>,
    mut state: ResMut<BenchmarkState>,
    _metrics: Res<FrameMetrics>,
) {
    match phase.get() {
        BenchmarkPhase::Idle => {
            // Transition to warmup handled by start command
        }
        BenchmarkPhase::WarmUp => {
            state.frame_counter += 1;
            if state.frame_counter >= WARMUP_FRAMES {
                state.frame_counter = 0;
                next_phase.set(BenchmarkPhase::Sampling);
            }
        }
        BenchmarkPhase::Sampling => {
            // Sampling is handled by collect_samples system
        }
        BenchmarkPhase::Adjusting => {
            // Adjustment is handled by adjust_entity_count system
        }
        BenchmarkPhase::Complete => {
            // Stay complete until reset
        }
    }
}

/// Handle transitions between phases
fn handle_phase_transitions(
    mut phase_events: MessageReader<StateTransitionEvent<BenchmarkPhase>>,
    mut metrics: ResMut<FrameMetrics>,
    mut state: ResMut<BenchmarkState>,
) {
    for event in phase_events.read() {
        match event.entered {
            Some(BenchmarkPhase::WarmUp) => {
                info!("Entering warm-up phase ({} frames)", WARMUP_FRAMES);
                state.frame_counter = 0;
            }
            Some(BenchmarkPhase::Sampling) => {
                info!("Entering sampling phase ({} frames)", SAMPLE_FRAMES);
                metrics.clear_samples();
                state.frame_counter = 0;
            }
            Some(BenchmarkPhase::Adjusting) => {
                info!("Adjusting entity count based on samples");
            }
            Some(BenchmarkPhase::Complete) => {
                info!("Benchmark complete!");
            }
            Some(BenchmarkPhase::Idle) => {
                info!("Benchmark idle");
            }
            None => {}
        }
    }
}

/// Collect frame time samples during sampling phase
fn collect_samples(
    time: Res<Time>,
    mut metrics: ResMut<FrameMetrics>,
    mut state: ResMut<BenchmarkState>,
    mut next_phase: ResMut<NextState<BenchmarkPhase>>,
) {
    metrics.add_sample(time.delta_secs_f64());
    state.frame_counter += 1;

    if state.frame_counter >= SAMPLE_FRAMES {
        state.frame_counter = 0;
        next_phase.set(BenchmarkPhase::Adjusting);
    }
}

/// Adjust entity count based on collected samples using binary search
fn adjust_entity_count(
    mut state: ResMut<BenchmarkState>,
    metrics: Res<FrameMetrics>,
    mut next_phase: ResMut<NextState<BenchmarkPhase>>,
    mut spawn_events: MessageWriter<SpawnEntitiesRequest>,
    mut despawn_events: MessageWriter<DespawnAllRequest>,
    mut complete_events: MessageWriter<BenchmarkComplete>,
    workload: Res<SelectedWorkload>,
    mut results: ResMut<BenchmarkResults>,
) {
    let stats = metrics.sample_stats();
    let exceeds_target = stats.median_exceeds(TARGET_FRAME_TIME_MS);

    info!(
        "Entity count: {} | Median frame time: {:.2}ms | Target: {:.2}ms | {}",
        state.entity_count,
        stats.median,
        TARGET_FRAME_TIME_MS,
        if exceeds_target { "OVER" } else { "UNDER" }
    );

    // Binary search logic
    if exceeds_target {
        // We're over the target, need fewer entities
        state.search_high = state.entity_count;
    } else {
        // We're under the target, can handle more
        state.search_low = state.entity_count;
    }

    // Check if we've converged (within 2% or absolute minimum gap)
    let gap = state.search_high - state.search_low;
    let relative_gap = gap as f64 / state.entity_count as f64;

    if relative_gap < 0.02 || gap < MIN_CONVERGENCE_GAP {
        // We've found the breakdown point
        let breakdown = if exceeds_target {
            state.search_low
        } else {
            state.entity_count
        };

        info!("Breakdown point found: {} entities", breakdown);

        // Calculate throughput at breakdown
        let throughput = breakdown as f64 * (1000.0 / stats.median);

        // Record results
        results.record_workload_result(*workload, breakdown, throughput, stats);

        // Signal completion
        complete_events.write(BenchmarkComplete {
            workload: *workload,
            breakdown_point: breakdown,
            throughput,
        });

        next_phase.set(BenchmarkPhase::Complete);
        return;
    }

    // Calculate next entity count
    let next_count = if !exceeds_target && state.search_high == MAX_ENTITY_COUNT {
        // Still in exponential growth phase
        ((state.entity_count as f64) * GROWTH_MULTIPLIER) as usize
    } else {
        // Binary search phase
        (state.search_low + state.search_high) / 2
    }
    .clamp(MIN_ENTITY_COUNT, MAX_ENTITY_COUNT);

    // Despawn all and spawn new count
    despawn_events.write(DespawnAllRequest);
    state.entity_count = next_count;
    spawn_events.write(SpawnEntitiesRequest { count: next_count });

    // Go back to warmup
    next_phase.set(BenchmarkPhase::WarmUp);
}

/// Handle spawn requests (actual spawning done by workload systems)
fn handle_spawn_requests(
    mut events: MessageReader<SpawnEntitiesRequest>,
) {
    for event in events.read() {
        info!("Spawn request: {} entities", event.count);
        // Actual spawning is handled by workload-specific systems
    }
}

/// Handle despawn requests
fn handle_despawn_requests(
    mut commands: Commands,
    mut events: MessageReader<DespawnAllRequest>,
    query: Query<Entity, With<BenchmarkEntity>>,
) {
    for _event in events.read() {
        let count = query.iter().count();
        info!("Despawning {} benchmark entities", count);
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Start a benchmark run
pub fn start_benchmark(
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_phase: ResMut<NextState<BenchmarkPhase>>,
    mut state: ResMut<BenchmarkState>,
    mut spawn_events: MessageWriter<SpawnEntitiesRequest>,
    mut metrics: ResMut<FrameMetrics>,
) {
    info!("Starting benchmark...");

    // Reset state
    state.reset();
    metrics.reset();

    // Spawn initial entities
    spawn_events.write(SpawnEntitiesRequest {
        count: state.entity_count,
    });

    // Transition to running
    next_app_state.set(AppState::Running);
    next_phase.set(BenchmarkPhase::WarmUp);
}

/// Stop/pause the benchmark
pub fn stop_benchmark(
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_phase: ResMut<NextState<BenchmarkPhase>>,
    mut despawn_events: MessageWriter<DespawnAllRequest>,
) {
    info!("Stopping benchmark...");
    despawn_events.write(DespawnAllRequest);
    next_phase.set(BenchmarkPhase::Idle);
    next_app_state.set(AppState::Paused);
}

/// Reset the benchmark
pub fn reset_benchmark(
    mut next_phase: ResMut<NextState<BenchmarkPhase>>,
    mut state: ResMut<BenchmarkState>,
    mut metrics: ResMut<FrameMetrics>,
    mut despawn_events: MessageWriter<DespawnAllRequest>,
) {
    info!("Resetting benchmark...");
    state.reset();
    metrics.reset();
    despawn_events.write(DespawnAllRequest);
    next_phase.set(BenchmarkPhase::Idle);
}
