"""Tests for constraint_theory_core.holonomy — cycle consistency."""

import pytest
from constraint_theory_core.holonomy import (
    cycle_holonomy,
    verify_consistency,
    isolate_fault,
    fault_boundaries,
)

# Consistent direction: sum ≡ 0 mod 48
GOOD_DIRS = [16, 16, 16]   # 48 ≡ 0
BAD_DIRS = [1, 2, 3]       # 6 ≠ 0
TRI = [(0, 1), (1, 2), (2, 0)]
TRI2 = [(0, 1), (1, 3), (3, 0)]


class TestCycleHolonomy:
    def test_consistent_triangle(self):
        assert cycle_holonomy(TRI, GOOD_DIRS) == 0

    def test_inconsistent_triangle(self):
        assert cycle_holonomy(TRI, BAD_DIRS) == 6

    def test_empty_cycle(self):
        assert cycle_holonomy([], []) == 0

    def test_single_edge(self):
        assert cycle_holonomy([(0, 1)], [5]) == 5

    def test_mismatched_lengths(self):
        with pytest.raises(ValueError):
            cycle_holonomy([(0, 1)], [1, 2])

    def test_invalid_direction(self):
        with pytest.raises(ValueError):
            cycle_holonomy([(0, 1)], [48])
        with pytest.raises(ValueError):
            cycle_holonomy([(0, 1)], [-1])

    def test_modulo_48(self):
        assert cycle_holonomy(TRI, [16, 16, 16]) == 0

    def test_zero_directions(self):
        assert cycle_holonomy(TRI, [0, 0, 0]) == 0


class TestVerifyConsistency:
    def test_all_consistent(self):
        tiles = [
            (TRI, GOOD_DIRS),
            (TRI2, [16, 16, 16]),
        ]
        assert verify_consistency(tiles)

    def test_one_inconsistent(self):
        tiles = [
            (TRI, GOOD_DIRS),
            (TRI2, BAD_DIRS),
        ]
        assert not verify_consistency(tiles)

    def test_empty_tiles(self):
        assert verify_consistency([])


class TestIsolateFault:
    def test_single_fault_at_end(self):
        tiles = [(TRI, GOOD_DIRS), (TRI, GOOD_DIRS), (TRI2, BAD_DIRS)]
        idx = isolate_fault(tiles)
        assert idx == 2

    def test_single_fault_at_start(self):
        tiles = [(TRI2, BAD_DIRS), (TRI, GOOD_DIRS), (TRI, GOOD_DIRS)]
        idx = isolate_fault(tiles)
        assert idx == 0

    def test_single_fault_in_middle(self):
        tiles = [(TRI, GOOD_DIRS), (TRI2, BAD_DIRS), (TRI, GOOD_DIRS)]
        idx = isolate_fault(tiles)
        assert idx == 1

    def test_empty_raises(self):
        with pytest.raises(ValueError):
            isolate_fault([])

    def test_all_consistent_raises(self):
        with pytest.raises(ValueError):
            isolate_fault([(TRI, GOOD_DIRS)])

    def test_large_bisection(self):
        good = (TRI, GOOD_DIRS)
        bad = (TRI, BAD_DIRS)
        tiles = [good] * 50 + [bad] + [good] * 49
        assert isolate_fault(tiles) == 50


class TestFaultBoundaries:
    def test_finds_all_faults(self):
        good = (TRI, GOOD_DIRS)
        bad = (TRI, BAD_DIRS)
        tiles = [good, bad, good, bad, bad]
        assert fault_boundaries(tiles) == [1, 3, 4]

    def test_none_faulty(self):
        good = (TRI, GOOD_DIRS)
        assert fault_boundaries([good, good]) == []

    def test_empty(self):
        assert fault_boundaries([]) == []
