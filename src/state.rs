//! Application state management for the benchmark suite.

use bevy::prelude::*;

/// Main application states
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    /// Initial state, showing menu
    #[default]
    Menu,
    /// Running a benchmark
    Running,
    /// Benchmark paused
    Paused,
    /// Showing results summary
    Results,
}

/// Benchmark execution phase
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum BenchmarkPhase {
    /// No benchmark active
    #[default]
    Idle,
    /// Warming up (skipping initial frames)
    WarmUp,
    /// Collecting samples
    Sampling,
    /// Adjusting entity count (binary search)
    Adjusting,
    /// Benchmark complete
    Complete,
}

/// Currently selected workload type
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Resource)]
pub enum SelectedWorkload {
    #[default]
    SimpleIteration,
    MultiComponentRead,
    PositionVelocity,
    SpawnDespawn,
    ComponentAddRemove,
    FragmentedArchetypes,
}

impl SelectedWorkload {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SimpleIteration => "Simple Iteration",
            Self::MultiComponentRead => "Multi-Component Read",
            Self::PositionVelocity => "Position/Velocity Update",
            Self::SpawnDespawn => "Spawn/Despawn Churn",
            Self::ComponentAddRemove => "Component Add/Remove",
            Self::FragmentedArchetypes => "Fragmented Archetypes",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::SimpleIteration => "Read-only iteration over single component",
            Self::MultiComponentRead => "Read 3 components per entity",
            Self::PositionVelocity => "Classic game loop: position += velocity",
            Self::SpawnDespawn => "Spawn and despawn entities each frame",
            Self::ComponentAddRemove => "Add/remove components on existing entities",
            Self::FragmentedArchetypes => "Entities spread across many archetypes",
        }
    }

    pub fn key_hint(&self) -> &'static str {
        match self {
            Self::SimpleIteration => "1",
            Self::MultiComponentRead => "2",
            Self::PositionVelocity => "3",
            Self::SpawnDespawn => "4",
            Self::ComponentAddRemove => "5",
            Self::FragmentedArchetypes => "6",
        }
    }

    pub fn all() -> &'static [SelectedWorkload] {
        &[
            Self::SimpleIteration,
            Self::MultiComponentRead,
            Self::PositionVelocity,
            Self::SpawnDespawn,
            Self::ComponentAddRemove,
            Self::FragmentedArchetypes,
        ]
    }

    pub fn from_key(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Digit1 | KeyCode::Numpad1 => Some(Self::SimpleIteration),
            KeyCode::Digit2 | KeyCode::Numpad2 => Some(Self::MultiComponentRead),
            KeyCode::Digit3 | KeyCode::Numpad3 => Some(Self::PositionVelocity),
            KeyCode::Digit4 | KeyCode::Numpad4 => Some(Self::SpawnDespawn),
            KeyCode::Digit5 | KeyCode::Numpad5 => Some(Self::ComponentAddRemove),
            KeyCode::Digit6 | KeyCode::Numpad6 => Some(Self::FragmentedArchetypes),
            _ => None,
        }
    }
}

/// Resource tracking the current benchmark configuration
#[derive(Resource)]
pub struct BenchmarkState {
    /// Current entity count being tested
    pub entity_count: usize,
    /// Lower bound for binary search
    pub search_low: usize,
    /// Upper bound for binary search
    pub search_high: usize,
    /// Frame counter for warm-up/sampling phases
    pub frame_counter: usize,
    /// Whether we're running in automated mode
    pub automated: bool,
    /// Index of current workload in automated suite
    pub suite_index: usize,
}

impl Default for BenchmarkState {
    fn default() -> Self {
        Self {
            entity_count: crate::config::INITIAL_ENTITY_COUNT,
            search_low: crate::config::MIN_ENTITY_COUNT,
            search_high: crate::config::MAX_ENTITY_COUNT,
            frame_counter: 0,
            automated: false,
            suite_index: 0,
        }
    }
}

impl BenchmarkState {
    pub fn reset(&mut self) {
        self.entity_count = crate::config::INITIAL_ENTITY_COUNT;
        self.search_low = crate::config::MIN_ENTITY_COUNT;
        self.search_high = crate::config::MAX_ENTITY_COUNT;
        self.frame_counter = 0;
    }

    pub fn reset_for_new_workload(&mut self) {
        self.entity_count = crate::config::INITIAL_ENTITY_COUNT;
        self.search_low = crate::config::MIN_ENTITY_COUNT;
        self.search_high = crate::config::MAX_ENTITY_COUNT;
        self.frame_counter = 0;
    }
}
