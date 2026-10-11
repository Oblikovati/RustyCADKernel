"""Martensite point cloud array alias/copy, selection and facing-aware appearance cases."""
from itertools import product


def append_point_cloud_requests(requests: list[str]) -> None:
    """Exercise every geometry overload and ownership transition; append_point_cloud_requests(rows)."""
    sequences = ((0, 2, 4, 1, 3), (2, 4, 3, 0, 1), (5, 0, 2, 6, 7, 1, 3), (3, 1, 1, 3))
    requests.append("v3_point_cloud_alias")
    requests.extend(f"v3_point_cloud_owner {layer}" for layer in (-1, 0, -2, -3))
    for linked, colors, normals, operations in product(range(2), range(3), range(3), sequences):
        requests.append(f"v3_ais_point_cloud {linked} {colors} {normals} {len(operations)} " + " ".join(map(str, operations)))
