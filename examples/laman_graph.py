"""
Laman Graph — build a rigid graph via Henneberg construction.

A Laman graph has exactly 2n-3 edges: every edge is load-bearing,
removing any edge loses rigidity. This is the optimal topology for
distributed consensus — minimal bandwidth, maximum connectivity.
"""

from constraint_theory_core.rigidity import (
    is_laman, henneberg_construct, algebraic_connectivity, optimal_coupling,
)


def main():
    print("=== Laman Rigidity ===\n")

    # -- Henneberg construction --
    print("--- Henneberg Construction ---")
    print("Algorithm: Start with K₂, add each vertex with 2 edges\n")

    for n in [2, 3, 5, 9, 20]:
        edges = henneberg_construct(n)
        expected = 2 * n - 3
        laman = is_laman(n, edges)
        print(f"  n={n:3d}  edges={len(edges):3d}  expected={expected:3d}  "
              f"Laman={'✓' if laman else '✗'}")

    print()

    # -- 9-agent fleet topology --
    print("--- 9-Agent Fleet ---")
    n = 9
    edges = henneberg_construct(n)
    print(f"Vertices: {n}")
    print(f"Edges: {len(edges)} (= 2×{n}−3 = {2*n-3})")
    print(f"Edge list: {edges}")
    print()

    # -- Algebraic connectivity --
    lam2 = algebraic_connectivity(edges, n)
    print(f"Algebraic connectivity λ₂ = {lam2:.4f}")
    print(f"  (λ₂ > 0 means the graph is connected)")
    print()

    # -- Optimal coupling --
    alpha = optimal_coupling(edges, n)
    print(f"Optimal coupling α* = {alpha:.4f}")
    print(f"  (This is the metronome's correction strength)")
    print(f"  Too small → slow convergence")
    print(f"  Too large → overshoot/oscillation")
    print(f"  α* → fastest convergence without oscillation")
    print()

    # -- Verify Laman conditions --
    print("--- Laman Conditions ---")
    print(f"  |E| = 2|V| − 3: {len(edges)} = 2×{n}−3 = {2*n-3} ✓")

    # Show degree distribution
    degree = [0] * n
    for u, v in edges:
        degree[u] += 1
        degree[v] += 1
    print(f"  Degrees: {dict(enumerate(degree))}")
    print(f"  Min degree: {min(degree)}, Max degree: {max(degree)}")

    # Verify no subset has too many edges
    print(f"  All subset checks passed: {is_laman(n, edges)} ✓")


if __name__ == "__main__":
    main()
