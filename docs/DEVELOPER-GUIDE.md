# Developer Guide — constraint-theory-core

Architecture, extension patterns, testing philosophy, and integration points.

---

## Architecture: Why These 5 Modules

The package is structured around a single insight: **constraint theory has exactly five orthogonal dimensions** — space, time, topology, agreement, and verification. Each module handles one dimension, and their compositions are theorems, not features.

### Module Dependency Graph

```
lattice (zero deps — pure geometry)
  ├── temporal (depends on lattice.snap)
  ├── rigidity (zero deps — pure graph theory)
  └── holonomy (depends on lattice.vector48_encode/decode)

metronome (depends on lattice, temporal, rigidity)
```

The dependency graph is a DAG. No cycles. No circular imports.

### Why This Decomposition?

1. **Lattice** is the foundation. The A₂ covering radius ρ = 1/√3 is the fundamental error bound. Everything else builds on this guarantee.

2. **Temporal** adds time. The deadband funnel ε(t) = ε₀ · e^(−λt) narrows the lattice tolerance exponentially. This is independent of topology — a single agent can use it.

3. **Rigidity** is pure graph theory. It doesn't know about lattices or time. It answers: "Is this graph minimally rigid?" The Laman condition |E| = 2n−3 is purely combinatorial.

4. **Metronome** is the composition. It uses lattice (snap phases), temporal (narrow tolerance), and rigidity (compute α*). This is where the three dimensions unify.

5. **Holonomy** is the verifier. After consensus runs, holonomy checks that every cycle in the tiling closes. It's orthogonal to the metronome — you can verify any constraint system, not just metronome-consensus ones.

### What This Isn't

- Not a general-purpose math library. The modules are specialized for constraint theory.
- Not a distributed systems framework. No networking, no message passing. Pure math.
- Not tied to any particular application. The primitives are application-agnostic.

---

## Adding New Constraint Types

### Pattern: Module + Tests

Each module follows the same pattern:

1. **Pure functions first.** `snap()`, `is_laman()`, `cycle_holonomy()` — no state, no side effects.
2. **Stateful wrappers second.** `TemporalAgent`, `Metronome` — compose pure functions with mutable state.
3. **Frozen dataclasses for results.** `A2Point`, `FunnelResult`, `MetronomeState` — immutable snapshots.

To add a new constraint type (e.g., hyperbolic lattice):

```python
# constraint_theory_core/hyperbolic.py

"""Hyperbolic lattice constraints — Poincaré disk quantization."""

from dataclasses import dataclass
from typing import Tuple

@dataclass(frozen=True, slots=True)
class HyperbolicPoint:
    """Point on the Poincaré disk."""
    r: float
    theta: float

def snap_hyperbolic(x: float, y: float) -> Tuple[HyperbolicPoint, float]:
    """Snap a point to the nearest hyperbolic lattice point."""
    ...

def hyperbolic_covering_radius() -> float:
    """Covering radius of the hyperbolic lattice."""
    ...
```

Then:
1. Add the public API to `__init__.py`
2. Write tests in `tests/test_hyperbolic.py`
3. Follow the property-based testing patterns below

### Extending the Lattice to Higher Dimensions

The current lattice is 2D (A₂ in the complex plane). To extend to A₃ (3D) or Aₙ (nD):

1. **Keep the same interface.** `snap(x, y, z) → (A3Point, error)` should mirror `snap(x, y) → (A2Point, error)`.
2. **Derive the covering radius.** For A₃, ρ = √(3/2)/√3. For Aₙ, use the general formula.
3. **Keep A2Point unchanged.** Higher-dimensional points should be separate types, not subclasses. Composition > inheritance.
4. **Update the dodecet.** A₃ has more minimal vectors (the root system has more elements).

The key invariant: **covering radius must be derived from first principles, not hardcoded**.

---

## Testing Philosophy

### 83 Tests, Zero External Deps (Beyond pytest)

Tests use only `pytest` and `math`. No mocking, no fixtures, no parametrized combinatorial explosions. Every test is a direct assertion about mathematical properties.

### Property-Based Testing Patterns

The tests use manual property checks (not Hypothesis, though it's in dev deps). Key patterns:

#### 1. Roundtrip Properties

```python
def test_dodecet_roundtrip():
    for i in range(12):
        pt = decode_dodecet(i)
        assert encode_dodecet(pt) == i
```

If `decode(encode(x)) == x` for all x, both functions are correct by composition.

#### 2. Coverage Guarantees

```python
def test_covering_radius_guarantee():
    rng = random.Random(42)
    for _ in range(1000):
        x, y = rng.uniform(-10, 10), rng.uniform(-10, 10)
        _, err = snap(x, y)
        assert err <= covering_radius() + 1e-10
```

Not "passes for test inputs" — "passes for ALL inputs" (tested statistically).

#### 3. Invariant Preservation

```python
def test_laman_positive():
    for n in range(3, 15):
        edges = henneberg_construct(n)
        lam2 = algebraic_connectivity(edges, n)
        assert lam2 > 0  # Connected graphs always have λ₂ > 0
```

#### 4. Edge Cases

```python
def test_empty_cycle():
    assert is_consistent([])  # Vacuous truth

def test_single_vertex():
    assert is_laman(1, [])  # Trivially Laman
```

### Running Tests

```bash
pytest                              # All 83 tests
pytest tests/test_lattice.py        # One module
pytest -k "test_snap"               # One test
pytest --tb=short                   # Compact tracebacks
pytest -x                           # Stop at first failure
```

### Adding Tests for New Modules

Follow the existing pattern:

```python
"""Tests for constraint_theory_core.hyperbolic — Poincaré disk quantization."""

import pytest
from constraint_theory_core.hyperbolic import snap_hyperbolic, HyperbolicPoint

class TestHyperbolicSnap:
    def test_origin(self):
        pt, err = snap_hyperbolic(0.0, 0.0)
        assert err == pytest.approx(0.0, abs=1e-10)

    def test_covering_guarantee(self):
        import random
        rng = random.Random(42)
        for _ in range(1000):
            x = rng.uniform(-0.99, 0.99)
            y = rng.uniform(-0.99, 0.99)
            _, err = snap_hyperbolic(x, y)
            assert err <= hyperbolic_covering_radius() + 1e-10
```

---

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| `snap(x, y)` | O(1) | Checks 7 candidates (1 + 6 neighbors) |
| `is_laman(n, edges)` | O(2^n) for n≤15, O(n+e) for n>15 | Brute-force subset check vs pebble game |
| `henneberg_construct(n)` | O(n) | One vertex at a time |
| `algebraic_connectivity(edges, n)` | O(n² × 100) | 100 power iterations, O(n²) per iteration |
| `optimal_coupling(edges, n)` | O(n²) | Calls algebraic_connectivity once |
| `Metronome.tick()` | O(1) | Phase advance + temporal decay |
| `Metronome.correct(phases)` | O(k) | k = number of neighbors |
| `verify_consistency(tiles)` | O(N × L) | N tiles, L edges per tile |
| `isolate_fault(tiles)` | O(N × L × log N) | Binary search with full-scan verification |
| `fault_boundaries(tiles)` | O(N × L) | Single scan |

### Memory

- `A2Point`: 2 ints (frozen dataclass with slots)
- `FunnelResult`: 5 fields (frozen dataclass with slots)
- `MetronomeState`: 5 fields (frozen dataclass with slots)
- `TemporalAgent`: ~100 bytes (floats + optional float)
- `Metronome`: ~200 bytes + correction history (grows linearly, manual reset)

No allocations in hot paths except the correction list. For long runs, call `reset()` periodically.

### Scaling Limits

- `is_laman`: Brute-force is exponential. Switches to pebble game at n > 15. The current pebble game is simplified (checks connectivity only). For full generality, implement the complete Jacobs–Hendrickson algorithm.
- `algebraic_connectivity`: Power iteration is fine for n ≤ 1000. For larger graphs, use sparse matrix libraries (scipy, numpy).
- `henneberg_construct`: Linear, scales to millions of vertices.

---

## Integration with Other Projects

### flux-tensor-midi

Musical constraint theory. Uses the lattice module to quantize MIDI note positions onto the Eisenstein lattice, then applies temporal deadband to smooth expression.

```python
# Conceptual integration
from constraint_theory_core import snap, TemporalAgent

agent = TemporalAgent(decay_rate=2.0)  # Fast decay for musical timing
note_x, note_y = midi_to_eisenstein(pitch, velocity)
result = agent.observe(note_x, note_y, t=beat_time)
```

### PLATO Tiling Architecture

PLATO tiles the constraint surface. Each tile is a cycle. Holonomy verifies that every tile closes.

```python
from constraint_theory_core.holonomy import verify_consistency, isolate_fault

# Each PLATO tile → (edges, directions)
tiles = plato_generate_tiles(...)
if not verify_consistency(tiles):
    fault_idx = isolate_fault(tiles)
    print(f"Tile {fault_idx} is broken")
```

### Fleet Agent (Cocapn Fleet)

The metronome is the consensus engine for the Cocapn fleet of AI agents. Each agent runs a `Metronome` instance, communicates phases via the Laman graph, and converges to agreement.

```python
from constraint_theory_core import Metronome, henneberg_construct

# Agent i's metronome
edges = henneberg_construct(n_agents)
agent = Metronome(
    T=heartbeat_interval,
    neighbors=my_neighbors,
    edges=edges,
    n_agents=n_agents,
)
```

### Extending for New Integrations

The package is designed to be a **library, not a framework**. It doesn't impose structure on the caller. To integrate:

1. Import the primitives you need
2. Compose them in your application logic
3. Don't subclass the module classes — compose them instead

```python
# Good: composition
from constraint_theory_core import snap, TemporalAgent
class MySensor:
    def __init__(self):
        self.agent = TemporalAgent(decay_rate=0.5)

# Avoid: inheritance
class MySensor(TemporalAgent):  # Couples you to internal state
    ...
```
