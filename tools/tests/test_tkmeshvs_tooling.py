"""The TKMeshVS verifier wiring: its numeric oracle entry, the closure it links and builds, and
a deterministic corpus that reaches every probe operation of both twins with purely numeric
requests."""
import math
import re
import unittest
from pathlib import Path
from coverage_report import PROBE_CORPORA
from native_build import MESHVS_CLOSURE_TARGETS, MESHVS_LINKED_TOOLKITS, MESHVS_TOOLKITS, V3D_LINKED_TOOLKITS
from tkmeshvs_corpus import append_tkmeshvs_requests
from verify_toolkit import NUMERIC_VERIFIERS, PROFILE_CLOSURES, linked_toolkits
from verify_tkmeshvs_numeric import ORACLE, corpus

ROOT = Path(__file__).resolve().parents[2]


def corpus_requests() -> list[str]:
    """The whole TKMeshVS corpus, one request per entry."""
    requests: list[str] = []
    append_tkmeshvs_requests(requests)
    return requests


class MeshVsVerifierTableTests(unittest.TestCase):
    def test_tkmeshvs_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKMeshVS']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkmeshvs/numeric')
        self.assertEqual(ORACLE.evidence, ROOT/evidence)

    def test_coverage_replays_the_tkmeshvs_corpus(self):
        evidence = {corpus.package: corpus.evidence for corpus in PROBE_CORPORA}
        self.assertEqual(evidence['martensite-tkmeshvs'], 'target/oracle-tkmeshvs/numeric')

    def test_the_oracle_links_the_tkmeshvs_externlib(self):
        self.assertEqual(linked_toolkits('TKMeshVS'), MESHVS_LINKED_TOOLKITS)
        self.assertEqual(tuple(ORACLE.libraries), MESHVS_LINKED_TOOLKITS)
        self.assertEqual(MESHVS_LINKED_TOOLKITS, ('TKMeshVS', *V3D_LINKED_TOOLKITS))

    def test_the_meshvs_closure_has_its_own_tree(self):
        self.assertEqual(MESHVS_TOOLKITS, ('TKMeshVS',))
        self.assertIn('gtest_main', MESHVS_CLOSURE_TARGETS)
        self.assertEqual(PROFILE_CLOSURES['TKMeshVS'][2], 'target/occt-meshvs-full')


class MeshVsCorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_numeric(self):
        first = corpus_requests()
        self.assertEqual(first, corpus_requests())
        self.assertEqual(corpus(), '\n'.join(first)+'\n')
        for request in first:
            for value in request.split()[1:]:
                # std::istream cannot read nan or inf.
                self.assertTrue(math.isfinite(float(value)), request)

    def test_corpus_reaches_every_probe_operation(self):
        native = (ROOT/'tools/oracle/tkmeshvs_probe.cxx').read_text()
        operations = set(re.findall(r'\{"(mv_\w+)",', native))
        twin = (ROOT/'crates/tkmeshvs/examples/numeric_probe.rs').read_text()
        self.assertEqual(operations, set(re.findall(r'\(\s*"(mv_\w+)",', twin)))
        self.assertEqual(operations, {request.split()[0] for request in corpus_requests()})

    def test_no_negative_buffer_size_is_requested(self):
        # The native allocator failure of a negative MeshVS_Buffer is not a ported behaviour.
        for request in corpus_requests():
            if request.startswith('mv_ds '):
                self.assertGreaterEqual(int(request.split()[4]), 0, request)


if __name__ == '__main__':
    unittest.main()
