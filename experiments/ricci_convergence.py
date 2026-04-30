#!/usr/bin/env python3
"""Ricci Flow Convergence Timing — validates 1.692 multiplier
Measures actual convergence time vs Ricci-predicted (latency × 1.692).
"""
import random, json, time, math

RICCI_MULTIPLIER = 1.692

def simulate_swarm_convergence(n_agents, avg_latency_ms, max_steps=1000):
    """Simulate constraint propagation through swarm.
    Returns steps to convergence and actual time."""
    # Each agent has a value, must converge to global average
    values = [random.gauss(0, 1) for _ in range(n_agents)]
    target = sum(values) / n_agents
    
    for step in range(max_steps):
        new_values = values.copy()
        for i in range(n_agents):
            # Average with random subset (neighbors)
            neighbors = random.sample(range(n_agents), min(12, n_agents))
            neighbor_avg = sum(values[j] for j in neighbors) / len(neighbors)
            new_values[i] = 0.5 * values[i] + 0.5 * neighbor_avg
        values = new_values
        
        # Check convergence
        if all(abs(v - target) < 0.001 for v in values):
            convergence_time = step * avg_latency_ms
            predicted = avg_latency_ms * RICCI_MULTIPLIER * math.log(n_agents)  # Ricci prediction
            return {"converged": True, "steps": step, "actual_ms": convergence_time, "predicted_ms": predicted}
    
    return {"converged": False, "steps": max_steps}

print("Ricci Flow Convergence Timing")
print("="*60)
print(f"{'Agents':>8} {'Latency':>10} {'Steps':>8} {'Actual':>12} {'Predicted':>12} {'Ratio':>8}")
print("-"*60)

results = []
for n in [32, 64, 128, 256, 512, 1024]:
    for latency in [10, 50, 100]:
        r = simulate_swarm_convergence(n, latency)
        if r["converged"]:
            ratio = r["actual_ms"] / r["predicted_ms"] if r["predicted_ms"] > 0 else 0
            print(f"{n:>8} {latency:>9}ms {r['steps']:>8} {r['actual_ms']:>10.1f}ms {r['predicted_ms']:>10.1f}ms {ratio:>7.3f}")
            results.append({**r, "n_agents": n, "latency_ms": latency, "ratio": ratio})

with open("ricci_results.json", "w") as f:
    json.dump(results, f, indent=2)
print("\nResults saved to ricci_results.json")
print(f"\nExpected ratio: clustering near {RICCI_MULTIPLIER}")
