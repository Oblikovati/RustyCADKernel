"""Legacy corpus CLI ordering and threshold behavior stay intact after extraction."""

from pathlib import Path
import tempfile
import unittest
from unittest import mock

import coverage_report


class NamedCoverageCorpusCommands:
    """Return only checked fake report contents, recording every intended command."""

    def __init__(self) -> None:
        """Keep the explicit observer state; see the enclosing named fixture."""

        self.calls: list[list[str]] = []

    def run(
        self,
        argv: list[str],
        stdin: str | None,
        environment: dict[str, str] | None = None,
    ) -> str:
        self.calls.append(argv)
        if argv[:3] == ["cargo", "llvm-cov", "report"]:
            output = Path(argv[-1])
            output.parent.mkdir()
            output.write_text("SF:/kernel/crates/cmath/src/lib.rs\nLF:4\nLH:4\n")
        return ""

    def replay(
        self, corpus: coverage_report.ProbeCorpus, run: coverage_report.Runner
    ) -> int:
        self.calls.append(["replay", corpus.package])
        return 2


class CoverageReportOrchestrationTests(unittest.TestCase):
    def test_corpus_accumulation_keeps_clean_workspace_features_and_all_probes(
        self,
    ) -> None:
        fake = NamedCoverageCorpusCommands()
        with tempfile.TemporaryDirectory() as tmp, mock.patch.object(
            coverage_report, "ROOT", Path(tmp)
        ):
            with mock.patch.object(
                coverage_report, "replay_probe", fake.replay
            ), mock.patch("sys.stdout"):
                report = coverage_report.run_coverage_corpora(fake.run)
            self.assertTrue(report.is_file())
        self.assertEqual(fake.calls[0], ["cargo", "llvm-cov", "clean", "--workspace"])
        self.assertEqual(
            fake.calls[2:-1],
            [["replay", corpus.package] for corpus in coverage_report.PROBE_CORPORA],
        )

    def test_workspace_corpus_arguments_preserve_the_original_all_feature_graph(
        self,
    ) -> None:
        expected = (
            "cargo llvm-cov --no-report --workspace --all-features --locked".split()
        )
        self.assertEqual(coverage_report.workspace_corpus_test_arguments(), expected)

    def test_totals_keep_exact_original_minimum_failure(self) -> None:
        lcov = "SF:/kernel/crates/cmath/src/lib.rs\nLF:4\nLH:3\n"
        with mock.patch("sys.stdout"):
            coverage_report.print_coverage_totals(lcov, 75.0)
            with self.assertRaises(SystemExit) as caught:
                coverage_report.print_coverage_totals(lcov, 80.0)
        self.assertEqual(
            str(caught.exception), "coverage 75.0% is below the 80.0% minimum"
        )

    def test_legacy_main_only_orchestrates_the_injected_existing_operations(
        self,
    ) -> None:
        fake = NamedCoverageCorpusCommands()
        with tempfile.TemporaryDirectory() as tmp, mock.patch.object(
            coverage_report, "ROOT", Path(tmp)
        ):
            with mock.patch.object(
                coverage_report, "run_command", fake.run
            ), mock.patch.object(coverage_report, "replay_probe", fake.replay):
                with mock.patch(
                    "sys.argv", ["coverage_report", "--minimum", "100"]
                ), mock.patch("sys.stdout"):
                    coverage_report.main()
        self.assertEqual(len(fake.calls), len(coverage_report.PROBE_CORPORA) + 3)


if __name__ == "__main__":
    unittest.main()
