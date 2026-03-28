# Master Schema - Constraint Theory Core

**Version:** 1.0.1
**Last Updated:** 2025-01-27

---

## Overview

This document provides the master schema linking all components of Constraint Theory across repositories and documentation. It serves as the authoritative reference for API consistency, data structures, and cross-ecosystem integration.

---

## 🔑 Key Formulas

### Hidden Dimensions Formula

The number of hidden dimensions required for precision ε is:

```
k = ⌈log₂(1/ε)⌉
```

This formula determines the computational depth needed to achieve a target precision level:

| Target Precision (ε) | Hidden Dimensions (k) |
|---------------------|---------------------|
| 0.1 | 4 |
| 0.01 | 7 |
| 0.001 | 10 |
| 0.0001 | 14 |

**Derivation**: From UNIFIED_QUANTIZATION_SYSTEM.md §4 - the hidden dimension count emerges from the information-theoretic bound on distinguishing Pythagorean ratios.

### Angular Resolution Formula

Maximum angular deviation from true input direction:

```
θ_max ≈ π / state_count = π / (5 × density)
```

### Noise Bound Theorem

For any snapped vector:

```
d_g(v, σ(v)) < π/(2n)  where n = manifold density
noise = 1 - cos(d_g) < 1 - cos(π/(2n))
```

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CONSTRAINT THEORY ECOSYSTEM                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                      RESEARCH & THEORY                               │    │
│  │  constraint-theory-research                                          │    │
│  │  ├── Mathematical Foundations (Ω-geometry, Φ-folding)                │    │
│  │  ├── Theoretical Guarantees (zero-hallucination proofs)             │    │
│  │  ├── arXiv Paper (arXiv:2503.15847)                                  │    │
│  │  └── Open Problems (3D, GPU, High-Dim)                               │    │
│  └────────────────────────────────┬────────────────────────────────────┘    │
│                                   │                                          │
│                                   ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         CORE IMPLEMENTATION                          │    │
│  │  constraint-theory-core (THIS REPO)                                  │    │
│  │  ├── PythagoreanManifold: Discrete lattice on S¹                     │    │
│  │  ├── KD-Tree: O(log n) nearest neighbor                              │    │
│  │  ├── SIMD: AVX2 batch processing                                     │    │
│  │  └── Hidden Dimensions: k = ⌈log₂(1/ε)⌉                             │    │
│  └────────────────────────────────┬────────────────────────────────────┘    │
│                                   │                                          │
│                    ┌──────────────┴──────────────┐                          │
│                    ▼                             ▼                          │
│  ┌─────────────────────────────┐  ┌─────────────────────────────┐          │
│  │     PYTHON BINDINGS         │  │      WEB VISUALIZATIONS     │          │
│  │  constraint-theory-python   │  │  constraint-theory-web      │          │
│  │  ├── PyO3 native bindings   │  │  ├── 49 interactive demos   │          │
│  │  ├── NumPy integration      │  │  ├── KD-tree visualizer     │          │
│  │  └── PyTorch compatible     │  │  └── No-install demos       │          │
│  └─────────────────────────────┘  └─────────────────────────────┘          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. PythagoreanManifold

**Purpose:** Discrete set of exact Pythagorean coordinates on the unit circle.

| Component | File | Description |
|-----------|------|-------------|
| Manifold struct | `src/manifold.rs` | Main data structure |
| Triple generation | Euclid's formula | a = m² - n², b = 2mn, c = m² + n² |
| KD-Tree index | `src/kdtree.rs` | O(log n) lookup |
| State count | ~5 × density | Valid Pythagorean points |

**Key Formula:**
```
k = ⌈log₂(1/ε)⌉  (Hidden dimensions for precision ε)
```

### 2. Snap Operation

**Purpose:** Project any 2D vector to nearest exact Pythagorean triple.

```
σ(v) = argmin_{p ∈ M} d_g(v, p)
```

| Method | Complexity | Use Case |
|--------|------------|----------|
| `snap()` | O(log n) | Single vector |
| `snap_batch_simd()` | O(m log n) | High throughput |
| `snap_batch()` | O(m log n) | Consensus-critical |

### 3. Performance Subsystem

| Component | File | Optimization |
|-----------|------|--------------|
| SIMD | `src/simd.rs` | AVX2 parallelism |
| KD-Tree | `src/kdtree.rs` | Cache-friendly layout |
| Edge cases | `src/edge_case_tests.rs` | NaN, zero, infinity |

---

## Module Dependencies

```
lib.rs
├── manifold.rs ──────────┬── kdtree.rs (lookup)
│                         └── simd.rs (batch)
├── curvature.rs ─────────── Ricci flow
├── cohomology.rs ────────── Sheaf cohomology
├── gauge.rs ─────────────── Gauge theory
├── percolation.rs ───────── Rigidity analysis
├── tile.rs ──────────────── Constraint blocks
└── edge_case_tests.rs ───── Edge case coverage
```

---

## Data Flow

### Single Snap Operation

```
Input: [f32; 2]
    │
    ▼
┌─────────────────┐
│  Normalize      │  v' = v / ||v||
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  KD-Tree Search │  O(log n)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Compute Noise  │  1 - (v' · p)
└────────┬────────┘
         │
         ▼
Output: ([f32; 2], f32)
```

### Batch Processing (SIMD)

```
Input: Vec<[f32; 2]>
    │
    ▼
┌─────────────────┐
│  Normalize      │  Vectorized
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  KD-Tree Search │  Parallel O(log n)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Compute Noise  │  Vectorized
└────────┬────────┘
         │
         ▼
Output: Vec<([f32; 2], f32)>
```

---

## Research Connections

### Grand Unified Constraint Theory (GUCT)

The core implementation realizes the mathematical framework described in the research repository:

| Concept | Implementation | Research Reference |
|---------|---------------|-------------------|
| Pythagorean Manifold | `PythagoreanManifold` | MATHEMATICAL_FOUNDATIONS_DEEP_DIVE.md §3 |
| Snap Operator | `snap()` | THEORETICAL_GUARANTEES.md §2 |
| Hidden Dimensions | k = ⌈log₂(1/ε)⌉ | UNIFIED_QUANTIZATION_SYSTEM.md §4 |
| Holonomy | `cohomology` module | HOLONOMIC_INFORMATION_THEORY.md |
| Noise Bounds | d_g < π/(2n) | THEORETICAL_GUARANTEES.md §4 |

### Key Theorems

1. **Exact Projection Theorem:** For any v ∈ S¹, σ(v) returns the nearest Pythagorean point.
2. **Bounded Noise Theorem:** d_g(v, σ(v)) < π/(2n) where n = manifold density.
3. **Zero Hallucination Guarantee:** All outputs satisfy constraints exactly.

---

## PythagoreanQuantizer Schema

The `PythagoreanQuantizer` is the core abstraction for quantizing continuous vectors to discrete Pythagorean coordinates.

### Schema Definition

```rust
/// PythagoreanQuantizer - Maps continuous 2D vectors to exact Pythagorean ratios
/// 
/// # Type Parameters
/// * `D` - Density parameter (number of Pythagorean triples to generate)
///
/// # Invariants
/// * All output vectors have exact rational representations
/// * Outputs are always normalized (lie on unit circle)
/// * Snap operation is deterministic across platforms
pub struct PythagoreanQuantizer<const D: usize> {
    /// Pre-computed valid states as normalized vectors
    valid_states: Vec<[f32; 2]>,
    /// KD-tree index for O(log n) lookup
    kdtree: KDTree,
    /// Hidden dimension count for precision bound
    hidden_dims: usize,
}
```

### JSON Schema (for WASM/Python bindings)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://superinstance.ai/schemas/pythagorean-quantizer.json",
  "title": "PythagoreanQuantizer",
  "type": "object",
  "properties": {
    "density": {
      "type": "integer",
      "minimum": 2,
      "maximum": 10000,
      "description": "Maximum m value in Euclid's formula"
    },
    "hidden_dimensions": {
      "type": "integer",
      "minimum": 1,
      "description": "Computed as ceil(log2(1/epsilon))"
    },
    "state_count": {
      "type": "integer",
      "minimum": 4,
      "description": "Number of valid Pythagorean vectors"
    },
    "max_angular_error": {
      "type": "number",
      "minimum": 0,
      "description": "Maximum angular deviation in radians"
    }
  },
  "required": ["density"]
}
```

### Snap Result Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://superinstance.ai/schemas/snap-result.json",
  "title": "SnapResult",
  "type": "object",
  "properties": {
    "snapped": {
      "type": "array",
      "items": { "type": "number" },
      "minItems": 2,
      "maxItems": 2,
      "description": "Snapped 2D vector on unit circle"
    },
    "noise": {
      "type": "number",
      "minimum": 0,
      "maximum": 1,
      "description": "Quantization noise (1 - resonance)"
    },
    "triple": {
      "type": "object",
      "properties": {
        "a": { "type": "integer" },
        "b": { "type": "integer" },
        "c": { "type": "integer" }
      },
      "description": "Underlying Pythagorean triple (a, b, c)"
    }
  },
  "required": ["snapped", "noise"]
}
```

---

## Cross-Repository Links

### From Core to Python

| Rust API | Python Equivalent | Notes |
|----------|-------------------|-------|
| `PythagoreanManifold::new(d)` | `PythagoreanManifold(density=d)` | Identical behavior |
| `manifold.snap([x, y])` | `manifold.snap(x, y)` | Python accepts two args |
| `manifold.snap_batch_simd(&v)` | `manifold.snap_batch(v)` | Python always uses SIMD when available |
| `manifold.state_count()` | `manifold.state_count` | Property in Python |
| `manifold.max_angular_error()` | `manifold.max_angular_error` | Property in Python |
| `PythagoreanTriple::new(a,b,c)` | `PythagoreanTriple(a, b, c)` | Identical behavior |
| `triple.to_vector()` | `triple.to_vector()` | Identical behavior |

#### Python Module Structure

```python
# constraint-theory-python
from constraint_theory import (
    PythagoreanManifold,
    PythagoreanTriple,
    snap,
    snap_batch,
    FastPercolation,
    RigidityResult,
)

# Create manifold
manifold = PythagoreanManifold(density=200)

# Single snap
snapped, noise = manifold.snap(0.6, 0.8)

# Batch snap
vectors = [[0.6, 0.8], [0.8, 0.6]]
results = manifold.snap_batch(vectors)
```

### From Core to Web

| Rust Concept | Web Demo | URL |
|--------------|----------|-----|
| KD-Tree lookup | `/simulators/kdtree/` | Interactive KD-tree visualization |
| Pythagorean snapping | `/simulators/pythagorean/` | Snap operation demo |
| Swarm behavior | `/simulators/swarm/` | Multi-agent constraint system |
| Ricci flow | `/simulators/ricci/` | Curvature evolution |
| Percolation | `/simulators/percolation/` | Rigidity analysis |

#### WebAssembly Exports

```typescript
// constraint-theory-web WASM bindings
export interface PythagoreanManifold {
  constructor(density: number);
  snap(x: number, y: number): SnapResult;
  snap_batch(vectors: Float32Array): SnapResult[];
  state_count: number;
  max_angular_error: number;
}

export interface SnapResult {
  snapped: [number, number];
  noise: number;
}
```

### From Core to Research

| Code | Paper Reference | Section |
|------|-----------------|--------|
| `manifold.rs` | paper1_constraint_theory_geometric_foundation.tex | §3 |
| `kdtree.rs` | paper2_pythagorean_snapping.tex | §2 |
| `curvature.rs` | paper3_deterministic_ai_practice.tex | §4.2 |
| `cohomology.rs` | HOLONOMIC_INFORMATION_THEORY.md | Full chapter |
| `percolation.rs` | THEORETICAL_GUARANTEES.md | §5 |
| Applications | paper3_deterministic_ai_practice.tex | Full paper |

---

## Configuration Reference

### Manifold Density

| Density | States | Angular Resolution | Memory |
|---------|--------|-------------------|--------|
| 50 | ~250 | ~1.4° | ~20 KB |
| 100 | ~500 | ~0.7° | ~40 KB |
| 200 | ~1000 | ~0.36° | ~80 KB |
| 500 | ~2500 | ~0.14° | ~200 KB |
| 1000 | ~5000 | ~0.07° | ~400 KB |

### SIMD Configuration

```rust
// Enable SIMD feature
cargo build --release --features simd

// SIMD is used when:
// 1. Architecture is x86_64
// 2. AVX2 is detected at runtime
// 3. Batch size >= 8
```

---

---

## Version Compatibility Matrix

### Core Version History

| Core Version | Rust Version | Release Date | Key Changes |
|--------------|--------------|--------------|-------------|
| 1.0.1 | 1.75+ | 2025-01-27 | KD-tree optimization, SIMD improvements |
| 1.0.0 | 1.75+ | 2025-01-15 | Initial stable release |

### Cross-Ecosystem Version Compatibility

| Core Version | Python Version | Web Version | Research Version | WASM |
|--------------|----------------|-------------|------------------|------|
| 1.0.1 | 1.0.1+ | Current | arXiv:2503.15847 | v1.0.1 |
| 1.0.0 | 1.0.0+ | Current | arXiv:2503.15847 | v1.0.0 |

### API Stability Guarantee

- **1.x.x**: Stable API, backward compatible additions only
- **Minor versions (1.1.x)**: New features, deprecations allowed
- **Patch versions (1.0.x)**: Bug fixes only
- **Breaking changes**: Reserved for 2.0.0

### Cross-Platform Compatibility

| Platform | Status | Rust Target | Notes |
|----------|--------|-------------|-------|
| Linux x86_64 | ✅ Stable | `x86_64-unknown-linux-gnu` | Primary development platform |
| macOS x86_64 | ✅ Stable | `x86_64-apple-darwin` | Intel Macs |
| macOS ARM64 | ✅ Stable | `aarch64-apple-darwin` | Apple Silicon (M1/M2/M3) |
| Windows x86_64 | ✅ Stable | `x86_64-pc-windows-msvc` | MSVC toolchain |
| WebAssembly | ✅ Stable | `wasm32-unknown-unknown` | Via constraint-theory-web |
| Linux ARM64 | ⚠️ Best Effort | `aarch64-unknown-linux-gnu` | AWS Graviton |

### SIMD Feature Support

| Feature | x86_64 | ARM64 | WASM | Fallback |
|---------|--------|-------|------|----------|
| AVX2 | ✅ | N/A | N/A | Scalar |
| AVX-512 | 🔜 Planned | N/A | N/A | Scalar |
| NEON | N/A | 🔜 Planned | N/A | Scalar |
| WASM SIMD | N/A | N/A | ✅ | Scalar |

### Compatibility Testing

Cross-platform determinism is verified by CI:

```bash
# Run cross-platform compatibility tests
cargo test test_cross_platform_determinism

# Verify SIMD/scalar equivalence
cargo test test_simd_scalar_equivalence
```

---

## Quick Reference

### Import Pattern

```rust
use constraint_theory_core::{PythagoreanManifold, snap};
```

### Basic Usage

```rust
let manifold = PythagoreanManifold::new(200);
let (snapped, noise) = snap(&manifold, [x, y]);
```

### Batch Processing

```rust
let results = manifold.snap_batch_simd(&vectors);
```

### Consensus-Critical

```rust
let mut results = vec![([0.0, 0.0], 0.0f32); vectors.len()];
manifold.snap_batch(&vectors, &mut results);  // Scalar, deterministic
```

### Error Handling (Recommended)

```rust
use constraint_theory_core::{PythagoreanManifold, CTResult, CTErr};

let manifold = PythagoreanManifold::new(200);

// Validate before snapping for consensus-critical code
match manifold.validate_input([x, y]) {
    Ok(()) => {
        let (snapped, noise) = manifold.snap([x, y]);
        // Process result
    }
    Err(reason) => {
        // Handle invalid input
        eprintln!("Invalid input: {}", reason);
    }
}
```

---

## See Also

- [API Documentation](https://docs.rs/constraint-theory-core)
- [Performance Guide](./PERFORMANCE.md)
- [Benchmarks](./BENCHMARKS.md)
- [Tutorial](./TUTORIAL.md)
- [Testing Methodology](./TESTING.md)
- [Security Policy](../SECURITY.md)

---

**Document Version:** 1.1
**Next Review:** 2025-04-01
