"""Martensite/OCCT selection schemes, style slots, callbacks and camera picking."""
from itertools import product


def append_ais_selection_actions_requests(requests: list[str]) -> None:
    """Append deterministic selection traces; append_ais_selection_actions_requests(rows)."""
    flags = (0, 2, 8, 32, 512, 1024, 1026, 1032, 2048, 4096)
    for flag, mask, scheme in product(flags, (0, 3, 6, 7), range(-1, 6)):
        steps = [(0, 3, 0), (11, 0, 0), (12, 0, 0), (0, 6, scheme),
                 (0, 0, scheme), (0, 7, scheme), (8, 0, 1), (9, 0, 1), (10, 0, 1)]
        requests.append(selection_request(flag, mask, 2, 0, 0, steps))
    setters = [(1, 0, 0), (2, 0, 0), (3, 1, 1), (3, 1, 0), (4, 0, 1),
               (5, 0, 1), (6, 0, 1), (7, 2, 1), (13, 0, 0), (5, 2, 1),
               (7, 1, 0), (7, -1, 1), (3, -1, 0), (10, 0, 1)]
    styles = [(21, 3, 0), (31, 0, 0), (31, 1, 0), (31, -1, 0), (32, 0, 0),
              (33, 0, 0), (33, 1, 0), (34, 0, 0), (34, 1, 0), (22, 7, 1),
              (0, 3, 0), (23, 1, 0), (24, 0, 1), (25, -1, 0), (26, 0, 0),
              (27, 3, 0), (28, 0, 3), (29, 0, 1), (30, 0, 1), (21, 7, 1)]
    for flag, mask, mode, mutation in product(flags, (3, 7), (-1, 2), (0, 3, 4)):
        requests.append(selection_request(flag, mask, mode, mutation, 0, setters))
        requests.append(selection_request(flag, mask, mode, mutation, 0, styles))
    for flag, mask, scheme in product((0, 2, 8, 512, 1024), (0, 3, 7), range(-1, 6)):
        steps = [(15, 1, 0), (16, 0, scheme), (17, 1, 0), (16, 0, scheme),
                 (17, 0, 0), (18, 0, scheme), (19, 0, scheme), (20, 0, scheme),
                 (18, 200, scheme), (19, 200, scheme), (20, 200, scheme),
                 (35, 0, 1), (35, 1, 1), (36, 0, 1), (36, 1, 1),
                 (37, 0, 1), (37, 1, 1), (39, 0, 0), (41, 0, 0), (40, 0, 0)]
        requests.append(selection_request(flag, mask, 2, 0, 0, steps))
    for flag, failing in product((0, 2, 1024), range(1, 7)):
        steps = [(0, 7, 0), (14, 0, 0), (8, 0, 0), (3, 1, 1), (10, 0, 1)]
        requests.append(selection_request(flag, 7, 2, 0, failing, steps))
    requests.append(selection_request(0, 7, 2, 0, 0,
        [(1, 0, 0), (38, 2, 0), (8, 0, 0), (11, 0, 0), (12, 0, 0)]))
    for flag in (0, 1024, 2048, 4096):
        requests.append(selection_request(flag, 7, 2, 0, 0,
            [(42, 0, 0), (43, 0, 0), (44, 0, 0), (25, 3, 0),
             (26, 0, 0), (29, 0, 0), (30, 0, 0)]))


def selection_request(flags: int, mask: int, mode: int, mutation: int,
                      failing: int, steps: list[tuple[int, int, int]]) -> str:
    """Encode independently reproducible selection operations and their arguments."""
    operations = " ".join(f"{op} {a} {b}" for op, a, b in steps)
    return f"v3_ais_selection_actions {flags} {mask} {mode} {mutation} {failing} {len(steps)} {operations}"
