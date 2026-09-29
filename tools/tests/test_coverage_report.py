"""Coverage counts probe lines only when the probe still matches the saved native output."""
import tempfile
import unittest
from pathlib import Path
from unittest import mock
import coverage_report
from probe_shards import ShardResult
from coverage_report import ProbeCorpus, check_output, crate_totals, percent, replay_probe, verifier_environment


class RecordingProbeRunner:
    """Return a fixed probe output and record the command it was asked to run."""

    def __init__(self, output: str):
        self.output = output
        self.commands: list[list[str]] = []

    def __call__(self, args, stdin):
        self.commands.append(args)
        return self.output


class CheckOutputTests(unittest.TestCase):
    def test_numeric_corpora_accept_nan_payload_differences_only(self):
        corpus = ProbeCorpus('p', 'numeric_probe', 'e')
        nan_a, nan_b = str(0x7ff8000000000001), str(0x7ff8000000000002)
        check_output(corpus, f'OK {nan_a}\n', f'OK {nan_b}\n', 'op 1\n')
        with self.assertRaises(RuntimeError):
            check_output(corpus, 'OK 4607182418800017408\n', 'OK 4607182418800017409\n', 'op 1\n')

    def test_exact_corpora_reject_any_text_difference(self):
        corpus = ProbeCorpus('p', 'oracle_probe', 'e', 'occt-output.txt', True)
        check_output(corpus, 'OK a\n', 'OK a\n', 'op\n')
        with self.assertRaises(RuntimeError):
            check_output(corpus, 'OK a\n', 'OK b\n', 'op\n')


class FixedShardRunner:
    """Answer every shard with its own lines mapped through a table, recording the environment."""

    def __init__(self, answers: dict[str, str]):
        self.answers = answers
        self.environments: list[dict[str, str]] = []

    def __call__(self, command, requests):
        self.environments.append(command.environment)
        return ShardResult(''.join(self.answers[line] for line in requests.splitlines(keepends=True)), '')


class ReplayProbeTests(unittest.TestCase):
    def test_replay_runs_the_probe_under_coverage_and_checks_it(self):
        with tempfile.TemporaryDirectory() as tmp, mock.patch.object(coverage_report, 'ROOT', Path(tmp)):
            (Path(tmp)/'ev').mkdir()
            (Path(tmp)/'ev/inputs.txt').write_text('op 1\nop 2\n')
            (Path(tmp)/'ev/native.txt').write_text('OK 1\nOK 2\n')
            corpus = ProbeCorpus('rustycad-x', 'numeric_probe', 'ev')
            builder = RecordingProbeRunner('')
            shards = FixedShardRunner({'op 1\n': 'OK 1\n', 'op 2\n': 'OK 2\n'})
            self.assertEqual(replay_probe(corpus, builder, shards), 2)
            self.assertEqual(builder.commands[0][:5], ['cargo', 'llvm-cov', 'run', '--no-report', '--locked'])
            self.assertTrue(all(e['LLVM_PROFILE_FILE'].endswith('-%p-%m.profraw') for e in shards.environments))
            with self.assertRaises(RuntimeError):
                replay_probe(corpus, builder, FixedShardRunner({'op 1\n': 'OK 1\n', 'op 2\n': 'OK 3\n'}))

    def test_missing_evidence_names_the_verifier_to_run(self):
        with tempfile.TemporaryDirectory() as tmp, mock.patch.object(coverage_report, 'ROOT', Path(tmp)):
            with self.assertRaisesRegex(FileNotFoundError, 'rustycad-x verifier'):
                replay_probe(ProbeCorpus('rustycad-x', 'numeric_probe', 'ev'), RecordingProbeRunner(''))


class EnvironmentTests(unittest.TestCase):
    def test_replay_uses_the_verifier_c_locale(self):
        # The TKernel OSD probe prints LANG and LC_ALL; the native output was made under C.
        with mock.patch.dict('os.environ', {'LANG': 'en_GB.UTF-8', 'LC_ALL': ''}):
            environment = verifier_environment()
        self.assertEqual((environment['LANG'], environment['LC_ALL']), ('C', 'C'))


class TotalsTests(unittest.TestCase):
    def test_lines_are_summed_per_crate(self):
        lcov = 'SF:/r/crates/tkmath/src/a.rs\nLF:10\nLH:8\nSF:/r/crates/tkmath/src/b.rs\nLF:10\nLH:2\nSF:/r/x.rs\nLF:4\nLH:4\n'
        self.assertEqual(crate_totals(lcov), {'tkmath': (20, 10), 'other': (4, 4)})
        self.assertEqual(percent(20, 10), 50.0)
        self.assertEqual(percent(0, 0), 100.0)


if __name__ == '__main__':
    unittest.main()
