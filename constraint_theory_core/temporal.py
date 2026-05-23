"""
Temporal Constraints — deadband funnel with exponential decay.

The funnel starts wide (permissive) and narrows monotonically:
ε(t) = ε₀ · e^(-λt)

When prediction error exceeds the deadband, an anomaly is detected
and the funnel widens. Otherwise, the system converges to zero drift.

This is the temporal dimension of the unified architecture: the metronome's
ε and δ are duals of the deadband parameters.

Example
-------
>>> from constraint_theory_core.temporal import TemporalAgent, FunnelPhase
>>> agent = TemporalAgent(decay_rate=0.1)
>>> result = agent.observe(0.1, 0.3, t=1.0)
>>> result.phase == FunnelPhase.APPROACH
True
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from enum import Enum
from typing import Optional

from .lattice import snap, soft_snap

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

SQRT_3: float = math.sqrt(3.0)
COVERING_RADIUS: float = 1.0 / SQRT_3


# ---------------------------------------------------------------------------
# Types
# ---------------------------------------------------------------------------

class FunnelPhase(Enum):
    """Phase of the deadband funnel at the current observation."""
    APPROACH = "approach"
    """Error is decreasing — converging toward the lattice point."""
    NARROWING = "narrowing"
    """Deadband is shrinking — tightening the constraint."""
    ANOMALY = "anomaly"
    """Error exceeded deadband — anomaly detected, funnel resets."""


@dataclass(frozen=True, slots=True)
class FunnelResult:
    """Result of a temporal observation.

    Attributes
    ----------
    phase : FunnelPhase
        Current phase of the funnel.
    error : float
        Quantization error at this observation.
    deadband : float
        Current deadband width.
    snapped_a : int
        Lattice coordinate a of the snap point.
    snapped_b : int
        Lattice coordinate b of the snap point.
    """
    phase: FunnelPhase
    error: float
    deadband: float
    snapped_a: int
    snapped_b: int

    def __repr__(self) -> str:
        return (
            f"FunnelResult(phase={self.phase!r}, error={self.error}, "
            f"deadband={self.deadband}, snapped_a={self.snapped_a}, "
            f"snapped_b={self.snapped_b})"
        )


# ---------------------------------------------------------------------------
# Temporal Agent
# ---------------------------------------------------------------------------

class TemporalAgent:
    """Deadband funnel agent with exponential temporal decay.

    The agent tracks a point on the Eisenstein lattice with a decaying
    deadband. When error stays below the deadband, the funnel narrows
    (precision improves). When error exceeds the deadband, an anomaly
    is flagged and the funnel resets.

    Parameters
    ----------
    decay_rate : float
        Exponential decay rate λ for ε(t) = ε₀ · e^(-λt).
    epsilon_0 : float
        Initial deadband width. Defaults to covering radius.
    delta : float
        Anomaly threshold (absolute). Defaults to covering radius.
    softness : float
        Soft-snap parameter ε in [0, 1].  0 = exact snap (rigid),
        1 = no snap (free).  Intermediate values soften the deadband
        enforcement by interpolating the snap target.
    """

    def __init__(
        self,
        decay_rate: float = 0.1,
        epsilon_0: float = COVERING_RADIUS,
        delta: float = COVERING_RADIUS,
        softness: float = 0.0,
    ) -> None:
        # Validate decay_rate
        if not isinstance(decay_rate, (int, float)):
            raise TypeError(
                f"decay_rate must be a number, got {type(decay_rate).__name__}"
            )
        if isinstance(decay_rate, float) and (math.isnan(decay_rate) or math.isinf(decay_rate)):
            raise ValueError(f"decay_rate must be finite, got {decay_rate}")
        if decay_rate < 0:
            raise ValueError(f"decay_rate must be non-negative, got {decay_rate}")

        # Validate epsilon_0
        if not isinstance(epsilon_0, (int, float)):
            raise TypeError(
                f"epsilon_0 must be a number, got {type(epsilon_0).__name__}"
            )
        if isinstance(epsilon_0, float) and (math.isnan(epsilon_0) or math.isinf(epsilon_0)):
            raise ValueError(f"epsilon_0 must be finite, got {epsilon_0}")
        if epsilon_0 <= 0:
            raise ValueError(f"epsilon_0 must be positive, got {epsilon_0}")

        self.decay_rate = decay_rate
        self.epsilon_0 = epsilon_0
        self.delta = delta
        self.softness = softness
        self._epsilon = epsilon_0
        self._last_t: Optional[float] = None
        self._last_error: float = 0.0
        self._anomalies: int = 0

    @property
    def epsilon(self) -> float:
        """Current deadband width."""
        return self._epsilon

    @property
    def anomaly_count(self) -> int:
        """Number of anomalies detected."""
        return self._anomalies

    def observe(self, x: float, y: float, t: float) -> FunnelResult:
        """Observe a point and update the funnel.

        Parameters
        ----------
        x : float
            Real coordinate.
        y : float
            Imaginary coordinate.
        t : float
            Current time (monotonically increasing).

        Returns
        -------
        FunnelResult
            Phase, error, deadband, and snap point.
        """
        pt, error = snap(x, y)

        # Apply soft deadband: widen effective deadband by softness
        effective_epsilon = self._epsilon * (1.0 + self.softness)

        # Decay the deadband
        if self._last_t is not None:
            dt = t - self._last_t
            if dt > 0:
                self._epsilon *= math.exp(-self.decay_rate * dt)
        self._last_t = t

        if error > self.delta:
            phase = FunnelPhase.ANOMALY
            self._anomalies += 1
            self._epsilon = self.epsilon_0  # reset
        elif error > effective_epsilon:
            phase = FunnelPhase.APPROACH
        else:
            phase = FunnelPhase.NARROWING

        self._last_error = error

        return FunnelResult(
            phase=phase,
            error=error,
            deadband=self._epsilon,
            snapped_a=pt.a,
            snapped_b=pt.b,
        )

    def decay(self, dt: float) -> None:
        """Manually decay the deadband by a time interval.

        Parameters
        ----------
        dt : float
            Time elapsed since last update.
        """
        if dt > 0:
            self._epsilon *= math.exp(-self.decay_rate * dt)

    def reset(self, _x: float, _y: float) -> None:
        """Reset the funnel to initial width.

        Parameters
        ----------
        _x : float
            Unused (for API compatibility).
        _y : float
            Unused (for API compatibility).
        """
        self._epsilon = self.epsilon_0
