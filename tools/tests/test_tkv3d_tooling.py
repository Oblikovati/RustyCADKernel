"""The TKV3d verifier wiring: its numeric oracle entry, the closure it links and builds, and a
deterministic corpus that reaches every probe operation of both twins with purely numeric
requests."""
import math
import re
import unittest
from pathlib import Path
from coverage_report import PROBE_CORPORA
from native_build import V3D_CLOSURE_TARGETS, V3D_LINKED_TOOLKITS, V3D_TOOLKITS
from tkv3d_corpus import append_tkv3d_requests
from verify_toolkit import NUMERIC_VERIFIERS, PROFILE_CLOSURES, linked_toolkits
from verify_tkv3d_numeric import ORACLE, corpus

ROOT = Path(__file__).resolve().parents[2]


def corpus_requests() -> list[str]:
    """The whole TKV3d corpus, one request per entry."""
    requests: list[str] = []
    append_tkv3d_requests(requests)
    return requests


class V3dVerifierTableTests(unittest.TestCase):
    def test_tkv3d_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKV3d']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkv3d/numeric')
        self.assertEqual(ORACLE.evidence, ROOT/evidence)

    def test_coverage_replays_the_tkv3d_corpus(self):
        evidence = {corpus.package: corpus.evidence for corpus in PROBE_CORPORA}
        self.assertEqual(evidence['rustycad-tkv3d'], 'target/oracle-tkv3d/numeric')

    def test_the_oracle_links_the_tkv3d_externlib(self):
        self.assertEqual(linked_toolkits('TKV3d'), V3D_LINKED_TOOLKITS)
        self.assertEqual(tuple(ORACLE.libraries), V3D_LINKED_TOOLKITS)
        self.assertEqual(V3D_LINKED_TOOLKITS[:2], ('TKV3d', 'TKService'))

    def test_the_v3d_closure_has_its_own_tree(self):
        # TKPrim builds the boxes of the unchanged AIS_ColoredShape_Test suite.
        self.assertEqual(V3D_TOOLKITS, ('TKV3d', 'TKPrim'))
        self.assertIn('TKPrim', V3D_LINKED_TOOLKITS)
        self.assertLess(V3D_LINKED_TOOLKITS.index('TKPrim'), V3D_LINKED_TOOLKITS.index('TKTopAlgo'))
        self.assertIn('gtest_main', V3D_CLOSURE_TARGETS)
        self.assertEqual(PROFILE_CLOSURES['TKV3d'][2], 'target/occt-v3d-full')



class V3dCorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_numeric(self):
        first = corpus_requests()
        self.assertEqual(first, corpus_requests())
        self.assertEqual(corpus(), '\n'.join(first)+'\n')
        for request in first:
            for value in request.split()[1:]:
                # std::istream cannot read nan or inf.
                self.assertTrue(math.isfinite(float(value)), request)

    def test_corpus_reaches_every_probe_operation(self):
        native = (ROOT/'tools/oracle/tkv3d_probe.cxx').read_text()
        operations = set(re.findall(r'\{"(v3_\w+)",', native))
        twin = (ROOT/'crates/tkv3d/examples/numeric_probe.rs').read_text()
        self.assertEqual(operations, set(re.findall(r'\(\s*"(v3_\w+)",', twin)))
        self.assertEqual(operations, {request.split()[0] for request in corpus_requests()})


if __name__ == '__main__':
    unittest.main()
