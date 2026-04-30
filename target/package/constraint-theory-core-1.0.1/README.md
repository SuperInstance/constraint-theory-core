# Constraint Theory

A Rust library that snaps continuous vectors to exact Pythagorean coordinates via O(log n) KD-tree lookup.

[![CI](https://github.com/SuperInstance/constraint-theory-core/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/constraint-theory-core/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/constraint-theory-core.svg)](https://crates.io/crates/constraint-theory-core)
[![docs.rs](https://docs.rs/constraint-theory-core/badge.svg)](https://docs.rs/constraint-theory-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## What It Does

Constraint Theory builds a discrete manifold of [Pythagorean triples](https://en.wikipedia.org/wiki/Pythagorean_triple) (integer-ratio points on the unit circle), indexes them in a KD-tree, and provides a "snap" operator that maps any continuous 2D vector to its nearest exact geometric state.

**Key property:** The output is always an exact rational coordinate — no floating-point drift. The constraint predicate `a² + b² = c²` is satisfied by construction, not validated after the fact.

---

## Quick Start

```rust
use constraint_theory_core::{PythagoreanManifold, snap};

// Build manifold: 200 density → ~1000 Pythagorean states
let manifold = PythagoreanManifold::new(200);

// Snap a continuous vector to its nearest exact state
let (snapped, noise) = snap(&manifold, [0.6, 0.8]);

// (0.6, 0.8) = (3/5, 4/5) — an exact Pythagorean triple
assert!(noise < 0.001);
```

### Batch Processing (SIMD)

```rust
use constraint_theory_core::PythagoreanManifold;

let manifold = PythagoreanManifold::new(200);
let vectors = vec![[0.6, 0.8], [0.8, 0.6], [0.1, 0.99]];
let results = manifold.snap_batch_simd(&vectors);

for (snapped, noise) in results {
    println!("({:.4}, {:.4}) noise={:.6}", snapped[0], snapped[1], noise);
}
```

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
constraint-theory-core = "0.1"
```

---

## Performance

| Operation | Time | Complexity |
|-----------|------|------------|
| Manifold build | ~50 μs | O(n log n) |
| Single snap (KD-tree) | ~100 ns | O(log n) |
| Batch snap (SIMD) | ~74 ns/op | O(log n) |

---

## Limitations

- **2D only** — Higher dimensions are an open research problem
- **~1000 states** at default density — finite resolution
- **Research-grade** — Core algorithms work but not battle-tested in production

---

## Ecosystem

- **[constraint-theory-python](https://github.com/SuperInstance/constraint-theory-python)** — Python bindings via PyO3
- **[constraint-theory-web](https://github.com/SuperInstance/constraint-theory-web)** — Interactive demos and visualizations
- **[constraint-theory-research](https://github.com/SuperInstance/constraint-theory-research)** — Mathematical foundations and papers

---

## Documentation

- **[Tutorial](docs/TUTORIAL.md)** — Step-by-step guide
- **[Benchmarks](docs/BENCHMARKS.md)** — Performance methodology
- **[API Docs](https://docs.rs/constraint-theory-core)** — Full API reference

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for build instructions, code style, and PR process.

Areas where contributions are especially valuable:
- Higher-dimensional generalizations (3D Pythagorean quadruples, nD)
- GPU implementations (CUDA, WebGPU)
- Real-world use cases and benchmarks

---

## Citation

```bibtex
@software{constraint_theory,
  title={Constraint Theory: Deterministic Manifold Snapping via Pythagorean Geometry},
  author={SuperInstance},
  year={2025},
  url={https://github.com/SuperInstance/constraint-theory-core},
  version={0.1.0}
}
```

## License

MIT — see [LICENSE](LICENSE).
