"""The TKService verifier wiring: its numeric oracle entry, the visualization closure it
links and builds, and a deterministic corpus that reaches every probe operation of both
twins with purely numeric requests."""
import math
import re
import unittest
from pathlib import Path
from coverage_report import PROBE_CORPORA
from native_build import (VISUALIZATION_CLOSURE_TARGETS, VISUALIZATION_LINKED_TOOLKITS,
                          VISUALIZATION_TOOLKITS)
from tkservice_corpus import append_tkservice_requests
from tkservice_corpus_grid import circular_request, rectangular_request
from verify_toolkit import NUMERIC_VERIFIERS, linked_toolkits
from verify_tkservice_numeric import ORACLE, corpus

ROOT = Path(__file__).resolve().parents[2]


def corpus_requests() -> list[str]:
    """The whole TKService corpus, one request per entry."""
    requests: list[str] = []
    append_tkservice_requests(requests)
    return requests


class ServiceVerifierTableTests(unittest.TestCase):
    def test_tkservice_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKService']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkservice/numeric')
        self.assertEqual(ORACLE.evidence, ROOT/evidence)

    def test_coverage_replays_the_tkservice_corpus(self):
        evidence = {corpus.package: corpus.evidence for corpus in PROBE_CORPORA}
        self.assertEqual(evidence['rustycad-tkservice'], 'target/oracle-tkservice/numeric')

    def test_the_oracle_links_only_the_visualization_closure(self):
        # TKService sits on TKMath and TKernel only (its EXTERNLIB), not on the geometry closure.
        self.assertEqual(linked_toolkits('TKService'), ('TKService', 'TKMath', 'TKernel'))
        self.assertEqual(tuple(ORACLE.libraries), VISUALIZATION_LINKED_TOOLKITS)

    def test_the_visualization_closure_builds_tkservice(self):
        self.assertEqual(VISUALIZATION_TOOLKITS, ('TKService',))
        self.assertIn('gtest_main', VISUALIZATION_CLOSURE_TARGETS)



class ServiceCorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_numeric(self):
        first = corpus_requests()
        self.assertEqual(first, corpus_requests())
        self.assertEqual(corpus(), '\n'.join(first)+'\n')
        for request in first:
            for value in request.split()[1:]:
                # std::istream cannot read nan or inf.
                self.assertTrue(math.isfinite(float(value)), request)

    def test_corpus_reaches_every_operation_of_both_probes(self):
        operations = {request.split()[0] for request in corpus_requests()}
        native = (ROOT/'tools/oracle/tkservice_probe.cxx').read_text()
        rust = (ROOT/'crates/tkservice/examples/numeric_probe.rs').read_text()
        self.assertEqual(set(re.findall(r'\{"(\w+)",', native)), operations)
        self.assertEqual(set(re.findall(r'\("(\w+)",', rust)), operations)

    def test_grid_requests_spell_every_double(self):
        self.assertEqual(rectangular_request([1, 2, 0, 0, 0, 0, 0], [0.5, -0.0]),
                         'sv_rgrid 1.0 2.0 0.0 0.0 0.0 0.0 0.0 0.5 -0.0')
        self.assertEqual(circular_request([1, 8, 0, 0, 0], [1, 2]),
                         'sv_cgrid 1.0 8.0 0.0 0.0 0.0 1.0 2.0')

    def test_no_unrotated_single_division_circular_grid(self):
        # Compute divides by zero natively for one division on an unrotated grid.
        for request in corpus_requests():
            fields = request.split()
            if fields[0] == 'sv_cgrid':
                self.assertFalse(float(fields[2]) == 1.0 and float(fields[5]) == 0.0, request)


if __name__ == '__main__':
    unittest.main()
