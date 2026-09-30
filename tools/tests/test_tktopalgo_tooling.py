"""The TKTopAlgo verifier wiring: its numeric oracle entry, the native libraries it links,
and a deterministic corpus that reaches every probe operation of both twins with purely
numeric requests."""
import re
import unittest
from pathlib import Path
from coverage_report import PROBE_CORPORA
from native_build import LINKED_TOOLKITS
from probe_shards import SHARD_COUNT
from tktopalgo_corpus import append_tktopalgo_requests
from tktopalgo_corpus_brepbuilderapi import make_vertex_request
from tktopalgo_corpus_brepclass import count_commands
from tktopalgo_corpus_faces import SEGMENT, named_faces, record_length, square_face
from verify_toolkit import NUMERIC_VERIFIERS
from verify_tktopalgo_numeric import ORACLE, corpus

ROOT = Path(__file__).resolve().parents[2]


def corpus_requests() -> list[str]:
    """The whole TKTopAlgo corpus, one request per entry."""
    requests: list[str] = []
    append_tktopalgo_requests(requests)
    return requests


class VerifierTableTests(unittest.TestCase):
    def test_tktopalgo_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKTopAlgo']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tktopalgo/numeric')
        self.assertEqual(ORACLE.evidence, ROOT/evidence)

    def test_coverage_replays_the_tktopalgo_corpus(self):
        evidence = {corpus.package: corpus.evidence for corpus in PROBE_CORPORA}
        self.assertEqual(evidence['rustycad-tktopalgo'], 'target/oracle-tktopalgo/numeric')

    def test_the_oracle_links_tktopalgo_and_every_toolkit_below_it(self):
        self.assertIn('TKTopAlgo', LINKED_TOOLKITS)
        self.assertEqual(ORACLE.libraries[0], 'TKTopAlgo')
        self.assertEqual(set(ORACLE.libraries) - set(LINKED_TOOLKITS), set())

    def test_ci_runs_the_tktopalgo_verifier(self):
        workflow = (ROOT/'.github/workflows/ci.yml').read_text()
        self.assertIn('tools/verify_toolkit.py TKTopAlgo', workflow)


class CorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_numeric(self):
        first = corpus_requests()
        self.assertEqual(first, corpus_requests())
        self.assertEqual(corpus(), '\n'.join(first)+'\n')
        for request in first:
            for value in request.split()[1:]:
                float(value)

    def test_corpus_fills_every_shard(self):
        # Round-robin sharding (tools/probe_shards.py) needs no minimum, but a corpus
        # smaller than the shard count would leave the parallel path untested.
        self.assertGreaterEqual(len(corpus_requests()), SHARD_COUNT // 4)

    def test_corpus_reaches_every_operation_of_both_probes(self):
        operations = {request.split()[0] for request in corpus_requests()}
        native = (ROOT/'tools/oracle/tktopalgo_probe.cxx').read_text()
        rust = (ROOT/'crates/tktopalgo/examples/numeric_probe.rs').read_text()
        self.assertEqual(set(re.findall(r'\{"(\w+)",', native)), operations)
        self.assertEqual(set(re.findall(r'\("(\w+)",', rust)), operations)

    def test_make_vertex_request_is_three_exact_coordinates(self):
        self.assertEqual(make_vertex_request([0.1, -0.0, 2]), 'tp_make_vertex 0.1 -0.0 2.0')


# The fields each BRepClass operation reads after its face record (brc_edge: its
# command count and commands come after `k mode n`).
BREPCLASS_FIELDS = {'brc_explorer': 2, 'brc_segment': 3, 'brc_checkpoint': 2, 'brc_intersect': 10,
                    'brc_intersect2': 8, 'brc_classify': 5, 'brc_classify3d': 6, 'brc_passive': 7}


class BRepClassCorpusTests(unittest.TestCase):
    def test_every_face_record_has_the_length_the_reader_consumes(self):
        for name, face in named_faces().items():
            with self.subTest(name):
                self.assertEqual(record_length(face.numbers()), len(face.numbers()))

    def test_square_record_is_a_plane_with_four_segment_edges(self):
        values = square_face().numbers()
        self.assertEqual(len(values), 70)
        self.assertEqual(values[8], 4)
        self.assertEqual(values[21], 4)
        self.assertEqual(values[22], SEGMENT)

    def test_every_request_has_the_fields_of_its_operation(self):
        for request in corpus_requests():
            op, *fields = request.split()
            if op not in BREPCLASS_FIELDS and op != 'brc_edge':
                continue
            values = [float(field) for field in fields]
            length = record_length(values)
            if op == 'brc_edge':
                script = values[length + 3:]
                self.assertEqual(count_commands(script), values[length + 2], request[:80])
                continue
            self.assertEqual(len(values) - length, BREPCLASS_FIELDS[op], request[:80])

    def test_the_corpus_classifies_on_every_face(self):
        faces = {' '.join(repr(float(v)) for v in face.numbers()) for face in named_faces().values()}
        classified = {request.split(' ', 1)[1] for request in corpus_requests() if request.startswith('brc_classify ')}
        for face in faces:
            self.assertTrue(any(line.startswith(face + ' ') for line in classified))

    def test_edge_script_commands_count_their_fields(self):
        self.assertEqual(count_commands([0, 1, -1.0, -1.0, 3.0, 2.0, 6, 1, 4, 0, 1]), 4)
        self.assertEqual(count_commands([3, 7, 0.5, 8, 1]), 3)


if __name__ == '__main__':
    unittest.main()
