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
    DODECET_DIRECTIONS,  # noqa: F401 — re-exported for external use
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
    soft_snap,
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
    soft_rigidity,
)

from .metronome import (
    Metronome,
    MetronomeState,
)

from .holonomy import (
    cycle_holonomy,
    fault_boundaries,
    isolate_fault,
    soft_verify_consistency,
    verify_consistency,
)

from .genre_brain import (
    GenreEpsilons,
    custom_genre,
    get_genre,
    list_genres,
    sweep_genre,
)

from .exercises import (
    generate_exercise,
)

__version__ = "0.1.0"

__all__ = [
    # Lattice
    "A2Point", "snap", "soft_snap", "covering_radius", "is_safe", "norm_sq",
    "decode_dodecet", "encode_dodecet", "vector48_encode", "vector48_decode",
    "holonomy_product", "is_consistent",
    "COVERING_RADIUS", "SAFE_THRESHOLD", "SQRT_3", "DIRECTION_COUNT",
    # Temporal
    "TemporalAgent", "FunnelPhase", "FunnelResult",
    # Rigidity
    "is_laman", "henneberg_construct", "algebraic_connectivity", "optimal_coupling", "soft_rigidity",
    # Metronome
    "Metronome", "MetronomeState",
    # Holonomy
    "cycle_holonomy", "verify_consistency", "soft_verify_consistency", "isolate_fault", "fault_boundaries",
    # Genre Brain
    "GenreEpsilons", "get_genre", "list_genres", "custom_genre", "sweep_genre",
    # Exercises
    "generate_exercise",
    # Meta
    "__version__",
]
