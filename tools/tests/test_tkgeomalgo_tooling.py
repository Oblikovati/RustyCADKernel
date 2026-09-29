"""The TKGeomAlgo verifier wiring: its numeric oracle entry, the numeric-only run of a
toolkit without mapped suites, and a deterministic corpus that reaches every probe
operation of both twins with purely numeric requests."""
import random
import re
import tempfile
import unittest
from pathlib import Path
from coverage_report import PROBE_CORPORA
from native_build import LINKED_TOOLKITS
from tkgeomalgo_corpus import append_tkgeomalgo_requests
from tkgeomalgo_corpus_intf import identity, point_pool, spoint, triangle_request
from tkgeomalgo_corpus_intf_poly import height_field
from tkgeomalgo_corpus_toptrans import fan
from verify_toolkit import NUMERIC_VERIFIERS, mapped_suites

ROOT = Path(__file__).resolve().parents[2]


def corpus_requests() -> list[str]:
    """The whole TKGeomAlgo corpus, one request per entry."""
    requests: list[str] = []
    append_tkgeomalgo_requests(requests)
    return requests


class VerifierTableTests(unittest.TestCase):
    def test_tkgeomalgo_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKGeomAlgo']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkgeomalgo/numeric')

    def test_coverage_replays_the_tkgeomalgo_corpus(self):
        evidence = {corpus.package: corpus.evidence for corpus in PROBE_CORPORA}
        self.assertEqual(evidence['rustycad-tkgeomalgo'], 'target/oracle-tkgeomalgo/numeric')

    def test_the_oracle_closure_links_tkgeomalgo(self):
        self.assertIn('TKGeomAlgo', LINKED_TOOLKITS)


class NumericOnlyRunTests(unittest.TestCase):
    def test_no_mapped_rows_runs_no_suite_and_hashes_the_closure(self):
        # A fake build tree holding one empty file per linked library.
        with tempfile.TemporaryDirectory() as directory:
            build = Path(directory)
            for name in LINKED_TOOLKITS:
                (build/f'lib{name}.so').write_bytes(b'')
            native, rust, approved, libraries = mapped_suites(build, build, build, [], 1, {})
        self.assertEqual((native, rust, approved), (set(), set(), {}))
        self.assertEqual([path.name for path in libraries], [f'lib{n}.so' for n in LINKED_TOOLKITS])


class CorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_numeric(self):
        first = corpus_requests()
        self.assertEqual(first, corpus_requests())
        for request in first:
            for value in request.split()[1:]:
                float(value)

    def test_corpus_reaches_every_operation_of_both_probes(self):
        operations = {request.split()[0] for request in corpus_requests()}
        native = (ROOT/'tools/oracle/tkgeomalgo_probe.cxx').read_text()
        rust = (ROOT/'crates/tkgeomalgo/examples/numeric_probe.rs').read_text()
        self.assertEqual(set(re.findall(r'\{"(\w+)",', native)), operations)
        self.assertEqual(set(re.findall(r'\("(\w+)",', rust)), operations)

    def test_triangle_request_is_twelve_coordinates(self):
        line = triangle_request([0, 0, 0], [1, 0, 0], [0, 1, 0], [0.25, 0.25, 0])
        self.assertEqual(line, 'intf_triangle 0.0 0.0 0.0 1.0 0.0 0.0 0.0 1.0 0.0 0.25 0.25 0.0')


class IntfCorpusTests(unittest.TestCase):
    def test_section_point_specs_have_the_probe_field_counts(self):
        rng = random.Random(1)
        for _ in range(50):
            spec = spoint(rng, identity(rng))
            self.assertEqual(len(spec), 13 if spec[0] == 0 else 10)

    def test_point_pool_repeats_identities(self):
        # Logically equal points drive the IsEqual branches of the records.
        pool = point_pool(random.Random(2), 6)
        self.assertEqual(pool[0], 6)

    def test_height_field_triangles_index_its_points(self):
        field = height_field(random.Random(3), 3, True)
        points = int(field[2])
        triangles_at = 3 + 3 * points
        count = int(field[triangles_at])
        indices = field[triangles_at + 1:triangles_at + 1 + 3 * count]
        self.assertEqual((points, count), (9, 8))
        self.assertTrue(all(1 <= i <= points for i in indices))


class TopTransCorpusTests(unittest.TestCase):
    def test_fan_directions_are_normal_to_the_tangent(self):
        self.assertEqual(fan(0.0), (0.0, 1.0, 0.0))


if __name__ == '__main__':
    unittest.main()
