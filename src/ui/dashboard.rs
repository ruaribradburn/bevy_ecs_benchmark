//! Main dashboard UI layout and update systems.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use crate::config::{colors, sizes, TARGET_FRAME_TIME_MS};
use crate::metrics::{format_count, format_throughput, FrameMetrics};
use crate::state::{BenchmarkPhase, BenchmarkState, SelectedWorkload};
use crate::ui::styles::*;

// =============================================================================
// UI Marker Components
// =============================================================================

#[derive(Component)]
pub struct UiRoot;

#[derive(Component)]
pub struct EntityCountText;

#[derive(Component)]
pub struct FrameTimeText;

#[derive(Component)]
pub struct ThroughputText;

#[derive(Component)]
pub struct PhaseText;

#[derive(Component)]
pub struct WorkloadText;

#[derive(Component)]
pub struct GraphContainer;

#[derive(Component)]
pub struct GraphBar {
    pub index: usize,
}

#[derive(Component)]
pub struct TargetLine;

#[derive(Component)]
pub struct ControlsHint;

#[derive(Component)]
pub struct WorkloadDescriptionText;

// =============================================================================
// UI Setup
// =============================================================================

pub fn setup_ui(mut commands: Commands) {
    // Root container - full screen
    commands
        .spawn((
            UiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
        ))
        .with_children(|parent| {
            // Left sidebar
            spawn_sidebar(parent);

            // Main content area (graph)
            spawn_main_content(parent);
        });
}

fn spawn_sidebar(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                width: sizes::SIDEBAR_WIDTH,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(colors::PANEL_BG),
        ))
        .with_children(|sidebar| {
            // Title
            sidebar.spawn((
                Text::new("ECS Benchmark"),
                title_text_font(),
                TextColor(colors::TEXT_PRIMARY),
            ));

            sidebar.spawn((
                Text::new("Bevy 0.17.3"),
                small_text_font(),
                TextColor(colors::TEXT_SECONDARY),
            ));

            // Spacing
            sidebar.spawn(section_spacing());

            // Current workload section
            spawn_workload_section(sidebar);

            // Metrics section
            spawn_metrics_section(sidebar);

            // Controls hint
            spawn_controls_section(sidebar);
        });
}

fn spawn_workload_section(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Text::new("Workload"),
        heading_text_font(),
        TextColor(colors::TEXT_SECONDARY),
    ));

    parent.spawn((
        WorkloadText,
        Text::new("Simple Iteration"),
        body_text_font(),
        TextColor(colors::ACCENT),
        Node {
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        },
    ));

    // Phase indicator
    parent.spawn((
        Text::new("Phase"),
        small_text_font(),
        TextColor(colors::TEXT_SECONDARY),
    ));

    parent.spawn((
        PhaseText,
        Text::new("Idle"),
        body_text_font(),
        TextColor(colors::TEXT_PRIMARY),
        Node {
            margin: UiRect::bottom(Val::Px(16.0)),
            ..default()
        },
    ));
}

fn spawn_metrics_section(parent: &mut ChildSpawnerCommands) {
    // Entity count - large prominent display
    parent.spawn((
        Text::new("Entities"),
        small_text_font(),
        TextColor(colors::TEXT_SECONDARY),
    ));

    parent.spawn((
        EntityCountText,
        Text::new("0"),
        large_metric_font(),
        TextColor(colors::TEXT_PRIMARY),
        Node {
            margin: UiRect::bottom(Val::Px(16.0)),
            ..default()
        },
    ));

    // Frame time
    parent.spawn((
        Text::new("Frame Time"),
        small_text_font(),
        TextColor(colors::TEXT_SECONDARY),
    ));

    parent.spawn((
        FrameTimeText,
        Text::new("0.00ms"),
        heading_text_font(),
        TextColor(colors::SUCCESS),
        Node {
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        },
    ));

    // Target indicator
    parent.spawn((
        Text::new(format!("Target: {:.1}ms (60 FPS)", TARGET_FRAME_TIME_MS)),
        small_text_font(),
        TextColor(colors::TEXT_SECONDARY),
        Node {
            margin: UiRect::bottom(Val::Px(16.0)),
            ..default()
        },
    ));

    // Throughput
    parent.spawn((
        Text::new("Throughput"),
        small_text_font(),
        TextColor(colors::TEXT_SECONDARY),
    ));

    parent.spawn((
        ThroughputText,
        Text::new("0/s"),
        heading_text_font(),
        TextColor(colors::ACCENT),
        Node {
            margin: UiRect::bottom(Val::Px(16.0)),
            ..default()
        },
    ));
}

fn spawn_controls_section(parent: &mut ChildSpawnerCommands) {
    parent.spawn(section_spacing());

    parent.spawn((
        Text::new("Controls"),
        heading_text_font(),
        TextColor(colors::TEXT_SECONDARY),
        Node {
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        },
    ));

    let controls = [
        ("1-6", "Select workload"),
        ("Space", "Start/Pause"),
        ("R", "Reset"),
        ("Enter", "Run full suite"),
        ("Up/Down", "Adjust count"),
        ("S", "Save results"),
        ("Esc", "Exit"),
    ];

    for (key, action) in controls {
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(format!("{:<8}", key)),
                    small_text_font(),
                    TextColor(colors::ACCENT),
                ));
                row.spawn((
                    Text::new(action),
                    small_text_font(),
                    TextColor(colors::TEXT_SECONDARY),
                ));
            });
    }
}

fn spawn_main_content(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(24.0)),
                ..default()
            },
        ))
        .with_children(|main| {
            // Graph title
            main.spawn((
                Text::new("Frame Time History"),
                heading_text_font(),
                TextColor(colors::TEXT_PRIMARY),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            // Graph container
            main.spawn((
                GraphContainer,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(300.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexEnd,
                    column_gap: Val::Px(1.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            ))
            .with_children(|graph| {
                // Spawn graph bars
                for i in 0..300 {
                    graph.spawn((
                        GraphBar { index: i },
                        Node {
                            width: Val::Px(2.0),
                            height: Val::Px(0.0),
                            ..default()
                        },
                        BackgroundColor(colors::GRAPH_LINE),
                    ));
                }
            });

            // Legend
            main.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            })
            .with_children(|legend| {
                // Frame time legend
                legend.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|item| {
                    item.spawn((
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(3.0),
                            ..default()
                        },
                        BackgroundColor(colors::GRAPH_LINE),
                    ));
                    item.spawn((
                        Text::new("Frame time"),
                        small_text_font(),
                        TextColor(colors::TEXT_SECONDARY),
                    ));
                });

                // Target legend
                legend.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|item| {
                    item.spawn((
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(3.0),
                            ..default()
                        },
                        BackgroundColor(colors::GRAPH_TARGET),
                    ));
                    item.spawn((
                        Text::new("60 FPS target"),
                        small_text_font(),
                        TextColor(colors::TEXT_SECONDARY),
                    ));
                });
            });

            // Workload selection hints
            main.spawn(section_spacing());
            spawn_workload_hints(main);
        });
}

fn spawn_workload_hints(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Text::new("Available Workloads"),
        heading_text_font(),
        TextColor(colors::TEXT_PRIMARY),
        Node {
            margin: UiRect::bottom(Val::Px(12.0)),
            ..default()
        },
    ));

    let workloads = SelectedWorkload::all();

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(16.0),
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|grid| {
            for workload in workloads {
                grid.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    min_width: Val::Px(200.0),
                    ..default()
                })
                .with_children(|item| {
                    // Key badge
                    item.spawn((
                        Node {
                            padding: UiRect::new(
                                Val::Px(8.0),
                                Val::Px(8.0),
                                Val::Px(4.0),
                                Val::Px(4.0),
                            ),
                            ..default()
                        },
                        BackgroundColor(colors::ACCENT),
                    ))
                    .with_children(|badge| {
                        badge.spawn((
                            Text::new(workload.key_hint()),
                            small_text_font(),
                            TextColor(colors::BACKGROUND),
                        ));
                    });

                    // Name
                    item.spawn((
                        Text::new(workload.name()),
                        small_text_font(),
                        TextColor(colors::TEXT_SECONDARY),
                    ));
                });
            }
        });

    // Workload description
    parent.spawn((
        WorkloadDescriptionText,
        Text::new(SelectedWorkload::default().description()),
        body_text_font(),
        TextColor(colors::TEXT_SECONDARY),
        Node {
            margin: UiRect::top(Val::Px(16.0)),
            ..default()
        },
    ));
}

// =============================================================================
// Update Systems
// =============================================================================

pub fn update_entity_count_display(
    state: Res<BenchmarkState>,
    query: Single<&mut Text, With<EntityCountText>>,
) {
    let mut text = query.into_inner();
    **text = format_count(state.entity_count);
}

pub fn update_frame_time_display(
    metrics: Res<FrameMetrics>,
    query: Single<(&mut Text, &mut TextColor), With<FrameTimeText>>,
) {
    let (mut text, mut color) = query.into_inner();
    let frame_time = metrics.current_frame_time;
    **text = format!("{:.2}ms", frame_time);
    color.0 = frame_time_color(frame_time, TARGET_FRAME_TIME_MS);
}

pub fn update_throughput_display(
    metrics: Res<FrameMetrics>,
    query: Single<&mut Text, With<ThroughputText>>,
) {
    let mut text = query.into_inner();
    **text = format_throughput(metrics.throughput);
}

pub fn update_phase_display(
    phase: Res<State<BenchmarkPhase>>,
    query: Single<(&mut Text, &mut TextColor), With<PhaseText>>,
) {
    let (mut text, mut color) = query.into_inner();
    let (phase_name, phase_color) = match phase.get() {
        BenchmarkPhase::Idle => ("Idle", colors::TEXT_SECONDARY),
        BenchmarkPhase::WarmUp => ("Warming up...", colors::WARNING),
        BenchmarkPhase::Sampling => ("Sampling", colors::ACCENT),
        BenchmarkPhase::Adjusting => ("Adjusting", colors::WARNING),
        BenchmarkPhase::Complete => ("Complete!", colors::SUCCESS),
    };
    **text = phase_name.to_string();
    color.0 = phase_color;
}

pub fn update_workload_display(
    workload: Res<SelectedWorkload>,
    query: Single<&mut Text, With<WorkloadText>>,
) {
    let mut text = query.into_inner();
    **text = workload.name().to_string();
}

pub fn update_workload_description_display(
    workload: Res<SelectedWorkload>,
    query: Single<&mut Text, With<WorkloadDescriptionText>>,
) {
    let mut text = query.into_inner();
    **text = workload.description().to_string();
}
