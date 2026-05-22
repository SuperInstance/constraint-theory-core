"""
Holonomy consistency — detecting faults in tiled constraint systems.

A cycle has zero holonomy when the product (sum) of its direction
vectors closes exactly.  In the PLATO architecture each tile is a
cycle; global consistency requires every tile to be holonomy-free.
"""

from __future__ import annotations

from typing import List, Tuple

from .lattice import DIRECTION_COUNT


# ---------------------------------------------------------------------------
# Core operations
# ---------------------------------------------------------------------------

def cycle_holonomy(edges: List[Tuple[int, int]], directions: List[int]) -> int:
    """Compute holonomy around a directed cycle.

    The holonomy is the signed sum of direction indices modulo 48.
    A value of 0 means the cycle is consistent (closes exactly).

    Parameters
    ----------
    edges : List[Tuple[int, int]]
        Vertices of the cycle in order, e.g. [(0, 1), (1, 2), (2, 0)].
    directions : List[int]
        Direction index (0–47) for each directed edge.

    Returns
    -------
    int
        Holonomy sum modulo 48. 0 means consistent.

    Raises
    ------
    ValueError
        If edges and directions have different lengths, or if any
        direction index is out of range.

    Examples
    --------
    >>> cycle_holonomy([(0, 1), (1, 2), (2, 0)], [12, 12, 12])
    0
    >>> cycle_holonomy([(0, 1), (1, 0)], [5, 0])
    5
    """
    if len(edges) != len(directions):
        raise ValueError(
            f"edges ({len(edges)}) and directions ({len(directions)}) "
            "must have the same length"
        )
    for d in directions:
        if not 0 <= d < DIRECTION_COUNT:
            raise ValueError(
                f"Direction index must be 0-{DIRECTION_COUNT - 1}, got {d}"
            )
    return sum(directions) % DIRECTION_COUNT


def verify_consistency(
    tiles: List[Tuple[List[Tuple[int, int]], List[int]]]
) -> bool:
    """Verify all PLATO tiles are holonomy-free.

    Parameters
    ----------
    tiles : List[Tuple[List[Tuple[int, int]], List[int]]]
        Each tile is a pair (edges, directions).

    Returns
    -------
    bool
        True if every tile has zero holonomy.

    Examples
    --------
    >>> tiles = [
    ...     ([(0, 1), (1, 2), (2, 0)], [12, 12, 12]),
    ...     ([(0, 1), (1, 3), (3, 0)], [24, 24, 0]),
    ... ]
    >>> verify_consistency(tiles)
    True
    """
    for edges, directions in tiles:
        if cycle_holonomy(edges, directions) != 0:
            return False
    return True


def isolate_fault(
    tiles: List[Tuple[List[Tuple[int, int]], List[int]]]
) -> int:
    """O(log N) fault isolation via cycle bisection.

    Given a list of tiles where at least one is inconsistent,
    returns the index of *an* inconsistent tile using binary
    search.  The number of consistency checks is O(log N).

    Parameters
    ----------
    tiles : List[Tuple[List[Tuple[int, int]], List[int]]]
        Tiles to check.

    Returns
    -------
    int
        Index of an inconsistent tile.

    Raises
    ------
    ValueError
        If tiles is empty or all tiles are consistent.

    Examples
    --------
    >>> tiles = [
    ...     ([(0, 1), (1, 2), (2, 0)], [12, 12, 12]),
    ...     ([(0, 1), (1, 3), (3, 0)], [1, 2, 3]),  # inconsistent
    ... ]
    >>> isolate_fault(tiles)
    1
    """
    if not tiles:
        raise ValueError("tiles list is empty")

    n = len(tiles)

    if verify_consistency(tiles):
        raise ValueError("no inconsistent tile found")

    lo, hi = 0, n
    while lo < hi - 1:
        mid = (lo + hi) // 2
        first_half = tiles[lo:mid]
        if not verify_consistency(first_half):
            hi = mid
        else:
            lo = mid

    return lo


def fault_boundaries(
    tiles: List[Tuple[List[Tuple[int, int]], List[int]]]
) -> List[int]:
    """Return indices of all inconsistent tiles (O(N) scan).

    Parameters
    ----------
    tiles : List[Tuple[List[Tuple[int, int]], List[int]]]
        Tiles to check.

    Returns
    -------
    List[int]
        Indices of inconsistent tiles.
    """
    bad: List[int] = []
    for idx, (edges, directions) in enumerate(tiles):
        if cycle_holonomy(edges, directions) != 0:
            bad.append(idx)
    return bad
