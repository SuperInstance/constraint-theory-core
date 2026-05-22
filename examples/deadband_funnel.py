"""
Deadband Funnel — watch epsilon narrow over time.

The temporal agent starts with a wide deadband (permissive) and
narrows it exponentially: ε(t) = ε₀ · e^(-λt). When error exceeds
the deadband, it's an anomaly and the funnel resets.
"""

from constraint_theory_core.temporal import TemporalAgent, FunnelPhase


def main():
    print("=== Deadband Funnel ===\n")

    agent = TemporalAgent(
        decay_rate=0.1,   # λ in ε(t) = ε₀ · e^(-λt)
        epsilon_0=0.5,    # starting deadband
        delta=0.577,      # anomaly threshold
    )

    print(f"Initial ε = {agent.epsilon:.4f}")
    print(f"Anomaly threshold δ = {agent.delta:.4f}")
    print()

    # Phase 1: Steady observations near origin — funnel narrows
    print("--- Phase 1: Converging (points near origin) ---")
    for t in range(1, 21):
        result = agent.observe(0.01, 0.01, t=float(t))
        phase_icon = {"approach": "→", "narrowing": "↓", "anomaly": "⚠"}
        icon = phase_icon.get(result.phase.value, "?")
        print(f"  t={t:2d}  {icon} {result.phase.value:10s}  "
              f"ε={result.deadband:.4f}  err={result.error:.4f}  "
              f"snap=({result.snapped_a}, {result.snapped_b})")

    print(f"\nAfter 20 safe observations: ε = {agent.epsilon:.6f}")
    print()

    # Phase 2: Anomaly — far-off point exceeds delta
    print("--- Phase 2: Anomaly (point far from lattice) ---")
    result = agent.observe(10.0, 10.0, t=21.0)
    print(f"  ⚠ ANOMALY!  err={result.error:.4f} > δ={agent.delta:.4f}")
    print(f"  Funnel reset to ε = {agent.epsilon:.4f}")
    print(f"  Anomaly count: {agent.anomaly_count}")
    print()

    # Phase 3: Resume narrowing
    print("--- Phase 3: Resume convergence ---")
    for t in range(22, 42):
        result = agent.observe(0.01, 0.01, t=float(t))
        if t % 5 == 0:
            print(f"  t={t:2d}  {result.phase.value:10s}  ε={result.deadband:.6f}")

    print(f"\nFinal ε = {agent.epsilon:.6f}")
    print(f"Total anomalies: {agent.anomaly_count}")


if __name__ == "__main__":
    main()
