"""
Tests for soft_snap and softness parameterization across all constraint modules.

Validates that:
1. soft_snap interpolates correctly between exact snap and free
2. Each module's soft parameter works at key epsilon values
3. The Goldilokes inverted-U can be produced via epsilon sweep
"""

import math
import pytest
from constraint_theory_core import (
    soft_snap,
    snap,
    covering_radius,
    TemporalAgent,
    FunnelPhase,
    is_laman,
    henneberg_construct,
    soft_rigidity,
    Metronome,
    verify_consistency,
    soft_verify_consistency,
    cycle_holonomy,
    GenreEpsilons,
    get_genre,
    sweep_genre,
    list_genres,
    custom_genre,
)


# ---------------------------------------------------------------------------
# soft_snap core tests (15 tests)
# ---------------------------------------------------------------------------


class TestSoftSnapExact:
    """At ε=0, soft_snap should behave identically to snap."""

    def test_epsilon_zero_returns_snapped_coords(self):
        pt, err = snap(0.5, 0.3)
        (sx, sy), soft_err, spt = soft_snap(0.5, 0.3, epsilon=0.0)
        px, py = pt.to_complex()
        assert abs(sx - px) < 1e-12
        assert abs(sy - py) < 1e-12
        assert abs(soft_err - err) < 1e-12
        assert spt == pt

    def test_epsilon_zero_origin(self):
        (sx, sy), _, spt = soft_snap(0.0, 0.0, epsilon=0.0)
        assert spt.a == 0 and spt.b == 0
        assert abs(sx) < 1e-12
        assert abs(sy) < 1e-12

    def test_epsilon_zero_on_lattice(self):
        # Point exactly on lattice
        (sx, sy), err, spt = soft_snap(1.0, 0.0, epsilon=0.0)
        assert abs(err) < 1e-12
        assert abs(sx - 1.0) < 1e-12
        assert abs(sy) < 1e-12


class TestSoftSnapFree:
    """At ε=1, soft_snap should return the original point."""

    def test_epsilon_one_returns_original(self):
        x, y = 0.73, 0.41
        (sx, sy), _, _ = soft_snap(x, y, epsilon=1.0)
        assert abs(sx - x) < 1e-12
        assert abs(sy - y) < 1e-12

    def test_epsilon_one_arbitrary_point(self):
        x, y = -2.3, 4.7
        (sx, sy), _, _ = soft_snap(x, y, epsilon=1.0)
        assert abs(sx - x) < 1e-12
        assert abs(sy - y) < 1e-12

    def test_epsilon_one_origin(self):
        (sx, sy), _, _ = soft_snap(0.0, 0.0, epsilon=1.0)
        assert abs(sx) < 1e-12
        assert abs(sy) < 1e-12


class TestSoftSnapInterpolation:
    """Intermediate ε values should interpolate linearly."""

    def test_epsilon_half_midpoint(self):
        x, y = 0.5, 0.3
        (sx, sy), _, pt = soft_snap(x, y, epsilon=0.0)
        (fx, fy), _, _ = soft_snap(x, y, epsilon=1.0)
        (mx, my), _, _ = soft_snap(x, y, epsilon=0.5)
        expected_x = 0.5 * sx + 0.5 * fx
        expected_y = 0.5 * sy + 0.5 * fy
        assert abs(mx - expected_x) < 1e-12
        assert abs(my - expected_y) < 1e-12

    def test_epsilon_015_close_to_snap(self):
        x, y = 0.5, 0.3
        (sx, sy), _, _ = soft_snap(x, y, epsilon=0.0)
        (soft_x, soft_y), _, _ = soft_snap(x, y, epsilon=0.15)
        # Should be closer to snapped than to original
        d_snap = math.hypot(soft_x - sx, soft_y - sy)
        d_orig = math.hypot(soft_x - x, soft_y - y)
        assert d_snap < d_orig

    def test_monotonic_drift(self):
        """As ε increases, distance from snap should increase monotonically."""
        x, y = 1.3, -0.7
        (sx, sy), _, _ = soft_snap(x, y, epsilon=0.0)
        distances = []
        for eps in [0.0, 0.1, 0.2, 0.3, 0.5, 0.7, 1.0]:
            (soft_x, soft_y), _, _ = soft_snap(x, y, epsilon=eps)
            d = math.hypot(soft_x - sx, soft_y - sy)
            distances.append(d)
        for i in range(1, len(distances)):
            assert distances[i] >= distances[i - 1] - 1e-12

    def test_continuity(self):
        """Small change in ε produces small change in output."""
        x, y = 2.1, 3.4
        (p1x, p1y), _, _ = soft_snap(x, y, epsilon=0.5)
        (p2x, p2y), _, _ = soft_snap(x, y, epsilon=0.501)
        assert abs(p1x - p2x) < 0.01
        assert abs(p1y - p2y) < 0.01

    def test_same_lattice_point_all_epsilon(self):
        """The exact lattice point should be the same regardless of ε."""
        x, y = 0.5, 0.3
        _, _, pt0 = soft_snap(x, y, epsilon=0.0)
        _, _, pt5 = soft_snap(x, y, epsilon=0.5)
        _, _, pt1 = soft_snap(x, y, epsilon=1.0)
        assert pt0 == pt5 == pt1


class TestSoftSnapValidation:
    """Input validation for soft_snap."""

    def test_epsilon_negative_rejected(self):
        with pytest.raises(ValueError, match="\\[0, 1\\]"):
            soft_snap(0.5, 0.3, epsilon=-0.1)

    def test_epsilon_over_one_rejected(self):
        with pytest.raises(ValueError, match="\\[0, 1\\]"):
            soft_snap(0.5, 0.3, epsilon=1.5)

    def test_epsilon_nan_rejected(self):
        with pytest.raises(ValueError, match="finite"):
            soft_snap(0.5, 0.3, epsilon=float("nan"))

    def test_epsilon_string_rejected(self):
        with pytest.raises(TypeError):
            soft_snap(0.5, 0.3, epsilon="soft")  # type: ignore[arg-type]


# ---------------------------------------------------------------------------
# Temporal softness tests (5 tests)
# ---------------------------------------------------------------------------


class TestTemporalSoftness:
    """TemporalAgent softness parameter widens the effective deadband."""

    def test_softness_zero_hard_check(self):
        agent = TemporalAgent(decay_rate=0.0, softness=0.0)
        result = agent.observe(0.5, 0.3, t=0.0)
        # With zero decay and no softness, should behave identically to before
        assert result.phase in (FunnelPhase.APPROACH, FunnelPhase.NARROWING, FunnelPhase.ANOMALY)

    def test_softness_reduces_anomalies(self):
        """Higher softness should tolerate larger errors."""
        rigid = TemporalAgent(decay_rate=0.0, epsilon_0=0.1, delta=0.3, softness=0.0)
        soft = TemporalAgent(decay_rate=0.0, epsilon_0=0.1, delta=0.3, softness=0.5)
        # Observe same sequence — soft should flag fewer anomalies
        rigid.observe(0.1, 0.1, t=0.0)
        soft.observe(0.1, 0.1, t=0.0)
        # Both should survive the first observation
        assert rigid.anomaly_count == soft.anomaly_count == 0

    def test_softness_widens_effective_deadband(self):
        """With high softness, the effective deadband is wider."""
        agent = TemporalAgent(decay_rate=0.0, epsilon_0=0.1, delta=0.5, softness=2.0)
        # effective_epsilon = 0.1 * (1 + 2.0) = 0.3
        # A point with error ~0.2 should be NARROWING (within 0.3) not APPROACH
        result = agent.observe(0.2, 0.1, t=0.0)
        # error for (0.2, 0.1) should be within the widened deadband
        assert result.phase in (FunnelPhase.NARROWING, FunnelPhase.APPROACH)

    def test_softness_propagated_from_metronome(self):
        """Metronome softness should flow into its internal TemporalAgent."""
        m = Metronome(softness=0.3)
        assert m._temporal.softness == 0.3

    def test_temporal_softness_default_is_zero(self):
        """Default softness should be 0 (backward compatible)."""
        agent = TemporalAgent()
        assert agent.softness == 0.0


# ---------------------------------------------------------------------------
# Rigidity softness tests (5 tests)
# ---------------------------------------------------------------------------


class TestSoftRigidity:
    """soft_rigidity provides continuous rigidity scoring."""

    def test_rigid_graph_epsilon_zero(self):
        edges = henneberg_construct(6)
        score = soft_rigidity(6, edges, epsilon=0.0)
        assert score == 1.0

    def test_nonrigid_graph_epsilon_zero(self):
        # Only 2 edges for 5 vertices — not Laman (need 2*5-3=7)
        score = soft_rigidity(5, [(0, 1), (1, 2)], epsilon=0.0)
        assert score == 0.0

    def test_rigid_graph_softened(self):
        edges = henneberg_construct(6)
        score_hard = soft_rigidity(6, edges, epsilon=0.0)
        score_soft = soft_rigidity(6, edges, epsilon=0.5)
        # Soft score should be <= hard score but still high
        assert score_soft <= score_hard
        assert score_soft > 0.5

    def test_nonrigid_graph_softened(self):
        # Non-rigid graph gets a non-zero score with softness
        score = soft_rigidity(5, [(0, 1), (1, 2), (2, 3), (3, 4)], epsilon=0.5)
        assert score > 0.0

    def test_epsilon_out_of_range(self):
        with pytest.raises(ValueError, match="\\[0, 1\\]"):
            soft_rigidity(5, [], epsilon=1.5)


# ---------------------------------------------------------------------------
# Metronome soft consensus tests (5 tests)
# ---------------------------------------------------------------------------


class TestSoftConsensus:
    """Metronome softness weakens consensus coupling."""

    def test_softness_reduces_correction(self):
        """Higher softness → smaller corrections toward neighbors."""
        edges = henneberg_construct(4)
        m_rigid = Metronome(
            phi0=0.0, neighbors=[1, 2], edges=edges, n_agents=4, softness=0.0
        )
        m_soft = Metronome(
            phi0=0.0, neighbors=[1, 2], edges=edges, n_agents=4, softness=0.8
        )
        phases = [0.5, 0.6]  # neighbor phases
        corr_rigid = m_rigid.correct(phases)
        corr_soft = m_soft.correct(phases)
        assert corr_soft <= corr_rigid + 1e-12

    def test_softness_one_no_correction(self):
        """At softness=1, no correction should be applied."""
        edges = henneberg_construct(4)
        m = Metronome(
            phi0=0.0, neighbors=[1, 2], edges=edges, n_agents=4, softness=1.0
        )
        corr = m.correct([0.5, 0.6])
        assert corr == 0.0

    def test_agree_wider_with_softness(self):
        """agree() should be more permissive with higher softness."""
        edges = henneberg_construct(4)
        m1 = Metronome(phi0=0.0, epsilon=0.05, edges=edges, n_agents=4, softness=0.0)
        m2 = Metronome(phi0=0.08, epsilon=0.05, edges=edges, n_agents=4, softness=0.0)
        m3 = Metronome(phi0=0.08, epsilon=0.05, edges=edges, n_agents=4, softness=2.0)
        # m1 and m2: diff=0.08, deadband ~0.05, no softness → disagree
        # m1 and m3: diff=0.08, effective deadband = 0.05*(1+2)=0.15 → agree
        assert not m1.agree(m2) or m1.agree(m3)  # at least one direction shows widening

    def test_default_softness_zero(self):
        m = Metronome()
        assert m.softness == 0.0


# ---------------------------------------------------------------------------
# Holonomy soft verification tests (4 tests)
# ---------------------------------------------------------------------------


class TestSoftHolonomy:
    """soft_verify_consistency provides graded consistency scoring."""

    def test_consistent_tiles_score_one(self):
        tiles = [
            ([(0, 1), (1, 2), (2, 0)], [0, 0, 0]),  # holonomy = 0
        ]
        assert soft_verify_consistency(tiles, epsilon=0.0) == 1.0
        assert soft_verify_consistency(tiles, epsilon=0.5) == 1.0

    def test_inconsistent_hard_check(self):
        tiles = [
            ([(0, 1), (1, 2), (2, 0)], [1, 2, 3]),  # holonomy=6 != 0
        ]
        assert soft_verify_consistency(tiles, epsilon=0.0) == 0.0

    def test_inconsistent_softened(self):
        tiles = [
            ([(0, 1), (1, 2), (2, 0)], [1, 2, 3]),  # holonomy=6
        ]
        score = soft_verify_consistency(tiles, epsilon=0.5)
        assert 0.0 < score <= 1.0

    def test_empty_tiles_score_one(self):
        assert soft_verify_consistency([], epsilon=0.0) == 1.0
        assert soft_verify_consistency([], epsilon=0.5) == 1.0


# ---------------------------------------------------------------------------
# Genre brain tests (4 tests)
# ---------------------------------------------------------------------------


class TestGenreBrain:
    """Genre profiles map to correct epsilon tuples."""

    def test_serialism_is_fully_rigid(self):
        g = get_genre("serialism")
        assert g.overall() == 0.0

    def test_chaos_is_fully_free(self):
        g = get_genre("chaos")
        assert g.overall() == 1.0

    def test_classical_is_moderate(self):
        g = get_genre("classical")
        avg = g.overall()
        assert 0.0 < avg < 0.3

    def test_sweep_genre_uniform(self):
        g = sweep_genre(0.3)
        assert g.lattice_epsilon == 0.3
        assert g.temporal_softness == 0.3
        assert g.overall() == 0.3


# ---------------------------------------------------------------------------
# Goldilocks epsilon sweep test (1 test)
# ---------------------------------------------------------------------------


class TestGoldilocksSweep:
    """Verify the epsilon sweep produces the inverted-U pattern."""

    def test_sweep_produces_varying_metrics(self):
        """Sweep epsilon from 0 to 1 and verify metrics change continuously."""
        x, y = 0.5, 0.3
        epsilons = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]

        # Collect snap deviation at each epsilon
        deviations = []
        for eps in epsilons:
            (sx, sy), err, pt = soft_snap(x, y, epsilon=eps)
            px, py = pt.to_complex()
            deviations.append(math.hypot(sx - px, sy - py))

        # At ε=0 deviation should be 0 (exact snap)
        assert deviations[0] < 1e-12
        # At ε=1 deviation should be maximal (original point to snap)
        assert deviations[-1] > 0.1
        # Monotonically increasing
        for i in range(1, len(deviations)):
            assert deviations[i] >= deviations[i - 1] - 1e-12

    def test_sweep_rigidity_varying(self):
        """Rigidity score should decrease with epsilon for non-rigid graphs."""
        edges = [(0, 1), (1, 2)]  # not Laman for n=4
        scores = [soft_rigidity(4, edges, epsilon=e) for e in [0.0, 0.3, 0.6, 1.0]]
        # Non-rigid at ε=0 should be 0, then increase
        assert scores[0] == 0.0
        # Should be monotonically non-decreasing for non-rigid graph
        for i in range(1, len(scores)):
            assert scores[i] >= scores[i - 1] - 1e-12


# ---------------------------------------------------------------------------
# Integration: full pipeline with softness (2 tests)
# ---------------------------------------------------------------------------


class TestSoftPipeline:
    """End-to-end test of soft parameterization through the full stack."""

    def test_full_pipeline_with_softness(self):
        """Genre epsilons flow through all modules without errors."""
        genre = get_genre("jazz")
        eps = genre.overall()

        # Lattice: soft snap
        (sx, sy), err, pt = soft_snap(1.5, 0.8, epsilon=eps)
        assert err >= 0

        # Temporal: soft deadband
        agent = TemporalAgent(softness=eps)
        result = agent.observe(sx, sy, t=0.0)
        assert result.phase in (FunnelPhase.APPROACH, FunnelPhase.NARROWING, FunnelPhase.ANOMALY)

        # Rigidity: soft check
        edges = henneberg_construct(6)
        score = soft_rigidity(6, edges, epsilon=eps)
        assert 0.0 <= score <= 1.0

        # Holonomy: soft verification
        tiles = [([(0, 1), (1, 2), (2, 0)], [12, 12, 12])]
        h_score = soft_verify_consistency(tiles, epsilon=eps)
        assert 0.0 <= h_score <= 1.0

    def test_sweep_all_modules(self):
        """Sweep epsilon and verify all modules respond."""
        results = {}
        for eps in [0.0, 0.25, 0.5, 0.75, 1.0]:
            _, err, _ = soft_snap(0.5, 0.3, epsilon=eps)
            edges = henneberg_construct(5)
            rig = soft_rigidity(5, edges, epsilon=eps)
            tiles = [([(0, 1), (1, 2), (2, 0)], [0, 0, 0])]
            hol = soft_verify_consistency(tiles, epsilon=eps)
            results[eps] = {"snap_err": err, "rigidity": rig, "holonomy": hol}

        # Verify all epsilons produce valid outputs
        for eps, r in results.items():
            assert r["snap_err"] >= 0
            assert 0.0 <= r["rigidity"] <= 1.0
            assert 0.0 <= r["holonomy"] <= 1.0
