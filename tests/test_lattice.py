"""Tests for constraint_theory_core.lattice — Eisenstein A₂ operations."""

import math
import pytest
from constraint_theory_core.lattice import (
    A2Point, snap, covering_radius, is_safe, norm_sq,
    decode_dodecet, encode_dodecet, vector48_encode, vector48_decode,
    holonomy_product, is_consistent, COVERING_RADIUS, SAFE_THRESHOLD,
)


class TestA2Point:
    def test_add(self):
        assert A2Point(1, 2) + A2Point(3, 4) == A2Point(4, 6)

    def test_sub(self):
        assert A2Point(5, 3) - A2Point(2, 1) == A2Point(3, 2)

    def test_neg(self):
        assert -A2Point(1, -2) == A2Point(-1, 2)

    def test_norm_sq_zero(self):
        assert A2Point(0, 0).norm_sq() == 0

    def test_norm_sq_positive(self):
        # (1,0): 1-0+0 = 1
        assert A2Point(1, 0).norm_sq() == 1
        # (1,1): 1-1+1 = 1
        assert A2Point(1, 1).norm_sq() == 1
        # (2,-1): 4+2+1 = 7
        assert A2Point(2, -1).norm_sq() == 7

    def test_to_complex_origin(self):
        x, y = A2Point(0, 0).to_complex()
        assert x == pytest.approx(0.0)
        assert y == pytest.approx(0.0)

    def test_to_complex_basis(self):
        x, y = A2Point(0, 1).to_complex()
        assert x == pytest.approx(-0.5)
        assert y == pytest.approx(math.sqrt(3) / 2)


class TestSnap:
    def test_origin(self):
        pt, err = snap(0.0, 0.0)
        assert pt == A2Point(0, 0)
        assert err == pytest.approx(0.0, abs=1e-10)

    def test_on_lattice(self):
        pt, err = snap(-0.5, math.sqrt(3) / 2)
        assert pt == A2Point(0, 1)
        assert err < 1e-10

    def test_covering_radius_guarantee(self):
        """Every point is within ρ = 1/√3 of a lattice point."""
        import random
        rng = random.Random(42)
        for _ in range(1000):
            x = rng.uniform(-10, 10)
            y = rng.uniform(-10, 10)
            _, err = snap(x, y)
            assert err <= covering_radius() + 1e-10, f"Error {err} exceeds covering radius"

    def test_safe_threshold(self):
        """Points near lattice centers snap safely."""
        pt, err = snap(0.1, 0.05)
        assert is_safe(err)


class TestDodecet:
    def test_roundtrip_all(self):
        for i in range(12):
            pt = decode_dodecet(i)
            assert encode_dodecet(pt) == i

    def test_invalid_index(self):
        with pytest.raises(ValueError):
            decode_dodecet(12)
        with pytest.raises(ValueError):
            decode_dodecet(-1)

    def test_invalid_encode(self):
        with pytest.raises(ValueError):
            encode_dodecet(A2Point(5, 5))


class TestVector48:
    def test_roundtrip(self):
        for i in range(48):
            angle = vector48_decode(i)
            assert vector48_encode(angle) == i

    def test_negative_angle(self):
        assert vector48_encode(-math.pi / 4) == vector48_encode(7 * math.pi / 4)

    def test_invalid_decode(self):
        with pytest.raises(ValueError):
            vector48_decode(48)


class TestHolonomy:
    def test_consistent_cycle(self):
        # 12+12+12+12 = 48 ≡ 0 mod 48
        assert is_consistent([12, 12, 12, 12])
        # 24+24 = 48 ≡ 0 mod 48
        assert is_consistent([24, 24])

    def test_inconsistent_cycle(self):
        assert not is_consistent([1, 2, 3])  # 6 mod 48 ≠ 0

    def test_empty_cycle(self):
        assert is_consistent([])

    def test_single_direction(self):
        # Single direction: holonomy = d mod 48
        # Only consistent if d ≡ 0 mod 48
        assert not is_consistent([1])
        assert is_consistent([0])
