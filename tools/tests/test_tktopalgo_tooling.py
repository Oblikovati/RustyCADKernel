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


if __name__ == '__main__':
    unittest.main()
