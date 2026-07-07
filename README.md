# Constraint Theory Core


## Meta

**Domain:** constraint-theory
**Depends on:** —
**Depended by:** —
**Implements:** Deterministic manifold snapping — maps continuous 2D vectors to exact Pythagorean coordinates on the unit circle, so a direction is defined by integers, not floats.
**Related:** —

[![GitHub stars](https://img.shields.io/github/stars/SuperInstance/constraint-theory-core?style=social)](https://github.com/SuperInstance/constraint-theory-core)
[![CI](https://github.com/SuperInstance/constraint-theory-core/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/constraint-theory-core/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/constraint-theory-core.svg)](https://crates.io/crates/constraint-theory-core)
[![docs.rs](https://docs.rs/constraint-theory-core/badge.svg)](https://docs.rs/constraint-theory-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**`cargo add constraint-theory-core`** · [Live Demos](https://constraint-theory-web.pages.dev) · [Docs](https://docs.rs/constraint-theory-core)

---

## The problem this crate solves

You have felt floating-point drift even if you've never called it that.

A spreadsheet that totals a column of cents and shows a phantom `0.00000001` in
the balance cell. A leaderboard where two players tie for first because their
browsers each computed `0.1 + 0.2` and got `0.30000000000000004` instead of
`0.3`. A multiplayer game where, after a few minutes of identical inputs, the
two consoles show an object in measurably different positions — not because
either machine is wrong, but because each took a slightly different path
through the same sequence of multiplies and adds, and the tiny rounding
differences compounded across a hundred thousand frames.

That last one is the real disease, and here is what it looks like when you
isolate it:

```rust
// A unit direction vector, the kind every game loop and every robot
// controller normalizes on every tick.
let mut vx = 3.0_f64 / 5.0;
let mut vy = 4.0_f64 / 5.0;            // starts at length exactly 1.0

// Rotate it by another clean 3-4-5 direction, 100,000 times —
// i.e. ~16 minutes of frames at 60 fps, with identical inputs.
let (c, s) = (0.6_f64, 0.8_f64);
for _ in 0..100_000 {
    let nx = vx * c - vy * s;
    let ny = vx * s + vy * c;
    vx = nx; vy = ny;
}

let mag = (vx * vx + vy * vy).sqrt();
println!("{:.16}", mag);               // -> 1.0000000000023512
println!("{}", mag == 1.0);            // -> false
```

Run that loop on two machines and they will disagree about where the object
is. Not because of a bug in anyone's code — because of IEEE 754, the standard
that defines how computers do floating-point arithmetic. Every individual
operation there is the *correctly rounded* nearest representable float; "a
little off in one operation" and "a little off, compounded a hundred thousand
times" are different stories. By the end, `==` no longer means what you wanted
it to mean, and two deterministic simulations have quietly diverged.

**Constraint Theory's answer:** stop representing the direction as a float at
all. Represent it as three whole numbers — a **Pythagorean triple**: three
integers `a`, `b`, `c` with `a² + b² = c²`, like `3, 4, 5` (the one most
people half-remember from school, because `3² + 4² = 9 + 16 = 25 = 5²`).
Normalize the triple to `(a/c, b/c)` — for `(3, 4, 5)` that's `(0.6, 0.8)` —
and you have a point on the unit circle *whose definition is the integers
`3, 4, 5`*, not a float computation. Integer arithmetic is exact on every
machine, in every frame, forever.

There are infinitely many such triples. This crate precomputes all the useful
ones up to a resolution you choose, indexes them in a **KD-tree** (a way of
organizing points in space so that "find the nearest one" visits only a
handful of candidates instead of checking every single point — the difference
between O(log N) and O(N) per lookup), and snaps any input vector to its
nearest exact neighbor. Snap once, and the result is reproducible on every
machine, every platform, every compile.

That is the whole idea. The rest of this document is the details.

## What this crate does, concretely

Constraint Theory replaces floating-point approximation with exact rational
arithmetic. It maps any continuous 2D direction vector to the nearest
Pythagorean rational point — `(3/5, 4/5)`, `(5/13, 12/13)`, `(8/17, 15/17)`,
and so on — where `a² + b² = c²` holds in **integers**, not floats.

This is not arbitrary-precision arithmetic (which chases perfect accuracy at
unbounded cost). It is a *finite lattice* of exact points: a fixed, enumerable
set chosen once at build time, indexed for fast lookup. You trade a sliver of
angular resolution (you can only land on a point that exists in the lattice)
for an absolute guarantee that the point you land on is defined by whole
numbers and is therefore identical everywhere.

### The mechanism in one paragraph

A **Pythagorean triple** is three whole numbers `(a, b, c)` with `a² + b² = c²`.
**Euclid's formula** — a 2300-year-old result — generates every one of them
from two integers `m > n > 0`: `(m²−n², 2mn, m²+n²)`. Plugging in `m=2, n=1`
gives `(3, 4, 5)`; `m=3, n=2` gives `(5, 12, 13)`; `m=4, n=1` gives `(15, 8, 17)`.
Normalize each to `(a/c, b/c)` and you get an exact rational point on the unit
circle. There are infinitely many triples, but only finitely many below any
chosen `m`. This crate enumerates them once, reflects them across all four
quadrants of the circle, hands the result to a **KD-tree**, and answers "what's
the nearest exact point to this input?" in O(log N). The snap is deterministic,
platform-independent, and exact.

## Install

```bash
cargo add constraint-theory-core
```

Zero runtime dependencies (`[dependencies]` in `Cargo.toml` is empty; `rand`
and `criterion` are dev-dependencies used only by tests and benchmarks). The
crate is `#![deny(missing_docs)]` at the root — every public item is
documented. The only `unsafe` code in the crate lives in `src/simd.rs`, where
it wraps AVX2 intrinsics behind a safe public API (`snap_batch_simd`); every
other module is plain safe Rust. Published on [crates.io](https://crates.io/crates/constraint-theory-core)
(current version 2.2.0), with API docs rendered at [docs.rs](https://docs.rs/constraint-theory-core).

## Verify it works

```rust
use constraint_theory_core::{PythagoreanManifold, snap};

fn main() {
    // density 200 -> 40,384 exact states across the unit circle, ~315 KB.
    let manifold = PythagoreanManifold::new(200);

    // Two slightly different readings of "the same" direction — sensor noise,
    // or the same joystick sampled on two different machines.
    let (a, _) = snap(&manifold, [0.601, 0.799]);
    let (b, _) = snap(&manifold, [0.599, 0.801]);

    assert_eq!(a, b);                       // both resolved to ONE lattice point
    println!("[{}, {}]", a[0], a[1]);       // [0.6, 0.8]

    // That point is (3/5, 4/5) — the 3-4-5 triple. Its source of truth is the
    // integers (3, 4, 5), so a² + b² = c² holds exactly in integer arithmetic:
    println!("{}", 3 * 3 + 4 * 4 == 5 * 5); // true
}
```

The whole pitch is in the `assert_eq!(a, b)`: two different float inputs,
arriving from two different places, collapse to the *same* exact point because
that point's identity is the integer triple `(3, 4, 5)`, not a float
computation that could round differently elsewhere.

```bash
cargo test    # 262 tests pass (2 ignored); see the Quality section for the breakdown
```

## What it's good for

**Deterministic multiplayer / lockstep simulation** — the same joystick input
produces the same direction on every client, every frame, with no
floating-point divergence. This is the use case the rotation-drift example
above was about: without exact snapping, two clients running the "same"
simulation drift apart over time and desync.

```rust
fn process_input(&mut self, joystick: [f32; 2]) {
    let (direction, noise) = self.manifold.snap(joystick);
    self.velocity = [direction[0] * SPEED, direction[1] * SPEED];
    // `direction` is identical on every machine. Always.
}
```

**Robotics** — repeatability. The same target direction produces the same
motion, down to the bit, across power cycles and across different controller
hardware. Useful when two arms must agree on where "forward" is, or when a
telemetry log must replay identically.

```rust
fn move_arm(&mut self, target: [f32; 2]) {
    let (direction, noise) = self.manifold.snap(target);
    if noise > 0.01 {
        log::warn!("High quantization noise — target sits between lattice points");
    }
}
```

**ML direction quantization** — reproducible training and inference. The same
embedding projected to 2D produces the same snapped token, every run, so a
model's behavior is bit-for-bit reproducible across machines and restarts.

```rust
let (quantized, _) = manifold.snap(project_to_2d(&embedding));
// Integer arithmetic from here. Deterministic across runs.
```

## Capabilities

| Feature | What it does |
|---|---|
| **Exact snapping** | Map any 2D vector to the nearest Pythagorean rational. O(log N) per lookup. |
| **KD-tree spatial index** | O(N log N) to build once, O(log N) to query, with deterministic tie-breaking so every machine agrees. |
| **SIMD batch** | Process 8 × f32 at once on AVX2, auto-detected at runtime. Note: the SIMD path may tie-break differently from the scalar path — use the scalar `snap_batch` for consensus-critical code. |
| **Holonomy verification** | *Holonomy* is what happens when you carry a direction around a closed loop of constraints and check whether it comes back to itself; if it doesn't, the constraint network is internally inconsistent. `HolonomyChecker` verifies this incrementally. |
| **Sheaf cohomology** | Counts the connected components (H₀) and independent cycles (H₁) of a constraint graph in O(1) via the Euler characteristic — a topological way to detect emergent structure. |
| **Ricci flow** | A curvature-smoothing process borrowed from differential geometry; here used to evolve a manifold toward a flatter, more consistent state. Convergence is guaranteed. |
| **Laman rigidity** | Laman's theorem characterizes which bar-and-joint frameworks hold their shape (a "rigid" graph vs. a "floppy" one); `FastPercolation` checks this in O(V²). |
| **Quantization** | `PythagoreanQuantizer` with four modes: Ternary `{−1, 0, 1}` (BitNet-style, for LLM weights), Polar (exact unit norm, for embeddings), Turbo (near-optimal distortion, for vector DBs), and Hybrid (auto-select). |
| **Hidden dimensions** | Lift a 2D point into Rⁿ⁺ᵏ to encode constraints exactly; the number of extra dimensions needed for precision ε is `k = ⌈log₂(1/ε)⌉`. |

## Benchmarks

The numbers below are order-of-magnitude figures recorded in `docs/BENCHMARKS.md`
on the reference machine; rerun `cargo bench` on your own hardware for current
numbers (the benchmark suite is a criterion bench at `benches/core_benchmarks.rs`).

| Operation | Time |
|---|---|
| Single snap (density 200) | ~100 ns |
| SIMD batch (1000 vectors) | ~74 ns/op |
| Manifold build (density 200, one-time) | ~2.8 ms |
| Ternary quantize (128-D) | ~50 ns |
| Holonomy check (cycle length 16) | ~300 ns |

The lattice size scales with density. These counts are exact at HEAD (verified
by `manifold.state_count()`, not estimated):

| Density | Exact states | Raw state vector memory |
|---|---|---|
| 50 | 2,494 | ~19 KB |
| 200 | 40,384 | ~315 KB |
| 500 | 252,829 | ~1.93 MB |

(Raw vector memory is `state_count × 8` bytes for the `Vec<[f32; 2]>`. The
KD-tree adds its own O(N) node storage on top.)

Full benchmarks and methodology: [docs/BENCHMARKS.md](./docs/BENCHMARKS.md).

## Core types

| Type | What it is |
|---|---|
| `PythagoreanManifold` | Precomputed exact points + KD-tree. The main entry point. ~315 KB at density 200. |
| `Tile` | 384-byte fundamental unit (verified at compile time by a `const _: () = assert!(size_of::<Tile>() == 384)`). Holds an origin, a payload, and constraints. |
| `ConstraintBlock` | 192 bytes of holonomy matrix, Ricci curvature, percolation probability, and gluing map (also compile-time size-checked). |
| `PythagoreanQuantizer` | The unified quantizer across all four modes (Ternary / Polar / Turbo / Hybrid). |
| `HolonomyChecker` | Incremental cycle verification: `apply()` a rotation, `check_partial()` mid-loop, `check_closed()` when the loop completes. |
| `FastPercolation` | Union-find with path compression, used to test Laman rigidity percolation. |
| `FastCohomology` | O(1) H₀ and H₁ via the Euler characteristic. |

## Architecture

```
src/
├── manifold.rs         PythagoreanManifold — triple generation, snapping
├── kdtree.rs           KD-tree — O(log N) spatial index
├── tile.rs             Tile (384 B), Origin (64 B), ConstraintBlock (192 B)
├── holonomy.rs         Holonomy verification, HolonomyChecker
├── cohomology.rs       H₀, H¹ via Euler characteristic
├── curvature.rs        RicciFlow — curvature evolution
├── percolation.rs      FastPercolation — Laman rigidity
├── gauge.rs            GaugeConnection — parallel transport
├── quantizer.rs        PythagoreanQuantizer — 4 modes
├── hidden_dimensions.rs  Precision encoding, k = ⌈log₂(1/ε)⌉
├── cache.rs            CachedLattice — thread-safe global cache
├── simd.rs             AVX2 batch snapping (x86_64, runtime-detected)
└── dcs.rs              Physical constants (Laman threshold, Ricci multiplier)
```

The crate also ships constraint-satisfaction tooling (modules `csp`, `ac3`,
`backtracking`, `cdcl`, `puzzle`, `sudoku`) used by the broader Constraint
Theory ecosystem — AC-3 arc consistency, MRV/LCV/FC backtracking, and a
1-UIP CDCL solver. These are out of scope for the snapping use case above but
are documented at [docs.rs](https://docs.rs/constraint-theory-core).

## The floating-point problem, in one place

```rust
// The bug every deterministic-simulation engineer has shipped:
let mut vx = 3.0_f64 / 5.0;
let mut vy = 4.0_f64 / 5.0;
let (c, s) = (0.6_f64, 0.8_f64);
for _ in 0..100_000 {
    let nx = vx * c - vy * s;
    let ny = vx * s + vy * c;
    vx = nx; vy = ny;
}
if (vx * vx + vy * vy).sqrt() == 1.0 { /* this never runs */ }
```

`0.6` and `0.8` cannot be represented exactly in IEEE 754 — `0.6` is stored as
a value very slightly *less* than 0.6, and `0.8` as a value very slightly
*more* than 0.8. A *single* `0.6² + 0.8²` happens to round back to exactly
`1.0` (the two errors cancel, which is a well-known coincidence, not a
guarantee). The disease isn't in any single operation — it's that you cannot
predict *which* operations will cancel and which will compound, so across a
long simulation the magnitude walks away from 1.0 and `==` silently stops
meaning what you wanted it to mean.

Constraint Theory's answer is not "make the floats more precise." It is to
make the direction's *identity* an integer triple. The lattice point `(0.6,
0.8)` is stored as a float for convenience, but its source of truth is the
triple `(3, 4, 5)`, and `3² + 4² = 5² = 25` holds exactly in integer
arithmetic on every processor ever manufactured. The float is a label; the
triple is the fact.

## Limitations (honest ones)

| Limitation | Why | Impact |
|---|---|---|
| **2D only** | Pythagorean triples live on the unit circle, which is a 2D object. | Not directly suitable for 3D; you'd project to 2D first or use a per-axis scheme. |
| **Finite lattice** | You can only land on a point the manifold precomputed. | At density 200 the ~40k states give ~0.009° average angular resolution. Higher density = finer resolution, more memory (see table above). |
| **Quantization noise** | Snapping moves the input to the nearest lattice point; the input is rarely *on* a point. | Always check the returned `noise` value; if it's high, your input sat between lattice points and the snap may not mean what you want. |
| **SIMD / scalar tie-breaking can differ** | The AVX2 batch path may break ties differently from the scalar path. | For consensus-critical code, use the scalar `snap_batch`, not `snap_batch_simd`. |

## Quality

| Metric | Value |
|---|---|
| Tests | 262 passing, 2 ignored (136 unit + 30 integration + 54 module-coverage + 42 doc-tests, as reported by `cargo test`'s four sections — rerun to verify) |
| Dependencies | Zero runtime dependencies |
| `unsafe` | Confined to `src/simd.rs` (AVX2 intrinsics), behind a safe public wrapper. Every other module is safe Rust. |
| Lints | `#![deny(missing_docs)]`, `#![warn(clippy::all)]` at the crate root |
| CI | GitHub Actions on Linux (stable + beta Rust); runs build, test, clippy, fmt check on every push and PR to `main` |
| Property tests | Edge cases covered in `src/edge_case_tests.rs` |

## Research

- [arXiv:2503.15847](https://arxiv.org/abs/2503.15847) — Constraint Theory: Deterministic Manifold Snapping via Pythagorean Geometry
- [Mathematical Foundations (45 pages)](https://github.com/SuperInstance/constraint-theory-research/blob/main/MATHEMATICAL_FOUNDATIONS_DEEP_DIVE.md)
- [Theoretical Guarantees](https://github.com/SuperInstance/constraint-theory-research/blob/main/guides/THEORETICAL_GUARANTEES.md)
- [Proofs and errata → constraint-theory-math](https://github.com/SuperInstance/constraint-theory-math)

## The ecosystem

| Repo | What it is |
|---|---|
| **[constraint-theory-core](https://github.com/SuperInstance/constraint-theory-core)** | This crate. Rust, zero deps, 262 tests. |
| **[constraint-theory-python](https://github.com/SuperInstance/constraint-theory-python)** | Python bindings. NumPy + PyTorch. |
| **[constraint-theory-web](https://github.com/SuperInstance/constraint-theory-web)** | 50 interactive demos. |
| **[constraint-theory-math](https://github.com/SuperInstance/constraint-theory-math)** | Proofs, sheaf cohomology, errata. |
| **[holonomy-consensus](https://github.com/SuperInstance/holonomy-consensus)** | Zero-holonomy consensus for distributed systems. |
| **[fleet-coordinate](https://github.com/SuperInstance/fleet-coordinate)** | Fleet coordination using Eisenstein spatial hashing. |
| **[constraint-theory-research](https://github.com/SuperInstance/constraint-theory-research)** | Papers, open problems, formal proofs. |

## Contributing

[Good First Issues](https://github.com/SuperInstance/constraint-theory-core/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) · [CONTRIBUTING.md](CONTRIBUTING.md)

```bash
rustup component add clippy rustfmt
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Citation

```bibtex
@software{constraint_theory,
  title={Constraint Theory: Deterministic Manifold Snapping via Pythagorean Geometry},
  author={SuperInstance},
  year={2025},
  url={https://github.com/SuperInstance/constraint-theory-core},
  version={2.2.0}
}
```

## License

MIT
