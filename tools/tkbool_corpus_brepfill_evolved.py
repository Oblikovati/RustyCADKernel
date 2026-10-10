"""BRepFill_Evolved wire/face, constructor/Perform and shell/solid comparisons."""
from __future__ import annotations
from tkbool_corpus_brepfill_advanced_evolved import SPINES
from tkbool_corpus_brepfill_generator import polyline_wire, circle_wire
from tktopalgo_shape_program import ShapeProgram, num

PROFILES = {
    'vertical': ([(0., 0., 0.), (0., 0., 1.)], False),
    'outward': ([(0., 0., 0.), (0., -0.5, 1.)], False),
    'inward': ([(0., 0., 0.), (0., 0.5, 1.)], False),
    'bent': ([(0., 0., 0.), (0., 0., 1.), (0., 0.5, 1.5)], False),
    'planar': ([(0., 0., 0.), (0., 0.5, 0.)], False),
    'triangle': ([(0., 0., 0.), (0., 0.5, 0.), (0., 0., 1.)], True),
}


def evolved_request(spine: str, profile: str, face: int, solid: int,
                    constructor: int, join: int = 0, moved: bool = False) -> str:
    """Make one independent request; e.g. evolved_request('square', 'vertical', 0, 1, 0)."""
    program = ShapeProgram()
    spine_index = (circle_wire(program, (0., 0., 0.), 4.) if spine == 'circle'
                   else polyline_wire(program, SPINES[spine], True))
    points, closed = PROFILES[profile]
    axes = (0., 0., 0., 0., 0., 1., 1., 0., 0.)
    if moved:
        points = [(3. + y, -2. + z, 5. + x) for x, y, z in points]
        axes = (3., -2., 5., 0., 1., 0., 0., 0., 1.)
    profile_index = polyline_wire(program, points, closed)
    return (f'bf_evolved {program.tokens()} {spine_index} {profile_index} '
            f'{face} {solid} {constructor} {join} '+ ' '.join(map(num, axes)))


def append_evolved_requests(requests: list[str]) -> None:
    """Cover every branch of the public facade and each kind of profile part."""
    requests.append('bf_evolved_empty')
    for spine in sorted(SPINES):
        for profile in sorted(PROFILES):
            for face in (0, 1):
                for solid in (0, 1):
                    requests.append(evolved_request(spine, profile, face, solid, 0))
    for face in (0, 1):
        for join in (0, 1, 2):
            requests.append(evolved_request('square', 'vertical', face, 1, 1, join))
    for profile in ('vertical', 'planar', 'outward'):
        for face in (0, 1):
            requests.append(evolved_request('circle', profile, face, 1, 1))
            requests.append(evolved_request('square', profile, face, 1, 0, moved=True))
