# Production Readiness Guide

**Version:** 1.0.1  
**Last Updated:** 2025-01-27

---

## Overview

This document covers production deployment considerations for `constraint-theory-core`, including security, memory safety, performance tuning, and debugging strategies.

---

## Security Considerations

### Input Validation

The library validates all inputs and handles edge cases gracefully:

```rust
use constraint_theory_core::{PythagoreanManifold, CTErr};

let manifold = PythagoreanManifold::new(200);

// NaN inputs are handled safely
let (snapped, noise) = manifold.snap([f32::NAN, 0.0]);
assert_eq!(noise, 1.0); // Error indicator

// Infinity inputs are handled safely
let (snapped, noise) = manifold.snap([f32::INFINITY, 0.0]);
assert_eq!(noise, 1.0); // Error indicator

// Zero vectors get a safe default
let (snapped, noise) = manifold.snap([0.0, 0.0]);
assert!(snapped[0].is_finite());
```

### Input Validation API

For consensus-critical systems, validate inputs explicitly:

```rust
let manifold = PythagoreanManifold::new(200);

// Recommended pattern for consensus systems
match manifold.validate_input([x, y]) {
    Ok(()) => {
        let (snapped, noise) = manifold.snap([x, y]);
        // Safe to use result
    }
    Err(reason) => {
        // Reject input before consensus
        log::warn!("Rejecting invalid input: {}", reason);
        return Err(ConsensusError::InvalidInput(reason));
    }
}
```

### Memory Safety Guarantees

#### No `unsafe` in Public API

All public APIs are safe Rust. Internal `unsafe` blocks are:
- Isolated to SIMD operations (`src/simd.rs`)
- Clearly documented with safety requirements
- Wrapped in safe public interfaces

#### Buffer Safety

```rust
// All batch operations have bounds checking
let manifold = PythagoreanManifold::new(200);
let vectors = vec![[0.6, 0.8]; 100];
let mut results = vec![([0.0, 0.0], 0.0f32); 100];

// Safe: Correct buffer size
manifold.snap_batch(&vectors, &mut results);

// Panic: Mismatched buffer sizes (debug builds)
// manifold.snap_batch(&vectors, &mut results[..50]);
```

#### Memory Layout

All data structures use explicit memory layouts:

```rust
// Tile is exactly 384 bytes, 64-byte aligned
assert_eq!(std::mem::size_of::<Tile>(), 384);
assert_eq!(std::mem::align_of::<Tile>(), 64);

// Origin is exactly 64 bytes, 64-byte aligned
assert_eq!(std::mem::size_of::<Origin>(), 64);

// ConstraintBlock is exactly 192 bytes
assert_eq!(std::mem::size_of::<ConstraintBlock>(), 192);
```

### Denial of Service Prevention

#### Manifold Size Limits

```rust
// Reasonable limits prevent memory exhaustion
let max_density = 10000;  // ~500KB memory
if density > max_density {
    return Err(CTErr::InvalidDimension);
}
```

#### Batch Size Recommendations

| Batch Size | Memory | Recommended Use |
|------------|--------|-----------------|
| < 1,000 | < 16 KB | Real-time |
| 1,000 - 100,000 | 16 KB - 1.6 MB | Batch processing |
| > 100,000 | > 1.6 MB | Consider streaming |

### Cryptographic Considerations

The snap operation is **deterministic** but **not cryptographic**:

- Results are reproducible across platforms (scalar path)
- SIMD paths may have platform-dependent tie-breaking
- Do NOT use for cryptographic hashing or key derivation

For cryptographic applications, use dedicated cryptographic libraries.

---

## Memory Safety Guarantees

### Rust Memory Safety

The library leverages Rust's memory safety guarantees:

1. **No Buffer Overflows**: All array accesses are bounds-checked
2. **No Use-After-Free**: Ownership system prevents dangling references
3. **No Data Races**: Thread-safe by design (all operations are immutable after creation)
4. **No Uninitialized Memory**: All structures are fully initialized

### Thread Safety

```rust
use std::sync::Arc;
use std::thread;

// Manifold is immutable after creation - safe to share
let manifold = Arc::new(PythagoreanManifold::new(200));

let handles: Vec<_> = (0..4)
    .map(|_| {
        let m = Arc::clone(&manifold);
        thread::spawn(move || {
            // Safe concurrent access
            m.snap([0.6, 0.8])
        })
    })
    .collect();

// No synchronization needed - read-only access
```

### SIMD Safety

SIMD operations require special attention:

```rust
// SIMD path is safe but may have platform differences
let results_simd = manifold.snap_batch_simd(&vectors);

// Scalar path is deterministic across platforms
let mut results_scalar = vec![([0.0, 0.0], 0.0f32); vectors.len()];
manifold.snap_batch(&vectors, &mut results_scalar);

// For consensus-critical code, use scalar path
```

### FFI Safety

When calling from other languages:

```rust
// C-compatible types for FFI
#[repr(C)]
pub struct SnapResultFFI {
    pub snapped_x: f32,
    pub snapped_y: f32,
    pub noise: f32,
}

#[no_mangle]
pub extern "C" fn ct_snap(
    manifold: *const PythagoreanManifold,
    x: f32,
    y: f32,
) -> SnapResultFFI {
    // ... safe wrapper implementation
}
```

---

## Performance Tuning Guide

### Density Selection

Choose manifold density based on precision requirements:

| Use Case | Recommended Density | Angular Error | Memory |
|----------|---------------------|---------------|--------|
| Animation | 50-100 | 0.7°-1.4° | 12-24 KB |
| Games | 100-200 | 0.36°-0.7° | 24-48 KB |
| Robotics | 200-500 | 0.14°-0.36° | 48-120 KB |
| ML/Scientific | 500-1000 | 0.07°-0.14° | 120-240 KB |

### SIMD Optimization

```rust
// Check SIMD availability
#[cfg(target_arch = "x86_64")]
if is_x86_feature_detected!("avx2") {
    // Use SIMD batch processing
    let results = manifold.snap_batch_simd(&vectors);
} else {
    // Fallback to scalar
    manifold.snap_batch(&vectors, &mut results);
}
```

### Memory Pre-allocation

```rust
// BAD: Allocate per iteration
for chunk in data.chunks(100) {
    let results = manifold.snap_batch_simd(chunk); // Allocates each time
}

// GOOD: Reuse buffer
let mut results = vec![([0.0, 0.0], 0.0f32); 100];
for chunk in data.chunks(100) {
    manifold.snap_batch_simd_into(chunk, &mut results);
    // Process results
}
```

### Cache Optimization

```rust
// L1 cache (32 KB) can hold ~4000 states
// L2 cache (256 KB) can hold ~32000 states

// For large batches, process in cache-friendly chunks
let chunk_size = 1000; // ~16 KB per chunk
for chunk in vectors.chunks(chunk_size) {
    manifold.snap_batch_simd_into(chunk, &mut results);
}
```

### Parallel Processing

```rust
use rayon::prelude::*;

// Parallel batch processing (each thread needs its own manifold clone)
let manifold = PythagoreanManifold::new(200);

let results: Vec<_> = vectors
    .par_chunks(1000)
    .flat_map(|chunk| {
        manifold.snap_batch_simd(chunk)
    })
    .collect();
```

### Performance Targets

| Metric | Target | Typical |
|--------|--------|---------|
| Single snap | < 1 μs | ~100 ns |
| Batch (1K) | < 100 μs | ~15 μs |
| Batch (100K) | < 10 ms | ~1.1 ms |
| Memory/state | < 100 bytes | ~80 bytes |

---

## Debugging Tips

### Common Issues and Solutions

#### SIMD Mismatch

```rust
// Symptom: SIMD and scalar produce different results
// Solution: Use scalar for consensus-critical code

let manifold = PythagoreanManifold::new(200);

// Check if SIMD is available
#[cfg(target_arch = "x86_64")]
println!("AVX2 available: {}", is_x86_feature_detected!("avx2"));

// Compare paths
let simd_results = manifold.snap_batch_simd(&vectors);
let mut scalar_results = vec![([0.0, 0.0], 0.0f32); vectors.len()];
manifold.snap_batch(&vectors, &mut scalar_results);

for (i, (simd, scalar)) in simd_results.iter().zip(scalar_results.iter()).enumerate() {
    if (simd.0[0] - scalar.0[0]).abs() > 0.001 {
        println!("Mismatch at index {}: SIMD={:?} Scalar={:?}", i, simd, scalar);
    }
}
```

#### NaN Propagation

```rust
// Symptom: Unexpected NaN in results
// Solution: Validate inputs

let manifold = PythagoreanManifold::new(200);

fn safe_snap(manifold: &PythagoreanManifold, v: [f32; 2]) -> Option<([f32; 2], f32)> {
    if !v[0].is_finite() || !v[1].is_finite() {
        return None;
    }
    let (snapped, noise) = manifold.snap(v);
    if noise >= 1.0 {
        return None; // Error indicator
    }
    Some((snapped, noise))
}
```

#### Performance Degradation

```rust
// Symptom: Slower than expected performance
// Diagnosis: Check these common causes

// 1. Creating manifold repeatedly (expensive!)
// BAD:
for v in vectors {
    let manifold = PythagoreanManifold::new(200); // O(n log n) each time!
    manifold.snap(v);
}

// GOOD:
let manifold = PythagoreanManifold::new(200); // Create once
for v in &vectors {
    manifold.snap(*v);
}

// 2. Not using SIMD for batch operations
// BAD:
for v in &vectors {
    manifold.snap(*v); // Scalar loop
}

// GOOD:
let results = manifold.snap_batch_simd(&vectors); // SIMD parallelism

// 3. Cache misses
// BAD: Random access pattern
let random_order: Vec<_> = vectors.choose_multiple(&mut rng, vectors.len());
for v in random_order {
    manifold.snap(*v);
}

// GOOD: Sequential access
for v in &vectors {
    manifold.snap(*v);
}
```

### Logging and Diagnostics

```rust
use log::{debug, info, trace};

fn debug_snap(manifold: &PythagoreanManifold, v: [f32; 2]) -> ([f32; 2], f32) {
    debug!("Input vector: {:?}", v);
    
    let norm = (v[0] * v[0] + v[1] * v[1]).sqrt();
    debug!("Input norm: {}", norm);
    
    if norm < 1e-10 {
        debug!("Zero vector detected, using default");
    }
    
    let (snapped, noise) = manifold.snap(v);
    
    debug!("Snapped: {:?}", snapped);
    debug!("Noise: {}", noise);
    
    (snapped, noise)
}
```

### Profiling

```bash
# Linux perf
perf record --call-graph=dwarf cargo test --release -- --ignored
perf report

# Flamegraph
cargo install flamegraph
cargo flamegraph --root -- test test_kdtree_performance

# Memory profiling with valgrind
valgrind --tool=massif target/release/deps/constraint_theory_core-*
```

### Debug Assertions

```rust
// Enable debug assertions in release for testing
// Add to Cargo.toml:
// [profile.release-with-debug]
// inherits = "release"
// debug-assertions = true

#[cfg(debug_assertions)]
fn verify_invariants(manifold: &PythagoreanManifold) {
    for state in manifold.states() {
        // All states should be normalized
        let norm = (state[0] * state[0] + state[1] * state[1]).sqrt();
        assert!((norm - 1.0).abs() < 0.001, "State not normalized: {:?}", state);
    }
}
```

---

## Production Checklist

### Before Deployment

- [ ] Choose appropriate manifold density for use case
- [ ] Enable SIMD feature for x86_64 deployments
- [ ] Implement input validation for consensus-critical paths
- [ ] Set up logging for debugging
- [ ] Run cross-platform compatibility tests
- [ ] Profile memory usage at expected scale
- [ ] Test edge cases (NaN, zero, infinity)
- [ ] Document expected performance characteristics

### Monitoring

```rust
// Key metrics to monitor
struct PerformanceMetrics {
    // Latency
    pub p50_snap_latency_ns: u64,
    pub p99_snap_latency_ns: u64,
    
    // Throughput
    pub snaps_per_second: f64,
    
    // Quality
    pub avg_noise: f32,
    pub max_noise: f32,
    
    // Errors
    pub invalid_input_count: u64,
    pub zero_vector_count: u64,
}
```

### Alerts

| Condition | Alert Level | Action |
|-----------|-------------|--------|
| p99 latency > 10 μs | Warning | Check for SIMD fallback |
| p99 latency > 100 μs | Critical | Investigate performance regression |
| Invalid input rate > 1% | Warning | Check input data quality |
| Memory usage > 1 MB | Warning | Consider reducing density |

---

## Version Compatibility

| Core Version | Minimum Rust | Tested Rust Versions |
|--------------|--------------|---------------------|
| 1.0.x | 1.75 | 1.75, 1.76, 1.77, nightly |

---

**Document Version:** 1.0  
**Next Review:** 2025-04-01
