"""
Holonomy Verification — find inconsistent cycles in a tiled system.

Each tile (cycle) has directions assigned to its edges. If the directions
sum to 0 mod 48, the cycle is holonomy-free (consistent). Otherwise,
it's broken. isolate_fault finds the bad tile in O(log N) via bisection.
"""

from constraint_theory_core.holonomy import (
    cycle_holonomy,
    verify_consistency,
    isolate_fault,
    fault_boundaries,
)


def main():
    print("=== Holonomy Verification ===\n")

    # Define some triangle tiles
    tri_a = [(0, 1), (1, 2), (2, 0)]
    tri_b = [(0, 1), (1, 3), (3, 0)]
    tri_c = [(2, 3), (3, 4), (4, 2)]
    tri_d = [(0, 2), (2, 4), (4, 0)]
    tri_e = [(1, 4), (4, 3), (3, 1)]

    # Consistent: 16+16+16 = 48 ≡ 0 mod 48
    good_dirs = [16, 16, 16]

    # Inconsistent: various non-zero sums
    bad_dirs_1 = [1, 2, 3]       # sum = 6
    bad_dirs_2 = [10, 10, 10]    # sum = 30
    bad_dirs_3 = [5, 5, 5]       # sum = 15

    # -- Single cycle check --
    print("--- Single Cycle Holonomy ---")
    for dirs, label in [
        (good_dirs, "good (16,16,16)"),
        (bad_dirs_1, "bad (1,2,3)"),
        (bad_dirs_2, "bad (10,10,10)"),
        ([0, 0, 0], "zero (0,0,0)"),
    ]:
        h = cycle_holonomy(tri_a, dirs)
        status = "✓ consistent" if h == 0 else f"✗ holonomy = {h}"
        print(f"  {label:20s} → {status}")
    print()

    # -- All-consistent system --
    print("--- All-Consistent System ---")
    tiles = [
        (tri_a, good_dirs),
        (tri_b, good_dirs),
        (tri_c, good_dirs),
    ]
    print(f"  3 tiles, all consistent: {verify_consistency(tiles)} ✓")
    print()

    # -- System with one fault --
    print("--- System with One Fault ---")
    tiles = [
        (tri_a, good_dirs),     # 0: consistent
        (tri_b, good_dirs),     # 1: consistent
        (tri_c, bad_dirs_1),    # 2: INCONSISTENT (sum=6)
        (tri_d, good_dirs),     # 3: consistent
        (tri_e, good_dirs),     # 4: consistent
    ]
    print(f"  Verify all consistent: {verify_consistency(tiles)}")
    idx = isolate_fault(tiles)
    print(f"  First fault at index: {idx} ← found via O(log N) bisection")
    all_faults = fault_boundaries(tiles)
    print(f"  All faults: {all_faults}")
    print()

    # -- Large system with single fault --
    print("--- Large System (100 tiles, 1 fault) ---")
    good = (tri_a, good_dirs)
    bad = (tri_a, bad_dirs_1)
    tiles = [good] * 50 + [bad] + [good] * 49  # fault at index 50

    print(f"  Total tiles: {len(tiles)}")
    print(f"  Verify consistent: {verify_consistency(tiles)}")
    fault_idx = isolate_fault(tiles)
    print(f"  Fault at index: {fault_idx} ← O(log 100) ≈ 7 checks")
    print()

    # -- Multiple faults --
    print("--- Multiple Faults ---")
    tiles = [
        (tri_a, good_dirs),     # 0: ✓
        (tri_b, bad_dirs_1),    # 1: ✗ (sum=6)
        (tri_c, good_dirs),     # 2: ✓
        (tri_d, bad_dirs_2),    # 3: ✗ (sum=30)
        (tri_e, bad_dirs_3),    # 4: ✗ (sum=15)
    ]
    faults = fault_boundaries(tiles)
    print(f"  Faulty tiles at indices: {faults}")
    print(f"  Holonomy values: {[cycle_holonomy(tiles[i][0], tiles[i][1]) for i in faults]}")
    print()
    print(f"  3 out of {len(tiles)} tiles are inconsistent "
          f"({len(faults)/len(tiles)*100:.0f}% fault rate)")


if __name__ == "__main__":
    main()
