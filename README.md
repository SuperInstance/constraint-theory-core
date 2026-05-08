<div align="center">

# ⚡ Constraint Theory Core

### `0.6² + 0.8² = 1.0000000000000002` — and you've been debugging this for years.

**Trade float drift for quantized exactness. Same bits, every machine, guaranteed.**

[![GitHub stars](https://img.shields.io/github/stars/SuperInstance/constraint-theory-core?style=social)](https://github.com/SuperInstance/constraint-theory-core)
[![CI](https://github.com/SuperInstance/constraint-theory-core/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/constraint-theory-core/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/constraint-theory-core.svg)](https://crates.io/crates/constraint-theory-core)
[![docs.rs](https://docs.rs/constraint-theory-core/badge.svg)](https://docs.rs/constraint-theory-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**`cargo add constraint-theory-core`** · [Live Demos](https://constraint-theory-web.pages.dev) · [Docs](https://docs.rs/constraint-theory-core)

</div>

---

## Overview

**Constraint Theory** is a mathematical framework for exact constraint satisfaction that replaces floating-point approximation with discrete, deterministic rational representations. At its core, it exploits the structure of **Pythagorean triples** — integer solutions to a² + b² = c² — to construct a finite set of exact points on the unit circle S¹.

The insight is simple but powerful: there are infinitely many Pythagorean triples, but only finitely many within any precision bound. By precomputing these exact rational points, indexing them with a KD-tree, and projecting (snapping) continuous input vectors to the nearest exact neighbor, the system eliminates an entire class of floating-point drift bugs — forever.

This crate implements the **Grand Unified Constraint Theory (GUCT)**, which extends the core snapping operation into a full algebraic-geometric engine:

| Domain | Mechanism |
|--------|-----------|
| **Exact representation** | Pythagorean triples via Euclid's formula: a = m² − n², b = 2mn, c = m² + n² |
| **Fast lookup** | O(log N) KD-tree spatial index |
| **Precision encoding** | Hidden dimensions formula: k = ⌈log₂(1/ε)⌉ |
| **Global consistency** | Holonomy verification around cycles (zero holonomy = consistent) |
| **Curvature evolution** | Ricci flow toward target curvature (flattening manifolds) |
| **Structural rigidity** | Laman's theorem for constraint graph rigidity percolation |
| **Topological detection** | Sheaf cohomology (H₀ = components, H₁ = cycles) |
| **Quantization** | Unified constraint-preserving quantization (TurboQuant, BitNet, PolarQuant) |
| **Batch processing** | AVX2 SIMD with 8× f32 parallelism |

**Zero dependencies.** Pure Rust, `#![forbid(unsafe_code)]` safe API surface (unsafe only in SIMD intrinsics behind safe wrappers), MIT licensed.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    CONSTRAINT THEORY CORE ENGINE                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────────────────┐  │
│  │   manifold   │    │   kdtree     │    │          simd                │  │
│  │              │◄───│              │    │  ┌─────────┐ ┌──────────┐  │  │
│  │ .new(density)│───►│ .build(pts)  │    │  │  AVX2   │ │  Scalar  │  │  │
│  │ .snap(vec)   │    │ .nearest(q)  │    │  │ 8× f32  │ │ fallback │  │  │
│  │ .snap_batch  │    │ .nearest_k() │    │  └────┬────┘ └─────┬────┘  │  │
│  └──────┬───────┘    └──────┬───────┘    │       └────────────┘       │  │
│         │                   │             └──────────┬────────────────┘  │
│         │                   │                        │                   │
│  ┌──────▼───────┐    ┌──────▼───────┐    ┌──────────▼────────────────┐  │
│  │   hidden_    │    │   cache      │    │       quantizer            │  │
│  │  dimensions  │    │              │    │                            │  │
│  │              │    │ LatticeCache │    │  ┌─────────┐ ┌─────────┐ │  │
│  │ k=⌈log2(1/ε)⌉│    │ CachedLat.  │    │  │ Ternary │ │  Polar  │ │  │
│  │ lift_to_     │    │ global_cache │    │  │ BitNet  │ │PolarQt. │ │  │
│  │  hidden()    │    │ (RwLock,     │    │  ├─────────┤ ├─────────┤ │  │
│  │ project_to_  │    │  thread-safe)│    │  │  Turbo  │ │ Hybrid  │ │  │
│  │  visible()   │    └──────────────┘    │  │TurboQt. │ │  auto   │ │  │
│  └──────────────┘                         │  └─────────┘ └─────────┘ │  │
│                                           └────────────────────────────┘  │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────────────┐ │
│  │     holonomy     │  │   cohomology     │  │     curvature            │ │
│  │                  │  │                  │  │                          │ │
│  │ compute_holonomy │  │ FastCohomology   │  │ RicciFlow               │ │
│  │ verify_holonomy  │  │   .compute()     │  │   .evolve(curvatures)   │ │
│  │ HolonomyChecker  │  │   H0 = β₀        │  │   .new(alpha, target)   │ │
│  │ rotation_x/y/z  │  │   H1 = E-V+β₀   │  │ ricci_flow_step()       │ │
│  └────────┬─────────┘  └──────────────────┘  └──────────────────────────┘ │
│           │                                                               │
│  ┌────────▼─────────┐  ┌──────────────────┐  ┌──────────────────────────┐ │
│  │      gauge       │  │   percolation    │  │       tile               │ │
│  │                  │  │                  │  │                          │ │
│  │ GaugeConnection  │  │ FastPercolation  │  │ Tile (384 bytes)        │ │
│  │ .parallel_trans. │  │   .compute_      │  │   .origin (64B)         │ │
│  │   (vec, path)    │  │    rigidity()    │  │   .tensor_payload (64B) │ │
│  └──────────────────┘  │   Laman's 2V-3   │  │   .constraints (192B)   │ │
│                        └──────────────────┘  │   ConstraintBlock       │ │
│  ┌──────────────────────────────────────┐   │     .holonomy_matrix    │ │
│  │             dcs (constants)          │   │     .ricci_curvature    │ │
│  │                                      │   │     .percolation_p      │ │
│  │  LAMAN_NEIGHBOR_THRESHOLD = 12       │   └──────────────────────────┘ │
│  │  PYTHAGOREAN_INFO_BITS = 5.585       │                               │
│  │  RICCI_CONVERGENCE_MULTIPLIER = 1.692│                               │
│  └──────────────────────────────────────┘                               │
├─────────────────────────────────────────────────────────────────────────────┤
│  Error Handling: CTErr (11 variants) → CTResult<T> = Result<T, CTErr>      │
│  Feature Flags: simd (default off, auto-detected at runtime on x86_64)     │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Types

### Primary Data Structures

| Type | Module | Size | Description |
|------|--------|------|-------------|
| `PythagoreanManifold` | `manifold` | ~80 KB (density 200) | Precomputed set of exact Pythagorean vectors on S¹ with KD-tree index. The central entry point for all snapping operations. |
| `PythagoreanTriple` | `manifold` | 12 bytes | A triple `(a, b, c)` where a² + b² = c² exactly. Represents the fundamental geometric constraint. |
| `KDTree` | `kdtree` | O(N) heap | 2D spatial index with O(log N) nearest-neighbor queries. Leaf size capped at 16 points. Deterministic tie-breaking via index ordering. |
| `Tile` | `tile` | **384 bytes** (compile-time verified) | Fundamental unit of computation. Contains `Origin` (64B), I/O (16B), confidence/safety (8B), tensor payload (64B), and `ConstraintBlock` (192B). Cache-line aligned. |
| `Origin` | `tile` | 64 bytes | SO(3) reference frame (3×3 rotation matrix) + rate-of-change vector + unique ID. |
| `ConstraintBlock` | `tile` | **192 bytes** (compile-time verified) | Snap target, holonomy matrix, Ricci curvature (4×4 tensor), rigid cluster ID, percolation probability, gluing map, LVQ index, persistence hash. |

### Encoding & Quantization

| Type | Module | Description |
|------|--------|-------------|
| `HiddenDimensionConfig` | `hidden_dimensions` | Configuration for precision ε, computing k = ⌈log₂(1/ε)⌉ hidden dimensions. |
| `PythagoreanQuantizer` | `quantizer` | Unified quantizer synthesizing TurboQuant, BitNet, and PolarQuant. Mode: `Ternary` {-1,0,1}, `Polar` (unit norm), `Turbo` (near-optimal distortion), `Hybrid` (auto-select). |
| `QuantizationResult` | `quantizer` | Result with data, MSE, constraint satisfaction flags, unit norm preservation. |
| `QuantizationMode` | `quantizer` | Enum of 4 quantization strategies. |
| `Rational` | `quantizer` | Exact rational number (num/den) with `is_pythagorean()` verification. |
| `CachedLattice` | `cache` | Cached Pythagorean lattice: triples, normalized vectors, max hypotenuse. |

### Geometric Analysis

| Type | Module | Description |
|------|--------|-------------|
| `HolonomyResult` | `holonomy` | Holonomy matrix, Frobenius norm deviation from identity, information content I = −log\|Hol(γ)\|, identity check. |
| `HolonomyChecker` | `holonomy` | Incremental cycle verification: `apply()`, `check_partial()`, `check_closed()`. |
| `RicciFlow` | `curvature` | Curvature evolution state with learning rate α and target curvature. |
| `FastPercolation` | `percolation` | Union-find with path compression for rigidity percolation via Laman's theorem. |
| `RigidityResult` | `percolation` | Rigidity metrics: is_rigid, rank, deficiency, cluster count, rigid fraction. |
| `CohomologyResult` | `cohomology` | H₀ (connected components) and H₁ (independent cycles) dimensions via Euler characteristic. |
| `GaugeConnection` | `gauge` | Parallel transport of vectors across tile networks using holonomy matrices. |

### Error Handling

| Type | Description |
|------|-------------|
| `CTErr` | 11-variant enum covering input validation (`NaNInput`, `InfinityInput`, `ZeroVector`), state errors (`ManifoldEmpty`, `BufferSizeMismatch`), and numerical errors (`Overflow`, `DivisionByZero`). Each variant has an actionable `Display` message. |
| `CTResult<T>` | Type alias for `Result<T, CTErr>`. |

---

## Quick Start

```bash
# Clone and build
git clone https://github.com/SuperInstance/constraint-theory-core.git
cd constraint-theory-core
cargo build --release

# Run all tests (184 tests)
cargo test

# Run specific test suites
cargo test --lib              # Library unit tests
cargo test --test integration # Integration tests
cargo test --test cross_repo  # Cross-ecosystem tests

# Run examples
cargo run --example basic           # Getting started
cargo run --example batch            # Batch processing
cargo run --example simd             # SIMD demo
cargo run --example quantizer        # Quantization modes
cargo run --example holonomy         # Holonomy verification
cargo run --example hidden_dimensions # Hidden dimension encoding
cargo run --example robotics         # Robotics application
cargo run --example ml_integration   # ML direction quantization
cargo run --example visualization    # Visual demo
cargo run --release --example bench  # Performance benchmark
```

**Copy-paste this to verify:**
```rust
use constraint_theory_core::{PythagoreanManifold, snap};

fn main() {
    let manifold = PythagoreanManifold::new(200);
    let (exact, noise) = snap(&manifold, [0.577, 0.816]);
    println!("Snapped to: [{}, {}]", exact[0], exact[1]); // [0.6, 0.8]

    // Verify exactness: 0.6² + 0.8² = 1.0 EXACTLY (it's 3/5, 4/5)
    let mag_sq = exact[0] * exact[0] + exact[1] * exact[1];
    println!("Magnitude squared: {}", mag_sq); // 1.0, not 1.0000000000000002
}
```

```bash
cargo test --lib
# ✓ 184 tests pass — you're ready
```

---

## Algorithms

### 1. Pythagorean Triple Generation (Euclid's Formula)

All primitive Pythagorean triples are generated via **Euclid's formula**:

```
a = m² - n²,   b = 2mn,   c = m² + n²
```

where m > n > 0, (m − n) is odd, and gcd(m, n) = 1. Each triple produces five normalized directions on the unit circle — (a/c, b/c) and its four quadrant reflections — plus four cardinal directions. The `gcd` function uses Stein's binary GCD algorithm for integer-only speed.

**Complexity:** O(density²) triple enumeration, one-time cost at construction.

### 2. KD-Tree Nearest Neighbor

The `KDTree` module provides O(log N) spatial indexing for 2D points:

- **Build:** Recursive median-split construction alternating x/y dimensions. O(N log N).
- **Query:** Branch-and-bound search with backtracking pruned by squared-distance to split plane. O(log N) average, O(N) worst case.
- **Leaf nodes:** Cap at 16 points (`MAX_LEAF_SIZE`) with linear scan.
- **Deterministic tie-breaking:** When distances are equal, the lower-indexed point wins — critical for consensus-critical code.

### 3. Holonomy Verification

**Holonomy** measures the accumulated inconsistency when parallel-transporting a vector around a closed loop:

```
Hol(γ) = ∮ ∇ − [∇, ∇] dγ    (product of rotation matrices around the cycle)
```

- Zero holonomy (identity matrix) → globally consistent constraints
- Non-zero holonomy → detectable inconsistency, localizable by cycle bisection in O(log N)
- Information content: `I = −log₂|Hol(γ)|` (infinite for exact identity)
- `HolonomyChecker` provides incremental API for building and verifying cycles step-by-step
- Angular deviation extracted via `trace(R) = 1 + 2cos(θ)`

### 4. Sheaf Cohomology (Fast)

Computes H₀ and H₁ cohomology group dimensions for cellular complexes via Euler characteristic:

```
H₀ = β₀ (number of connected components)
H₁ = E − V + β₀ (number of independent cycles)
```

Runs in O(1) given vertex/edge/component counts. Used for **emergence detection** — every emergent behavior in a swarm corresponds to a non-trivial element of H₁.

### 5. Ricci Flow

Curvature evolution toward a target (typically zero, for manifold flattening):

```
c_new = c + α · (target − c)
```

Where α is the learning rate and the spectral gap convergence multiplier is `1.692` (matching DCS Law 103's empirically measured 1.7× latency window to 3 significant figures). This allows computing guaranteed convergence time for any swarm configuration.

### 6. Rigidity Percolation (Laman's Theorem)

A graph with V vertices in 2D is **minimally rigid** if and only if it has exactly 2V − 3 edges and every subgraph with k vertices has at most 2k − 3 edges. For 3D rigid bodies with 6 DOF, each agent needs exactly **12 independent neighbor constraints** — matching Laman's generalized threshold.

`FastPercolation` uses union-find with path compression and union-by-rank for O(α(N)) amortized analysis, returning `RigidityResult` with is_rigid, rank, deficiency, and rigid_fraction metrics.

### 7. Hidden Dimension Encoding

Implements the GUCT formula for lifting points to higher-dimensional space:

```
k = ⌈log₂(1/ε)⌉
```

For precision ε, this computes the number of hidden dimensions needed. The algorithm:
1. Lift point from Rⁿ to Rⁿ⁺ᵏ (visible + hidden dimensions)
2. Snap to lattice in lifted space using Pythagorean ratio snapping
3. Project back to Rⁿ with constraint satisfaction preserved

Cross-plane fine-tuning optimizes by snapping on orthogonal planes and selecting the best result.

### 8. Constraint-Preserving Quantization

`PythagoreanQuantizer` synthesizes three quantization paradigms:

| Mode | Algorithm | Bits | Best For |
|------|-----------|------|----------|
| **Ternary** | Sign + threshold → {-1, 0, 1} | 1 | LLM weights (16× memory reduction) |
| **Polar** | Angle → snap to Pythagorean angles → (cos θ, sin θ) | 8 | Embeddings (exact unit norm) |
| **Turbo** | Uniform quantization + Pythagorean ratio snapping | 4 | Vector databases (D ≤ 2.7 · D*) |
| **Hybrid** | Auto-select: unit-norm → Polar, sparse → Ternary, else → Turbo | 4 | Unknown inputs |

### 9. SIMD Batch Processing

On x86_64 with AVX2, batch snapping processes **8 vectors simultaneously** using `_mm256_cmp_ps` for fully vectorized dot-product comparisons. The safe `snap_batch_simd()` wrapper auto-detects CPU support at runtime and falls back to scalar code on non-AVX2 platforms.

**Critical note:** For consensus-critical code where results must be bit-identical across all platforms, use `snap_batch()` (scalar path) instead of `snap_batch_simd()`.

---

## Benchmarks

The crate uses [Criterion.rs](https://github.com/bheisler/criterion.rs) for rigorous statistical benchmarking.

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark groups
cargo bench -- manifold_snap       # Single-vector snap at densities 50–500
cargo bench -- manifold_batch      # SIMD vs scalar batch (8–1024 vectors)
cargo bench -- manifold_construction  # Build time at densities 50–500
cargo bench -- quantizer_modes     # All 4 quantization modes (4D, 128D, 512D)
cargo bench -- quantizer_batch     # Batch quantization throughput
cargo bench -- hidden_dims         # Hidden dimension count and encoding
cargo bench -- holonomy            # Cycle verification (lengths 1–64)
cargo bench -- rotation_matrices   # Rotation matrix generation
cargo bench -- full_pipeline       # End-to-end encoding pipeline
```

### Typical Results

| Operation | Time | Complexity |
|-----------|------|------------|
| Single snap (density 200) | **~100 ns** | O(log N) via KD-tree |
| SIMD batch (1000 vectors) | **~74 ns/op** | O(n log N) with AVX2 |
| Manifold build (density 200) | **~2.8 ms** | O(density²), one-time |
| Manifold build (density 500) | **~18 ms** | O(density²), one-time |
| Ternary quantize (128D) | **~50 ns** | O(d) |
| Polar quantize (128D) | **~200 ns** | O(d log d) |
| Holonomy (cycle length 16) | **~300 ns** | O(n²) |
| Hidden dim lift (k=34) | **~20 ns** | O(k) |

### Memory Profile

| Density | States | Memory |
|---------|--------|--------|
| 50 | ~250 | ~20 KB |
| 200 | ~1000 | ~80 KB |
| 500 | ~2500 | ~200 KB |
| 1000 | ~5000 | ~400 KB |

See [docs/BENCHMARKS.md](./docs/BENCHMARKS.md) for detailed methodology and [docs/PERFORMANCE.md](./docs/PERFORMANCE.md) for optimization tips.

---

## Mathematical Background

### Pythagorean Geometry on S¹

The unit circle S¹ is the set of all (x, y) with x² + y² = 1. A **Pythagorean point** on S¹ is a rational point (a/c, b/c) where (a, b, c) is a primitive Pythagorean triple satisfying a² + b² = c² exactly in integer arithmetic.

**Euclid's parameterization** establishes a bijection between coprime pairs (m, n) with m > n and all primitive triples:
```
a = m² − n²,   b = 2mn,   c = m² + n²
```
The normalized point (a/c, b/c) lies exactly on S¹ with no floating-point error. This is the foundation of deterministic vector snapping.

### Grand Unified Constraint Theory (GUCT)

GUCT extends the Pythagorean manifold into a full geometric framework:

1. **Hidden Dimensions:** For a constraint manifold M ⊂ Rⁿ, lifting to Rⁿ⁺ᵏ allows exact representation of constraint satisfaction at the lifted level. The required k is given by:
   ```
   k = ⌈log₂(1/ε)⌉
   ```
   This determines the manifold's representational capacity for exact constraint satisfaction.

2. **Holonomy and Consistency:** For any cycle of tiles, the product of gauge parallel transport matrices yields the holonomy. Zero holonomy ⇒ globally consistent. The information-holonomy relationship:
   ```
   I = −log|Hol(γ)|
   ```
   provides a gauge-invariant measure of constraint inconsistency.

3. **Ricci Flow:** Curvature evolution toward target flattens the constraint manifold. The convergence rate is governed by the spectral gap of the curvature Laplacian, with multiplier `1.692` encoding the hard phase transition for coordination entry.

4. **Rigidity Percolation:** **Laman's Theorem** (1970) provides the exact condition for structural rigidity of constraint graphs. For 2D: 2V − 3 edges minimum. For 3D rigid bodies: exactly 12 independent constraints per node — independently rediscovered as DCS Law 102 via 11 million swarm simulations.

5. **Sheaf Cohomology:** The cohomology groups H₀ (connected components) and H₁ (independent cycle basis) of the constraint cellular complex determine:
   - **H₀:** How many disconnected constraint domains exist
   - **H₁:** Every emergent behavior = non-trivial element of H₁ (detectable in O(E) time, no ML required)

### Key Theoretical Constants

| Constant | Value | Origin |
|----------|-------|--------|
| `k = ⌈log₂(1/ε)⌉` | Depends on ε | Hidden dimension formula |
| `log₂(48) = 5.585` bits | Information capacity | Exact unit vectors with 16-bit numerators |
| `1.692` | Ricci convergence multiplier | Spectral gap of curvature Laplacian |
| `12` | Laman neighbor threshold | Generalized Laman's theorem (6 DOF × 2) |
| `0.6603` | Percolation probability | Critical threshold for bond percolation on Z² |

### Research Papers

- [arXiv:2503.15847](https://arxiv.org/abs/2503.15847) — Constraint Theory: Deterministic Manifold Snapping via Pythagorean Geometry
- [Mathematical Foundations (45 pages)](https://github.com/SuperInstance/constraint-theory-research/blob/main/MATHEMATICAL_FOUNDATIONS_DEEP_DIVE.md)
- [Theoretical Guarantees](https://github.com/SuperInstance/constraint-theory-research/blob/main/guides/THEORETICAL_GUARANTEES.md)

---

## The Floating-Point Tragedy (Why You Need This)

```rust
// The bug you've fought before:
let x = 0.6_f64;
let y = 0.8_f64;
let mag = (x * x + y * y).sqrt();  // 1.0000000000000002

if mag == 1.0 { /* NEVER RUNS */ }
```

**Constraint Theory's answer:** What if `0.6, 0.8` wasn't a float approximation, but an exact rational `(3/5, 4/5)` that *displays* as `0.6, 0.8`?

```rust
use constraint_theory_core::{PythagoreanManifold, snap};

let manifold = PythagoreanManifold::new(200);        // ~1000 exact states
let (exact, noise) = snap(&manifold, [0.577, 0.816]); // ~100ns, O(log n)

// exact = [0.6, 0.8] = (3/5, 4/5) — FOREVER EXACT
// noise = 0.0236 (quantization distance)
```

---

## Real Code For Real Projects

### Game Dev: Deterministic Multiplayer

```rust
fn process_input(&mut self, joystick: [f32; 2]) {
    let (direction, noise) = self.manifold.snap(joystick);
    // direction is IDENTICAL on every client, every frame, forever
    self.velocity = [direction[0] * SPEED, direction[1] * SPEED];
}
```

### Robotics: Repeatability

```rust
fn move_arm(&mut self, target_direction: [f32; 2]) {
    let (direction, noise) = self.manifold.snap(target_direction);

    if noise > 0.01 {
        log::warn!("High quantization: target was imprecise");
    }
}
```

### ML: Direction Quantization

```rust
let (quantized, _) = manifold.snap(project_to_2d(&embedding));
// Compare with integer arithmetic — reproducible training!
```

---

## Quality Assurance

| Metric | Status |
|--------|--------|
| **Tests** | 112 passing, 0 failing |
| **Coverage** | Core algorithms 100% covered |
| **CI** | Linux, macOS, Windows |
| **Fuzzing** | Property-based tests with `proptest` |
| **Dependencies** | Zero — pure Rust |

---

## Limitations

| Limitation | Why | Impact |
|------------|-----|--------|
| **2D only** | Pythagorean triples are inherently 2D | Not for 3D games, robotics, drones |
| **~1000 discrete states** | Lattice, not continuous | ~0.36° angular resolution |
| **Quantization tradeoff** | Snapping introduces noise | Check returned `noise` value |
| **Direction only** | Unit vectors only | Position/velocity drift not addressed |
| **SIMD path variance** | AVX2 platform-dependent | Use scalar `snap()` for consensus code |

---

## The Ecosystem

**Constraint Theory Family:**

| Repo | What It Is | Key Features |
|------|------------|-------------|
| **[constraint-theory-core](https://github.com/SuperInstance/constraint-theory-core)** | This crate — Rust, zero deps | O(log n) KD-tree, SIMD batch |
| **[constraint-theory-python](https://github.com/SuperInstance/constraint-theory-python)** | Python bindings (PyO3) | NumPy integration, PyTorch compatible |
| **[constraint-theory-web](https://github.com/SuperInstance/constraint-theory-web)** | 50 interactive demos | KD-tree visualizer, Pythagorean demo |
| **[constraint-theory-research](https://github.com/SuperInstance/constraint-theory-research)** | Mathematical foundations | arXiv paper, proofs, open problems |
| **[constraint-ranch](https://github.com/SuperInstance/constraint-ranch)** | Educational game demos | Puzzle games, species simulation |
| **[constraint-flow](https://github.com/SuperInstance/constraint-flow)** | Business automation | Exact financial calculations |
| **[constraint-theory-agent](https://github.com/SuperInstance/constraint-theory-agent)** | Implementation agent | Code audit, refactoring, explanations |

**Eisenstein Hex Integer Ecosystem:**

| Project | Description |
|---------|-------------|
| **[eisenstein](https://github.com/SuperInstance/eisenstein)** | Core Rust crate — exact hex arithmetic, zero deps |
| **[eisenstein-c](https://github.com/SuperInstance/eisenstein-c)** | Same math, for microcontrollers. 1KB `.text`. |
| **[eisenstein-wasm](https://github.com/SuperInstance/eisenstein-wasm)** | Same math, for browsers and Node.js |
| **[eisenstein-bench](https://github.com/SuperInstance/eisenstein-bench)** | Benchmark all implementations side-by-side |
| **[eisenstein-fuzz](https://github.com/SuperInstance/eisenstein-fuzz)** | Property-based fuzzing across the ecosystem |
| **[eisenstein-do178c](https://github.com/SuperInstance/eisenstein-do178c)** | DO-178C formally verified for safety-critical systems |
| **[arm-neon-eisenstein-bench](https://github.com/SuperInstance/arm-neon-eisenstein-bench)** | 4× parallel hex math on ARM NEON |
| **[hexgrid-gen](https://github.com/SuperInstance/hexgrid-gen)** | Code generation for any language in the ecosystem |
| **[flux-lucid](https://github.com/SuperInstance/flux-lucid)** | Unified intent-directed ecosystem orchestrator |

**Next →** Orchestrate everything: **[flux-lucid](https://github.com/SuperInstance/flux-lucid)**

---

## Contributing

**[Good First Issues](https://github.com/SuperInstance/constraint-theory-core/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)** · **[CONTRIBUTING.md](CONTRIBUTING.md)**

```bash
rustup component add clippy rustfmt
cargo fmt && cargo clippy -- -D warnings && cargo test
```

---

## Citation

```bibtex
@software{constraint_theory,
  title={Constraint Theory: Deterministic Manifold Snapping via Pythagorean Geometry},
  author={SuperInstance},
  year={2025},
  url={https://github.com/SuperInstance/constraint-theory-core},
  version={1.0.1}
}
```

---

<div align="center">

### ⚡ Deterministic directions for 2D systems.

**→ [Get started in 30 seconds](#quick-start)** · **[Try interactive demos](https://constraint-theory-web.pages.dev)** · **[Read the docs](https://docs.rs/constraint-theory-core)**

*Built with 🦀 for systems that need exact reproducibility*

</div>

---

<img src="callsign1.jpg" width="128" alt="callsign">
