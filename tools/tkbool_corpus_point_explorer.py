"""PointExplorer cases avoid reading native default points' uninitialized tolerance."""
from itertools import product


def append_point_explorer_requests(requests: list[str]) -> None:
    """Append traversal and live-edit cases; e.g. append_point_explorer_requests([])."""
    for count, mask, kept, query in product((0, 1, 4), (0, 5, 15), (0, 1), (-1, 0, 1, 2, 4, 5)):
        requests.append(f'ds_point_explorer {count} {mask} {kept} {query}')
    requests.append('ds_point_explorer_mutation')
