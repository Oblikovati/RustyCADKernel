"""Deterministic requests for the Media package without FFmpeg (tools/oracle/tkservice_probe_media.cxx).

`sv_media` compares the time texts (FormatTime and FormatTimeProgress, with the unsigned
wrap of negative times), the error text, a stopped playback timer, a buffer pool through a
sequence of sizes, and the FFmpeg-free answers of the frame, packet, scaler and contexts.
`sv_mtex` compares the media texture set: its plane textures, the YUV programs (text for
text) and the front/back frame queue.
"""
from __future__ import annotations
import random

from tkservice_corpus_camera import numbers

TIMES = [0.0, 0.0004, 0.25, 0.5, 1.0, 9.75, 59.99, 61.25, 600.0, 3599.5, 3725.5, 90000.0, -1.5, -0.25, 1e10]
DURATIONS = [0.0, 1.0, 60.0, 3600.5, -2.0, 7322.0]
CODES = [-2, 0, 2, 26, -1094995529]


def append_media_requests(requests: list[str]) -> None:
    """Append seeded Media requests."""
    rng = random.Random(20261015)
    for index in range(40):
        t = TIMES[index % len(TIMES)] if index < len(TIMES) else rng.choice(TIMES)
        sizes = [rng.choice([0, 0, 64, 100, 1024]) for _ in range(rng.randrange(0, 5))]
        requests.append('sv_media ' + numbers([t, rng.choice(DURATIONS), rng.choice(CODES), len(sizes)] + sizes))
    # sv_mtex: a media texture set with its frame queue locked and released 0..5 times.
    for nb_locks in range(6):
        requests.append('sv_mtex ' + numbers([nb_locks]))
