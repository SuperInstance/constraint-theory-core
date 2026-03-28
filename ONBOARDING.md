# Onboarding Guide: constraint-theory-core

**Repository:** https://github.com/SuperInstance/constraint-theory-core
**Language:** Rust
**Version:** 1.0.1
**Last Updated:** 2025-01-27
**Part of:** Grand Unified Constraint Theory (GUCT)

---

## Welcome to Constraint Theory Core

This repository contains the **core Rust implementation** of Constraint Theory - a mathematical framework for exact constraint satisfaction using Pythagorean geometry, hidden dimensions, and holographic encoding.

### What You'll Learn

1. How to build and test the library
2. Core concepts: manifolds, snapping, holonomy
3. Using the PythagoreanManifold API
4. Understanding the hidden dimensions formula: k = ⌈log₂(1/ε)⌉
5. Performance characteristics and benchmarking
6. Contributing to the codebase

---

## Prerequisites

### Required

- **Rust 1.70+** (use `rustup` to install)
- **Cargo** (comes with Rust)
- **Git**

### Optional (for GPU support)

- **CUDA 12.0+**
- **NVIDIA GPU** with compute capability 7.0+

### Installation Check

```bash
# Verify Rust installation
rustc --version   # Should show 1.70 or higher
cargo --version   # Should work

# Clone the repository
git clone https://github.com/SuperInstance/constraint-theory-core.git
cd constraint-theory-core
```

---

## Quick Start (5 Minutes)

### 1. Build the Library

```bash
cargo build --release
```

### 2. Run Tests

```bash
cargo test
```

### 3. Run an Example

```bash
# Basic usage example
cargo run --example basic

# Pythagorean snapping demo
cargo run --example visualization

# Benchmark performance
cargo run --example bench
```

### 4. Your First Snap

```rust
use constraint_theory_core::{PythagoreanManifold, snap};

fn main() {
    // Create a Pythagorean manifold with density 200 (~1000 valid states)
    let manifold = PythagoreanManifold::new(200);
    
    // A point near the unit circle (noisy input)
    let input = [0.577_f32, 0.816];
    
    // Snap to nearest Pythagorean triple
    let (snapped, noise) = snap(&manifold, input);
    
    println!("Input:  ({}, {})", input[0], input[1]);
    println!("Snapped: ({}, {})", snapped[0], snapped[1]);
    println!("Noise:   {:.6}", noise);
    // Output: Snapped: (0.6, 0.8) - the 3-4-5 triangle!
}
```

---

## Core Concepts

### 1. Pythagorean Manifold

A **Pythagorean manifold** is a discrete set of points on the unit circle S¹ where each point corresponds to an exact Pythagorean triple (a/c, b/c) with a² + b² = c².

```rust
use constraint_theory_core::PythagoreanManifold;

// Create a manifold with ~1000 valid Pythagorean states
let manifold = PythagoreanManifold::new(200);
println!("Manifold has {} exact states", manifold.state_count());

// Every state satisfies x² + y² = 1 EXACTLY
let (snapped, noise) = manifold.snap([0.6, 0.8]);
let mag_sq = snapped[0] * snapped[0] + snapped[1] * snapped[1];
assert!((mag_sq - 1.0).abs() < 1e-6, "Exact unit norm!");
```

### 2. Pythagorean Snapping

**Pythagorean snapping** projects any unit vector to the nearest valid Pythagorean triple using O(log n) KD-tree lookup.

```rust
use constraint_theory_core::{PythagoreanManifold, snap};

let manifold = PythagoreanManifold::new(200);

// Snap a point to the nearest valid state
let input = [0.707_f32, 0.707];  // ~45 degrees, not exact
let (snapped, noise) = snap(&manifold, input);

// Result is a Pythagorean triple: (a/c, b/c) where a² + b² = c²
println!("Snapped to: ({:.4}, {:.4})", snapped[0], snapped[1]);
println!("Noise: {:.6}", noise);
// The noise tells you how far from the nearest exact state
```

### 3. Hidden Dimensions Formula

**Hidden dimensions** encode precision logarithmically as part of the Grand Unified Constraint Theory (GUCT). For precision ε, the required hidden dimension count is:

```
k = ⌈log₂(1/ε)⌉
```

This formula determines the number of additional dimensions needed to represent precision exactly:

| Precision (ε) | k = ⌈log₂(1/ε)⌉ | Hidden Dims |
|--------------|-----------------|-------------|
| 1e-3 | ⌈9.97⌉ = 10 | 10 |
| 1e-6 | ⌈19.93⌉ = 20 | 20 |
| 1e-9 | ⌈29.90⌉ = 30 | 30 |
| 1e-10 | ⌈33.22⌉ = 34 | 34 |

**Mathematical Foundation:**

For a constraint manifold M ⊂ Rⁿ, lifting to Rⁿ⁺ᵏ allows exact representation of:
- Constraint satisfaction at the lifted level
- Holonomy group structure for global consistency
- Quantization noise bounds: d_g(v, σ(v)) < π/(2n)

See the [research documentation](https://github.com/SuperInstance/constraint-theory-research) for formal proofs.

### 4. Holonomy

**Holonomy** measures the consistency of constraint satisfaction around cycles. Zero holonomy = globally consistent. This is implemented in the `cohomology` module for advanced use cases.

```rust
use constraint_theory_core::PythagoreanManifold;

// For standard usage, holonomy is automatically satisfied
// by construction of the Pythagorean manifold
let manifold = PythagoreanManifold::new(200);

// Every snap operation preserves exact constraint satisfaction
let (v1, _) = manifold.snap([0.6, 0.8]);
let (v2, _) = manifold.snap([0.8, 0.6]);

// Both are exact Pythagorean triples
// v1² + v2² = 1.0 exactly for both
```

For advanced holonomy computations and cycle checking, see the `cohomology` module in the source code.

### 5. SIMD Batch Processing

For high-throughput applications, use SIMD batch processing:

```rust
use constraint_theory_core::PythagoreanManifold;

let manifold = PythagoreanManifold::new(200);

// Process multiple vectors at once with SIMD optimization
let vectors = vec![[0.6, 0.8], [0.8, 0.6], [0.1, 0.99], [0.99, 0.1]];
let results = manifold.snap_batch_simd(&vectors);

for (snapped, noise) in results {
    println!("Snapped: ({:.3}, {:.3}), Noise: {:.6}", snapped[0], snapped[1], noise);
}
```

**Performance:** ~74 ns/op with SIMD vs ~100 ns/op for single operations.

**Note:** For consensus-critical code, use `snap_batch()` (scalar path) to ensure deterministic tie-breaking across platforms.

---

## Project Structure

```
constraint-theory-core/
├── Cargo.toml              # Package manifest
├── src/
│   ├── lib.rs              # Library entry point
│   ├── manifold.rs         # PythagoreanManifold implementation
│   ├── kdtree.rs           # KD-tree for O(log n) lookup
│   ├── hidden_dimensions.rs # Hidden dimension encoding (GUCT)
│   ├── quantizer.rs        # PythagoreanQuantizer (TurboQuant/BitNet/PolarQuant)
│   ├── holonomy.rs         # Holonomy verification for constraint consistency
│   ├── cohomology.rs       # Sheaf cohomology (advanced)
│   ├── curvature.rs        # Ricci flow evolution
│   ├── gauge.rs            # Gauge theory operations
│   ├── percolation.rs      # Rigidity percolation
│   ├── tile.rs             # ConstraintBlock and Tile
│   ├── simd.rs             # SIMD optimizations
│   └── edge_case_tests.rs  # Edge case coverage
├── examples/
│   ├── basic.rs            # Getting started
│   ├── batch.rs            # Batch processing
│   ├── robotics.rs         # Robotics application
│   ├── ml_integration.rs   # Machine learning
│   ├── simd.rs             # SIMD demo
│   ├── bench.rs            # Performance benchmark
│   └── visualization.rs    # Visual demo
├── docs/
│   ├── TUTORIAL.md         # Extended tutorial
│   ├── PERFORMANCE.md      # Performance guide
│   ├── BENCHMARKS.md       # Benchmark results
│   ├── TESTING.md          # Testing methodology
│   └── DISCLAIMERS.md      # Important clarifications
└── benches/
    └── (integrated in examples)
```

---

## Key APIs

### PythagoreanManifold

```rust
use constraint_theory_core::PythagoreanManifold;

impl PythagoreanManifold {
    /// Create a Pythagorean manifold with specified density
    /// Higher density = more valid states = finer angular resolution
    pub fn new(density: usize) -> Self;
    
    /// Get the number of valid states in the manifold
    pub fn state_count(&self) -> usize;
    
    /// Get reference to all valid states
    pub fn states(&self) -> &[[f32; 2]];
    
    /// Snap a vector to nearest Pythagorean triple (O(log n) via KD-tree)
    /// Returns (snapped_vector, noise) where noise = 1 - resonance
    pub fn snap(&self, vector: [f32; 2]) -> ([f32; 2], f32);
    
    /// SIMD-optimized batch snapping
    pub fn snap_batch_simd(&self, vectors: &[[f32; 2]]) -> Vec<([f32; 2], f32)>;
    
    /// SIMD batch snapping with pre-allocated buffer
    pub fn snap_batch_simd_into(&self, vectors: &[[f32; 2]], results: &mut [([f32; 2], f32)]);
    
    /// Scalar batch snapping (deterministic, for consensus-critical code)
    pub fn snap_batch(&self, vectors: &[[f32; 2]], results: &mut [([f32; 2], f32)]);
    
    /// Validate input for consensus-critical systems
    pub fn validate_input(&self, vector: [f32; 2]) -> Result<(), &'static str>;
    
    /// Get maximum angular error for this density
    pub fn max_angular_error(&self) -> f32;
    
    /// Get recommended noise threshold for a use case
    pub fn recommended_noise_threshold(use_case: &str) -> f32;
}

/// Convenience function for snapping
pub fn snap(manifold: &PythagoreanManifold, vector: [f32; 2]) -> ([f32; 2], f32);
```

### PythagoreanTriple

```rust
use constraint_theory_core::manifold::PythagoreanTriple;

impl PythagoreanTriple {
    /// Create a new Pythagorean triple (a, b, c) where a² + b² = c²
    pub fn new(a: f32, b: f32, c: f32) -> Self;
    
    /// Check if the triple satisfies a² + b² = c²
    pub fn is_valid(&self) -> bool;
    
    /// Convert to normalized 2D vector [a/c, b/c]
    pub fn to_vector(&self) -> [f32; 2];
}
```

### Hidden Dimensions

The hidden dimensions module implements the Grand Unified Constraint Theory (GUCT) approach to exact constraint satisfaction.

```rust
use constraint_theory_core::{hidden_dim_count, holographic_accuracy, HiddenDimensionConfig};

// Compute hidden dimensions for precision ε: k = ⌈log₂(1/ε)⌉
let k = hidden_dim_count(1e-10);
assert_eq!(k, 34);

// Compute holographic accuracy
let accuracy = holographic_accuracy(10, 12);

// Use config for encoding
let config = HiddenDimensionConfig::new(1e-6);
let encoded = config.encode(&[0.6, 0.8]);
```

### PythagoreanQuantizer

Unified quantization with constraint preservation, integrating TurboQuant, BitNet, and PolarQuant technologies.

```rust
use constraint_theory_core::{PythagoreanQuantizer, QuantizationMode};

// Create quantizer for different use cases
let llm_quantizer = PythagoreanQuantizer::for_llm();     // Ternary {-1, 0, 1}
let embed_quantizer = PythagoreanQuantizer::for_embeddings(); // Polar, unit norm
let db_quantizer = PythagoreanQuantizer::for_vector_db();     // Turbo, near-optimal

// Quantize with constraint preservation
let vector = vec![0.6, 0.8, 0.0, 0.0];
let result = embed_quantizer.quantize(&vector);

// Result preserves unit norm exactly
assert!(result.unit_norm_preserved);
```

**Quantization Modes:**
| Mode | Description | Best For |
|------|-------------|----------|
| Ternary | {-1, 0, 1} values | LLM weights, sparse data |
| Polar | Exact unit norm | Embeddings, direction vectors |
| Turbo | Near-optimal distortion | Vector databases, general |
| Hybrid | Auto-select | Unknown input characteristics |

### Holonomy Verification

Holonomy measures constraint consistency around cycles. Zero holonomy = globally consistent.

```rust
use constraint_theory_core::{compute_holonomy, verify_holonomy, HolonomyChecker};

// Compute holonomy for a cycle of transformations
let result = compute_holonomy(&cycle);
if result.is_identity() {
    println!("Constraints are globally consistent");
}

// Use incremental checker for complex cycles
let mut checker = HolonomyChecker::default_tolerance();
checker.apply(&rotation1);
checker.apply(&rotation2);
let final_result = checker.check_closed();
```

---

## Performance

### Benchmarks (Typical Results)

| Operation | Time | Rate | Notes |
|-----------|------|------|-------|
| Single snap | ~100 ns | ~10M/sec | KD-tree O(log n) |
| Batch SIMD (1000) | ~74 ns/op | ~13.5M/sec | AVX2 parallel |
| Manifold build (density=200) | ~2.8 ms | - | One-time cost |
| Memory usage (density=200) | ~80 KB | - | Linear with density |

**Key insight:** KD-tree provides O(log n) vs O(n) brute force. At 1000 states, that's ~109x faster.

See [docs/BENCHMARKS.md](./docs/BENCHMARKS.md) for detailed methodology.

### SIMD Optimization

The library uses SIMD for batch operations:

```bash
# Build with SIMD (default in release mode)
cargo build --release --features simd

# SIMD is automatically used when:
# 1. x86_64 architecture with AVX2
# 2. Batch size >= 8 vectors
```

### SIMD Warning

⚠️ **For consensus-critical code**, use the scalar `snap_batch()` instead of `snap_batch_simd()`:
- SIMD may have platform-dependent tie-breaking
- Scalar path ensures identical results across all platforms

---

## Integration Examples

### With Python (via constraint-theory-python)

```python
from constraint_theory import PythagoreanManifold

# Create manifold
manifold = PythagoreanManifold(density=200)

# Snap to nearest Pythagorean triple
x, y, noise = manifold.snap(0.577, 0.816)
print(f"Snapped: ({x}, {y})")  # (0.6, 0.8)

# Batch snap
results = manifold.snap_batch([[0.6, 0.8], [0.8, 0.6]])
```

See [constraint-theory-python](https://github.com/SuperInstance/constraint-theory-python) for more.

### With Web (via constraint-theory-web)

See the [interactive demos](https://constraint-theory-web.pages.dev) for browser-based visualizations.

```javascript
// Example: KD-tree visualization
// https://constraint-theory-web.pages.dev/simulators/kdtree/
```

---

## Testing

### Run All Tests

```bash
# Unit tests
cargo test

# Run examples
cargo run --example basic
cargo run --example batch
cargo run --example robotics

# Performance benchmark
cargo run --release --example bench
```

### Write Your Own Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_snap() {
        let manifold = PythagoreanManifold::new(200);
        let (snapped, noise) = manifold.snap([0.6, 0.8]);
        
        // Verify exact unit norm
        let mag_sq = snapped[0] * snapped[0] + snapped[1] * snapped[1];
        assert!((mag_sq - 1.0).abs() < 1e-6);
        assert!(noise < 0.001);
    }
}
```

---

## Contributing

### Development Setup

```bash
# Install development tools
rustup component add clippy rustfmt

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Run all checks
cargo test && cargo clippy && cargo fmt -- --check
```

### Pull Request Checklist

- [ ] Code formatted with `cargo fmt`
- [ ] No clippy warnings
- [ ] All tests pass
- [ ] New code has tests
- [ ] Documentation updated
- [ ] CHANGELOG.md updated

### Adding New Features

1. **New Lattice Type**
   ```rust
   // src/lattice/my_lattice.rs
   pub struct MyLattice {
       // ...
   }
   
   impl Lattice for MyLattice {
       fn snap(&self, point: &Point) -> Point;
       fn density(&self) -> f64;
   }
   ```

2. **New Quantization Mode**
   ```rust
   // src/quantizer.rs
   pub enum QuantizationMode {
       // ... existing modes
       MyMode,
   }
   
   impl PythagoreanQuantizer {
       fn quantize_my_mode(&self, data: &[f64]) -> QuantizationResult {
           // Implementation
       }
   }
   ```

---

## Troubleshooting

### Common Issues

**1. High Noise Values**
```
Problem: Snapping gives unexpectedly high noise
Solution: Ensure input is a direction (will be normalized internally)
```
```rust
// The snap function normalizes internally, but check your inputs
let (snapped, noise) = manifold.snap([0.6, 0.8]);
if noise > 0.1 {
    println!("Warning: High quantization noise = {:.4}", noise);
}
```

**2. SIMD Inconsistency Across Platforms**
```
Problem: Different results on different machines
Solution: Use scalar path for consensus-critical code
```
```rust
// For consensus-critical code:
let mut results = vec![([0.0, 0.0], 0.0f32); vectors.len()];
manifold.snap_batch(&vectors, &mut results);  // Scalar, deterministic
```

**3. Memory with Large Densities**
```
Problem: Memory usage too high
Solution: Use lower density (memory scales linearly)
```
| Density | States | Memory |
|---------|--------|--------|
| 200 | ~1000 | ~80 KB |
| 500 | ~2500 | ~200 KB |
| 1000 | ~5000 | ~400 KB |

---

## Resources

### Documentation

- [Tutorial](./docs/TUTORIAL.md) - Step-by-step guide
- [Performance Guide](./docs/PERFORMANCE.md) - Optimization tips
- [Benchmarks](./docs/BENCHMARKS.md) - Detailed methodology
- [Testing Guide](./docs/TESTING.md) - Testing methodology

### Ecosystem

| Repo | Description |
|------|-------------|
| [constraint-theory-core](https://github.com/SuperInstance/constraint-theory-core) | This repo - Rust crate |
| [constraint-theory-python](https://github.com/SuperInstance/constraint-theory-python) | Python bindings |
| [constraint-theory-web](https://github.com/SuperInstance/constraint-theory-web) | 49 interactive demos |
| [constraint-theory-research](https://github.com/SuperInstance/constraint-theory-research) | Mathematical foundations |

### Research

- [Mathematical Foundations Deep Dive](https://github.com/SuperInstance/constraint-theory-research/blob/main/MATHEMATICAL_FOUNDATIONS_DEEP_DIVE.md)
- [Theoretical Guarantees](https://github.com/SuperInstance/constraint-theory-research/blob/main/guides/THEORETICAL_GUARANTEES.md)
- [arXiv Paper](https://arxiv.org/abs/2503.15847)

---

## License

MIT License - See [LICENSE](./LICENSE) for details.

---

## Next Steps

1. ✅ Build and test the library
2. ✅ Run the examples
3. 📖 Read the [Tutorial](./docs/TUTORIAL.md)
4. 🚀 Build something amazing!

**Welcome to Constraint Theory!** 🎉
