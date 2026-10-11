"""The TKFillet verifier wiring: its numeric oracle entry, the native libraries it links
and a deterministic corpus that reaches every probe operation of both twins with
purely numeric requests."""
import re
import unittest
from pathlib import Path
from coverage_report import PROBE_CORPORA
from native_build import LINKED_TOOLKITS
from tkfillet_corpus import append_tkfillet_requests
from tkfillet_corpus_chfids import law_request, spine_request
from verify_toolkit import NUMERIC_VERIFIERS
from verify_tkfillet_numeric import ORACLE, corpus

ROOT = Path(__file__).resolve().parents[2]


def corpus_requests() -> list[str]:
    """The whole TKFillet corpus, one request per entry."""
    requests: list[str] = []
    append_tkfillet_requests(requests)
    return requests


class FilletVerifierTableTests(unittest.TestCase):
    def test_tkfillet_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKFillet']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkfillet/numeric')
        self.assertEqual(ORACLE.evidence, ROOT/evidence)

    def test_coverage_replays_the_tkfillet_corpus(self):
        evidence = {corpus.package: corpus.evidence for corpus in PROBE_CORPORA}
        self.assertEqual(evidence['rustycad-tkfillet'], 'target/oracle-tkfillet/numeric')

    def test_the_oracle_links_tkfillet_and_every_toolkit_below_it(self):
        self.assertEqual(ORACLE.libraries[0], 'TKFillet')
        self.assertEqual(set(ORACLE.libraries) - set(LINKED_TOOLKITS), set())
        # Dependants come first in the link order: TKFillet sits on TKBO.
        self.assertLess(LINKED_TOOLKITS.index('TKFillet'), LINKED_TOOLKITS.index('TKBO'))



class FilletCorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_numeric(self):
        first = corpus_requests()
        self.assertEqual(first, corpus_requests())
        self.assertEqual(corpus(), '\n'.join(first)+'\n')
        for request in first:
            for value in request.split()[1:]:
                float(value)

    def test_corpus_reaches_every_operation_of_both_probes(self):
        operations = {request.split()[0] for request in corpus_requests()}
        native = (ROOT/'tools/oracle/tkfillet_probe.cxx').read_text()
        rust = (ROOT/'crates/tkfillet/examples/numeric_probe.rs').read_text()
        self.assertEqual(set(re.findall(r'\{"(\w+)",', native)), operations)
        self.assertEqual(set(re.findall(r'\(\s*"(\w+)",', rust)), operations)

    def test_spine_request_counts_its_points_and_abscissae(self):
        request = spine_request([(0, 0, 0), (1, 0, 0)], 1, False, (True, False), (2, 1.0), [0.5])
        self.assertEqual(request, 'fl_spine 2.0 0.0 0.0 0.0 1.0 0.0 0.0 1.0 0.0 1.0 0.0 2.0 1.0 1.0 0.5')

    def test_law_request_counts_its_pairs_and_edges(self):
        request = law_request(2, True, [(0.0, 1.0)], [(1, 2.0)], (0.0, 2.0, True), [1.0])
        self.assertEqual(request, 'fl_fil_law 2.0 1.0 1.0 0.0 1.0 1.0 1.0 2.0 0.0 2.0 1.0 1.0 1.0')


if __name__ == '__main__':
    unittest.main()
