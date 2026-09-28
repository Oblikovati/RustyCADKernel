"""The TKBRep verifier wiring: its numeric oracle entry, the native-blocked rows that are
kept out of the native gate, and a deterministic corpus that reaches every probe
operation with purely numeric requests."""
import re
import unittest
from pathlib import Path
from tkbrep_corpus import EDGE, REVERSED, VERTEX, Script, append_tkbrep_requests
from verify_toolkit import BLOCKED_STATUS, NUMERIC_VERIFIERS, gated_rows

ROOT = Path(__file__).resolve().parents[2]


def row(upstream_id: str, source: str, status: str = 'translated') -> dict:
    return {'upstream_id': upstream_id, 'source': source, 'status': status,
            'native_block_reason': 'needs TKPrim' if status == BLOCKED_STATUS else None}


class VerifierTableTests(unittest.TestCase):
    def test_tkbrep_runs_its_numeric_oracle(self):
        script, evidence = NUMERIC_VERIFIERS['TKBRep']
        self.assertTrue((ROOT/script).is_file())
        self.assertEqual(evidence, 'target/oracle-tkbrep/numeric')


class GatedRowsTests(unittest.TestCase):
    def test_blocked_rows_are_separated_from_the_gate(self):
        rows = [row('A.a', 'a.cxx'), row('B.b', 'b.cxx', BLOCKED_STATUS)]
        gated, blocked = gated_rows(rows)
        self.assertEqual([r['upstream_id'] for r in gated], ['A.a'])
        self.assertEqual([r['upstream_id'] for r in blocked], ['B.b'])

    def test_a_source_is_gated_whole_or_not_at_all(self):
        rows = [row('A.a', 'a.cxx'), row('A.b', 'a.cxx', BLOCKED_STATUS)]
        with self.assertRaises(ValueError):
            gated_rows(rows)

    def test_a_toolkit_needs_at_least_one_gated_suite(self):
        with self.assertRaises(ValueError):
            gated_rows([row('B.b', 'b.cxx', BLOCKED_STATUS)])


class CorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_and_numeric(self):
        first: list[str] = []
        second: list[str] = []
        append_tkbrep_requests(first)
        append_tkbrep_requests(second)
        self.assertEqual(first, second)
        for request in first:
            op, *fields = request.split()
            self.assertTrue(op.startswith('br_'))
            for value in fields:
                float(value)

    def test_corpus_reaches_every_probe_operation(self):
        requests: list[str] = []
        append_tkbrep_requests(requests)
        probe = (ROOT/'tools/oracle/tkbrep_probe.cxx').read_text()
        operations = set(re.findall(r'\{"(br_\w+)"', probe))
        self.assertEqual({request.split()[0] for request in requests}, operations)

    def test_script_numbers_follow_the_probe_layout(self):
        script = Script()
        t = script.translation(1.0, 2.0, 3.0)
        vertex = script.make(VERTEX)
        edge = script.make(EDGE)
        script.add(edge, vertex, REVERSED, t)
        self.assertEqual(script.numbers(), [1, 0, 1.0, 2.0, 3.0, 3, 1, VERTEX, 1, EDGE, 2, edge, vertex, REVERSED, t])


if __name__ == '__main__':
    unittest.main()
