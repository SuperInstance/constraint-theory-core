#!/bin/bash
# constraint-theory-core quickstart — snap points, converge funnels, check rigidity
set -e
echo "🧮 Constraint Theory Core — Quick Start"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

pip install -e . --quiet 2>/dev/null

python3 -c "
from constraint_theory_core import snap, is_safe, covering_radius, TemporalAgent, FunnelPhase
from constraint_theory_core import is_laman, henneberg_construct

# Snap a point to the Eisenstein lattice
pt, err = snap(0.6, 0.8)
print(f'📍 snap(0.6, 0.8) → A2({pt.a}, {pt.b}), error={err:.6f}')
print(f'   covering_radius = {covering_radius():.6f}')
print(f'   is_safe(error)?  {is_safe(err)}')

# Temporal agent with deadband funnel
agent = TemporalAgent(decay_rate=0.1)
for i in range(10):
    result = agent.observe(0.5 + 0.05*i, 0.5 + 0.03*i, t=float(i))
    if i >= 8:
        print(f'   step {i}: error={result.error:.6f}, deadband={result.deadband:.6f}, phase={result.phase.name}')
print(f'🎯 Final: error={result.error:.6f}, deadband={result.deadband:.6f}')

# Laman rigidity
graph = henneberg_construct(4)
print(f'🔗 Laman graph (n=4): {graph}  rigid={is_laman(4, graph)}')

print()
print('✅ constraint-theory-core works!')
"
