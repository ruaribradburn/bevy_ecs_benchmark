//! Main benchmark plugin that coordinates all subsystems.

use bevy::prelude::*;

use crate::benchmark::results::{BenchmarkComplete, BenchmarkResults, SaveResultsRequest};
use crate::benchmark::runner::{BenchmarkRunnerPlugin, DespawnAllRequest, SpawnEntitiesRequest};
use crate::benchmark::workloads::{ComponentToggleState, SpawnDespawnState, WorkloadsPlugin};
use crate::components::BenchmarkEntity;
use crate::config::TARGET_FRAME_TIME_MS;
use crate::metrics::FrameMetrics;
use crate::state::{AppState, BenchmarkPhase, BenchmarkState, SelectedWorkload};
use crate::ui::BenchmarkUiPlugin;

/// Main plugin for the benchmark suite
pub struct BenchmarkPlugin;

impl Plugin for BenchmarkPlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .init_state::<AppState>()
            .init_state::<BenchmarkPhase>()
            // Resources
            .init_resource::<SelectedWorkload>()
            .init_resource::<BenchmarkState>()
            .init_resource::<FrameMetrics>()
            .init_resource::<BenchmarkResults>()
            .init_resource::<SpawnDespawnState>()
            .init_resource::<ComponentToggleState>()
            // Events
            .add_message::<SaveResultsRequest>()
            // Sub-plugins
            .add_plugins(BenchmarkRunnerPlugin)
            .add_plugins(WorkloadsPlugin)
            .add_plugins(BenchmarkUiPlugin)
            // Core systems
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (
                    handle_input,
                    update_metrics,
                    handle_benchmark_complete,
                    handle_save_request,
                ),
            );
    }
}

/// Setup the camera for UI rendering
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Handle keyboard input for benchmark control
fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    app_state: Res<State<AppState>>,
    _phase: Res<State<BenchmarkPhase>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_phase: ResMut<NextState<BenchmarkPhase>>,
    mut workload: ResMut<SelectedWorkload>,
    mut state: ResMut<BenchmarkState>,
    mut metrics: ResMut<FrameMetrics>,
    mut spawn_events: MessageWriter<SpawnEntitiesRequest>,
    mut despawn_events: MessageWriter<DespawnAllRequest>,
    mut save_events: MessageWriter<SaveResultsRequest>,
    mut results: ResMut<BenchmarkResults>,
    mut exit: MessageWriter<AppExit>,
) {
    // Escape to exit
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
        return;
    }

    // Workload selection (1-6)
    for key in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Numpad1,
        KeyCode::Numpad2,
        KeyCode::Numpad3,
        KeyCode::Numpad4,
        KeyCode::Numpad5,
        KeyCode::Numpad6,
    ] {
        if keyboard.just_pressed(key) {
            if let Some(new_workload) = SelectedWorkload::from_key(key) {
                // Stop current benchmark if running
                if *app_state.get() == AppState::Running {
                    despawn_events.write(DespawnAllRequest);
                    next_phase.set(BenchmarkPhase::Idle);
                    next_app_state.set(AppState::Menu);
                }

                *workload = new_workload;
                state.reset_for_new_workload();
                info!("Selected workload: {}", new_workload.name());
            }
        }
    }

    // Space to start/pause
    if keyboard.just_pressed(KeyCode::Space) {
        match app_state.get() {
            AppState::Menu | AppState::Paused => {
                info!("Starting benchmark: {}", workload.name());
                state.reset();
                metrics.reset();
                spawn_events.write(SpawnEntitiesRequest {
                    count: state.entity_count,
                });
                next_app_state.set(AppState::Running);
                next_phase.set(BenchmarkPhase::WarmUp);
            }
            AppState::Running => {
                info!("Pausing benchmark");
                next_app_state.set(AppState::Paused);
                next_phase.set(BenchmarkPhase::Idle);
            }
            AppState::Results => {
                // Return to menu
                next_app_state.set(AppState::Menu);
            }
        }
    }

    // R to reset
    if keyboard.just_pressed(KeyCode::KeyR) {
        info!("Resetting benchmark");
        despawn_events.write(DespawnAllRequest);
        state.reset();
        metrics.reset();
        next_phase.set(BenchmarkPhase::Idle);
        next_app_state.set(AppState::Menu);
    }

    // Enter to run full automated suite
    if keyboard.just_pressed(KeyCode::Enter) {
        if *app_state.get() != AppState::Running {
            info!("Starting automated benchmark suite");
            results.start_new_report(TARGET_FRAME_TIME_MS);
            state.automated = true;
            state.suite_index = 0;
            *workload = *SelectedWorkload::all().first().unwrap();

            // Start first benchmark
            state.reset();
            metrics.reset();
            spawn_events.write(SpawnEntitiesRequest {
                count: state.entity_count,
            });
            next_app_state.set(AppState::Running);
            next_phase.set(BenchmarkPhase::WarmUp);
        }
    }

    // Up/Down to manually adjust entity count
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let step = if shift {
        crate::config::MANUAL_STEP_SIZE_LARGE
    } else {
        crate::config::MANUAL_STEP_SIZE
    };

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        let new_count = (state.entity_count + step).min(crate::config::MAX_ENTITY_COUNT);
        if *app_state.get() == AppState::Running {
            despawn_events.write(DespawnAllRequest);
            state.entity_count = new_count;
            spawn_events.write(SpawnEntitiesRequest { count: new_count });
            next_phase.set(BenchmarkPhase::WarmUp);
        } else {
            state.entity_count = new_count;
        }
        info!("Entity count: {}", state.entity_count);
    }

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        let new_count = state.entity_count.saturating_sub(step).max(crate::config::MIN_ENTITY_COUNT);
        if *app_state.get() == AppState::Running {
            despawn_events.write(DespawnAllRequest);
            state.entity_count = new_count;
            spawn_events.write(SpawnEntitiesRequest { count: new_count });
            next_phase.set(BenchmarkPhase::WarmUp);
        } else {
            state.entity_count = new_count;
        }
        info!("Entity count: {}", state.entity_count);
    }

    // S to save results
    if keyboard.just_pressed(KeyCode::KeyS) {
        save_events.write(SaveResultsRequest);
    }
}

/// Update frame metrics each frame
fn update_metrics(
    time: Res<Time>,
    mut metrics: ResMut<FrameMetrics>,
    _state: Res<BenchmarkState>,
    query: Query<&BenchmarkEntity>,
) {
    let entity_count = query.iter().count();
    metrics.record_frame(time.delta_secs_f64(), entity_count);
}

/// Handle benchmark completion - advance to next workload in automated mode
fn handle_benchmark_complete(
    mut events: MessageReader<BenchmarkComplete>,
    mut state: ResMut<BenchmarkState>,
    mut workload: ResMut<SelectedWorkload>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_phase: ResMut<NextState<BenchmarkPhase>>,
    mut spawn_events: MessageWriter<SpawnEntitiesRequest>,
    mut despawn_events: MessageWriter<DespawnAllRequest>,
    mut metrics: ResMut<FrameMetrics>,
) {
    for event in events.read() {
        info!(
            "Benchmark complete for {}: {} entities @ {:.0} entities/s",
            event.workload.name(),
            event.breakdown_point,
            event.throughput
        );

        if state.automated {
            // Advance to next workload
            state.suite_index += 1;
            let workloads = SelectedWorkload::all();

            if state.suite_index < workloads.len() {
                // More workloads to test
                *workload = workloads[state.suite_index];
                info!("Advancing to next workload: {}", workload.name());

                despawn_events.write(DespawnAllRequest);
                state.reset_for_new_workload();
                metrics.reset();

                // Small delay, then start next
                spawn_events.write(SpawnEntitiesRequest {
                    count: state.entity_count,
                });
                next_phase.set(BenchmarkPhase::WarmUp);
            } else {
                // Suite complete
                info!("Automated suite complete!");
                state.automated = false;
                next_app_state.set(AppState::Results);
                next_phase.set(BenchmarkPhase::Complete);
            }
        }
    }
}

/// Handle save results request
fn handle_save_request(
    mut events: MessageReader<SaveResultsRequest>,
    results: Res<BenchmarkResults>,
) {
    for _event in events.read() {
        match results.save_report() {
            Ok(filename) => {
                info!("Results saved to: {}", filename);
            }
            Err(e) => {
                error!("Failed to save results: {}", e);
            }
        }
    }
}
