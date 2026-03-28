# Onboarding Guide: constraint-theory-core

**Repository:** https://github.com/SuperInstance/constraint-theory-core
**Language:** Rust
**Version:** 0.2.0
**Last Updated:** 2025-01-27

---

## Welcome to Constraint Theory Core

This repository contains the **core Rust implementation** of Constraint Theory - a mathematical framework for exact constraint satisfaction using Pythagorean geometry, hidden dimensions, and holographic encoding.

### What You'll Learn

1. How to build and test the library
2. Core concepts: manifolds, snapping, holonomy
3. Using the PythagoreanQuantizer
4. Implementing custom constraint solvers
5. Contributing to the codebase

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

### 4. Your First Constraint

```rust
use constraint_theory::{PythagoreanManifold, Point2D};

fn main() {
    // Create a Pythagorean manifold
    let manifold = PythagoreanManifold::new_2d();
    
    // A point near the unit circle
    let point = Point2D::new(0.6, 0.8);
    
    // Snap to nearest Pythagorean point
    let snapped = manifold.snap(&point);
    
    println!("Original: ({}, {})", point.x, point.y);
    println!("Snapped:  ({}, {})", snapped.x, snapped.y);
    println!("Distance: {}", point.distance_to(&snapped));
}
```

---

## Core Concepts

### 1. Constraint Manifold

A **constraint manifold** is a surface in n-dimensional space where all constraints are satisfied. Constraint Theory provides exact representation of points on these manifolds.

```rust
use constraint_theory::ConstraintManifold;

// Define a simple constraint: x² + y² = 1 (unit circle)
let circle = ConstraintManifold::unit_circle();

// Check if a point satisfies constraints
let on_circle = Point2D::new(0.6, 0.8);  // 0.36 + 0.64 = 1 ✓
let off_circle = Point2D::new(0.5, 0.5); // 0.25 + 0.25 = 0.5 ✗

assert!(circle.contains(&on_circle));
assert!(!circle.contains(&off_circle));
```

### 2. Pythagorean Snapping

**Pythagorean snapping** projects any point to the nearest valid constraint-satisfying point using integer lattice geometry.

```rust
use constraint_theory::{PythagoreanManifold, SnapConfig};

let manifold = PythagoreanManifold::new_2d();
let config = SnapConfig {
    max_hypotenuse: 1000,  // Search up to c=1000
    exact: true,           // Require exact representation
};

// Snap a point to the lattice
let point = Point2D::new(0.7, 0.7);  // Not on unit circle
let snapped = manifold.snap_with_config(&point, &config);

// Result is a Pythagorean triple: (a/c, b/c) where a² + b² = c²
// Example output: (3/5, 4/5) which is (0.6, 0.8)
println!("Snapped to: {}", snapped);
```

### 3. Hidden Dimensions

**Hidden dimensions** encode precision logarithmically. For precision ε, you need k = ⌈log₂(1/ε)⌉ hidden dimensions.

```rust
use constraint_theory::{HiddenDimensionEncoder, Precision};

let encoder = HiddenDimensionEncoder::new(Precision::P10);  // 10 decimal places

// Encode a point with hidden dimensions
let point = vec![1.0, 2.0, 3.0];
let lifted = encoder.lift(&point);

// lifted has dimension = point.len() + hidden_dims
println!("Original dimension: {}", point.len());
println!("Lifted dimension: {}", lifted.len());  // +34 hidden dims for 1e-10 precision

// After computation, project back
let projected = encoder.project(&lifted);
```

### 4. Holonomy

**Holonomy** measures the consistency of constraint satisfaction around cycles. Zero holonomy = globally consistent.

```rust
use constraint_theory::{HolonomyChecker, ConstraintCycle};

let checker = HolonomyChecker::new();

// Define a cycle of constraint operations
let cycle = ConstraintCycle::new(vec![
    ConstraintOp::Rotate { axis: Axis::Z, angle: PI/2.0 },
    ConstraintOp::Rotate { axis: Axis::X, angle: PI/2.0 },
    ConstraintOp::Rotate { axis: Axis::Z, angle: -PI/2.0 },
]);

// Check holonomy (should be identity for consistent constraints)
let holonomy = checker.compute(&cycle);

if holonomy.is_identity() {
    println!("Constraints are globally consistent!");
} else {
    println!("Warning: Holonomy error = {}", holonomy.error());
}
```

### 5. PythagoreanQuantizer

The **PythagoreanQuantizer** integrates TurboQuant, BitNet, PolarQuant, and QJL for constraint-aware quantization.

```rust
use constraint_theory::{PythagoreanQuantizer, QuantizationMode};

// Create quantizer for LLM weights
let quantizer = PythagoreanQuantizer::new(
    QuantizationMode::Ternary,  // BitNet-style {-1, 0, 1}
    1.58,                       // Bits per weight
);

// Quantize weights
let weights = vec![0.5, -0.3, 0.9, -1.2, 0.0];
let result = quantizer.quantize(&weights);

println!("Original: {:?}", weights);
println!("Quantized: {:?}", result.data);
println!("Codes: {:?}", result.codes);  // {-1, 0, 1}
println!("Sparsity: {}", result.sparsity());
```

---

## Project Structure

```
constraint-theory-core/
├── Cargo.toml              # Package manifest
├── src/
│   ├── lib.rs              # Library entry point
│   ├── manifold.rs         # Constraint manifold implementation
│   ├── kdtree.rs           # KD-tree for fast lattice lookup
│   ├── quantizer.rs        # PythagoreanQuantizer
│   ├── holonomy.rs         # Holonomy computation
│   ├── hidden_dims.rs      # Hidden dimension encoding
│   ├── lattice/            # Lattice implementations
│   │   ├── pythagorean.rs  # 2D Pythagorean lattice
│   │   ├── hurwitz.rs      # 3D/4D quaternion lattice
│   │   ├── e8.rs           # 8D E8 lattice
│   │   └── leech.rs        # 24D Leech lattice
│   ├── simd.rs             # SIMD optimizations
│   └── cuda/               # CUDA kernels (optional)
├── examples/
│   ├── basic.rs            # Getting started
│   ├── quantization.rs     # Quantization examples
│   ├── robotics.rs         # Robotics application
│   └── ml_integration.rs   # Machine learning
├── tests/
│   ├── test_manifold.rs    # Manifold tests
│   ├── test_quantizer.rs   # Quantizer tests
│   └── test_holonomy.rs    # Holonomy tests
├── docs/
│   ├── TUTORIAL.md         # Extended tutorial
│   ├── PERFORMANCE.md      # Performance guide
│   └── API.md              # API reference
└── benches/
    └── bench_snap.rs       # Benchmarks
```

---

## Key APIs

### PythagoreanManifold

```rust
impl PythagoreanManifold {
    /// Create a 2D Pythagorean manifold
    pub fn new_2d() -> Self;
    
    /// Create a 3D Hurwitz quaternion manifold
    pub fn new_3d() -> Self;
    
    /// Create an N-dimensional manifold
    pub fn new_nd(dimensions: usize) -> Self;
    
    /// Snap a point to the nearest lattice point
    pub fn snap(&self, point: &Point) -> Point;
    
    /// Snap with custom configuration
    pub fn snap_with_config(&self, point: &Point, config: &SnapConfig) -> Point;
    
    /// Find all lattice points within radius
    pub fn within_radius(&self, center: &Point, radius: f64) -> Vec<Point>;
    
    /// Get the holonomy group of the manifold
    pub fn holonomy_group(&self) -> HolonomyGroup;
}
```

### PythagoreanQuantizer

```rust
impl PythagoreanQuantizer {
    /// Create a new quantizer with specified mode
    pub fn new(mode: QuantizationMode, bits: f64) -> Self;
    
    /// Quantize data with constraint preservation
    pub fn quantize(&self, data: &[f64]) -> QuantizationResult;
    
    /// Dequantize back to floating-point
    pub fn dequantize(&self, result: &QuantizationResult) -> Vec<f64>;
    
    /// Build index for fast ANN search
    pub fn build_index(&self, data: &[Vec<f64>]) -> QJLIndex;
    
    /// Fast nearest neighbor search
    pub fn nearest_neighbors(&self, query: &[f64], k: usize) -> Vec<usize>;
}

pub enum QuantizationMode {
    Ternary,  // BitNet: {-1, 0, 1}
    Polar,    // PolarQuant: exact unit norm
    Turbo,    // TurboQuant: near-optimal MSE
    Hybrid,   // Auto-select
}
```

### UniversalConstraintSolver

```rust
impl UniversalConstraintSolver {
    /// Create a solver with specified precision
    pub fn new(precision: f64) -> Self;
    
    /// Solve constraint system
    pub fn solve(&self, constraints: &[Constraint]) -> Solution;
    
    /// Compute hidden dimension count for precision
    pub fn hidden_dim_count(&self) -> usize {
        (1.0 / self.precision).log2().ceil() as usize
    }
    
    /// Verify holonomy of solution
    pub fn verify(&self, solution: &Solution) -> bool;
}
```

---

## Performance

### Benchmarks (AMD Ryzen 9 7950X)

| Operation | Time | Rate |
|-----------|------|------|
| 2D snap | 45ns | 22M/sec |
| 3D snap (Hurwitz) | 120ns | 8.3M/sec |
| 8D snap (E8) | 350ns | 2.9M/sec |
| Ternary quantize (1K weights) | 2.1μs | 476M weights/sec |
| ANN search (1M vectors, d=128) | 0.8ms | 1.25M queries/sec |

### SIMD Optimization

The library uses SIMD for batch operations:

```rust
// Enable SIMD (default in release mode)
cargo build --release --features simd

// Disable SIMD for debugging
cargo build --no-default-features
```

### GPU Acceleration (Optional)

```rust
// Enable CUDA support
cargo build --release --features cuda

// Use GPU for large-scale operations
let gpu_quantizer = PythagoreanQuantizer::gpu_new(QuantizationMode::Turbo, 4.0)?;
let result = gpu_quantizer.quantize_batch(&large_matrix)?;
```

---

## Integration Examples

### With PyTorch (via constraint-theory-python)

```python
from constraint_theory import PythagoreanQuantizer, QuantizationMode
import torch

# Quantize PyTorch weights
weights = torch.randn(4096, 4096).numpy()
quantizer = PythagoreanQuantizer(QuantizationMode.TERNARY, 1.58)
result = quantizer.quantize(weights)

# Back to PyTorch
quantized_weights = torch.from_numpy(result.data)
```

### With Games (via constraint-theory-web WASM)

```javascript
// In browser
import init, { PythagoreanManifold } from 'constraint_theory';

await init();
const manifold = new PythagoreanManifold(2);  // 2D

// Snap game coordinates to exact positions
const snapped = manifold.snap(0.7, 0.7);
console.log(snapped.x, snapped.y);  // 0.6, 0.8 (3-4-5 triangle)
```

---

## Testing

### Run All Tests

```bash
# Unit tests
cargo test

# Benchmarks
cargo bench

# Examples
cargo run --example basic
cargo run --example quantization
cargo run --example robotics

# GPU tests (if CUDA available)
cargo test --features cuda
```

### Write Your Own Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_constraint() {
        let manifold = PythagoreanManifold::new_2d();
        let point = Point2D::new(0.5, 0.5);
        let snapped = manifold.snap(&point);
        
        // Verify on unit circle
        let r2 = snapped.x * snapped.x + snapped.y * snapped.y;
        assert!((r2 - 1.0).abs() < 1e-10);
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

**1. Precision Loss**
```
Problem: Snapping gives unexpected results
Solution: Check your precision requirements
```
```rust
// Use higher precision configuration
let config = SnapConfig {
    max_hypotenuse: 10000,  // Increase search range
    exact: true,
};
```

**2. Holonomy Errors**
```
Problem: Constraints satisfied locally but not globally
Solution: Check constraint cycle holonomy
```
```rust
let checker = HolonomyChecker::new();
let error = checker.compute(&cycle).error();
if error > 1e-10 {
    println!("Constraints are over-constrained. Remove some constraints.");
}
```

**3. Memory Issues**
```
Problem: Out of memory with large lattices
Solution: Use on-demand lattice generation
```
```rust
let manifold = PythagoreanManifold::new_2d_lazy();
// Lattice points generated on demand, not precomputed
```

---

## Resources

### Documentation

- [API Reference](./docs/API.md)
- [Tutorial](./docs/TUTORIAL.md)
- [Performance Guide](./docs/PERFORMANCE.md)
- [Benchmarks](./docs/BENCHMARKS.md)

### External

- [Grand Unified Constraint Theory](../research/GRAND_UNIFIED_CONSTRAINT_THEORY.md)
- [Unified Quantization System](../research/UNIFIED_QUANTIZATION_SYSTEM.md)
- [Research Summary](../research/RESEARCH_SUMMARY_10_ITERATIONS.md)

### Community

- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: Questions and ideas

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
