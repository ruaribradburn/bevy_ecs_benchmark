//! Concrete component definitions for benchmarks.
//!
//! This module contains optimized component definitions demonstrating
//! cache-friendly patterns and efficient ECS design.

use bevy::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

// =============================================================================
// Marker Components
// =============================================================================

/// Minimal marker component for pure iteration overhead testing
#[derive(Component, Default, Clone, Copy)]
pub struct BenchmarkEntity;

/// Marker for entities that should be despawned
#[derive(Component)]
pub struct MarkedForDespawn;

// =============================================================================
// Fast Random Number Generator Resource
// =============================================================================

/// OPTIMIZATION: Fast, seedable RNG resource for bulk entity generation.
///
/// Using `thread_rng()` in each component's `random()` method creates a new
/// RNG instance per call, which has significant overhead. This resource uses
/// Xoshiro256++, a fast PRNG that's ~3x faster than ChaCha (thread_rng default).
///
/// Benefits:
/// - Single RNG instance reused across all spawn operations
/// - Deterministic results when seeded (useful for reproducible benchmarks)
/// - Significantly reduced overhead in bulk spawn operations
///
/// Usage:
/// ```rust
/// fn spawn_system(mut rng: ResMut<FastRng>, mut commands: Commands) {
///     let pos = Position::random_with(&mut rng.0);
/// }
/// ```
#[derive(Resource)]
pub struct FastRng(pub Xoshiro256PlusPlus);

impl Default for FastRng {
    fn default() -> Self {
        // Seed from entropy for production, use fixed seed for reproducible benchmarks
        Self(Xoshiro256PlusPlus::seed_from_u64(42))
    }
}

impl FastRng {
    /// Create with a specific seed for reproducible benchmarks
    pub fn with_seed(seed: u64) -> Self {
        Self(Xoshiro256PlusPlus::seed_from_u64(seed))
    }

    /// Create with random seed from system entropy
    pub fn from_entropy() -> Self {
        Self(Xoshiro256PlusPlus::from_entropy())
    }
}

// =============================================================================
// Transform-like Components (common game patterns)
// =============================================================================

/// Position component - simulates Transform position
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Generate random position using thread_rng (convenient but slower)
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self::random_with(&mut rng)
    }

    /// OPTIMIZATION: Generate random position using provided RNG.
    ///
    /// This avoids the overhead of creating a new thread_rng() for each call.
    /// Use with FastRng resource for bulk spawning operations.
    #[inline]
    pub fn random_with<R: Rng>(rng: &mut R) -> Self {
        Self {
            x: rng.gen_range(-1000.0..1000.0),
            y: rng.gen_range(-1000.0..1000.0),
            z: rng.gen_range(-1000.0..1000.0),
        }
    }
}

/// Velocity component for movement systems
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Generate random velocity using thread_rng (convenient but slower)
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self::random_with(&mut rng)
    }

    /// OPTIMIZATION: Generate random velocity using provided RNG.
    #[inline]
    pub fn random_with<R: Rng>(rng: &mut R) -> Self {
        Self {
            x: rng.gen_range(-10.0..10.0),
            y: rng.gen_range(-10.0..10.0),
            z: rng.gen_range(-10.0..10.0),
        }
    }
}

/// Acceleration component for physics-like updates
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Acceleration {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Acceleration {
    /// Generate random acceleration using thread_rng (convenient but slower)
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self::random_with(&mut rng)
    }

    /// OPTIMIZATION: Generate random acceleration using provided RNG.
    #[inline]
    pub fn random_with<R: Rng>(rng: &mut R) -> Self {
        Self {
            x: rng.gen_range(-1.0..1.0),
            y: rng.gen_range(-1.0..1.0),
            z: rng.gen_range(-1.0..1.0),
        }
    }
}

// =============================================================================
// Data-heavy Components
// =============================================================================

/// OPTIMIZATION: Cache-aligned component for optimal memory access.
///
/// The `#[repr(C, align(64))]` ensures this component:
/// - Is laid out in memory exactly as defined (C representation)
/// - Starts on a 64-byte boundary (typical CPU cache line size)
///
/// Benefits:
/// - Prevents false sharing in parallel iteration
/// - Ensures the entire payload fits in a single cache line
/// - Reduces cache misses when iterating sequentially
///
/// The 16 f32 values = 64 bytes exactly matches one cache line.
#[derive(Component, Clone, Copy)]
#[repr(C, align(64))]
pub struct DataPayload {
    pub values: [f32; 16],
}

impl Default for DataPayload {
    fn default() -> Self {
        Self { values: [0.0; 16] }
    }
}

impl DataPayload {
    /// Generate random payload using thread_rng (convenient but slower)
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self::random_with(&mut rng)
    }

    /// OPTIMIZATION: Generate random payload using provided RNG.
    #[inline]
    pub fn random_with<R: Rng>(rng: &mut R) -> Self {
        let mut values = [0.0; 16];
        for v in &mut values {
            *v = rng.gen_range(-100.0..100.0);
        }
        Self { values }
    }

    /// Perform some computation to prevent optimization
    #[inline(always)]
    pub fn process(&mut self) {
        for i in 0..15 {
            self.values[i] = self.values[i].mul_add(0.99, self.values[i + 1] * 0.01);
        }
        self.values[15] = self.values[15].mul_add(0.99, self.values[0] * 0.01);
    }
}

/// Even larger component for stress testing (spans multiple cache lines)
#[derive(Component, Clone, Copy)]
#[repr(C, align(64))]
pub struct HeavyPayload {
    pub data: [f32; 64],
}

impl Default for HeavyPayload {
    fn default() -> Self {
        Self { data: [0.0; 64] }
    }
}

// =============================================================================
// Archetype Fragmentation Components
// =============================================================================

/// Variant components for creating archetype fragmentation
/// These are mutually exclusive to create different archetypes
#[derive(Component, Default, Clone, Copy)]
pub struct VariantA;

#[derive(Component, Default, Clone, Copy)]
pub struct VariantB;

#[derive(Component, Default, Clone, Copy)]
pub struct VariantC;

#[derive(Component, Default, Clone, Copy)]
pub struct VariantD;

#[derive(Component, Default, Clone, Copy)]
pub struct VariantE;

#[derive(Component, Default, Clone, Copy)]
pub struct VariantF;

#[derive(Component, Default, Clone, Copy)]
pub struct VariantG;

#[derive(Component, Default, Clone, Copy)]
pub struct VariantH;

/// OPTIMIZATION: Single component with bitflags instead of multiple marker components.
///
/// Using separate marker components (VariantA-H) creates 2^8 = 256 possible archetypes,
/// causing severe fragmentation and cache-unfriendly iteration patterns.
///
/// This single component approach keeps all entities in ONE archetype while still
/// allowing variant-specific behavior through bitwise checks.
///
/// Trade-offs:
/// - PRO: All entities in single archetype = optimal cache utilization
/// - PRO: No archetype migration when changing variants
/// - CON: Cannot use query filters like `With<VariantA>` (must check at runtime)
/// - CON: Slightly more CPU work per entity for bitwise checks
///
/// Use marker components when:
/// - You need query-level filtering for different systems
/// - Variants are rarely changed after spawn
///
/// Use EntityVariant when:
/// - Entities frequently change variants
/// - You're iterating over all variants together
/// - Archetype count is becoming a performance issue
#[derive(Component, Clone, Copy, Default)]
pub struct EntityVariant(pub u8);

impl EntityVariant {
    pub const NONE: u8 = 0;
    pub const A: u8 = 1 << 0;
    pub const B: u8 = 1 << 1;
    pub const C: u8 = 1 << 2;
    pub const D: u8 = 1 << 3;
    pub const E: u8 = 1 << 4;
    pub const F: u8 = 1 << 5;
    pub const G: u8 = 1 << 6;
    pub const H: u8 = 1 << 7;

    /// Create a variant with the given flags
    #[inline]
    pub fn new(flags: u8) -> Self {
        Self(flags)
    }

    /// Check if this variant has a specific flag
    #[inline]
    pub fn has(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    /// Add a flag to this variant
    #[inline]
    pub fn add(&mut self, flag: u8) {
        self.0 |= flag;
    }

    /// Remove a flag from this variant
    #[inline]
    pub fn remove(&mut self, flag: u8) {
        self.0 &= !flag;
    }

    /// Generate a random variant combination
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self(rng.gen())
    }

    /// Generate using provided RNG
    #[inline]
    pub fn random_with<R: Rng>(rng: &mut R) -> Self {
        Self(rng.gen())
    }
}

// =============================================================================
// Toggle Component (for add/remove tests)
// =============================================================================

/// Component that gets toggled on/off for structural change testing
#[derive(Component, Default, Clone, Copy)]
pub struct ToggleComponent {
    pub value: u32,
}

/// Secondary toggle for more complex structural tests
#[derive(Component, Default, Clone, Copy)]
pub struct SecondaryToggle {
    pub active: bool,
}

// =============================================================================
// Counter Component (for iteration verification)
// =============================================================================

/// Simple counter to verify iteration actually happens
#[derive(Component, Default, Clone, Copy)]
pub struct Counter {
    pub value: u64,
}

impl Counter {
    #[inline(always)]
    pub fn increment(&mut self) {
        self.value = self.value.wrapping_add(1);
    }
}

// =============================================================================
// Health/Stats Components (game-like patterns)
// =============================================================================

/// Health component - common game pattern
#[derive(Component, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

/// Generic stats component
#[derive(Component, Clone, Copy, Default)]
pub struct Stats {
    pub strength: f32,
    pub speed: f32,
    pub defense: f32,
}

impl Stats {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self::random_with(&mut rng)
    }

    #[inline]
    pub fn random_with<R: Rng>(rng: &mut R) -> Self {
        Self {
            strength: rng.gen_range(1.0..100.0),
            speed: rng.gen_range(1.0..100.0),
            defense: rng.gen_range(1.0..100.0),
        }
    }
}
