"""
Metronome architecture — distributed consensus via Laman-coupled oscillators.

Each agent maintains a phase φ on the Eisenstein lattice.  The metronome
parameters θ = (T, φ₀, ε, δ) are:
- T     : period (seconds per tick)
- φ₀    : reference phase (radians)
- ε     : deadband tolerance (from TemporalAgent)
- δ     : anomaly threshold

Agents tick() forward, agree() with neighbors, and correct() via
Laman-neighbor coupling with strength α* = 2/(λ₂ + λₙ).
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import List, Optional, Tuple

from .lattice import A2Point, snap
from .rigidity import optimal_coupling
from .temporal import TemporalAgent, FunnelPhase


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

TWO_PI: float = 2.0 * math.pi


# ---------------------------------------------------------------------------
# Core types
# ---------------------------------------------------------------------------

@dataclass(frozen=True, slots=True)
class MetronomeState:
    """Snapshot of a metronome agent's state.

    Attributes
    ----------
    phase : float
        Current phase in radians [0, 2π).
    tick_count : int
        Number of ticks elapsed.
    converged : bool
        Whether the agent has reached consensus.
    epsilon : float
        Current deadband width.
    anomaly_count : int
        Number of anomalies detected.
    """
    phase: float
    tick_count: int
    converged: bool
    epsilon: float
    anomaly_count: int


# ---------------------------------------------------------------------------
# Metronome agent
# ---------------------------------------------------------------------------

class Metronome:
    """Distributed metronome agent.

    Parameters
    ----------
    T : float
        Period in seconds.
    phi0 : float
        Initial phase in radians.
    epsilon : float
        Deadband tolerance.
    delta : float
        Anomaly threshold.
    neighbors : List[int]
        Indices of Laman neighbors for correction.
    edges : List[Tuple[int, int]]
        Full edge list for optimal coupling computation.
    n_agents : int
        Total number of agents in the fleet.

    Example
    -------
    >>> m = Metronome(T=1.0, phi0=0.0, epsilon=0.1, delta=0.5)
    >>> m.tick()
    6.283185307179586
    """

    def __init__(
        self,
        T: float = 1.0,
        phi0: float = 0.0,
        epsilon: float = 0.577,
        delta: float = 0.577,
        neighbors: Optional[List[int]] = None,
        edges: Optional[List[Tuple[int, int]]] = None,
        n_agents: int = 1,
    ) -> None:
        self.T = T
        self.phi0 = phi0 % TWO_PI
        self.epsilon = epsilon
        self.delta = delta
        self.neighbors = neighbors if neighbors is not None else []
        self.edges = edges if edges is not None else []
        self.n_agents = n_agents

        self._phi: float = self.phi0
        self._t: float = 0.0
        self._tick_count: int = 0
        self._converged: bool = False
        self._corrections: List[float] = []
        self._temporal = TemporalAgent(
            decay_rate=1.0 / self.T if self.T > 0 else 1.0,
            epsilon_0=self.epsilon,
            delta=self.delta,
        )
        self._alpha = (
            optimal_coupling(self.edges, self.n_agents)
            if self.edges and self.n_agents > 1
            else 0.0
        )

    # -----------------------------------------------------------------------
    # Properties
    # -----------------------------------------------------------------------

    @property
    def phase(self) -> float:
        """Current phase in radians [0, 2π)."""
        return self._phi

    @property
    def tick_count(self) -> int:
        """Number of ticks elapsed."""
        return self._tick_count

    @property
    def converged(self) -> bool:
        """Whether the agent has reached consensus."""
        return self._converged

    @property
    def corrections(self) -> List[float]:
        """History of applied correction magnitudes."""
        return self._corrections.copy()

    @property
    def anomaly_count(self) -> int:
        """Number of anomalies detected by the temporal agent."""
        return self._temporal.anomaly_count

    # -----------------------------------------------------------------------
    # Core operations
    # -----------------------------------------------------------------------

    def tick(self) -> float:
        """Advance the metronome by one period.

        Returns
        -------
        float
            Current phase after tick.
        """
        self._t += self.T
        self._phi = (self._phi + TWO_PI) % TWO_PI
        self._tick_count += 1
        self._temporal.decay(self.T)
        return self._phi

    def agree(self, other: Metronome) -> bool:
        """Check if two metronomes are in sync.

        Two agents agree when their phase difference is within the
        current deadband ε.

        Parameters
        ----------
        other : Metronome
            Another metronome agent.

        Returns
        -------
        bool
            True if phase difference ≤ deadband.
        """
        diff = _circular_distance(self._phi, other._phi)
        return diff <= self._temporal.epsilon

    def correct(self, neighbor_phases: List[float]) -> float:
        """Apply Laman-neighbor correction.

        Computes the circular mean of neighbor phases and nudges the
        local phase toward consensus using the optimal coupling α*.

        Parameters
        ----------
        neighbor_phases : List[float]
            Phases of Laman neighbors in radians.

        Returns
        -------
        float
            Applied correction magnitude.
        """
        if not neighbor_phases or self._alpha <= 0:
            return 0.0

        # Consensus target: circular mean of neighbors
        avg_phase = _circular_mean(neighbor_phases)

        # Shortest signed difference
        diff = _circular_diff(avg_phase, self._phi)

        # Apply correction with optimal coupling
        correction = self._alpha * diff
        self._phi = (self._phi + correction) % TWO_PI
        self._corrections.append(abs(correction))

        # Convergence tracking: three consecutive tiny corrections
        if len(self._corrections) >= 3:
            recent = self._corrections[-3:]
            if all(c < self.epsilon * 0.1 for c in recent):
                self._converged = True

        return abs(correction)

    def observe(self, x: float, y: float) -> FunnelPhase:
        """Observe an external reference point and snap phase.

        Parameters
        ----------
        x : float
            Real coordinate.
        y : float
            Imaginary coordinate.

        Returns
        -------
        FunnelPhase
            Phase of the temporal observation.
        """
        result = self._temporal.observe(x, y, t=self._t)
        # Map snapped lattice point to phase angle
        pt = A2Point(result.snapped_a, result.snapped_b)
        cx, cy = pt.to_complex()
        self._phi = math.atan2(cy, cx) % TWO_PI
        return result.phase

    def reset(self) -> None:
        """Reset metronome to initial phase and clear history."""
        self._phi = self.phi0
        self._t = 0.0
        self._tick_count = 0
        self._converged = False
        self._corrections.clear()
        self._temporal.reset(0.0, 0.0)

    def state(self) -> MetronomeState:
        """Return a snapshot of the current state."""
        return MetronomeState(
            phase=self._phi,
            tick_count=self._tick_count,
            converged=self._converged,
            epsilon=self._temporal.epsilon,
            anomaly_count=self._temporal.anomaly_count,
        )


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------

def _circular_mean(angles: List[float]) -> float:
    """Compute circular mean of angles in radians."""
    if not angles:
        return 0.0
    sx = sum(math.cos(a) for a in angles)
    sy = sum(math.sin(a) for a in angles)
    return math.atan2(sy, sx)


def _circular_diff(target: float, current: float) -> float:
    """Compute shortest signed difference target - current, in [-π, π]."""
    diff = target - current
    while diff > math.pi:
        diff -= TWO_PI
    while diff < -math.pi:
        diff += TWO_PI
    return diff


def _circular_distance(a: float, b: float) -> float:
    """Shortest absolute distance between two angles in [0, 2π)."""
    diff = abs(a - b)
    return min(diff, TWO_PI - diff)
