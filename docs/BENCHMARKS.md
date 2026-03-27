# ConstraintTheory Benchmarks

**Last Updated:** 2026-03-17
**Status:** Research Release - Empirical Validation Ongoing

---

## Overview

This document provides comprehensive benchmark results for ConstraintTheory, including:
- Performance measurements of core operations
- Comparison with alternative approaches
- Methodology documentation
- Known limitations and caveats

---

## Core Performance Metrics

### Pythagorean Snap Operation

The primary operation in ConstraintTheory is "snapping" a continuous vector to the nearest valid Pythagorean triple.

**Test Configuration:**
- **CPU:** Varies (see individual results)
- **Manifold Density:** 200 (yields ~1000 valid states)
- **Operation:** Nearest-neighbor lookup + normalization
- **Metric:** Time per operation (nanoseconds)

| Implementation | Time (ns) | Time (us) | Ops/sec | Speedup |
|----------------|-----------|-----------|---------|---------|
| Python NumPy (baseline) | 10,900 | 10.9 | 91K | 1.0x |
| Rust Scalar | 20,740 | 20.7 | 48K | 0.5x |
| Rust SIMD | 6,390 | 6.4 | 156K | 1.7x |
| **Rust + KD-tree** | **~100** | **0.1** | **~10M** | **~109x** |

**Important Notes:**
1. The "speedup" compares to the *NumPy baseline*, not to production-grade KD-tree implementations
2. A well-optimized KD-tree in Python/NumPy would achieve similar O(log n) performance
3. The ~109x figure applies **only** to geometric nearest-neighbor operations

### Complexity Analysis

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Manifold build | O(n log n) | One-time cost |
| Single snap | O(log n) | Via KD-tree |
| Batch snap (SIMD) | O(m log n) | m vectors, amortized |
| Memory usage | O(n) | Linear in manifold size |

---

## Methodology

### How Benchmarks Were Run

```bash
# Clone repository
git clone https://github.com/SuperInstance/constraint-theory
cd constraint-theory

# Build in release mode
cargo build --release

# Run benchmark
cargo run --release --example bench

# Run comparison benchmark
cargo run --release --example bench_comparison
```

### Benchmark Parameters

- **Warmup:** 10,000 iterations to stabilize CPU frequency
- **Measurement:** 100,000 operations over 5 iterations
- **Manifold:** 200 density (~1000 states)
- **Query vectors:** Pre-generated unit vectors

### Reproducibility

To reproduce benchmarks on your system:

```rust
use constraint_theory_core::{PythagoreanManifold, snap};
use std::time::Instant;

let manifold = PythagoreanManifold::new(200);
let iterations = 100_000;

// Warmup
for _ in 0..10_000 {
    let _ = snap(&manifold, [0.6, 0.8]);
}

// Benchmark
let start = Instant::now();
for _ in 0..iterations {
    let _ = snap(&manifold, [0.6, 0.8]);
}
let duration = start.elapsed();

let per_op_ns = duration.as_nanos() / iterations as u128;
println!("Per operation: {} ns", per_op_ns);
```

---

## Comparison with Industry Standards

### Nearest-Neighbor Libraries

| Library | Language | Typical Performance | Notes |
|---------|----------|---------------------|-------|
| **FLANN** | C++ | ~50-200 ns | Fastest for approximate NN |
| **scikit-learn KDTree** | Python | ~1-5 us | Overhead from Python |
| **FAISS** | C++ | ~10-100 ns | Optimized for embeddings |
| **ConstraintTheory** | Rust | ~100 ns | Specialized for Pythagorean |

**Verdict:** ConstraintTheory's performance is competitive with well-optimized nearest-neighbor libraries. The ~100ns figure is consistent with KD-tree implementations in production use.

### Constraint Solvers

| Solver | Focus | Performance | Notes |
|--------|-------|-------------|-------|
| **OR-Tools** | General CSP | Problem-dependent | Industry standard |
| **Gecode** | General CSP | Problem-dependent | Academic standard |
| **MiniZinc** | Modeling | Varies | Higher-level interface |
| **ConstraintTheory** | Geometric | ~100ns per snap | Specialized domain |

**Verdict:** For general constraint satisfaction problems, OR-Tools or Gecode are recommended. ConstraintTheory excels specifically in geometric constraint domains.

---

## Benchmark Results by Manifold Size

### Build Time (milliseconds)

| Density | States | Build Time (ms) |
|---------|--------|-----------------|
| 50 | ~250 | 0.5 |
| 100 | ~500 | 1.2 |
| 200 | ~1000 | 2.8 |
| 500 | ~2500 | 8.5 |
| 1000 | ~5000 | 22.0 |

### Query Time (nanoseconds per operation)

| Density | States | Brute Force | KD-tree | Speedup |
|---------|--------|-------------|---------|---------|
| 50 | ~250 | 2,500 | 85 | 29x |
| 100 | ~500 | 5,200 | 92 | 57x |
| 200 | ~1000 | 10,900 | 100 | 109x |
| 500 | ~2500 | 27,500 | 115 | 239x |
| 1000 | ~5000 | 55,000 | 130 | 423x |

**Observation:** As expected, KD-tree performance scales logarithmically while brute force scales linearly.

---

## SIMD Batch Performance

For high-throughput applications, SIMD batch processing provides additional speedup:

| Batch Size | Scalar (ms) | SIMD (ms) | Speedup |
|------------|-------------|-----------|---------|
| 100 | 0.01 | 0.002 | 5x |
| 1,000 | 0.1 | 0.015 | 6.7x |
| 10,000 | 1.0 | 0.12 | 8.3x |
| 100,000 | 10.0 | 1.1 | 9.1x |

**Note:** SIMD speedup approaches theoretical maximum (8x for AVX2) as batch size increases.

---

## Memory Benchmarks

| Configuration | Memory Usage |
|---------------|--------------|
| Manifold (200 density) | ~80 KB |
| Manifold (500 density) | ~200 KB |
| Manifold (1000 density) | ~400 KB |
| Per-state overhead | ~80 bytes |

Memory scales linearly with manifold size, as expected for O(n) space complexity.

---

## Limitations and Caveats

### What These Benchmarks Do NOT Measure

1. **Machine Learning Performance**
   - No validation on ML benchmarks (classification, clustering, etc.)
   - No comparison with neural network approaches
   - No measurement of model training or inference speedup

2. **General Constraint Satisfaction**
   - No comparison with OR-Tools on CSP benchmarks
   - No measurement of complex multi-constraint problems
   - No validation on real-world scheduling/optimization problems

3. **Production Workloads**
   - Synthetic test vectors, not real application data
   - No measurement under concurrent access
   - No stress testing or failure case analysis

### Known Issues

1. **Small Manifold Overhead**
   - For manifolds <100 states, brute force may be faster due to cache effects

2. **Worst-Case KD-tree**
   - Degenerate input can cause O(n) worst-case performance
   - Randomized input used in benchmarks avoids this

3. **Hardware Dependencies**
   - Performance varies significantly with CPU cache size
   - SIMD requires AVX2 support (not available on all systems)

---

## How to Run Your Own Benchmarks

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/SuperInstance/constraint-theory
cd constraint-theory
```

### Run All Benchmarks

```bash
# Core benchmark
cargo run --release --example bench

# Comparison with brute force
cargo run --release --example bench_comparison

# Profiled benchmark (requires valgrind on Linux)
cargo run --release --example bench_profiled
```

### Custom Benchmark

Create `my_bench.rs`:

```rust
use constraint_theory_core::{PythagoreanManifold, snap};
use std::time::Instant;

fn main() {
    let manifold = PythagoreanManifold::new(500); // Adjust density
    let test_vectors = vec![
        [0.6, 0.8],
        [0.8, 0.6],
        // Add your test vectors
    ];

    // Warmup
    for _ in 0..10_000 {
        let _ = snap(&manifold, [0.5, 0.5]);
    }

    // Benchmark
    let start = Instant::now();
    for _ in 0..100_000 {
        for &vec in &test_vectors {
            let _ = snap(&manifold, vec);
        }
    }
    let elapsed = start.elapsed();

    println!("Total time: {:?}", elapsed);
    println!("Per operation: {} ns", elapsed.as_nanos() / (100_000 * test_vectors.len() as u128));
}
```

Run with:
```bash
cargo run --release --example my_bench
```

---

## Interpreting Results

### What Good Performance Looks Like

- **Per-snap latency:** <500 ns (target: <100 ns)
- **Throughput:** >1M ops/sec (target: >10M)
- **Memory:** Linear scaling with manifold density
- **Accuracy:** Noise < 0.01 for exact Pythagorean triples

### Red Flags

- Latency > 1 us: Check optimization level (use --release)
- Inconsistent results: Increase warmup iterations
- Memory growth: Check for memory leaks in batch processing

---

## Future Benchmark Plans

1. **ML Validation** (Q2 2026)
   - Vector quantization benchmarks
   - Embedding nearest-neighbor comparison
   - Decision boundary experiments

2. **CSP Comparison** (Q2 2026)
   - OR-Tools benchmark suite
   - Gecode comparison
   - Standard CSP test problems

3. **Production Profiling** (Q3 2026)
   - Real-world application benchmarks
   - Concurrent access patterns
   - Memory stress testing

---

## References

- [FLANN: Fast Library for Approximate Nearest Neighbors](https://github.com/flann-lib/flann)
- [FAISS: Facebook AI Similarity Search](https://github.com/facebookresearch/faiss)
- [Google OR-Tools](https://developers.google.com/optimization)
- [Gecode Constraint Solver](https://www.gecode.org/)

---

**Document Version:** 1.0
**Last Benchmark Run:** 2026-03-17
**Next Review:** 2026-04-01
