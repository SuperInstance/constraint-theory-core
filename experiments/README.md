# Constraint Theory Experiments

GPU-ready simulations for validating the 5 DCS convergence constants.

## Experiments

| Script | Validates | What It Measures |
|--------|-----------|-----------------|
| `rigidity_simulation.py` | Law 102 (k=12) | Phase transition at 12 neighbors |
| `ricci_convergence.py` | Law 103 (1.692x) | Convergence time vs Ricci prediction |

## Running

```bash
# CPU reference (any machine)
python3 rigidity_simulation.py
python3 ricci_convergence.py

# GPU accelerated (Forgemaster's RTX 4050)
# TODO: CUDA versions for massive parallel trials
```

## Expected Results

### Rigidity Percolation
Sharp phase transition at k=12. Below: mostly non-rigid, poor coordination. Above: rigid, good coordination. k=12 is the cliff edge.

### Ricci Convergence
Actual/predicted ratio should cluster near 1.692 across all swarm sizes and latencies. This would confirm the Ricci flow spectral gap matches Law 103.

## For Forgemaster
These are CPU reference implementations. Port to CUDA for 1000x speedup:
- 10,000 trials per k-value → millions of graph evaluations
- GPU-parallel convergence simulation
- Overnight runs → comprehensive statistical validation

The results feed directly into the convergence paper (Section 5: Experiments).
