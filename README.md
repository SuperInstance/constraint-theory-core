# constraint-theory-core

Unified mathematical library for constraint theory — Eisenstein lattices, deadband funnels, Laman rigidity, distributed consensus, and holonomy verification. Five composable modules. 83 tests. Zero external dependencies.

## What It Does

Every distributed system has constraints: agents must agree on time, sensors must stay within tolerance, and the communication graph must carry enough information without waste. This package provides the mathematical primitives that make those constraints precise and provable.

The five modules compose into a unified architecture:

- **Lattice** quantizes continuous space onto the Eisenstein A₂ lattice — every point is within ρ = 1/√3 of a lattice point, guaranteed by geometry.
- **Temporal** wraps that quantization in a narrowing deadband funnel — ε(t) = ε₀ · e^(−λt) — so drift converges to zero or triggers an anomaly.
- **Rigidity** ensures the communication graph is minimally rigid (Laman) — exactly 2n−3 edges, every edge load-bearing, no waste.
- **Metronome** drives distributed consensus across Laman-connected agents — tick, agree, correct with optimal coupling α* = 2/(λ₂ + λₙ).
- **Holonomy** verifies cycle consistency in tiled constraint systems — detects and isolates faults in O(log N) via binary bisection.

## Architecture

```
                    ┌─────────────┐
                    │   Lattice   │
                    │  (Eisenstein │
                    │    A₂ snap)  │
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
       ┌──────▼──────┐    │    ┌───────▼───────┐
       │   Temporal   │    │    │   Holonomy    │
       │  (deadband   │    │    │  (cycle verify)│
       │   funnel)    │    │    └───────┬───────┘
       └──────┬──────┘    │            │
              │           │            │
       ┌──────▼───────────▼──┐  ┌─────▼─────────┐
       │     Metronome       │  │   Rigidity    │
       │  (distributed       │  │  (Laman graph) │
       │   consensus)        │  └───────┬───────┘
       └──────────┬──────────┘          │
                  │                     │
                  └──────┬──────────────┘
                         │
                  optimal coupling α*
```

**Compositions:**
- **Lattice + Temporal = bounded drift** — snap to lattice, then narrow the deadband over time.
- **Rigidity + Holonomy = zero-comm consensus** — Laman topology ensures minimal edges; holonomy verifies every cycle closes.
- **All five = full metronome** — agents snap to lattice, narrow via temporal, couple via Laman rigidity, agree via metronome, verify via holonomy.

## Installation

```bash
pip install constraint-theory-core
```

Requires Python ≥ 3.10. No external dependencies.

## Quick Start

### Lattice — Snap to Eisenstein Lattice

```python
from constraint_theory_core import snap, covering_radius, is_safe

# Any point in the plane snaps to the nearest A₂ lattice point
pt, error = snap(0.5, 0.3)
print(f"Snapped to ({pt.a}, {pt.b}), error = {error:.4f}")

# Error is always ≤ 1/√3 ≈ 0.577 (covering radius guarantee)
assert error <= covering_radius()

# If error < 1/2√3 ≈ 0.289, the snap is unambiguous
print(f"Safe: {is_safe(error)}")
```

### Temporal — Deadband Funnel

```python
from constraint_theory_core.temporal import TemporalAgent, FunnelPhase

agent = TemporalAgent(decay_rate=0.1)

for t in range(1, 50):
    result = agent.observe(0.01, 0.01, t=float(t))
    print(f"t={t:2d}  phase={result.phase.value:10s}  ε={result.deadband:.4f}  err={result.error:.4f}")
    if result.phase == FunnelPhase.ANOMALY:
        print("  ⚠ Anomaly detected!")
```

### Rigidity — Laman Graphs

```python
from constraint_theory_core import henneberg_construct, is_laman, algebraic_connectivity, optimal_coupling

# Build a minimally rigid graph for 9 agents
edges = henneberg_construct(9)
print(f"Edges: {len(edges)} (expected {2*9-3})")
assert is_laman(9, edges)

# Compute convergence properties
lam2 = algebraic_connectivity(edges, 9)
alpha = optimal_coupling(edges, 9)
print(f"λ₂ = {lam2:.4f}, α* = {alpha:.4f}")
```

### Metronome — Distributed Consensus

```python
from constraint_theory_core import Metronome
import math

# 9 agents on a Laman graph
edges = henneberg_construct(9)
agents = [
    Metronome(T=1.0, phi0=math.pi * i / 9, epsilon=0.5, delta=0.6,
              neighbors=_get_neighbors(edges, i),
              edges=edges, n_agents=9)
    for i in range(9)
]

# Run consensus
for tick in range(100):
    for agent in agents:
        agent.tick()
    phases = [a.phase for a in agents]
    for agent in agents:
        neighbor_phases = [phases[j] for j in agent.neighbors]
        agent.correct(neighbor_phases)

# Check convergence
print(f"All converged: {all(a.converged for a in agents)}")
```

### Holonomy — Cycle Verification

```python
from constraint_theory_core import verify_consistency, isolate_fault, fault_boundaries

# Each tile = (edges, direction_indices)
tiles = [
    ([(0,1), (1,2), (2,0)], [16, 16, 16]),  # consistent: 48 ≡ 0 mod 48
    ([(0,1), (1,3), (3,0)], [16, 16, 16]),  # consistent
    ([(2,3), (3,4), (4,2)], [1, 2, 3]),      # INCONSISTENT: 6 ≠ 0
]

print(f"All consistent: {verify_consistency(tiles)}")   # False
print(f"First fault at index: {isolate_fault(tiles)}")   # 2
print(f"All faults: {fault_boundaries(tiles)}")          # [2]
```

## Modules

| Module | What | Key Functions |
|--------|------|---------------|
| `lattice` | Eisenstein A₂ lattice | `snap`, `covering_radius`, `is_safe`, `A2Point`, `encode_dodecet`, `vector48_encode` |
| `temporal` | Deadband funnel | `TemporalAgent`, `FunnelPhase`, `FunnelResult` |
| `rigidity` | Laman graph topology | `is_laman`, `henneberg_construct`, `algebraic_connectivity`, `optimal_coupling` |
| `metronome` | Distributed consensus | `Metronome`, `MetronomeState` |
| `holonomy` | Cycle verification | `cycle_holonomy`, `verify_consistency`, `isolate_fault`, `fault_boundaries` |

## Equations

| Concept | Equation | Meaning |
|---------|----------|---------|
| A₂ covering radius | ρ = 1/√3 ≈ 0.577 | Max distance from any point to nearest lattice point |
| Safe threshold | ρ/2 ≈ 0.289 | Below this, snap is unambiguous |
| Eisenstein norm | ‖(a,b)‖² = a² − ab + b² | Squared norm on the lattice |
| Deadband decay | ε(t) = ε₀ · e^(−λt) | Funnel narrows exponentially |
| Laman condition | \|E\| = 2n − 3 | Exactly the right number of edges |
| Algebraic connectivity | λ₂ (2nd Laplacian eigenvalue) | Convergence rate of consensus |
| Optimal coupling | α* = 2/(λ₂ + λₙ) | Fastest convergence without oscillation |
| Holonomy | Σ directions mod 48 | 0 means cycle closes exactly |

## Testing

```bash
pip install -e ".[dev]"
pytest                          # 83 tests
pytest -v --tb=short            # verbose
```

## Related Repos

- [flux-tensor-midi](https://github.com/SuperInstance/flux-tensor-midi) — Musical constraint theory
- [plato](https://github.com/SuperInstance/plato) — PLATO tiling architecture
- [forgemaster](https://github.com/SuperInstance/forgemaster) — Constraint-theory specialist agent

## License

MIT
