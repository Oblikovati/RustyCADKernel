"""Canonical recognition: analytic outputs, failure state and caller-provided fitting seeds."""
from __future__ import annotations
from tktopalgo_shape_program import COMPOUND, SHELL, WIRE, num
from tkshhealing_scene_edges import edge_scene


def append_canonical_requests(requests: list[str]) -> None:
    """Exercise each recognition target and retain the native state between calls."""
    scene = edge_scene()
    program, shapes = scene.program, scene.shapes
    shapes['empty_wire'] = program.container(WIRE)
    shapes['empty_shell'] = program.container(SHELL)
    shapes['invalid'] = program.container(COMPOUND)
    shapes['no_surface'] = program.face(-1)
    for name, children in [('same', ['f_plane', 'f_plane']),
                           ('shifted', ['f_plane', 'f_plane_moved']),
                           ('different', ['f_plane', 'f_cyl'])]:
        shell = program.container(SHELL)
        for child in children:
            program.add(shell, shapes[child])
        shapes['shell_' + name] = shell
    for name in ('line', 'circle', 'seam', 'bspline'):
        wire = program.container(WIRE)
        program.add(wire, shapes[name])
        shapes['wire_' + name] = wire
    cone = program.surface(4, 0., 0., 0., 0., 0., 1., 2., 0.5)
    shapes['f_cone'] = program.face(cone)
    ellipse = program.curve(2, 0., 0., 0., 0., 0., 1., 2., 1.)
    shapes['ellipse'] = program.edge()
    program.edge_curve(shapes['ellipse'], ellipse)
    program.edge_range(shapes['ellipse'], 0., 6.)
    for name, children in [('same', ['line', 'line']), ('parallel', ['line', 'far']),
                           ('different', ['line', 'circle'])]:
        wire = program.container(WIRE)
        for child in children:
            program.add(wire, shapes[child])
        shapes['wire_' + name] = wire
    # Multiple supports select by deviation from the caller's sample surface.
    shapes['multi_support'] = program.edge()
    program.edge_pcurve(shapes['multi_support'], scene.pcurves['line'], scene.surfaces['cyl'])
    program.edge_pcurve(shapes['multi_support'], scene.pcurves['line'], cone)
    program.edge_pcurve(shapes['multi_support'], scene.pcurves['line'], scene.surfaces['sphere'])
    prefix = 'sh_canonical ' + program.tokens()
    selected = ['f_plane', 'f_plane_moved', 'f_cyl', 'f_cyl_rev', 'f_sphere',
                'empty_wire', 'empty_shell', 'no_surface', 'invalid',
                'shell_same', 'shell_shifted', 'shell_different',
                'line', 'line_located', 'circle', 'seam', 'pc_only', 'bare',
                'f_cone', 'ellipse', 'wire_same', 'wire_parallel', 'wire_different', 'multi_support',
                'bspline', 'wire_line', 'wire_circle', 'wire_seam', 'wire_bspline']
    for name in selected:
        for op in range(2, 9):
            for tolerance in (1e-7, 1e-3):
                recognize = [op, tolerance, 1e101, 0.5]
                steps = [[0, shapes[name]], recognize, recognize, [1], recognize,
                         [0, shapes['invalid']], recognize, [1], recognize]
                tokens = [item for step in steps for item in step]
                requests.append(prefix + ' -1 ' + str(len(steps)) + ' ' + ' '.join(map(num, tokens)))
    for name in ('circle', 'wire_circle'):
        for op in (3, 4, 5):
            for radius in (1.0, 1.1, 0.9):
                for tolerance in (1e-7, 0.1):
                    steps = [op, tolerance, radius, 0.5]
                    requests.append(prefix + f' {shapes[name]} 1 ' + ' '.join(map(num, steps)))
    # Empty construction and ClearStatus remain observable before a shape is set.
    requests.append(prefix + ' -1 4 6 1e-7 1 0.5 2 1e-7 1 0.5 1 6 1e-7 1 0.5')
