"""
Metronome Consensus — 9 agents converging via Laman-coupled correction.

Each agent has a phase φ ∈ [0, 2π). They tick forward, check agreement
with neighbors, and correct toward consensus using optimal coupling α*.
"""

import math
from constraint_theory_core import Metronome, henneberg_construct


def neighbors_of(edges, i):
    """Get neighbor indices for vertex i."""
    return [v if u == i else u for u, v in edges if u == i or v == i]


def circular_spread(phases):
    """Measure how spread out phases are (0 = aligned, π = maximally spread)."""
    sx = sum(math.cos(p) for p in phases)
    sy = sum(math.sin(p) for p in phases)
    return math.sqrt(sx*sx + sy*sy) / len(phases)


def main():
    print("=== Metronome Consensus ===\n")

    N = 9
    edges = henneberg_construct(N)

    print(f"Topology: {N} agents, {len(edges)} Laman edges")
    print(f"Edges: {edges}\n")

    # Create agents with spread-out phases
    agents = [
        Metronome(
            T=1.0,
            phi0=math.pi * i / N,  # evenly spread around circle
            epsilon=0.5,
            delta=0.6,
            neighbors=neighbors_of(edges, i),
            edges=edges,
            n_agents=N,
        )
        for i in range(N)
    ]

    initial_phases = [a.phase for a in agents]
    print(f"Initial phases: {[f'{p:.3f}' for p in initial_phases]}")
    print(f"Initial spread: {circular_spread(initial_phases):.4f}")
    print()

    # Run consensus
    print("--- Running Consensus ---")
    for tick in range(1, 51):
        # Tick all agents
        for a in agents:
            a.tick()

        # Exchange phases and correct
        phases = [a.phase for a in agents]
        for a in agents:
            nbr_phases = [phases[j] for j in a.neighbors]
            a.correct(nbr_phases)

        # Report every 10 ticks
        if tick % 10 == 0:
            phases = [a.phase for a in agents]
            spread = circular_spread(phases)
            converged = sum(1 for a in agents if a.converged)
            max_corr = max(
                a.corrections[-1] if a.corrections else 0
                for a in agents
            )
            print(f"  tick {tick:2d}: spread={spread:.4f}  "
                  f"converged={converged}/{N}  "
                  f"max_corr={max_corr:.6f}")

    # Final state
    print()
    final_phases = [a.phase for a in agents]
    print(f"Final phases: {[f'{p:.3f}' for p in final_phases]}")
    print(f"Final spread: {circular_spread(final_phases):.4f}")
    print(f"All converged: {all(a.converged for a in agents)}")

    # Per-agent detail
    print("\n--- Agent Detail ---")
    for i, a in enumerate(agents):
        s = a.state()
        print(f"  Agent {i}: φ={s.phase:.4f}  ticks={s.tick_count}  "
              f"converged={s.converged}  ε={s.epsilon:.4f}  "
              f"anomalies={s.anomaly_count}")


if __name__ == "__main__":
    main()
