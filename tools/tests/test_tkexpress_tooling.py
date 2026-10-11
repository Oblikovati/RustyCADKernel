"""The TKExpress verifier wiring: its numeric oracle entry, the closure it links and
builds (TKExpress over TKernel only), and a deterministic corpus that reaches every
probe operation of both twins with well-formed requests."""
import re
import unittest
from pathlib import Path
from coverage_report import PROBE_CORPORA
from native_build import EXPRESS_CLOSURE_TARGETS, EXPRESS_LINKED_TOOLKITS, EXPRESS_TOOLKITS
from tkexpress_corpus import append_tkexpress_requests, bits, hex_text
from verify_toolkit import NUMERIC_VERIFIERS, PROFILE_CLOSURES, linked_toolkits
from verify_tkexpress_numeric import ORACLE, corpus

ROOT = Path(__file__).resolve().parents[2]


def corpus_requests() -> list[str]:
    """The whole TKExpress corpus, one request per entry."""
    requests: list[str] = []
    append_tkexpress_requests(requests)
    return requests


class ExpressVerifierTableTests(unittest.TestCase):
    def test_tkexpress_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKExpress']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkexpress/numeric')
        self.assertEqual(ORACLE.evidence, ROOT/evidence)

    def test_coverage_replays_the_tkexpress_corpus(self):
        evidence = {corpus.package: corpus.evidence for corpus in PROBE_CORPORA}
        self.assertEqual(evidence['rustycad-tkexpress'], 'target/oracle-tkexpress/numeric')

    def test_the_oracle_links_only_tkexpress_and_tkernel(self):
        # TKExpress sits on TKernel only (its EXTERNLIB).
        self.assertEqual(linked_toolkits('TKExpress'), ('TKExpress', 'TKernel'))
        self.assertEqual(tuple(ORACLE.libraries), EXPRESS_LINKED_TOOLKITS)

    def test_the_express_closure_builds_tkexpress(self):
        self.assertEqual(EXPRESS_TOOLKITS, ('TKExpress',))
        self.assertIn('gtest_main', EXPRESS_CLOSURE_TARGETS)
        toolkits, targets, tree = PROFILE_CLOSURES['TKExpress']
        self.assertEqual((toolkits, targets, tree),
                         (EXPRESS_TOOLKITS, EXPRESS_CLOSURE_TARGETS, 'target/occt-express-full'))



class ExpressCorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic(self):
        first = corpus_requests()
        self.assertEqual(first, corpus_requests())
        self.assertEqual(corpus(), '\n'.join(first)+'\n')

    def test_requests_hold_one_line_of_tokens(self):
        for request in corpus_requests():
            self.assertNotIn('\n', request)
            for token in request.split()[1:]:
                self.assertRegex(token, r'^(x[0-9a-f]*|-?[0-9]+)$', request[:80])

    def test_corpus_reaches_every_operation_of_both_probes(self):
        operations = {request.split()[0] for request in corpus_requests()}
        native = (ROOT/'tools/oracle/tkexpress_probe.cxx').read_text()
        rust = (ROOT/'crates/tkexpress/examples/numeric_probe.rs').read_text()
        self.assertEqual(set(re.findall(r'\{"(\w+)",', native)), operations)
        self.assertEqual(set(re.findall(r'\("(\w+)",', rust)), operations)

    def test_corpus_reaches_the_bison_stack_limit(self):
        texts = [request for request in corpus_requests() if request.startswith('ex_text ')]
        self.assertIn('ex_text ' + hex_text('-' * 9997 + 'x'), texts)
        self.assertIn('ex_text ' + hex_text('-' * 9998 + 'x'), texts)

    def test_series_requests_never_build_an_empty_sum(self):
        # Expr_Sum::String reads Operand(1) unchecked natively (No_Standard_OutOfRange).
        for request in corpus_requests():
            for token in request.split()[1:]:
                if token.startswith('x'):
                    text = bytes.fromhex(token[1:]).decode('utf-8', 'replace')
                    self.assertNotRegex(text, r'(Sum|Prod)\[.*,\s*0\]', request[:80])

    def test_encoders(self):
        self.assertEqual(hex_text('x+1'), 'x782b31')
        self.assertEqual(bits(1.0), '4607182418800017408')
        self.assertEqual(bits(-0.0), '9223372036854775808')
