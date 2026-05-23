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


def soft_verify_consistency(
    tiles: List[Tuple[List[Tuple[int, int]], List[int]]],
    epsilon: float = 0.0,
) -> float:
    """Soft holonomy verification with continuous tolerance.

    At ε=0 this is equivalent to ``verify_consistency`` (hard check).
    As ε increases, holonomy deviations are tolerated proportionally,
    returning a score in [0, 1] instead of a boolean.

    Score::

        1.0 - ε * (total_holonomy_deviation / max_deviation)

    where max_deviation = 48 (full rotation per tile).

    Parameters
    ----------
    tiles : List[Tuple[List[Tuple[int, int]], List[int]]]
        Each tile is a pair (edges, directions).
    epsilon : float
        Softness in [0, 1].

    Returns
    -------
    float
        Consistency score in [0, 1].  1.0 = fully consistent.
    """
    if not 0.0 <= epsilon <= 1.0:
        raise ValueError(f"epsilon must be in [0, 1], got {epsilon}")

    if not tiles:
        return 1.0

    if epsilon == 0.0:
        return 1.0 if verify_consistency(tiles) else 0.0

    total_deviation = 0
    for edges, directions in tiles:
        h = cycle_holonomy(edges, directions)
        # Deviation from zero: use min(h, 48-h) since holonomy wraps
        deviation = min(h, DIRECTION_COUNT - h)
        total_deviation += deviation

    max_deviation = len(tiles) * (DIRECTION_COUNT // 2)
    if max_deviation == 0:
        return 1.0

    raw_score = 1.0 - total_deviation / max_deviation
    # Soften: blend between hard score and 1.0
    return epsilon * 1.0 + (1.0 - epsilon) * raw_score


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
