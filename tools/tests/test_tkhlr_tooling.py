"""The TKHLR verifier wiring: its numeric oracle entry, the native libraries it links
and builds, and a deterministic corpus that reaches every probe operation of both twins
with purely numeric requests."""
import re
import unittest
from pathlib import Path
from coverage_report import PROBE_CORPORA
from native_build import ADDITIONAL_TOOLKITS, CLOSURE_TARGETS, LINKED_TOOLKITS
from probe_shards import SHARD_COUNT
from tkhlr_corpus import append_tkhlr_requests
from tkhlr_corpus_frame import frame_request
from verify_toolkit import NUMERIC_VERIFIERS
from verify_tkhlr_numeric import ORACLE, corpus

ROOT = Path(__file__).resolve().parents[2]


def corpus_requests() -> list[str]:
    """The whole TKHLR corpus, one request per entry."""
    requests: list[str] = []
    append_tkhlr_requests(requests)
    return requests


class HlrVerifierTableTests(unittest.TestCase):
    def test_tkhlr_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKHLR']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkhlr/numeric')
        self.assertEqual(ORACLE.evidence, ROOT/evidence)

    def test_coverage_replays_the_tkhlr_corpus(self):
        evidence = {corpus.package: corpus.evidence for corpus in PROBE_CORPORA}
        self.assertEqual(evidence['rustycad-tkhlr'], 'target/oracle-tkhlr/numeric')

    def test_the_oracle_links_tkhlr_and_every_toolkit_below_it(self):
        self.assertIn('TKHLR', LINKED_TOOLKITS)
        self.assertEqual(ORACLE.libraries[0], 'TKHLR')
        self.assertEqual(set(ORACLE.libraries) - set(LINKED_TOOLKITS), set())
        # Dependants come first in the link order: TKHLR sits on TKTopAlgo.
        self.assertLess(LINKED_TOOLKITS.index('TKHLR'), LINKED_TOOLKITS.index('TKTopAlgo'))

    def test_the_closure_builds_tkhlr(self):
        # TKHLR is a sibling of TKPrim and TKHelix over TKTopAlgo; no target builds another.
        self.assertIn('TKHLR', ADDITIONAL_TOOLKITS)
        self.assertIn('TKHLR', CLOSURE_TARGETS)

    def test_ci_runs_the_tkhlr_verifier(self):
        workflow = (ROOT/'.github/workflows/ci.yml').read_text()
        self.assertIn('tools/verify_toolkit.py TKHLR', workflow)


class HlrCorpusTests(unittest.TestCase):
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
        native = (ROOT/'tools/oracle/tkhlr_probe.cxx').read_text()
        rust = (ROOT/'crates/tkhlr/examples/numeric_probe.rs').read_text()
        self.assertEqual(set(re.findall(r'\{"(\w+)",', native)), operations)
        self.assertEqual(set(re.findall(r'\("(\w+)",', rust)), operations)

    def test_frame_request_is_nine_exact_numbers(self):
        self.assertEqual(frame_request([0.1, -0.0, 2], [0, 0, 1], [1, 0, 0]),
                         'hl_frame 0.1 -0.0 2.0 0.0 0.0 1.0 1.0 0.0 0.0')


if __name__ == '__main__':
    unittest.main()
