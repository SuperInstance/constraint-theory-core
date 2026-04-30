# Deployment Guide

**Version:** 1.0.1
**Last Updated:** 2025-01-27

---

## Overview

This document covers production deployment for `constraint-theory-core`, including CI/CD pipelines, release process, security audit checklist, and performance tuning.

---

## CI/CD Pipeline

### GitHub Actions Configuration

The complete CI/CD pipeline is defined in `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  release:
    types: [published]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
        include:
          - os: ubuntu-latest
            rust: nightly
            experimental: true

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-registry-

      - name: Cache cargo index
        uses: actions/cache@v4
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache target directory
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --verbose --all-features

      - name: Run tests (release)
        run: cargo test --release --verbose

      - name: Run clippy
        run: cargo clippy --all-targets -- -D warnings

      - name: Check formatting
        run: cargo fmt -- --check

  benchmark:
    name: Benchmark
    runs-on: ubuntu-latest
    needs: test
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo bench --no-run

      - name: Store benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/results.json
          fail-on-alert: true
          github-token: ${{ secrets.GITHUB_TOKEN }}

  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    needs: test

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --out Xml --all-features

      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
          fail_ci_if_error: true

  docs:
    name: Documentation
    runs-on: ubuntu-latest
    needs: test

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build documentation
        run: cargo doc --no-deps --all-features

      - name: Deploy to GitHub Pages
        if: github.event_name == 'push' && github.ref == 'refs/heads/main'
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./target/doc

  release:
    name: Release
    runs-on: ubuntu-latest
    needs: [test, benchmark, coverage]
    if: github.event_name == 'release'

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Publish to crates.io
        run: cargo publish --token ${{ secrets.CRATES_IO_TOKEN }}

      - name: Create release notes
        run: |
          echo "## Release ${{ github.event.release.tag_name }}" >> release_notes.md
          echo "" >> release_notes.md
          cargo changelog --latest >> release_notes.md
```

### Cross-Platform Test Matrix

| Platform | Architecture | Rust Version | Status |
|----------|--------------|--------------|--------|
| Ubuntu 22.04 | x86_64 | stable, beta, nightly | Required |
| macOS 13 | x86_64 | stable | Required |
| macOS 14 | ARM64 | stable | Required |
| Windows Server 2022 | x86_64 | stable | Required |

### SIMD Feature Testing

```yaml
  simd-test:
    name: SIMD Tests
    runs-on: ubuntu-latest
    needs: test

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Test with SIMD
        run: cargo test --features simd

      - name: Test without SIMD
        run: cargo test --no-default-features

      - name: Compare SIMD vs Scalar
        run: cargo test test_simd_scalar_equivalence --release
```

---

## Release Process

### Version Management

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR (X.0.0):** Breaking API changes
- **MINOR (0.X.0):** New features, backward compatible
- **PATCH (0.0.X):** Bug fixes, backward compatible

### Release Checklist

#### Pre-Release

- [ ] Update version in `Cargo.toml`
- [ ] Update version in `src/lib.rs` constants
- [ ] Update `CHANGELOG.md` with release notes
- [ ] Run full test suite: `cargo test --all-features`
- [ ] Run benchmarks: `cargo bench`
- [ ] Check documentation: `cargo doc`
- [ ] Run security audit: `cargo audit`
- [ ] Check for outdated dependencies: `cargo outdated`

#### Release

```bash
# 1. Create release branch
git checkout -b release/v1.0.2

# 2. Update version
# Edit Cargo.toml and src/lib.rs

# 3. Update changelog
# Edit CHANGELOG.md

# 4. Commit changes
git commit -am "chore: bump version to 1.0.2"

# 5. Push and create PR
git push origin release/v1.0.2
gh pr create --title "Release v1.0.2" --body "Release checklist complete"

# 6. After PR approval and merge:
git checkout main
git pull

# 7. Create tag
git tag -a v1.0.2 -m "Release v1.0.2"
git push origin v1.0.2

# 8. Create GitHub release
gh release create v1.0.2 --title "v1.0.2" --notes-file release_notes.md

# 9. CI automatically publishes to crates.io
```

### Changelog Format

```markdown
# Changelog

All notable changes to this project will be documented in this file.

## [1.0.2] - 2025-01-30

### Added
- New `validate_input()` method for consensus-critical code

### Changed
- Improved KD-tree cache locality

### Fixed
- Edge case with zero vectors now returns correct default

### Performance
- 15% improvement in batch processing throughput

## [1.0.1] - 2025-01-27

### Added
- Initial stable release
- KD-tree O(log n) lookup
- SIMD batch processing
- Cross-platform determinism

[1.0.2]: https://github.com/SuperInstance/constraint-theory-core/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/SuperInstance/constraint-theory-core/releases/tag/v1.0.1
```

---

## Security Audit Checklist

### Code Review

- [ ] **No unsafe in public API** - All public functions are safe
- [ ] **Documented unsafe blocks** - All `unsafe` blocks have safety comments
- [ ] **Input validation** - NaN, infinity, zero vectors handled
- [ ] **Bounds checking** - Array accesses are bounds-checked
- [ ] **Integer overflow** - Use checked/saturating math where needed

### Dependency Audit

```bash
# Check for known vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated

# Check for duplicate dependencies
cargo tree --duplicates

# Check license compatibility
cargo license
```

### Memory Safety

| Check | Status | Notes |
|-------|--------|-------|
| No use-after-free | Pass | Rust ownership system |
| No buffer overflow | Pass | Bounds-checked arrays |
| No null dereference | Pass | No null pointers |
| No data races | Pass | Immutable after creation |
| No uninitialized memory | Pass | All values initialized |

### Denial of Service Prevention

```rust
// Limit manifold size
const MAX_DENSITY: usize = 10_000;

impl PythagoreanManifold {
    pub fn new(density: usize) -> Self {
        let density = density.min(MAX_DENSITY);
        // ...
    }
}

// Limit batch size
const MAX_BATCH_SIZE: usize = 1_000_000;

impl PythagoreanManifold {
    pub fn snap_batch_simd(&self, vectors: &[[f32; 2]]) -> Vec<([f32; 2], f32)> {
        let vectors = &vectors[..vectors.len().min(MAX_BATCH_SIZE)];
        // ...
    }
}
```

### Fuzzing

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzzing
cargo fuzz run snap_fuzz -- -max_total_time=3600
```

```rust
// fuzz/fuzz_targets/snap_fuzz.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use constraint_theory_core::PythagoreanManifold;

fuzz_target!(|data: [f32; 2]| {
    let manifold = PythagoreanManifold::new(200);
    let (snapped, noise) = manifold.snap(data);

    // Verify invariants
    assert!(snapped[0].is_finite());
    assert!(snapped[1].is_finite());
    assert!(noise >= 0.0 && noise <= 2.0);

    // Verify unit norm
    let norm_sq = snapped[0] * snapped[0] + snapped[1] * snapped[1];
    assert!((norm_sq - 1.0).abs() < 0.001);
});
```

### Security Policy

See [SECURITY.md](../SECURITY.md) for:
- Vulnerability reporting process
- Supported versions
- Security update policy

---

## Performance Tuning Guide

### Density Selection

Choose manifold density based on precision requirements:

| Use Case | Density | States | Angular Error | Memory | Lookup Time |
|----------|---------|--------|---------------|--------|-------------|
| Animation | 50-100 | 250-500 | 0.7°-1.4° | 12-24 KB | ~80 ns |
| Games | 100-200 | 500-1000 | 0.36°-0.7° | 24-48 KB | ~100 ns |
| Robotics | 200-500 | 1000-2500 | 0.14°-0.36° | 48-120 KB | ~120 ns |
| ML/Scientific | 500-1000 | 2500-5000 | 0.07°-0.14° | 120-240 KB | ~150 ns |

### SIMD Optimization

```rust
// Check SIMD availability at runtime
#[cfg(target_arch = "x86_64")]
fn has_avx2() -> bool {
    is_x86_feature_detected!("avx2")
}

// Use appropriate path
if has_avx2() && vectors.len() >= 8 {
    // SIMD path (faster but may vary slightly)
    manifold.snap_batch_simd(&vectors);
} else {
    // Scalar path (deterministic)
    manifold.snap_batch(&vectors, &mut results);
}
```

### Memory Pre-allocation

```rust
// BAD: Allocation per iteration
for chunk in data.chunks(100) {
    let results = manifold.snap_batch_simd(chunk);
    // ...
}

// GOOD: Reuse buffer
let mut results = vec![([0.0, 0.0], 0.0f32); 100];
for chunk in data.chunks(100) {
    manifold.snap_batch_simd_into(chunk, &mut results);
    // ...
}
```

### Cache Optimization

```rust
// L1 cache (32 KB) fits ~4000 states
// L2 cache (256 KB) fits ~32000 states
// For large batches, use cache-friendly chunks

let chunk_size = 1000; // ~16 KB per chunk
for chunk in vectors.chunks(chunk_size) {
    manifold.snap_batch_simd(chunk);
}
```

### Parallel Processing

```rust
use rayon::prelude::*;

// Parallel batch processing
let manifold = Arc::new(PythagoreanManifold::new(200));

let results: Vec<_> = vectors
    .par_chunks(1000)
    .flat_map(|chunk| {
        manifold.snap_batch_simd(chunk)
    })
    .collect();
```

### Compiler Optimization

```toml
# Cargo.toml

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"

[profile.release-with-debug]
inherits = "release"
debug = true
debug-assertions = true
```

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Profile with perf (Linux)
perf record --call-graph=dwarf cargo test --release -- --ignored
perf report

# Generate flamegraph
cargo install flamegraph
cargo flamegraph --root -- test test_kdtree_performance

# Memory profiling with valgrind
valgrind --tool=massif target/release/deps/constraint_theory_core-*
```

---

## Monitoring in Production

### Key Metrics

```rust
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

### Alerting Thresholds

| Condition | Alert Level | Action |
|-----------|-------------|--------|
| p99 latency > 1 μs | Warning | Check for SIMD fallback |
| p99 latency > 10 μs | Critical | Investigate performance regression |
| Invalid input rate > 1% | Warning | Check input data quality |
| Memory usage > 500 KB | Warning | Consider reducing density |

### Logging

```rust
use log::{debug, info, trace};

fn debug_snap(manifold: &PythagoreanManifold, v: [f32; 2]) -> ([f32; 2], f32) {
    debug!("Input vector: {:?}", v);

    let norm = (v[0] * v[0] + v[1] * v[1]).sqrt();
    debug!("Input norm: {}", norm);

    let (snapped, noise) = manifold.snap(v);

    debug!("Snapped: {:?}", snapped);
    debug!("Noise: {}", noise);

    (snapped, noise)
}
```

---

## Troubleshooting

### Common Issues

| Issue | Symptom | Solution |
|-------|---------|----------|
| SIMD mismatch | Different results on different machines | Use scalar path for consensus |
| Memory growth | Increasing memory usage | Check for manifold recreation |
| Slow first call | High latency on first snap | Pre-warm manifold at startup |
| Cache misses | Poor batch performance | Use smaller chunk sizes |

### Debug Commands

```bash
# Check test coverage
cargo tarpaulin --out Html

# Check for undefined behavior
cargo miri test

# Check for memory leaks (requires valgrind)
valgrind --leak-check=full target/release/deps/constraint_theory_core-*

# Profile memory usage
cargo install cargo-profiler
cargo profiler callgrind --bin target/release/constraint_theory_core
```

---

## See Also

- [PRODUCTION_READINESS.md](PRODUCTION_READINESS.md) - Production guide
- [PERFORMANCE.md](PERFORMANCE.md) - Performance characteristics
- [TESTING.md](TESTING.md) - Testing methodology
- [SECURITY.md](../SECURITY.md) - Security policy

---

**Document Version:** 1.0
**Next Review:** 2025-04-01
