"""The TKBRep verifier wiring: its numeric oracle entry, the native closure its unchanged
suites link against, and a deterministic corpus that reaches every probe operation with
purely numeric requests."""
import re
import unittest
from pathlib import Path
from tkbrep_corpus import EDGE, REVERSED, VERTEX, Script, append_tkbrep_requests
from tkbrep_brep_corpus import FORWARD, BRepScript
from tkbrep_brepgraph_corpus import (ADD, AUTO, PERFORM, TRACK, append_tkbrep_brepgraph_requests,
                                     request as graph_request)
from tkbrep_brepgraph_walk_corpus import (ADD_EMPTY, DOCUMENT_ROOT,
                                          append_tkbrep_brepgraph_walk_requests,
                                          request as walk_request)
from tkbrep_bintools_bytes_corpus import append_tkbrep_bintools_bytes_requests, shape_set
from tkbrep_bintools_scene_corpus import NORMALS, numbers as scene_numbers
from native_build import LINKED_TOOLKITS, closure_flags
from verify_toolkit import NUMERIC_VERIFIERS

ROOT = Path(__file__).resolve().parents[2]


class VerifierTableTests(unittest.TestCase):
    def test_tkbrep_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKBRep']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkbrep/numeric')


class NativeClosureTests(unittest.TestCase):
    def test_closure_builds_and_links_tkprim_for_the_unchanged_tkbrep_suites(self):
        # TopExp_Test builds its shapes with BRepPrimAPI_MakeBox (TKPrim).
        flags = closure_flags(['FoundationClasses', 'ModelingData'])
        # Other toolkits join the closure after these two (TKShHealing for its verifier).
        additional = next(flag for flag in flags if flag.startswith('-DBUILD_ADDITIONAL_TOOLKITS='))
        self.assertEqual(additional.split('=')[1].split()[:2], ['TKTopAlgo', 'TKPrim'])
        self.assertEqual(LINKED_TOOLKITS[0], 'TKPrim')
        self.assertEqual(LINKED_TOOLKITS[-1], 'TKernel')


class CorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_numeric(self):
        first: list[str] = []
        second: list[str] = []
        append_tkbrep_requests(first)
        append_tkbrep_requests(second)
        self.assertEqual(first, second)
        for request in first:
            op, *fields = request.split()
            self.assertTrue(op.startswith('br_'))
            for value in fields:
                float(value)

    def test_corpus_reaches_every_probe_operation(self):
        requests: list[str] = []
        append_tkbrep_requests(requests)
        probe = (ROOT/'tools/oracle/tkbrep_probe.cxx').read_text()
        operations = set(re.findall(r'\{"(br_\w+)"', probe))
        self.assertEqual({request.split()[0] for request in requests}, operations)

    def test_script_numbers_follow_the_probe_layout(self):
        script = Script()
        t = script.translation(1.0, 2.0, 3.0)
        vertex = script.make(VERTEX)
        edge = script.make(EDGE)
        script.add(edge, vertex, REVERSED, t)
        self.assertEqual(script.numbers(), [1, 0, 1.0, 2.0, 3.0, 3, 1, VERTEX, 1, EDGE, 2, edge, vertex, REVERSED, t])


class BRepScriptTests(unittest.TestCase):
    def test_brep_script_numbers_follow_the_probe_layout(self):
        # Locations, then geometries (one registry per kind), then commands; graph
        # commands are shifted by 100.
        script = BRepScript()
        t = script.translation(1.0, 0.0, 0.0)
        line = script.line((0.0, 0.0, 0.0), (1.0, 0.0, 0.0))
        plane = script.plane((0.0, 0.0, 0.0), (0.0, 0.0, 1.0))
        edge = script.edge(line, t)
        vertex = script.vertex((0.0, 0.0, 0.0))
        script.add(edge, vertex, FORWARD)
        self.assertEqual((line, plane, edge, vertex), (0, 0, 0, 1))
        self.assertEqual(script.numbers(), [1, 0, 1.0, 0.0, 0.0, 2, 0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
                                            10, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 3, 4, line, t, 1e-7,
                                            1, 0.0, 0.0, 0.0, 1e-7, 102, edge, vertex, FORWARD, 0])


class BinToolsCorpusTests(unittest.TestCase):
    def test_scene_numbers_put_the_tweaks_between_geometries_and_commands(self):
        script = BRepScript()
        tri = script.triangulation([(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)], [(1, 2, 3)], 0)
        face = script.new(7, tri)
        numbers = scene_numbers(script, [[NORMALS, tri]], face)
        self.assertEqual(numbers[:2], [0, 1])
        tweaks_at = 2+len(script.geometries[0])
        self.assertEqual(numbers[tweaks_at:tweaks_at+3], [1, NORMALS, tri])
        self.assertEqual(numbers[tweaks_at+3:], [1, 7, tri, face])

    def test_byte_requests_hold_bytes_and_reader_arguments(self):
        requests: list[str] = []
        append_tkbrep_bintools_bytes_requests(requests)
        for request in requests:
            op, reader, zeros, count, *rest = request.split()
            self.assertEqual(op, 'br_bin_read')
            data = [int(v) for v in rest[:int(count)]]
            self.assertTrue(all(0 <= byte <= 255 for byte in data))
            extra = rest[int(count):]
            self.assertEqual(len(extra), 2 if reader == '1' else 0, request[:80])
            self.assertEqual(zeros, '0')

    def test_grouped_file_is_header_tables_shapes_and_root(self):
        data = shape_set(3, [b'V'], root=b'R')
        self.assertTrue(data.startswith(b'\nOpen CASCADE Topology V3 (c)\nLocations 0\nCurve2ds 0\n'))
        self.assertTrue(data.endswith(b'Triangulations 0\n\nTShapes 1\nVR'))


class BRepGraphCorpusTests(unittest.TestCase):
    def test_request_is_the_script_then_the_counted_actions(self):
        script = BRepScript()
        vertex = script.vertex((1.0, 2.0, 3.0))
        line = graph_request(script, [[PERFORM, vertex, 0], [ADD, vertex, AUTO | TRACK]])
        self.assertEqual(line, 'br_bgraph 0 0 1 1 1.0 2.0 3.0 1e-07 2 0 0 0 3 0 9')

    def test_every_request_is_a_bgraph_request(self):
        requests: list[str] = []
        append_tkbrep_brepgraph_requests(requests)
        self.assertTrue(requests)
        self.assertTrue(all(request.startswith('br_bgraph ') for request in requests))


class BRepGraphWalkCorpusTests(unittest.TestCase):
    def test_request_is_the_script_the_actions_then_the_edits(self):
        script = BRepScript()
        vertex = script.vertex((1.0, 2.0, 3.0))
        line = walk_request(script, [[PERFORM, vertex, 0]], [[ADD_EMPTY], [DOCUMENT_ROOT, 0]])
        self.assertEqual(line, 'br_bgraph_walk 0 0 1 1 1.0 2.0 3.0 1e-07 1 0 0 0 2 1 2 0')

    def test_every_request_is_a_walk_request_with_assemblies(self):
        requests: list[str] = []
        append_tkbrep_brepgraph_walk_requests(requests)
        self.assertTrue(all(request.startswith('br_bgraph_walk ') for request in requests))
        # The first requests edit products after the population.
        self.assertTrue(requests[0].split()[-1] != '0')


if __name__ == '__main__':
    unittest.main()
