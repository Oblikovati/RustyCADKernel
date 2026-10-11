"""The TKBool verifier wiring: its numeric oracle entry, the native libraries it links
and builds, and a deterministic corpus that reaches every probe operation of both twins
with purely numeric requests."""
import re
import unittest
from pathlib import Path
from coverage_report import PROBE_CORPORA
from native_build import ADDITIONAL_TOOLKITS, CLOSURE_TARGETS, LINKED_TOOLKITS
from probe_shards import SHARD_COUNT
from tkbool_corpus import append_tkbool_requests
from verify_toolkit import NUMERIC_VERIFIERS
from verify_tkbool_numeric import ORACLE, corpus

ROOT = Path(__file__).resolve().parents[2]


def corpus_requests() -> list[str]:
    """The whole TKBool corpus, one request per entry."""
    requests: list[str] = []
    append_tkbool_requests(requests)
    return requests


class BoolVerifierTableTests(unittest.TestCase):
    def test_tkbool_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKBool']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkbool/numeric')
        self.assertEqual(ORACLE.evidence, ROOT/evidence)

    def test_coverage_replays_the_tkbool_corpus(self):
        evidence = {corpus.package: corpus.evidence for corpus in PROBE_CORPORA}
        self.assertEqual(evidence['rustycad-tkbool'], 'target/oracle-tkbool/numeric')

    def test_the_oracle_links_tkbool_and_every_toolkit_below_it(self):
        self.assertIn('TKBool', LINKED_TOOLKITS)
        self.assertEqual(ORACLE.libraries[0], 'TKBool')
        self.assertEqual(set(ORACLE.libraries) - set(LINKED_TOOLKITS), set())
        # Dependants come first in the link order: TKBool sits on TKBO, TKOffset on TKBool.
        self.assertLess(LINKED_TOOLKITS.index('TKBool'), LINKED_TOOLKITS.index('TKBO'))
        self.assertLess(LINKED_TOOLKITS.index('TKOffset'), LINKED_TOOLKITS.index('TKBool'))

    def test_the_closure_builds_tkbool(self):
        # TKOffset depends on TKBool, so building TKOffset builds TKBool.
        self.assertIn('TKOffset', ADDITIONAL_TOOLKITS)
        self.assertIn('TKOffset', CLOSURE_TARGETS)



class BoolCorpusTests(unittest.TestCase):
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
        native = (ROOT/'tools/oracle/tkbool_probe.cxx').read_text()
        rust = (ROOT/'crates/tkbool/examples/numeric_probe.rs').read_text()
        self.assertEqual(set(re.findall(r'\{"(\w+)",', native)), operations)
        self.assertEqual(set(re.findall(r'\(\s*"(\w+)",', rust)), operations)


if __name__ == '__main__':
    unittest.main()
