# Cross-Ecosystem Integration Tests

**Version:** 1.0.1  
**Last Updated:** 2025-01-27

---

## Overview

This document describes integration tests for verifying consistency across the Constraint Theory ecosystem:
- **constraint-theory-core** (Rust)
- **constraint-theory-python** (Python bindings)
- **constraint-theory-web** (WASM/JavaScript)

---

## API Consistency Verification

### Core API Surface

All implementations must expose identical functionality:

| Function | Rust | Python | WASM |
|----------|------|--------|------|
| Create manifold | `PythagoreanManifold::new(density)` | `PythagoreanManifold(density=d)` | `new PythagoreanManifold(density)` |
| Single snap | `manifold.snap([x, y])` | `manifold.snap(x, y)` | `manifold.snap(x, y)` |
| Batch snap | `manifold.snap_batch_simd(&v)` | `manifold.snap_batch(v)` | `manifold.snap_batch(vectors)` |
| State count | `manifold.state_count()` | `manifold.state_count` | `manifold.state_count` |
| Max error | `manifold.max_angular_error()` | `manifold.max_angular_error` | `manifold.max_angular_error` |
| Validate input | `manifold.validate_input([x, y])` | `manifold.validate_input(x, y)` | `manifold.validate_input(x, y)` |

### Test Vectors

The following test vectors are used for cross-ecosystem verification:

```rust
/// Reference test cases that must produce identical results across all implementations
pub const CROSS_PLATFORM_TEST_VECTORS: &[([f32; 2], [f32; 2], f32)] = &[
    // (input, expected_snapped, max_noise)
    // 3-4-5 triangle
    ([0.6, 0.8], [0.6, 0.8], 0.001),
    // 4-3-5 triangle (swapped)
    ([0.8, 0.6], [0.8, 0.6], 0.001),
    // 7-24-25 triangle
    ([0.28, 0.96], [0.28, 0.96], 0.01),
    // Axis vectors
    ([1.0, 0.0], [1.0, 0.0], 0.001),
    ([0.0, 1.0], [0.0, 1.0], 0.001),
    // Negative quadrant
    ([-0.6, -0.8], [-0.6, -0.8], 0.001),
    // Approximate vectors
    ([0.577, 0.816], [0.6, 0.8], 0.05),
    ([0.707, 0.707], [0.6, 0.8], 0.1), // 45 degrees
];

/// Edge cases that must be handled consistently
pub const EDGE_CASE_VECTORS: &[([f32; 2], &'static str)] = &[
    ([0.0, 0.0], "zero_vector"),
    ([f32::NAN, 0.0], "nan_x"),
    ([0.0, f32::NAN], "nan_y"),
    ([f32::INFINITY, 0.0], "infinity_x"),
    ([f32::NEG_INFINITY, 0.0], "neg_infinity_x"),
];
```

---

## Rust Core Tests

### Cross-Platform Determinism Test

```rust
#[test]
fn test_cross_platform_determinism() {
    let manifold = PythagoreanManifold::new(200);
    
    for (input, expected, max_noise) in CROSS_PLATFORM_TEST_VECTORS {
        let (snapped, noise) = manifold.snap(*input);
        
        // Verify snapped vector
        assert!(
            (snapped[0] - expected[0]).abs() < 0.01,
            "X mismatch for input {:?}: got {:?}, expected {:?}",
            input, snapped, expected
        );
        assert!(
            (snapped[1] - expected[1]).abs() < 0.01,
            "Y mismatch for input {:?}: got {:?}, expected {:?}",
            input, snapped, expected
        );
        
        // Verify noise bound
        assert!(
            noise < *max_noise,
            "Noise {} exceeds max {} for input {:?}",
            noise, max_noise, input
        );
    }
}
```

### SIMD/Scalar Equivalence Test

```rust
#[test]
fn test_simd_scalar_equivalence() {
    let manifold = PythagoreanManifold::new(200);
    
    // Generate test vectors
    let vectors: Vec<[f32; 2]> = (0..1000)
        .map(|i| {
            let angle = i as f32 * 0.00628;
            [angle.cos(), angle.sin()]
        })
        .collect();
    
    // Scalar results
    let mut scalar_results = vec![([0.0, 0.0], 0.0f32); vectors.len()];
    manifold.snap_batch(&vectors, &mut scalar_results);
    
    // SIMD results
    let simd_results = manifold.snap_batch_simd(&vectors);
    
    // Compare
    for (i, (scalar, simd)) in scalar_results.iter().zip(simd_results.iter()).enumerate() {
        assert!(
            (scalar.0[0] - simd.0[0]).abs() < 0.001,
            "SIMD X mismatch at index {}", i
        );
        assert!(
            (scalar.0[1] - simd.0[1]).abs() < 0.001,
            "SIMD Y mismatch at index {}", i
        );
        assert!(
            (scalar.1 - simd.1).abs() < 0.001,
            "SIMD noise mismatch at index {}", i
        );
    }
}
```

---

## Python Binding Tests

### Test Script

```python
# test_cross_ecosystem.py
import constraint_theory as ct
import math

# Reference test vectors (must match Rust)
CROSS_PLATFORM_TEST_VECTORS = [
    # (input, expected_snapped, max_noise)
    ((0.6, 0.8), (0.6, 0.8), 0.001),
    ((0.8, 0.6), (0.8, 0.6), 0.001),
    ((0.28, 0.96), (0.28, 0.96), 0.01),
    ((1.0, 0.0), (1.0, 0.0), 0.001),
    ((0.0, 1.0), (0.0, 1.0), 0.001),
    ((-0.6, -0.8), (-0.6, -0.8), 0.001),
    ((0.577, 0.816), (0.6, 0.8), 0.05),
]

def test_python_rust_consistency():
    """Verify Python bindings produce same results as Rust core."""
    manifold = ct.PythagoreanManifold(density=200)
    
    for input_vec, expected, max_noise in CROSS_PLATFORM_TEST_VECTORS:
        snapped, noise = manifold.snap(input_vec[0], input_vec[1])
        
        # Verify snapped vector
        assert abs(snapped[0] - expected[0]) < 0.01, \
            f"X mismatch for {input_vec}: got {snapped}, expected {expected}"
        assert abs(snapped[1] - expected[1]) < 0.01, \
            f"Y mismatch for {input_vec}: got {snapped}, expected {expected}"
        
        # Verify noise bound
        assert noise < max_noise, \
            f"Noise {noise} exceeds max {max_noise} for {input_vec}"

def test_edge_cases():
    """Verify edge cases match Rust behavior."""
    manifold = ct.PythagoreanManifold(density=200)
    
    # Zero vector
    snapped, noise = manifold.snap(0.0, 0.0)
    assert snapped[0] == 1.0 and snapped[1] == 0.0  # Default
    assert noise == 0.0
    
    # NaN input - should raise or return error indicator
    try:
        snapped, noise = manifold.snap(float('nan'), 0.0)
        assert noise == 1.0  # Error indicator
    except ValueError:
        pass  # Acceptable to raise

def test_batch_consistency():
    """Verify batch operations match single operations."""
    manifold = ct.PythagoreanManifold(density=200)
    
    vectors = [[0.6, 0.8], [0.8, 0.6], [0.28, 0.96]]
    
    # Single snaps
    single_results = [manifold.snap(v[0], v[1]) for v in vectors]
    
    # Batch snap
    batch_results = manifold.snap_batch(vectors)
    
    for i, (single, batch) in enumerate(zip(single_results, batch_results)):
        assert abs(single[0][0] - batch[0][0]) < 0.001, f"Batch X mismatch at {i}"
        assert abs(single[0][1] - batch[0][1]) < 0.001, f"Batch Y mismatch at {i}"
        assert abs(single[1] - batch[1]) < 0.001, f"Batch noise mismatch at {i}"

if __name__ == "__main__":
    test_python_rust_consistency()
    test_edge_cases()
    test_batch_consistency()
    print("All cross-ecosystem tests passed!")
```

---

## WASM Integration Tests

### Test Script

```typescript
// test_cross_ecosystem.ts
import { PythagoreanManifold } from 'constraint-theory-web';

// Reference test vectors (must match Rust and Python)
const CROSS_PLATFORM_TEST_VECTORS: [number[], number[], number][] = [
    [[0.6, 0.8], [0.6, 0.8], 0.001],
    [[0.8, 0.6], [0.8, 0.6], 0.001],
    [[0.28, 0.96], [0.28, 0.96], 0.01],
    [[1.0, 0.0], [1.0, 0.0], 0.001],
    [[0.0, 1.0], [0.0, 1.0], 0.001],
];

describe('WASM Cross-Ecosystem Tests', () => {
    let manifold: PythagoreanManifold;

    beforeAll(() => {
        manifold = new PythagoreanManifold(200);
    });

    test('wasm_rust_consistency', () => {
        for (const [input, expected, maxNoise] of CROSS_PLATFORM_TEST_VECTORS) {
            const result = manifold.snap(input[0], input[1]);
            
            expect(Math.abs(result.snapped[0] - expected[0])).toBeLessThan(0.01);
            expect(Math.abs(result.snapped[1] - expected[1])).toBeLessThan(0.01);
            expect(result.noise).toBeLessThan(maxNoise);
        }
    });

    test('wasm_batch_consistency', () => {
        const vectors = new Float32Array([0.6, 0.8, 0.8, 0.6, 0.28, 0.96]);
        const results = manifold.snap_batch(vectors);
        
        // First result
        expect(Math.abs(results[0].snapped[0] - 0.6)).toBeLessThan(0.01);
        expect(Math.abs(results[0].snapped[1] - 0.8)).toBeLessThan(0.01);
        
        // Second result
        expect(Math.abs(results[1].snapped[0] - 0.8)).toBeLessThan(0.01);
        expect(Math.abs(results[1].snapped[1] - 0.6)).toBeLessThan(0.01);
    });

    test('wasm_state_count', () => {
        expect(manifold.state_count).toBeGreaterThan(0);
    });

    test('wasm_max_angular_error', () => {
        expect(manifold.max_angular_error).toBeGreaterThan(0);
        expect(manifold.max_angular_error).toBeLessThan(0.01);
    });
});
```

---

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/cross-ecosystem.yml
name: Cross-Ecosystem Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run Rust tests
        run: |
          cd constraint-theory-core
          cargo test
          cargo test test_cross_platform_determinism
          cargo test test_simd_scalar_equivalence

  python-tests:
    runs-on: ubuntu-latest
    needs: rust-tests
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      - name: Install Python bindings
        run: |
          cd constraint-theory-python
          pip install -e .
      - name: Run Python tests
        run: |
          cd constraint-theory-python
          python test_cross_ecosystem.py

  wasm-tests:
    runs-on: ubuntu-latest
    needs: rust-tests
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - name: Build WASM
        run: |
          cd constraint-theory-web
          wasm-pack build
      - name: Run WASM tests
        run: |
          cd constraint-theory-web
          npm install
          npm test

  consistency-check:
    runs-on: ubuntu-latest
    needs: [rust-tests, python-tests, wasm-tests]
    steps:
      - name: All ecosystem tests passed
        run: echo "Cross-ecosystem consistency verified"
```

---

## Version Compatibility Matrix

### API Version Requirements

| Core Version | Python Version | WASM Version | Notes |
|--------------|----------------|--------------|-------|
| 1.0.1 | 1.0.1 | 1.0.1 | Current stable |
| 1.0.0 | 1.0.0 | 1.0.0 | Initial release |

### Breaking Change Policy

1. **Major version (2.0.0)**: API-breaking changes
2. **Minor version (1.1.0)**: New features, deprecated features
3. **Patch version (1.0.1)**: Bug fixes only

### Deprecation Policy

- Deprecated features documented for 2 releases
- Breaking changes require ecosystem-wide coordination
- All implementations must pass cross-ecosystem tests before release

---

## Test Report Format

### Expected Output

```
=== Cross-Ecosystem Integration Test Report ===
Date: 2025-01-27
Commit: abc123

[Rust Core]
  ✓ test_cross_platform_determinism (23 vectors)
  ✓ test_simd_scalar_equivalence (1000 vectors)
  ✓ test_edge_cases (6 cases)
  Duration: 45ms

[Python Bindings]
  ✓ test_python_rust_consistency (7 vectors)
  ✓ test_edge_cases (3 cases)
  ✓ test_batch_consistency (3 vectors)
  Duration: 120ms

[WASM]
  ✓ wasm_rust_consistency (5 vectors)
  ✓ wasm_batch_consistency (3 vectors)
  ✓ wasm_state_count
  ✓ wasm_max_angular_error
  Duration: 85ms

[Summary]
  Total tests: 12
  Passed: 12
  Failed: 0
  Overall: PASS
```

---

## Troubleshooting Cross-Ecosystem Issues

### Common Issues

| Symptom | Likely Cause | Solution |
|---------|--------------|----------|
| Rust/Python mismatch | Different density | Ensure same density parameter |
| SIMD/Scalar mismatch | Tie-breaking order | Use scalar for consensus |
| WASM NaN handling | Different NaN behavior | Validate inputs before snapping |
| Batch size mismatch | Buffer allocation | Check array lengths |

### Debug Commands

```bash
# Run Rust tests with verbose output
cargo test -- --nocapture

# Run Python tests with debug
python -m pytest test_cross_ecosystem.py -v

# Run WASM tests with logging
WASM_LOG=debug npm test
```

---

**Document Version:** 1.0  
**Next Review:** 2025-04-01
