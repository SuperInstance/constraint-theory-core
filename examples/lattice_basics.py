"""
Lattice Basics — snap points to the Eisenstein A₂ lattice.

Every point in the plane is within ρ = 1/√3 ≈ 0.577 of a lattice point.
This is a geometric guarantee, not an approximation.
"""

from constraint_theory_core.lattice import (
    A2Point, snap, covering_radius, is_safe, norm_sq,
    decode_dodecet, encode_dodecet, vector48_encode, vector48_decode,
)
import math


def main():
    print("=== Eisenstein A₂ Lattice ===\n")

    # -- A2Point basics --
    print("--- A2Point ---")
    p = A2Point(3, -1)
    print(f"Point: {p}")
    print(f"Norm² = {p.norm_sq()} (= 3² - 3(-1) + (-1)² = {3*3 - 3*(-1) + (-1)**2})")
    x, y = p.to_complex()
    print(f"Cartesian: ({x:.4f}, {y:.4f})")

    # Arithmetic
    q = A2Point(1, 2)
    print(f"\n{p} + {q} = {p + q}")
    print(f"{p} - {q} = {p - q}")
    print(f"-{p} = {-p}")
    print()

    # -- Snap --
    print("--- Snap ---")
    rho = covering_radius()
    print(f"Covering radius ρ = 1/√3 ≈ {rho:.4f}")
    print(f"Safe threshold ρ/2 ≈ {rho/2:.4f}\n")

    test_points = [
        (0.0, 0.0),
        (1.0, 0.0),
        (0.5, 0.5),
        (-0.3, 0.7),
        (0.73, -0.42),
        (2.0, 1.0),
    ]

    for x, y in test_points:
        pt, err = snap(x, y)
        safe = "✓" if is_safe(err) else "○"
        print(f"  snap({x:5.2f}, {y:5.2f}) → ({pt.a:3d}, {pt.b:3d})  "
              f"err={err:.4f}  safe={safe}")

    print()

    # -- Dodecet --
    print("--- Dodecet (12 directions) ---")
    for i in range(12):
        d = decode_dodecet(i)
        assert encode_dodecet(d) == i
        ns = d.norm_sq()
        print(f"  dir[{i:2d}] = ({d.a:3d}, {d.b:3d})  norm²={ns}")
    print()

    # -- Vector48 --
    print("--- Vector48 (48 Pythagorean directions) ---")
    print(f"Resolution: {360/48:.1f}° per step\n")

    for deg in [0, 45, 90, 180, 270]:
        rad = math.radians(deg)
        idx = vector48_encode(rad)
        back = vector48_decode(idx)
        print(f"  {deg:3d}° → index {idx:2d} → {math.degrees(back):6.1f}°")


if __name__ == "__main__":
    main()
