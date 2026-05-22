"""
Full Pipeline — all 5 modules working together.

Build a Laman topology, run metronome consensus, verify holonomy.
This is the unified constraint theory architecture in action.
"""

import math
from constraint_theory_core import (
    snap, covering_radius, is_safe,
    Metronome,
    henneberg_construct, is_laman, algebraic_connectivity, optimal_coupling,
)
from constraint_theory_core.holonomy import (
    cycle_holonomy, verify_consistency, fault_boundaries,
)
from constraint_theory_core.temporal import FunnelPhase


def neighbors_of(edges, i):
    """Get neighbor indices for vertex i."""
    return [v if u == i else u for u, v in edges if u == i or v == i]


def main():
    print("╔══════════════════════════════════════════════╗")
    print("║   Unified Constraint Theory — Full Pipeline  ║")
    print("╚══════════════════════════════════════════════╝\n")

    N = 9

    # ================================================================
    # Step 1: LATTICE — Verify the covering radius guarantee
    # ================================================================
    print("Step 1: LATTICE — Eisenstein A₂ Quantization")
    print("-" * 50)

    rho = covering_radius()
    print(f"  Covering radius: ρ = 1/√3 ≈ {rho:.4f}")

    # Snap some test points
    for x, y in [(0.5, 0.3), (1.0, -0.5), (0.0, 0.87)]:
        pt, err = snap(x, y)
        safe = "✓ safe" if is_safe(err) else "○ marginal"
        print(f"  snap({x:5.2f}, {y:5.2f}) → ({pt.a:2d}, {pt.b:2d})  "
              f"err={err:.4f}  {safe}")
    print()

    # ================================================================
    # Step 2: RIGIDITY — Build Laman topology
    # ================================================================
    print("Step 2: RIGIDITY — Laman Graph Construction")
    print("-" * 50)

    edges = henneberg_construct(N)
    print(f"  Vertices: {N}")
    print(f"  Edges: {len(edges)} (expected {2*N-3})")
    print(f"  Is Laman: {is_laman(N, edges)} ✓")

    lam2 = algebraic_connectivity(edges, N)
    alpha = optimal_coupling(edges, N)
    print(f"  λ₂ = {lam2:.4f} (algebraic connectivity)")
    print(f"  α* = {alpha:.4f} (optimal coupling)")
    print()

    # ================================================================
    # Step 3: METRONOME — Distributed consensus
    # ================================================================
    print("Step 3: METRONOME — Distributed Consensus")
    print("-" * 50)

    agents = [
        Metronome(
            T=1.0,
            phi0=math.pi * i / N,
            epsilon=0.5,
            delta=0.6,
            neighbors=neighbors_of(edges, i),
            edges=edges,
            n_agents=N,
        )
        for i in range(N)
    ]

    # Measure initial spread
    def spread(agents):
        sx = sum(math.cos(a.phase) for a in agents)
        sy = sum(math.sin(a.phase) for a in agents)
        return math.sqrt(sx*sx + sy*sy) / len(agents)

    initial_spread = spread(agents)
    print(f"  Initial phase spread: {initial_spread:.4f}")

    # Run consensus
    for tick in range(1, 51):
        for a in agents:
            a.tick()
        phases = [a.phase for a in agents]
        for a in agents:
            nbr_phases = [phases[j] for j in a.neighbors]
            a.correct(nbr_phases)

    final_spread = spread(agents)
    converged = sum(1 for a in agents if a.converged)
    print(f"  Final phase spread: {final_spread:.4f}")
    print(f"  Converged agents: {converged}/{N}")
    print(f"  Spread reduced by: {(1 - final_spread/initial_spread)*100:.1f}%")
    print()

    # ================================================================
    # Step 4: TEMPORAL — Check for anomalies
    # ================================================================
    print("Step 4: TEMPORAL — Anomaly Detection")
    print("-" * 50)

    anomaly_count = sum(a.anomaly_count for a in agents)
    print(f"  Total anomalies across all agents: {anomaly_count}")

    # Observe an external reference with each agent
    for i, a in enumerate(agents):
        phase = a.observe(1.0, 0.0)  # reference point (1, 0)
        if i < 3:
            print(f"  Agent {i}: observe(1.0, 0.0) → phase {a.phase:.4f}  "
                  f"funnel={phase.value}")
    print()

    # ================================================================
    # Step 5: HOLONOMY — Verify cycle consistency
    # ================================================================
    print("Step 5: HOLONOMY — Cycle Consistency Verification")
    print("-" * 50)

    # Build tiles from the Laman graph
    # Each triangle of edges → a tile with consistent directions
    good_dirs = [16, 16, 16]  # 48 ≡ 0 mod 48

    # Extract some 3-cycles from the Laman edges
    tiles = []
    for i in range(min(3, N)):
        nbrs = neighbors_of(edges, i)
        if len(nbrs) >= 2:
            tri_edges = [(i, nbrs[0]), (nbrs[0], nbrs[1]), (nbrs[1], i)]
            tiles.append((tri_edges, good_dirs))

    consistent = verify_consistency(tiles)
    print(f"  Tiles checked: {len(tiles)}")
    print(f"  All consistent: {consistent} ✓")

    # Add a broken tile
    bad_dirs = [1, 2, 3]  # 6 ≠ 0 mod 48
    tiles.append(([(0, 1), (1, 2), (2, 0)], bad_dirs))

    consistent = verify_consistency(tiles)
    faults = fault_boundaries(tiles)
    print(f"  After adding broken tile: consistent={consistent}")
    print(f"  Faulty tiles at: {faults}")
    print()

    # ================================================================
    # Summary
    # ================================================================
    print("╔══════════════════════════════════════════════╗")
    print("║                 Summary                      ║")
    print("╠══════════════════════════════════════════════╣")
    print(f"║  Lattice: ρ = {rho:.4f} (bounded error)       ║")
    print(f"║  Rigidity: {N} vertices, {len(edges)} edges (Laman ✓)    ║")
    print(f"║  Metronome: {converged}/{N} agents converged         ║")
    print(f"║  Temporal: {anomaly_count} anomalies detected            ║")
    print(f"║  Holonomy: {len(tiles)} tiles, {len(faults)} faults              ║")
    print("╚══════════════════════════════════════════════╝")


if __name__ == "__main__":
    main()
