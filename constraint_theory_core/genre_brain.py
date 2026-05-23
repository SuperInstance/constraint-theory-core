"""
Genre Brain — maps musical genres to per-constraint epsilon values.

Each genre has a characteristic "softness profile" — how strictly each
constraint dimension is enforced.  This is the bridge between artistic
intent and the mathematical parameters of the constraint system.

The Goldilocks hypothesis predicts an inverted-U: the best-sounding
music sits in the middle (moderate epsilon), not too rigid (boring)
and not too free (chaotic).

Parameters per genre
--------------------
lattice_epsilon : float
    Softness of lattice snap (0=exact, 1=free).
temporal_softness : float
    Softness of temporal deadband.
rigidity_epsilon : float
    Softness of Laman rigidity check.
consensus_softness : float
    Softness of metronome consensus.
holonomy_epsilon : float
    Softness of holonomy verification.
"""

from __future__ import annotations

from dataclasses import dataclass, asdict
from typing import Any, Dict


@dataclass(frozen=True, slots=True)
class GenreEpsilons:
    """Per-constraint softness profile for a genre.

    All values are floats in [0, 1].
    """

    lattice_epsilon: float
    temporal_softness: float
    rigidity_epsilon: float
    consensus_softness: float
    holonomy_epsilon: float

    def __post_init__(self) -> None:
        for field_name in (
            "lattice_epsilon",
            "temporal_softness",
            "rigidity_epsilon",
            "consensus_softness",
            "holonomy_epsilon",
        ):
            val = getattr(self, field_name)
            if not isinstance(val, (int, float)):
                raise TypeError(f"{field_name} must be a number, got {type(val).__name__}")
            if not 0.0 <= float(val) <= 1.0:
                raise ValueError(f"{field_name} must be in [0, 1], got {val}")

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    def overall(self) -> float:
        """Average epsilon across all constraints — single 'softness' number."""
        return (
            self.lattice_epsilon
            + self.temporal_softness
            + self.rigidity_epsilon
            + self.consensus_softness
            + self.holonomy_epsilon
        ) / 5.0


# ---------------------------------------------------------------------------
# Built-in genre profiles
# ---------------------------------------------------------------------------

GENRES: Dict[str, GenreEpsilons] = {
    "serialism": GenreEpsilons(
        lattice_epsilon=0.0,
        temporal_softness=0.0,
        rigidity_epsilon=0.0,
        consensus_softness=0.0,
        holonomy_epsilon=0.0,
    ),
    "strict_counterpoint": GenreEpsilons(
        lattice_epsilon=0.0,
        temporal_softness=0.05,
        rigidity_epsilon=0.0,
        consensus_softness=0.05,
        holonomy_epsilon=0.0,
    ),
    "classical": GenreEpsilons(
        lattice_epsilon=0.10,
        temporal_softness=0.10,
        rigidity_epsilon=0.10,
        consensus_softness=0.10,
        holonomy_epsilon=0.10,
    ),
    "romantic": GenreEpsilons(
        lattice_epsilon=0.20,
        temporal_softness=0.25,
        rigidity_epsilon=0.15,
        consensus_softness=0.20,
        holonomy_epsilon=0.15,
    ),
    "jazz": GenreEpsilons(
        lattice_epsilon=0.30,
        temporal_softness=0.35,
        rigidity_epsilon=0.25,
        consensus_softness=0.30,
        holonomy_epsilon=0.25,
    ),
    "free_jazz": GenreEpsilons(
        lattice_epsilon=0.50,
        temporal_softness=0.55,
        rigidity_epsilon=0.45,
        consensus_softness=0.50,
        holonomy_epsilon=0.45,
    ),
    "minimalism": GenreEpsilons(
        lattice_epsilon=0.05,
        temporal_softness=0.15,
        rigidity_epsilon=0.05,
        consensus_softness=0.10,
        holonomy_epsilon=0.05,
    ),
    "ambient": GenreEpsilons(
        lattice_epsilon=0.60,
        temporal_softness=0.65,
        rigidity_epsilon=0.50,
        consensus_softness=0.55,
        holonomy_epsilon=0.50,
    ),
    "noise": GenreEpsilons(
        lattice_epsilon=0.90,
        temporal_softness=0.90,
        rigidity_epsilon=0.85,
        consensus_softness=0.90,
        holonomy_epsilon=0.85,
    ),
    "chaos": GenreEpsilons(
        lattice_epsilon=1.0,
        temporal_softness=1.0,
        rigidity_epsilon=1.0,
        consensus_softness=1.0,
        holonomy_epsilon=1.0,
    ),
}


def get_genre(name: str) -> GenreEpsilons:
    """Look up a built-in genre profile.

    Parameters
    ----------
    name : str
        Genre name (case-insensitive).

    Returns
    -------
    GenreEpsilons

    Raises
    ------
    KeyError
        If the genre is not found.
    """
    key = name.lower().replace(" ", "_").replace("-", "_")
    if key not in GENRES:
        raise KeyError(
            f"Unknown genre {name!r}. Available: {sorted(GENRES.keys())}"
        )
    return GENRES[key]


def list_genres() -> Dict[str, GenreEpsilons]:
    """Return all built-in genre profiles."""
    return dict(GENRES)


def custom_genre(
    lattice_epsilon: float = 0.0,
    temporal_softness: float = 0.0,
    rigidity_epsilon: float = 0.0,
    consensus_softness: float = 0.0,
    holonomy_epsilon: float = 0.0,
) -> GenreEpsilons:
    """Create a custom genre profile with explicit epsilon values."""
    return GenreEpsilons(
        lattice_epsilon=lattice_epsilon,
        temporal_softness=temporal_softness,
        rigidity_epsilon=rigidity_epsilon,
        consensus_softness=consensus_softness,
        holonomy_epsilon=holonomy_epsilon,
    )


def sweep_genre(base_overall: float) -> GenreEpsilons:
    """Create a uniform-epsilon genre for Goldilocks sweeps.

    All five constraint dimensions get the same epsilon value,
    producing a single control variable for the inverted-U test.

    Parameters
    ----------
    base_overall : float
        Epsilon value applied to all constraints.

    Returns
    -------
    GenreEpsilons
    """
    return GenreEpsilons(
        lattice_epsilon=base_overall,
        temporal_softness=base_overall,
        rigidity_epsilon=base_overall,
        consensus_softness=base_overall,
        holonomy_epsilon=base_overall,
    )
