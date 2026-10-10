"""Sparse and filtered TopOpeBRepDS_CurveExplorer traversal against native OCCT."""
from itertools import product


def append_curve_explorer_requests(requests: list[str]) -> None:
    """Append deterministic binding/filter cases; e.g. append_curve_explorer_requests([])."""
    for count, gap, mask, only_kept, query in product(
        (0, 1, 4), (0, 2), (0, 5, 15), (0, 1), (-1, 0, 1, 2, 4, 6, 7)
    ):
        requests.append(f'ds_curve_explorer {count} {gap} {mask} {only_kept} {query}')
    requests.extend(f'ds_curve_explorer_mutation {gap}' for gap in (0, 2, 5))
