"""AIS_MediaPlayer without a media input: the overlay persistence and layer, the frame fitted
into several areas, the presentation and selection of the frame rectangle and the removal from
the context on close (see tools/oracle/tkv3d_probe_ais_media_player.cxx).

A request is `v3_ais_media_player X Y W H Count Op...`.
"""
from __future__ import annotations

AREAS = ('0 0 640 480', '10 20 100 300', '-5 7 1 1', '3 4 0 9')
SEQUENCES = ((0, 0, 4), (2, 0, 4, 1), (6, 5, 1, 2, 3, 1, 1), (3, 1, 2, 3, 1), (0, 2, 0, 4))


def append_ais_media_player_requests(requests: list[str]) -> None:
    """Every operation sequence on every area."""
    for area in AREAS:
        for ops in SEQUENCES:
            requests.append(f'v3_ais_media_player {area} {len(ops)} {" ".join(map(str, ops))}')
