"""Tests for constraint_theory_core.metronome — distributed consensus."""

import math
import pytest
from constraint_theory_core.metronome import Metronome, MetronomeState
from constraint_theory_core.temporal import FunnelPhase


class TestMetronomeInit:
    def test_defaults(self):
        m = Metronome()
        assert m.T == 1.0
        assert m.phase == pytest.approx(0.0)
        assert m.tick_count == 0
        assert not m.converged

    def test_custom_phase(self):
        m = Metronome(phi0=math.pi)
        assert m.phase == pytest.approx(math.pi)

    def test_state_snapshot(self):
        m = Metronome(T=2.0, phi0=math.pi / 2)
        s = m.state()
        assert isinstance(s, MetronomeState)
        assert s.phase == pytest.approx(math.pi / 2)
        assert s.tick_count == 0
        assert not s.converged


class TestTick:
    def test_phase_wraps(self):
        m = Metronome(T=1.0, phi0=0.0)
        # After one tick phase goes 0 → 2π → wraps to 0
        phase = m.tick()
        assert phase == pytest.approx(0.0, abs=1e-12)
        assert m.tick_count == 1

    def test_multiple_ticks(self):
        m = Metronome(T=1.0, phi0=math.pi / 4)
        for _ in range(5):
            m.tick()
        assert m.tick_count == 5
        # Phase after 5 full rotations should be same modulo 2π
        assert m.phase == pytest.approx(math.pi / 4)

    def test_temporal_decay_on_tick(self):
        m = Metronome(T=1.0, epsilon=1.0)
        eps_before = m.state().epsilon
        m.tick()
        eps_after = m.state().epsilon
        assert eps_after < eps_before


class TestAgree:
    def test_agree_same_phase(self):
        m1 = Metronome(phi0=0.5)
        m2 = Metronome(phi0=0.5)
        assert m1.agree(m2)
        assert m2.agree(m1)

    def test_agree_within_epsilon(self):
        m1 = Metronome(phi0=0.0, epsilon=0.1)
        m2 = Metronome(phi0=0.05, epsilon=0.1)
        assert m1.agree(m2)

    def test_disagree_outside_epsilon(self):
        m1 = Metronome(phi0=0.0, epsilon=0.01)
        m2 = Metronome(phi0=1.0, epsilon=0.01)
        assert not m1.agree(m2)

    def test_agree_wraparound(self):
        # Phases near 0 and 2π should be close
        m1 = Metronome(phi0=0.0, epsilon=0.1)
        m2 = Metronome(phi0=2.0 * math.pi - 0.05, epsilon=0.1)
        assert m1.agree(m2)


class TestCorrect:
    def test_no_neighbors_no_correction(self):
        m = Metronome()
        corr = m.correct([])
        assert corr == 0.0
        assert m.corrections == []

    def test_correction_moves_toward_mean(self):
        m = Metronome(phi0=0.0, epsilon=0.1, neighbors=[1, 2], edges=[(0,1),(1,2),(0,2)], n_agents=3)
        # Neighbors clustered around π
        corr = m.correct([math.pi, math.pi, math.pi])
        assert corr > 0.0
        # Phase should have moved toward π
        assert m.phase > 0.0

    def test_convergence_tracked(self):
        m = Metronome(phi0=0.0, epsilon=1.0, neighbors=[1], edges=[(0,1)], n_agents=2)
        # Simulate tiny corrections
        for _ in range(3):
            m._corrections.append(0.001)
        # Manually trigger convergence check via correct
        m.correct([0.0])
        assert m.converged

    def test_correction_with_edges(self):
        edges = [(0, 1), (1, 2), (0, 2)]
        m = Metronome(
            phi0=0.0,
            epsilon=0.1,
            neighbors=[1, 2],
            edges=edges,
            n_agents=3,
        )
        corr = m.correct([math.pi / 2, math.pi / 2])
        assert corr >= 0.0


class TestObserve:
    def test_observe_snaps_phase(self):
        m = Metronome(phi0=0.0)
        # (1, 0) in complex plane → phase 0
        phase = m.observe(1.0, 0.0)
        assert m.phase == pytest.approx(0.0, abs=1e-10)

    def test_observe_anomaly(self):
        m = Metronome(delta=0.01)
        phase = m.observe(10.0, 10.0)
        assert phase == FunnelPhase.ANOMALY
        assert m.anomaly_count == 1


class TestReset:
    def test_reset_clears_state(self):
        m = Metronome(phi0=0.5, T=1.0)
        m.tick()
        m.correct([1.0])
        m.reset()
        assert m.phase == pytest.approx(0.5)
        assert m.tick_count == 0
        assert not m.converged
        assert m.corrections == []
