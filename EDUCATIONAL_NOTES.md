# Educational-clarity pass — README.md

Rewrite goal, per `STYLE_BRIEF.md`: a reader with no background in floating-point
arithmetic, Pythagorean triples, or spatial data structures should be able to
follow `README.md` start to finish and come away understanding *why this crate
exists and what its mechanism actually is* — taught in place, not simplified
away. A working engineer who already knows all of it should read the same
document without feeling condescended to. One document, both audiences, no
two-tier "simple version / real version" split.

This file explains the specific choices made and, importantly, the places where
verification against the source and the live crate forced a correction to a
claim the old README had been carrying.

## Goal of the rewrite

The old README opened with `0.6² + 0.8² = 1.0000000000000002` as its signature
"you've felt this" moment, then used the terms *Pythagorean triple*, *KD-tree*,
*rational arithmetic*, and *manifold* without building any of them up. It was
right in spirit and wrong in several specifics. The rewrite keeps the spirit
(relatable floating-point pain → exact-integer answer) and fixes the specifics.

## The headline correction: the original opening equation was false

The old README's opening line — and the load-bearing claim of its
"The Floating-Point Problem" section — was:

> `0.6² + 0.8² = 1.0000000000000002`

This is **not true** in Rust, in either precision:

```rust
let x = 0.6_f64;
let y = 0.8_f64;
let sq = x * x + y * y;
assert!(sq == 1.0);          // passes. bits == 0x3ff0000000000000
```

The reason is a well-known IEEE 754 coincidence: `0.6` rounds to a value
slightly *less* than 0.6 (`0.59999999999999997780...`), `0.8` rounds to a value
slightly *more* than 0.8 (`0.80000000000000004441...`), and when squared and
summed the two errors happen to cancel, landing back on exactly `1.0`. The value
`1.0000000000000002` is a *real* representable `f64` (it is the next double
after `1.0`), and it shows up in many drift scenarios — just not in this
specific equation. (The `STYLE_BRIEF.md` itself cites this equation as a model
"oh, THAT" moment; the brief's *principle* is right, its *example* is not.)

The task instruction was explicit: "don't take the README's own claims about
test counts on faith, verify them," and "`0.6² + 0.8² = 1.0000000000000002`
line lands as the 'oh, THAT' moment rather than an assumed-already-annoying
fact." Verified rather than assumed, the line is false, so it had to be
replaced — the `STYLE_BRIEF.md` rule "don't drop accuracy to make something
feel simple; find a better explanation instead" applies in the other direction
here too: a false headline is not made acceptable by being relatable.

**The replacement.** The rewrite leads with the same kind of buildup the task
asked for (a running total off by a fraction of a cent; a multiplayer game
where two consoles disagree about a position) and lands its "oh, THAT" moment on
a drift example that *is* reproducible and that *is* the actual disease this
crate treats — accumulation across many operations:

```rust
let mut vx = 3.0_f64 / 5.0;
let mut vy = 4.0_f64 / 5.0;
let (c, s) = (0.6_f64, 0.8_f64);
for _ in 0..100_000 {
    let nx = vx * c - vy * s;
    let ny = vx * s + vy * c;
    vx = nx; vy = ny;
}
println!("{:.16}", (vx * vx + vy * vy).sqrt());  // 1.0000000000023512
```

This runs (verified against the live crate's environment) and prints exactly
`1.0000000000023512`, with `mag == 1.0` evaluating to `false`. It is a stronger
example than the one it replaces because it is (a) true, (b) the literal use
case the crate is sold for (multiplayer / robotics / repeated integration), and
(c) honest about the real lesson — the disease is not "every float operation is
wrong," it is "you cannot predict which operations will cancel and which will
compound." That sentence appears verbatim in the new "Floating-Point Problem"
section. The `0.6`/`0.8`/`3-4-5` framing is preserved for the *solution* half,
because the triple `(3, 4, 5)` really is the crate's source of truth; only the
false single-equation claim was removed.

## Other source-level corrections (every claim below was verified against the live crate)

The headline was the largest error, but not the only one. Several other
specifics in the old README did not match the current source:

- **Test count: 184 → 262.** The old README stated "184 tests pass" in three
  places (Verify, Quality, Ecosystem). `cargo test` on HEAD reports four
  sections — 136 unit (`src/lib.rs`), 30 integration (`tests/integration_tests.rs`),
  54 module-coverage (`tests/module_coverage_tests.rs`), 42 doc-tests — for 262
  passing and 2 ignored. All three stale mentions are updated.

- **Lattice size: ~1000 → 40,384 at density 200.** `PythagoreanManifold::new(200).state_count()`
  is 40,384, not "~1000". The old README undercounted by ~40× across the board:
  density 50 → 2,494 (not ~250), density 200 → 40,384 (not ~1000), density 500 →
  252,829 (not ~2500). The density/states/memory table now uses the exact
  `state_count()` values and computes memory as `state_count × 8` bytes for the
  `Vec<[f32; 2]>`, with a note that the KD-tree adds O(N) node storage on top.

- **Memory at density 200: ~80 KB → ~315 KB.** Follows from the corrected
  state count (`40384 × 8 = 323072 bytes ≈ 315 KB` for the raw state vectors
  alone). The "~80 KB" figure appeared in Install, Core Types, and the density
  table; all updated.

- **Angular resolution at density 200: ~0.36° → ~0.009°.** Average angular gap
  is `360° / 40384 ≈ 0.0089°`. The "~0.36°" figure in Limitations was stale by
  the same ~40× factor as the state count.

- **`#![forbid(unsafe_code)]` is not present anywhere in the crate.** A grep for
  `forbid` across `src/` returns nothing. The actual lint posture is
  `#![deny(missing_docs)]` + `#![warn(clippy::all)]` at the crate root
  (`src/lib.rs:134`). `unsafe` exists in exactly one module — `src/simd.rs`,
  where it wraps AVX2 intrinsics behind the safe public `snap_batch_simd` —
  which is itself consistent with the old Quality table's row "unsafe: Only in
  SIMD intrinsics, behind safe wrappers." The new README states the real
  situation and drops the inaccurate `forbid` claim.

- **CI platforms: "Linux, macOS, Windows" → "Linux (stable + beta)."**
  `.github/workflows/ci.yml` runs a single `ubuntu-latest` job across `stable`
  and `beta` Rust. There is no macOS or Windows job. The Quality row is updated
  to match the workflow file.

- **The "Verify It Works" code example did not do what it claimed.** The old
  block asserted that `snap(&manifold, [0.577, 0.816])` returns `[0.6, 0.8]`. It
  does not — that input snaps to `[0.5773387, 0.8165048]`, a *different* exact
  triple (noise = 0), not the 3-4-5 point. The new example uses two nearby
  inputs, `[0.601, 0.799]` and `[0.599, 0.801]`, both of which *do* snap to
  `[0.6, 0.8]`, and the example's load-bearing line is now `assert_eq!(a, b)` —
  demonstrating the crate's actual value (two different float readings collapse
  to one integer-defined point) rather than the value the old example claimed
  (which, as verified above, plain floats already give for free via the
  cancellation coincidence).

- **Benchmark numbers: kept but explicitly attributed.** The old README stated
  the timing table (`~100 ns` single snap, `~74 ns/op` SIMD batch, etc.) as
  flat fact. The reference numbers in `docs/BENCHMARKS.md` are tagged
  "Last Updated: 2025-01-27, Version: 1.0.1" (the current crate is 2.2.0) and
  that doc itself repeats the wrong "~1000 states" premise. Fresh numbers could
  not be obtained within the session — `cargo bench` pulls in `criterion` and
  did not finish compiling in the available time. The rewrite therefore keeps
  the order-of-magnitude figures (they remain plausible — KD-tree depth scales
  as `log₂(40384) ≈ 16`, consistent with a sub-microsecond lookup) but labels
  them as recorded in `docs/BENCHMARKS.md` on the reference machine and points
  readers at `cargo bench` to verify on their own hardware. The lattice-size
  numbers next to them are exact and freshly verified, so the two tables draw a
  clear line between "checked at HEAD" and "recorded earlier, rerun to confirm."

## Choices that follow the `STYLE_BRIEF.md` directly

These are the pedagogical moves the brief asks for, applied to this crate:

- **Pythagorean triple defined before it is named.** The first time the phrase
  appears, it is glossed inline in the same clause: "a Pythagorean triple: three
  integers `a, b, c` with `a² + b² = c²`, like `3, 4, 5` (the one most people
  half-remember from school, because `3² + 4² = 9 + 16 = 25 = 5²`)." The
  mechanism paragraph later expands with Euclid's formula and three concrete
  `(m, n)` examples that the reader can verify on a napkin.

- **KD-tree defined in one sentence at first use.** "(a way of organizing points
  in space so that 'find the nearest one' visits only a handful of candidates
  instead of checking every single point — the difference between O(log N) and
  O(N) per lookup)." The capabilities table later restates the O(N log N) build
  / O(log N) query contract.

- **Every other piece of jargon gets the same inline treatment.** Holonomy,
  sheaf cohomology, Ricci flow, and Laman rigidity each get a one-clause gloss
  in the Capabilities table, because that table is where a newcomer would
  otherwise bounce off first.

- **Motivate before mechanizing.** The opener spends a paragraph on three felt
  experiences of drift (the cents column, the leaderboard tie, the multiplayer
  desync) before any code or any crate name does any work. The "what it's good
  for" section leads each bullet with the concrete problem (lockstep
  desync, replayability, reproducible training) before the code that solves it.

- **One concrete example per abstract claim.** The 3-4-5 triple is the example
  the whole document returns to — in the opener, in the mechanism paragraph, in
  the verified demo, and in the floating-point section — so the reader always
  has one handhold.

## What was not changed

No source code, no `Cargo.toml`, no other documentation file, no benchmark, no
test, no CI workflow, no link, no license, no citation, and no capability claim
was modified. Only `README.md` was rewritten. No new feature, capability, or
promise was invented. Every capability, type, and module name in the rewritten
document is verified against the source (`src/lib.rs` re-exports, the
compile-time `size_of` assertions in `src/tile.rs`, the public `snap`/`snap_batch`/
`snap_batch_simd` API in `src/manifold.rs`, and the module list in the crate
root). The Meta block, badges, ecosystem table structure, research links,
contributing instructions, citation, and license are preserved.

## Verification done during this pass

- `cargo build` — succeeds, zero runtime dependencies confirmed.
- `cargo test` — 262 passing, 2 ignored; section breakdown captured.
- Every code block in the new README was extracted into a scratch binary
  against the live crate and run: the rotation-drift block prints
  `1.0000000000023512` and `mag == 1.0` is `false`, as claimed; the "Verify it
  works" block's `assert_eq!(a, b)` passes and prints `[0.6, 0.8]`; the
  use-case snippets (`process_input`, `move_arm`, ML quantization) compile
  against the real `PythagoreanManifold` API; the floating-point-problem block's
  "this never runs" branch is confirmed unreachable.
- `manifold.state_count()` checked at densities 50, 200, 500.
- The `0.6² + 0.8²` claim checked in both `f32` and `f64`, with bit-pattern
  output, to confirm the headline-correction section above.
- Crates.io publication verified via `https://crates.io/api/v1/crates/constraint-theory-core`
  (version 2.2.0, published 2026-05-03 by SuperInstance).
- `.github/workflows/ci.yml` read to confirm the CI matrix is Linux-only.
- `src/lib.rs` lint attributes and `src/simd.rs` `unsafe` usage read to confirm
  the real safety story.
- `src/tile.rs` `const _: () = assert!(size_of::<...>())` lines read to confirm
  the 384 / 64 / 192 byte claims.

## Verification limits

`cargo bench` did not finish compiling within the session (it pulls in
`criterion`'s dependency tree, which is heavy). The timing numbers in the
Benchmarks table are therefore presented as recorded-in-`docs/BENCHMARKS.md`
order-of-magnitude figures rather than freshly measured ones, and the README
says so explicitly. Everything else in the rewrite is verified at HEAD.
