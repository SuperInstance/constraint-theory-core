# Ecosystem Overview

**Version:** 1.0.1
**Last Updated:** 2025-01-27

---

## Overview

This document provides a comprehensive overview of the Constraint Theory ecosystem, including all related repositories, use cases, and cross-repository integration patterns.

---

## Ecosystem Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                         CONSTRAINT THEORY ECOSYSTEM                                  │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                        RESEARCH & THEORY                                     │    │
│  │  ┌───────────────────────────────────────────────────────────────────────┐  │    │
│  │  │  constraint-theory-research                                            │  │    │
│  │  │  ├── Mathematical Foundations (Ω-geometry, Φ-folding operator)         │  │    │
│  │  │  ├── Theoretical Guarantees (zero-hallucination proofs)               │  │    │
│  │  │  ├── arXiv:2503.15847                                                  │  │    │
│  │  │  └── Open Problems (3D, GPU, High-Dim)                                 │  │    │
│  │  └───────────────────────────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────┬────────────────────────────────────────┘    │
│                                       │                                              │
│                                       ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                         CORE IMPLEMENTATION                                  │    │
│  │  ┌───────────────────────────────────────────────────────────────────────┐  │    │
│  │  │  constraint-theory-core (THIS REPO)                                    │  │    │
│  │  │  ├── Rust crate, zero dependencies                                     │  │    │
│  │  │  ├── O(log n) KD-tree lookup                                          │  │    │
│  │  │  ├── SIMD batch processing (AVX2)                                      │  │    │
│  │  │  └── ~100ns per snap operation                                         │  │    │
│  │  └───────────────────────────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────┬────────────────────────────────────────┘    │
│                                       │                                              │
│                    ┌──────────────────┴──────────────────┐                          │
│                    ▼                                     ▼                          │
│  ┌─────────────────────────────┐       ┌─────────────────────────────┐              │
│  │     PYTHON BINDINGS         │       │      WEB VISUALIZATIONS     │              │
│  │  constraint-theory-python   │       │  constraint-theory-web      │              │
│  │  ├── PyO3 native bindings   │       │  ├── 50 interactive demos   │              │
│  │  ├── NumPy integration      │       │  ├── KD-tree visualizer     │              │
│  │  ├── PyTorch compatible     │       │  ├── Pythagorean demo       │              │
│  │  └── pip install ready      │       │  └── No-install demos       │              │
│  └─────────────────────────────┘       └─────────────────────────────┘              │
│                                                                                      │
│                    ┌──────────────────┬──────────────────┐                          │
│                    ▼                  ▼                  ▼                          │
│  ┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐           │
│  │     GAME DEV        │ │      ROBOTICS       │ │    ML / SCIENCE     │           │
│  │  constraint-ranch   │ │                     │ │                     │           │
│  │  ├── Puzzle games   │ │                     │ │                     │           │
│  │  ├── Educational    │ │                     │ │                     │           │
│  │  └── Species demo   │ │                     │ │                     │           │
│  └─────────────────────┘ └─────────────────────┘ └─────────────────────┘           │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

---

## Repository Overview

### Core Repository

| Repository | Description | Language | Status |
|------------|-------------|----------|--------|
| [constraint-theory-core](https://github.com/SuperInstance/constraint-theory-core) | **This repo** - Core Rust implementation | Rust | Stable v1.0.1 |

**Key Features:**
- Zero dependencies
- O(log n) KD-tree lookup
- SIMD batch processing
- Cross-platform determinism
- ~100ns per snap operation

### Ecosystem Repositories

| Repository | Description | Language | Install |
|------------|-------------|----------|---------|
| [constraint-theory-python](https://github.com/SuperInstance/constraint-theory-python) | Python bindings | Python/Rust | `pip install constraint-theory` |
| [constraint-theory-web](https://github.com/SuperInstance/constraint-theory-web) | Interactive demos | HTML/JS | Live at [constraint-theory.superinstance.ai](https://constraint-theory.superinstance.ai) |
| [constraint-theory-research](https://github.com/SuperInstance/constraint-theory-research) | Mathematical foundations | Markdown/LaTeX | - |
| [constraint-ranch](https://github.com/SuperInstance/constraint-ranch) | Educational game demos | TypeScript | - |

---

## Use Case Examples

### Game Development

**Repository:** [constraint-ranch](https://github.com/SuperInstance/constraint-ranch)

Constraint Theory enables deterministic multiplayer physics:

```typescript
// From constraint-ranch puzzle logic
import { PythagoreanManifold } from 'constraint-theory-wasm';

class DeterministicPhysics {
    private manifold: PythagoreanManifold;

    constructor() {
        this.manifold = new PythagoreanManifold(200);
    }

    // Same input → same output on every client
    processInput(joystick: [number, number]): [number, number] {
        const [x, y, noise] = this.manifold.snap(joystick[0], joystick[1]);
        return [x, y];
    }

    // All clients see identical physics
    updateEntity(entity: Entity, direction: [number, number]) {
        const [exactX, exactY] = this.processInput(direction);
        entity.velocity = [exactX * SPEED, exactY * SPEED];
    }
}
```

**Key Benefits:**
- No "rubber banding" from float reconciliation
- Identical physics on all platforms
- Simplified network code

### Machine Learning

**Repository:** [constraint-theory-python](https://github.com/SuperInstance/constraint-theory-python)

Reproducible data augmentation and training:

```python
from constraint_theory import PythagoreanManifold
import numpy as np

manifold = PythagoreanManifold(500)

def deterministic_augment(direction: np.ndarray) -> np.ndarray:
    """Reproducible data augmentation for ML training."""
    sx, sy, noise = manifold.snap(direction[0], direction[1])
    return np.array([sx, sy])

# Paper reviewers can reproduce exact training runs
def augment_batch(directions: np.ndarray) -> np.ndarray:
    results = manifold.snap_batch(directions)
    return np.array([[sx, sy] for sx, sy, _ in results])

# PyTorch integration
import torch

def snap_tensor(tensor: torch.Tensor) -> torch.Tensor:
    """Snap PyTorch tensor to exact coordinates."""
    numpy_arr = tensor.detach().cpu().numpy()
    vectors = numpy_arr.reshape(-1, 2)
    results = manifold.snap_batch(vectors.tolist())
    snapped = np.array([[sx, sy] for sx, sy, _ in results])
    return torch.from_numpy(snapped.reshape(tensor.shape))
```

**Key Benefits:**
- Reproducible Monte Carlo simulations
- Paper reviewers can reproduce exact results
- Deterministic training data augmentation

### Robotics

**From constraint-theory-core examples:**

```rust
use constraint_theory_core::PythagoreanManifold;

struct RobotArm {
    manifold: PythagoreanManifold,
}

impl RobotArm {
    fn move_to_direction(&mut self, target: [f32; 2]) -> Motion {
        let (exact, noise) = self.manifold.snap(target);

        if noise > 0.01 {
            log::warn!("High quantization noise: {}", noise);
        }

        // Same motion, same result, every time
        Motion {
            direction: exact,
            confidence: 1.0 - noise,
        }
    }
}
```

**Key Benefits:**
- Repeatable arm movements
- No accumulated floating-point drift
- Simplified calibration

### Scientific Computing

**From constraint-theory-python examples:**

```python
import numpy as np
from constraint_theory import PythagoreanManifold

manifold = PythagoreanManifold(300)

def monte_carlo_simulation(n_samples: int, seed: int = 42) -> dict:
    """Reproducible Monte Carlo simulation."""
    np.random.seed(seed)

    # Generate random directions
    angles = np.random.uniform(0, 2 * np.pi, n_samples)
    directions = np.column_stack([np.cos(angles), np.sin(angles)])

    # Snap to exact states
    results = manifold.snap_batch(directions)
    snapped = np.array([[sx, sy] for sx, sy, _ in results])

    # Compute statistics
    return {
        'mean_direction': snapped.mean(axis=0),
        'n_samples': n_samples,
        'reproducible': True,
        'seed': seed,
    }
```

**Key Benefits:**
- Reproducible across laptop, server, and HPC
- Identical results on any platform
- Simplified peer review

---

## Cross-Repository Links

### API Compatibility Matrix

| Feature | Rust Core | Python | WASM | TypeScript |
|---------|-----------|--------|------|------------|
| Snap single | `manifold.snap([x, y])` | `manifold.snap(x, y)` | `manifold.snap(x, y)` | `manifold.snap([x, y])` |
| Snap batch | `manifold.snap_batch_simd(&v)` | `manifold.snap_batch(v)` | `manifold.snap_batch(v)` | `manifold.snapBatch(v)` |
| State count | `manifold.state_count()` | `manifold.state_count` | `manifold.state_count()` | `manifold.stateCount()` |
| Hidden dims | `hidden_dimensions(ε)` | `hidden_dimensions(ε)` | - | - |

### Data Flow Between Repositories

```
constraint-theory-research
        │
        │  Theory (papers, proofs)
        ▼
constraint-theory-core
        │
        ├──────────────────┬──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
constraint-      constraint-         constraint-
theory-python    theory-web          ranch
        │                  │                  │
        │                  │                  │
        ▼                  ▼                  ▼
   ML/Science         Education         Game Dev
```

### Shared Constants

All repositories use identical constants:

```rust
// Rust
pub const VERSION: &str = "1.0.1";
pub const MAX_DENSITY: usize = 10_000;
pub const DEFAULT_DENSITY: usize = 200;
```

```python
# Python
__version__ = "1.0.1"
MAX_DENSITY = 10_000
DEFAULT_DENSITY = 200
```

```typescript
// TypeScript
export const VERSION = "1.0.1";
export const MAX_DENSITY = 10_000;
export const DEFAULT_DENSITY = 200;
```

---

## Quick Reference

### Installation

| Platform | Command |
|----------|---------|
| **Rust** | `cargo add constraint-theory-core` |
| **Python** | `pip install constraint-theory` |
| **JavaScript** | `npm install constraint-theory-wasm` (planned) |
| **Browser** | [Live Demo](https://constraint-theory.superinstance.ai) |

### Basic Usage (All Platforms)

**Rust:**
```rust
use constraint_theory_core::{PythagoreanManifold, snap};

let manifold = PythagoreanManifold::new(200);
let (snapped, noise) = snap(&manifold, [0.577, 0.816]);
println!("Snapped: {:?}", snapped);  // [0.6, 0.8]
```

**Python:**
```python
from constraint_theory import PythagoreanManifold

manifold = PythagoreanManifold(200)
x, y, noise = manifold.snap(0.577, 0.816)
print(f"Snapped: ({x}, {y})")  # (0.6, 0.8)
```

**JavaScript/WASM:**
```javascript
import { PythagoreanManifold } from 'constraint-theory-core';

const manifold = new PythagoreanManifold(200);
const [x, y, noise] = manifold.snap(0.577, 0.816);
console.log(`Snapped: (${x}, ${y})`);  // (0.6, 0.8)
```

### Performance Targets

| Operation | Target | Typical |
|-----------|--------|---------|
| Single snap | < 1 μs | ~100 ns |
| Batch (1K) | < 100 μs | ~15 μs |
| Batch (100K) | < 10 ms | ~1.1 ms |
| Memory/state | < 100 bytes | ~80 bytes |

### Key Formulas

| Formula | Description |
|---------|-------------|
| `k = ⌈log₂(1/ε)⌉` | Hidden dimensions for precision ε |
| `θ_max ≈ π / (5 × density)` | Maximum angular error |
| `noise = 1 - cos(d_g)` | Noise from geodesic distance |

---

## Integration Patterns

### Pattern 1: Shared Manifold

For applications with consistent precision requirements:

```rust
// Create once, use everywhere
lazy_static! {
    static ref MANIFOLD: PythagoreanManifold = PythagoreanManifold::new(200);
}

fn any_function() -> ([f32; 2], f32) {
    MANIFOLD.snap([0.577, 0.816])
}
```

### Pattern 2: Precision Tiering

For applications with varying precision needs:

```rust
struct TieredManifolds {
    low: PythagoreanManifold,    // density=50
    medium: PythagoreanManifold, // density=200
    high: PythagoreanManifold,   // density=500
}

impl TieredManifolds {
    fn snap(&self, v: [f32; 2], precision: Precision) -> ([f32; 2], f32) {
        match precision {
            Precision::Low => self.low.snap(v),
            Precision::Medium => self.medium.snap(v),
            Precision::High => self.high.snap(v),
        }
    }
}
```

### Pattern 3: Cross-Platform Validation

For consensus-critical systems:

```rust
fn validated_snap(manifold: &PythagoreanManifold, v: [f32; 2]) -> Result<([f32; 2], f32), Error> {
    // Validate input
    manifold.validate_input(v)?;

    // Snap (scalar path for determinism)
    let (snapped, noise) = manifold.snap(v);

    // Verify output
    let norm_sq = snapped[0] * snapped[0] + snapped[1] * snapped[1];
    if (norm_sq - 1.0).abs() > 0.001 {
        return Err(Error::InvariantViolation);
    }

    Ok((snapped, noise))
}
```

---

## Contributing to the Ecosystem

### Adding New Language Bindings

1. **Create FFI layer** in `constraint-theory-core/src/ffi.rs`
2. **Create binding repository** following naming convention
3. **Add CI/CD** for cross-platform testing
4. **Document API** with examples
5. **Submit PR** to update this ecosystem document

### Adding New Demos

1. **Create demo** in `constraint-theory-web/experiments/`
2. **Add cross-link** from this document
3. **Document mathematical concept** in `constraint-theory-research`

### Reporting Cross-Repo Issues

- **API inconsistency:** Report in both affected repos
- **Performance regression:** Benchmark in core, verify in bindings
- **Documentation gaps:** Cross-reference all repos

---

## See Also

- [MASTER_SCHEMA.md](MASTER_SCHEMA.md) - Technical specification
- [INTEGRATION.md](INTEGRATION.md) - Integration guide
- [RESEARCH_FOUNDATIONS.md](RESEARCH_FOUNDATIONS.md) - Theoretical foundations
- [DEPLOYMENT.md](DEPLOYMENT.md) - Deployment guide

---

## External Resources

- **arXiv Paper:** [arXiv:2503.15847](https://arxiv.org/abs/2503.15847)
- **Live Demos:** [constraint-theory.superinstance.ai](https://constraint-theory.superinstance.ai)
- **API Docs:** [docs.rs/constraint-theory-core](https://docs.rs/constraint-theory-core)

---

**Document Version:** 1.0
**Next Review:** 2025-04-01
