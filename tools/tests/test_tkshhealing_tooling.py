"""The TKShHealing verifier wiring: its numeric oracle entry, the native libraries it links,
and a deterministic corpus that reaches every probe operation of both twins with purely
numeric requests."""
import re
import unittest
from pathlib import Path
from coverage_report import PROBE_CORPORA
from native_build import ADDITIONAL_TOOLKITS, CLOSURE_TARGETS, LINKED_TOOLKITS
from probe_shards import SHARD_COUNT
from tkshhealing_corpus import append_tkshhealing_requests
from tkshhealing_corpus_shapeextend import STATUS_COUNT, status_request
from verify_toolkit import NUMERIC_VERIFIERS
from verify_tkshhealing_numeric import ORACLE, corpus

ROOT = Path(__file__).resolve().parents[2]


def corpus_requests() -> list[str]:
    """The whole TKShHealing corpus, one request per entry."""
    requests: list[str] = []
    append_tkshhealing_requests(requests)
    return requests


class VerifierTableTests(unittest.TestCase):
    def test_tkshhealing_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKShHealing']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkshhealing/numeric')
        self.assertEqual(ORACLE.evidence, ROOT/evidence)

    def test_coverage_replays_the_tkshhealing_corpus(self):
        evidence = {corpus.package: corpus.evidence for corpus in PROBE_CORPORA}
        self.assertEqual(evidence['rustycad-tkshhealing'], 'target/oracle-tkshhealing/numeric')

    def test_the_oracle_links_tkshhealing_and_every_toolkit_below_it(self):
        self.assertIn('TKShHealing', LINKED_TOOLKITS)
        self.assertEqual(ORACLE.libraries[0], 'TKShHealing')
        self.assertEqual(set(ORACLE.libraries) - set(LINKED_TOOLKITS), set())

    def test_the_closure_builds_tkshhealing(self):
        # A closure built from the sources (CI) must hold libTKShHealing for the probe.
        self.assertIn('TKShHealing', ADDITIONAL_TOOLKITS)
        self.assertIn('TKShHealing', CLOSURE_TARGETS)
        self.assertIn('TKPrim', CLOSURE_TARGETS)

    def test_ci_runs_the_tkshhealing_verifier(self):
        workflow = (ROOT/'.github/workflows/ci.yml').read_text()
        self.assertIn('tools/verify_toolkit.py TKShHealing', workflow)


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
        self.assertGreaterEqual(len(corpus_requests()), SHARD_COUNT)

    def test_corpus_reaches_every_operation_of_both_probes(self):
        operations = {request.split()[0] for request in corpus_requests()}
        native = (ROOT/'tools/oracle/tkshhealing_probe.cxx').read_text()
        rust = (ROOT/'crates/tkshhealing/examples/numeric_probe.rs').read_text()
        self.assertEqual(set(re.findall(r'\{"(\w+)",', native)), operations)
        self.assertEqual(set(re.findall(r'\("(\w+)",', rust)), operations)

    def test_status_requests_cover_every_enumerator(self):
        self.assertEqual(status_request(-1, 18), 'sh_status -1 18')
        statuses = {int(request.split()[2]) for request in corpus_requests()
                    if request.startswith('sh_status ')}
        self.assertEqual(statuses, set(range(STATUS_COUNT)))


if __name__ == '__main__':
    unittest.main()
