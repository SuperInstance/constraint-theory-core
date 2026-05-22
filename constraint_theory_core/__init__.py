"""
constraint_theory_core — unified geometric constraint theory.

The one package that brings together:
- Eisenstein A₂ lattice quantization
- Deadband temporal funnels
- Laman rigidity for topology
- Metronome consensus
- Holonomy consistency verification

All constants derived from first principles. Zero external dependencies.

Example
-------
>>> from constraint_theory_core import snap, covering_radius, is_safe
>>> pt, err = snap(0.5, 0.3)
>>> is_safe(err)
True
"""

from .lattice import (
    A2Point,
    COVERING_RADIUS,
    DIRECTION_COUNT,
    DODECET_DIRECTIONS,
    SAFE_THRESHOLD,
    SQRT_3,
    covering_radius,
    decode_dodecet,
    encode_dodecet,
    holonomy_product,
    is_consistent,
    is_safe,
    norm_sq,
    snap,
    vector48_decode,
    vector48_encode,
)

from .temporal import (
    FunnelPhase,
    FunnelResult,
    TemporalAgent,
)

from .rigidity import (
    algebraic_connectivity,
    henneberg_construct,
    is_laman,
    optimal_coupling,
)

from .metronome import (
    Metronome,
    MetronomeState,
)

from .holonomy import (
    cycle_holonomy,
    fault_boundaries,
    isolate_fault,
    verify_consistency,
)

__version__ = "0.1.0"

__all__ = [
    # Lattice
    "A2Point", "snap", "covering_radius", "is_safe", "norm_sq",
    "decode_dodecet", "encode_dodecet", "vector48_encode", "vector48_decode",
    "holonomy_product", "is_consistent",
    "COVERING_RADIUS", "SAFE_THRESHOLD", "SQRT_3", "DIRECTION_COUNT",
    # Temporal
    "TemporalAgent", "FunnelPhase", "FunnelResult",
    # Rigidity
    "is_laman", "henneberg_construct", "algebraic_connectivity", "optimal_coupling",
    # Metronome
    "Metronome", "MetronomeState",
    # Holonomy
    "cycle_holonomy", "verify_consistency", "isolate_fault", "fault_boundaries",
    # Meta
    "__version__",
]
