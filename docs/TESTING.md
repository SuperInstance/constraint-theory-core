# Testing Methodology

**Version:** 1.0.1  
**Last Updated:** 2025-01-17

---

## Overview

This document describes the testing methodology for `constraint-theory-core`, including:

- Unit testing strategy
- Integration testing approach
- Edge case coverage
- Performance testing methodology
- SIMD correctness verification

---

## Test Organization

```
src/
├── lib.rs              # Core module tests
├── manifold.rs         # Manifold tests
├── simd.rs             # SIMD tests
├── kdtree.rs           # KD-tree tests
├── curvature.rs        # Curvature tests
├── cohomology.rs       # Cohomology tests
├── gauge.rs            # Gauge tests
├── percolation.rs      # Percolation tests
├── tile.rs             # Tile tests
└── edge_case_tests.rs  # Edge case coverage

examples/
├── bench.rs            # Benchmark examples
├── bench_comparison.rs # Comparison benchmarks
└── bench_profiled.rs   # Profiled benchmarks
```

---

## Running Tests

### All Tests

```bash
# Run all unit and integration tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run in release mode (for performance tests)
cargo test --release
```

### Specific Tests

```bash
# Test specific module
cargo test manifold

# Test specific test
cargo test test_snap_exact_triple

# Run ignored tests (performance benchmarks)
cargo test --release -- --ignored
```

### Test Categories

```bash
# Unit tests only
cargo test --lib

# Documentation tests
cargo test --doc

# All tests including examples
cargo test --all-targets
```

---

## Unit Testing Strategy

### Core Principles

1. **Deterministic Tests**: All tests must be deterministic
2. **Fast Execution**: Unit tests run in < 1ms each
3. **Isolated**: No shared state between tests
4. **Self-Documenting**: Test names describe the scenario

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_<operation>_<scenario>() {
        // Arrange
        let manifold = PythagoreanManifold::new(200);
        let input = [0.6, 0.8];

        // Act
        let (snapped, noise) = manifold.snap(input);

        // Assert
        assert!(noise < 0.001);
        assert!((snapped[0] - 0.6).abs() < 0.01);
    }
}
```

### Test Categories

| Category | Purpose | Example |
|----------|---------|---------|
| Correctness | Verify mathematical accuracy | `test_snap_exact_triple` |
| Boundary | Test edge conditions | `test_snap_zero_vector` |
| Invariant | Check preserved properties | `test_snap_preserves_normalization` |
| Roundtrip | Verify reversibility | N/A (one-way operation) |
| Integration | Test module interactions | `test_kdtree_correctness` |

---

## Edge Case Testing

### Edge Cases Covered

The `edge_case_tests.rs` module covers:

#### Input Edge Cases

```rust
// Zero vector
let (snapped, noise) = manifold.snap([0.0, 0.0]);
assert_eq!(snapped, [1.0, 0.0]); // Default fallback

// Very small values
let (snapped, _) = manifold.snap([1e-10, 1e-10]);

// Very large values
let (snapped, _) = manifold.snap([1e10, 1e10]);

// NaN handling
// NaN inputs are normalized or rejected

// Infinity handling
// Infinite inputs are handled gracefully
```

#### Boundary Conditions

```rust
// Unit circle boundaries
test_snap_unit_x();    // [1.0, 0.0]
test_snap_unit_y();    // [0.0, 1.0]
test_snap_neg_unit_x(); // [-1.0, 0.0]
test_snap_neg_unit_y(); // [0.0, -1.0]

// Density boundaries
test_manifold_empty();    // density = 0
test_manifold_minimum();  // density = 1
test_manifold_large();    // density = 1000
```

#### Numerical Stability

```rust
// Near-zero denominators
test_division_stability();

// Precision preservation
test_precision_maintenance();

// Accumulated error bounds
test_error_bounds();
```

---

## SIMD Correctness Testing

### SIMD vs Scalar Verification

All SIMD implementations are verified against scalar equivalents:

```rust
#[test]
fn test_simd_vs_scalar() {
    let manifold = PythagoreanManifold::new(200);
    let vectors = generate_test_vectors(1000);

    let mut scalar_results = vec![([0.0, 0.0], 0.0f32); vectors.len()];
    let mut simd_results = vec![([0.0, 0.0], 0.0f32); vectors.len()];

    manifold.snap_batch(&vectors, &mut scalar_results);
    manifold.snap_batch_simd_into(&vectors, &mut simd_results);

    for (i, (scalar, simd)) in scalar_results.iter().zip(simd_results.iter()).enumerate() {
        assert!(
            (scalar.0[0] - simd.0[0]).abs() < 0.001,
            "X mismatch at index {}", i
        );
        assert!(
            (scalar.0[1] - simd.0[1]).abs() < 0.001,
            "Y mismatch at index {}", i
        );
        assert!(
            (scalar.1 - simd.1).abs() < 0.001,
            "Noise mismatch at index {}", i
        );
    }
}
```

### SIMD Edge Cases

```rust
// Batch size not divisible by SIMD width
test_simd_remainder_handling();

// Empty batch
test_simd_empty_batch();

// Single element batch
test_simd_single_element();

// Non-aligned memory
test_simd_unaligned_memory();
```

---

## KD-Tree Testing

### Correctness Verification

```rust
#[test]
fn test_kdtree_correctness() {
    let manifold = PythagoreanManifold::new(200);

    // For each test vector, verify KD-tree matches linear search
    for test_vector in TEST_VECTORS {
        let (kd_snapped, kd_noise) = manifold.snap(test_vector);

        // Brute force search
        let bf_result = brute_force_search(&manifold, test_vector);

        assert_eq!(kd_snapped, bf_result.snapped);
        assert!((kd_noise - bf_result.noise).abs() < 0.001);
    }
}
```

### Performance Testing

```rust
#[test]
#[ignore] // Run with --ignored flag
fn test_kdtree_performance() {
    let manifold = PythagoreanManifold::new(500);
    let iterations = 100_000;

    // Warmup
    for _ in 0..1000 {
        let _ = manifold.snap([0.6, 0.8]);
    }

    // Benchmark
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = manifold.snap([0.6, 0.8]);
    }
    let elapsed = start.elapsed();

    let per_op_ns = elapsed.as_nanos() / iterations as u128;
    assert!(per_op_ns < 1000, "KD-tree too slow: {} ns/op", per_op_ns);
}
```

---

## Integration Testing

### Module Integration

```rust
#[test]
fn test_full_pipeline() {
    // Create manifold
    let manifold = PythagoreanManifold::new(200);

    // Create tiles from snapped vectors
    let vectors = vec![[0.6, 0.8], [0.8, 0.6]];
    let snapped: Vec<_> = vectors.iter()
        .map(|v| manifold.snap(*v).0)
        .collect();

    // Verify tiles
    for snapped_vec in snapped {
        let tile = Tile::from_vector(snapped_vec);
        assert!(tile.is_valid());
    }
}
```

### Cross-Platform Testing

Tests verify consistent behavior across:

- Linux x86_64
- macOS x86_64 (Intel)
- macOS ARM64 (Apple Silicon)
- Windows x86_64

---

## Performance Testing

### Benchmark Methodology

1. **Warmup**: 10,000 iterations to stabilize CPU frequency
2. **Measurement**: 100,000+ operations over multiple runs
3. **Environment**: Release mode, isolated CPU core
4. **Metrics**: Mean, P50, P95, P99, max

### Performance Test Commands

```bash
# Run performance tests
cargo test --release -- --ignored test_kdtree_performance

# Run with criterion (if configured)
cargo bench

# Profile with perf (Linux)
perf record cargo test --release -- --ignored
perf report
```

### Performance Assertions

```rust
// Target: < 100ns per snap operation
assert!(per_op_ns < 1000);

// Target: > 10M ops/sec throughput
assert!(ops_per_sec > 10_000_000);

// Target: SIMD speedup > 5x
assert!(simd_speedup > 5.0);
```

---

## Test Coverage

### Coverage Goals

| Component | Target Coverage | Current |
|-----------|-----------------|---------|
| Core API | 100% | ~95% |
| Edge Cases | 100% | ~90% |
| SIMD Paths | 100% | ~85% |
| Error Paths | 100% | ~80% |

### Running Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

---

## Continuous Integration

### CI Test Matrix

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, nightly]
    include:
      - rust: stable
        features: default
      - rust: nightly
        features: simd
```

### CI Test Steps

1. **Format Check**: `cargo fmt --check`
2. **Lint**: `cargo clippy -- -D warnings`
3. **Unit Tests**: `cargo test`
4. **Doc Tests**: `cargo test --doc`
5. **Examples**: `cargo build --examples`

---

## Test Data Generation

### Deterministic Test Vectors

```rust
fn generate_test_vectors(count: usize) -> Vec<[f32; 2]> {
    (0..count)
        .map(|i| {
            let angle = (i as f32) * 0.001 * std::f32::consts::TAU;
            [angle.cos(), angle.sin()]
        })
        .collect()
}
```

### Known Pythagorean Triples

```rust
const KNOWN_TRIPLES: &[[f32; 2]] = &[
    [0.6, 0.8],   // 3-4-5
    [0.8, 0.6],   // 4-3-5
    [0.28, 0.96], // 7-24-25
    [0.35, 0.93666], // 20-21-29
];
```

---

## Debugging Failed Tests

### Verbose Output

```bash
# Show print statements
cargo test -- --nocapture

# Show test execution order
cargo test -- --test-threads=1

# Show backtraces
RUST_BACKTRACE=1 cargo test
```

### Common Issues

| Symptom | Likely Cause | Solution |
|---------|--------------|----------|
| SIMD mismatch | CPU feature | Check `is_avx2_available()` |
| Floating point drift | FPU precision | Use `approx` crate |
| Race condition | Shared state | Use thread-local manifold |
| Timeout | Infinite loop | Check KD-tree termination |

---

## Adding New Tests

### Test Naming Convention

```rust
// Format: test_<module>_<scenario>_<expected_result>
#[test]
fn test_manifold_snap_exact_triple_returns_zero_noise() { }

#[test]
fn test_manifold_snap_zero_vector_returns_default() { }
```

### Test Template

```rust
#[test]
fn test_<feature>() {
    // Arrange
    let input = ...;
    let expected = ...;

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected);
    
    // Optional: Additional checks
    assert!(result.is_valid());
}
```

---

## Test Maintenance

### Regular Tasks

- **Weekly**: Review and update edge cases
- **Monthly**: Run full performance regression suite
- **Per Release**: Update coverage reports
- **Quarterly**: Review and clean up ignored tests

### Test Deprecation

Tests are deprecated (marked `#[ignore]`) when:
- Feature is deprecated
- Test is redundant with newer tests
- Test is flaky and not fixable

---

**Document Version:** 1.0  
**Next Review:** 2025-04-01
