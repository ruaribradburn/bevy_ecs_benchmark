# Bevy ECS Benchmark Suite

A comprehensive stress testing and benchmarking suite for Bevy's Entity Component System. Designed to measure ECS throughput independently of rendering overhead.

## Features

- **Multiple Workload Types**: Test iteration, mutation, spawning/despawning, and structural changes
- **Archetype Fragmentation Testing**: Measure performance impact of component variety
- **Automated Breakdown Detection**: Binary search to find entity limits at target frame rates
- **Real-time Dashboard**: Visual feedback with frame time graphs and throughput metrics
- **Extensible Architecture**: Easy to add custom workloads and components
- **Results Export**: Save benchmark results to JSON for comparison

## Requirements

- Rust 1.75+ (2021 edition)
- Bevy 0.17.3

## Quick Start

```bash
# Clone the repository
git clone https://github.com/your-username/bevy_ecs_benchmark
cd bevy_ecs_benchmark

# Run in release mode (IMPORTANT for accurate results)
cargo run --release
```

> ⚠️ **Always benchmark in release mode!** Debug builds are 10-100x slower.

## Controls

| Key | Action |
|-----|--------|
| `1-6` | Select workload type |
| `Space` | Start/pause current benchmark |
| `R` | Reset current test |
| `Enter` | Run full automated suite |
| `Up/Down` | Manually adjust entity count |
| `S` | Save results to file |
| `Escape` | Exit |

## Workload Types

### 1. Simple Iteration (`1`)
Read-only iteration over entities with a single component. Tests raw query iteration speed.

### 2. Multi-Component Read (`2`)
Iteration over entities with 3 components. Tests cache efficiency with larger archetypes.

### 3. Position/Velocity Update (`3`)
Classic game loop pattern: read velocity, write position. Tests mutation throughput.

### 4. Spawn/Despawn Churn (`4`)
Continuously spawn and despawn entities. Tests command queue and archetype management.

### 5. Component Add/Remove (`5`)
Add and remove components from existing entities. Tests archetype migration cost.

### 6. Fragmented Archetypes (`6`)
Entities distributed across many archetypes. Tests query matching with fragmentation.

## Architecture

```
src/
├── main.rs                 # Entry point
├── lib.rs                  # Library exports
├── plugin.rs               # Main benchmark plugin
├── state.rs                # Application state machine
├── config.rs               # Configuration constants
│
├── benchmark/
│   ├── mod.rs
│   ├── runner.rs           # Benchmark execution logic
│   ├── results.rs          # Results collection and export
│   └── workloads/
│       ├── mod.rs          # Workload trait and registry
│       ├── iteration.rs    # Read-only iteration tests
│       ├── mutation.rs     # Write operation tests
│       ├── structural.rs   # Spawn/despawn/component tests
│       └── fragmentation.rs # Archetype fragmentation tests
│
├── components/
│   ├── mod.rs
│   └── test_components.rs  # Benchmark-specific components
│
├── ui/
│   ├── mod.rs
│   ├── dashboard.rs        # Main UI layout
│   ├── graph.rs            # Frame time visualization
│   └── styles.rs           # UI styling constants
│
└── metrics/
    ├── mod.rs
    └── frame_metrics.rs    # Performance measurement
```

## Extending with Custom Workloads

1. Create a new file in `src/benchmark/workloads/`
2. Implement the `Workload` trait:

```rust
use crate::benchmark::workloads::{Workload, WorkloadSystems};

pub struct MyCustomWorkload;

impl Workload for MyCustomWorkload {
    fn name(&self) -> &'static str {
        "My Custom Workload"
    }

    fn description(&self) -> &'static str {
        "Description of what this tests"
    }

    fn setup_systems(&self) -> WorkloadSystems {
        WorkloadSystems {
            spawn: spawn_my_entities.into(),
            update: update_my_entities.into(),
            cleanup: cleanup_my_entities.into(),
        }
    }
}
```

3. Register in `src/benchmark/workloads/mod.rs`

## Interpreting Results

The benchmark finds the **breakdown point**: the entity count at which frame time exceeds the target threshold (default: 16.6ms for 60 FPS).

Results include:
- **Breakdown Point**: Maximum sustainable entity count
- **Peak Throughput**: Entities processed per second at breakdown
- **Frame Time Distribution**: Min/max/median frame times

## Output Example

```json
{
  "timestamp": "2025-01-15T10:30:00Z",
  "system_info": {
    "os": "Linux",
    "cpu_cores": 8
  },
  "results": [
    {
      "workload": "Position/Velocity Update",
      "breakdown_point": 2500000,
      "throughput_eps": 150000000,
      "frame_time_ms": {
        "min": 14.2,
        "max": 17.1,
        "median": 16.4
      }
    }
  ]
}
```

## Tips for Accurate Benchmarking

1. **Close other applications** to reduce system noise
2. **Disable power saving** modes for consistent CPU performance
3. **Run multiple times** and compare results
4. **Watch for thermal throttling** on extended runs
5. **Use `--release`** - this cannot be overstated!

## License

MIT OR Apache-2.0 (same as Bevy)
