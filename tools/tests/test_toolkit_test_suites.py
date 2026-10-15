"""Toolkit consolidation preserves discovery, isolation, and legacy case identity."""

from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from test_target_selection import (
    cargo_case_command,
    compiled_case_target,
    original_case_names,
    package_manifests,
    passed_case_names,
)
from toolkit_test_suites import (
    INDEX_SIZE,
    configured_suite_manifests,
    suite_contents,
    suite_sources,
    synchronize_suite,
)
from legacy_test_targets import CargoTestLister
from release_verification_plan import coverage_gate, rust_gates

EXPECTED_COVERAGE_ARGV = (
    "cargo",
    "llvm-cov",
    "nextest",
    "--workspace",
    "--all-features",
    "--locked",
    "--profile",
    "ci",
    "-E",
    "not test(=bvh_bug30655::BVH_PairDistanceTest__Bug30655_SphereDistance) & not test(=sketch_region_prism::owned_request::complete::cancellation_tests::every_complete_checkpoint_cancel_and_native_failure_is_terminal_and_reusable)",
    "--lcov",
    "--output-path",
    "target/lcov.info",
)


class ToolkitCheckoutFake:
    """Small on-disk Cargo checkout with case sources and an isolated process test."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.directory = root / "crates" / "custom"
        self.tests = self.directory / "tests"
        self.tests.mkdir(parents=True)
        self.manifest = self.directory / "Cargo.toml"
        self.manifest.write_text(
            '[package]\nname = "custom-package"\nautotests = false\n'
            '[[test]]\nname = "toolkit"\npath = "tests/toolkit.rs"\n'
            '[[test]]\nname = "fatal"\npath = "tests/fatal.rs"\n'
        )
        for name in ["alpha", "fatal"]:
            (self.tests / f"{name}.rs").write_text("#[test]\nfn case() {}\n")


class CompiledToolkitListerFake(CargoTestLister):
    """Lists known binary cases while recording build requests."""

    def __init__(self, root: Path) -> None:
        super().__init__(root, {})
        self.builds: list[tuple[str, list[str]]] = []

    def build(self, package: str, targets: list[str]) -> dict[str, Path]:
        self.builds.append((package, targets))
        return {target: Path(target) for target in targets}

    def output(self, command: list[str]) -> str:
        if command[0] == "fatal":
            return "fatal_case: test\n"
        return "alpha::case: test\nalpha::nested::case: test\nforeign::case: test\n"


class ToolkitSuiteTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.checkout = ToolkitCheckoutFake(Path(self.temporary.name))
        package_manifests.cache_clear()

    def test_discovery_includes_directory_targets_and_excludes_isolation(self) -> None:
        nested = self.checkout.tests / "nested"
        nested.mkdir()
        (nested / "main.rs").write_text("#[test]\nfn nested_case() {}\n")
        names = [
            source.relative_to(self.checkout.tests).as_posix()
            for source in suite_sources(self.checkout.manifest)
        ]
        self.assertEqual(names, ["alpha.rs", "nested/main.rs"])
        generated = suite_contents(suite_sources(self.checkout.manifest))
        self.assertIn(
            '#[path = "../nested/main.rs"]\npub mod nested;',
            generated["suites/cases_00.rs"],
        )

    def test_nested_case_keeps_implicit_child_directory_at_toolkit_root(self) -> None:
        nested = self.checkout.tests / "alpha"
        nested.mkdir()
        (nested / "child.rs").write_text("#[test]\nfn child_case() {}\n")
        (self.checkout.tests / "alpha.rs").write_text("mod child;\n")
        generated = suite_contents(suite_sources(self.checkout.manifest))
        self.assertIn("mod alpha;\n", generated["toolkit.rs"])
        self.assertNotIn("#[path", generated["toolkit.rs"])
        self.assertEqual(set(generated), {"toolkit.rs"})
        synchronize_suite(self.checkout.manifest, True)
        self.assertEqual(synchronize_suite(self.checkout.manifest, False), [])

    def test_fixture_directory_does_not_change_flat_case_registration(self) -> None:
        fixtures = self.checkout.tests / "alpha"
        fixtures.mkdir()
        (fixtures / "mod.rs").write_text(
            "// Explicit legacy wrapper; not an implicit child.\n"
        )
        generated = suite_contents(suite_sources(self.checkout.manifest))
        self.assertIn("pub mod alpha;", generated["suites/cases_00.rs"])
        self.assertNotIn("mod alpha;", generated["toolkit.rs"])

    def test_public_scoped_and_directory_children_keep_conventional_roots(self) -> None:
        child = self.checkout.tests / "alpha" / "child"
        child.mkdir(parents=True)
        (child / "mod.rs").write_text("#[test]\nfn child_case() {}\n")
        for visibility in ["", "pub ", "pub(crate) "]:
            (self.checkout.tests / "alpha.rs").write_text(f"{visibility}mod child;\n")
            generated = suite_contents(suite_sources(self.checkout.manifest))
            self.assertIn("mod alpha;\n", generated["toolkit.rs"])

    def test_explicit_paths_and_inline_children_remain_indexed(self) -> None:
        child = self.checkout.tests / "alpha"
        child.mkdir()
        (child / "child.rs").write_text("// Existing auxiliary file.\n")
        for declaration in [
            '#[path = "alpha/child.rs"]\nmod child;\n',
            "mod alpha {\n    pub mod child;\n}\n",
        ]:
            (self.checkout.tests / "alpha.rs").write_text(declaration)
            generated = suite_contents(suite_sources(self.checkout.manifest))
            self.assertIn("pub mod alpha;", generated["suites/cases_00.rs"])
            self.assertNotIn("mod alpha;", generated["toolkit.rs"])

    def test_deleted_isolated_source_is_an_error(self) -> None:
        (self.checkout.tests / "fatal.rs").unlink()
        with self.assertRaisesRegex(ValueError, "fatal.rs.*existing test file"):
            suite_sources(self.checkout.manifest)

    def test_new_toolkit_cannot_bypass_consolidation(self) -> None:
        self.assertEqual(
            configured_suite_manifests(self.checkout.root), [self.checkout.manifest]
        )
        self.checkout.manifest.write_text('[package]\nname = "custom-package"\n')
        with self.assertRaisesRegex(
            ValueError, "expected autotests=false and a toolkit target"
        ):
            configured_suite_manifests(self.checkout.root)

    def test_disabled_discovery_requires_a_declared_toolkit_target(self) -> None:
        self.checkout.manifest.write_text(
            '[package]\nname = "custom-package"\nautotests = false\n'
        )
        with self.assertRaisesRegex(ValueError, "expected an explicit toolkit target"):
            configured_suite_manifests(self.checkout.root)

    def test_indexes_are_bounded_and_keep_every_case_namespace(self) -> None:
        generated = suite_contents(
            [Path(f"case_{index}.rs") for index in range(INDEX_SIZE + 1)]
        )
        self.assertEqual(len(generated), 3)
        self.assertEqual(
            sum(text.count("\npub mod ") for text in generated.values()), INDEX_SIZE + 1
        )
        self.assertTrue(
            all(len(text.splitlines()) < 500 for text in generated.values())
        )

    def test_synchronization_detects_new_cases_and_removes_stale_indexes(self) -> None:
        self.assertTrue(synchronize_suite(self.checkout.manifest, False))
        synchronize_suite(self.checkout.manifest, True)
        self.assertEqual(synchronize_suite(self.checkout.manifest, False), [])
        (self.checkout.tests / "new_case.rs").write_text("#[test]\nfn new_case() {}\n")
        stale = self.checkout.tests / "suites" / "cases_99.rs"
        stale.write_text("obsolete")
        self.assertTrue(synchronize_suite(self.checkout.manifest, False))
        synchronize_suite(self.checkout.manifest, True)
        self.assertFalse(stale.exists())
        self.assertEqual(synchronize_suite(self.checkout.manifest, False), [])

    def test_package_lookup_and_compiled_target_preserve_isolation(self) -> None:
        self.assertEqual(
            package_manifests(self.checkout.root),
            {"custom-package": self.checkout.manifest},
        )
        self.assertEqual(
            compiled_case_target(self.checkout.root, "custom-package", "alpha"),
            "toolkit",
        )
        self.assertEqual(
            compiled_case_target(self.checkout.root, "custom-package", "fatal"), "fatal"
        )
        self.assertEqual(
            compiled_case_target(self.checkout.root, "absent", "alpha"), "alpha"
        )

    def test_command_selects_whole_original_namespace_and_retains_flags(self) -> None:
        command = cargo_case_command(
            self.checkout.root, "custom-package", "alpha", ["--list"]
        )
        self.assertEqual(command[-5:], ["--test", "toolkit", "--", "alpha::", "--list"])
        isolated = cargo_case_command(
            self.checkout.root, "custom-package", "fatal", ["--list"]
        )
        self.assertEqual(isolated[-4:], ["--test", "fatal", "--", "--list"])

    def test_identity_removes_only_selected_prefix_and_keeps_unexpected_cases(
        self,
    ) -> None:
        names = {"alpha::nested::case", "other::case", "case"}
        self.assertEqual(
            original_case_names(names, "alpha"), {"nested::case", "other::case", "case"}
        )

    def test_substring_filter_excludes_suffix_collision_without_skipping_selected_cases(
        self,
    ) -> None:
        (self.checkout.tests / "prefixed_alpha.rs").write_text(
            "#[test]\nfn foreign_case() {}\n"
        )
        command = cargo_case_command(
            self.checkout.root, "custom-package", "alpha", ["--list"]
        )
        self.assertEqual(
            command[-5:], ["--", "alpha::", "--skip", "prefixed_alpha::", "--list"]
        )

    def test_legacy_lister_builds_shared_toolkit_once_and_preserves_nested_identity(
        self,
    ) -> None:
        lister = CompiledToolkitListerFake(self.checkout.root)
        listed = lister.list_tests(
            [("custom-package", "alpha"), ("custom-package", "fatal")]
        )
        self.assertEqual(lister.builds, [("custom-package", ["fatal", "toolkit"])])
        self.assertEqual(
            listed,
            {
                ("custom-package", "alpha"): {"case", "nested::case"},
                ("custom-package", "fatal"): {"fatal_case"},
            },
        )


class LibtestResultTests(unittest.TestCase):
    def test_a_case_that_prints_before_its_status_still_passes(self):
        # Font_FontMgr warnings print straight to stdout between the name and the status
        # (the TKV3d StdPrs_BRepFont GTests); failed and ignored cases never pass.
        log = (
            "running 4 tests\n"
            "test fonts::glyph ... Font_FontMgr, warning: unable to find font\n"
            "ok\n"
            "test fonts::plain ... ok\n"
            "test fonts::broken ... FAILED\n"
            "test fonts::skipped ... ignored\n"
            "\n"
            "test result: FAILED. 2 passed; 1 failed; 1 ignored\n"
        )
        self.assertEqual(passed_case_names(log), {"fonts::glyph", "fonts::plain"})


class ToolkitWorkflowTests(unittest.TestCase):
    def test_ci_has_one_linux_workspace_run_and_one_instrumented_coverage_run(
        self,
    ) -> None:
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/ci.yml").read_text()
        self.assertNotIn("matrix.shard", workflow)
        self.assertIn("LLVM_PROFILE_FILE_NAME: martensite-%4m.profraw", workflow)
        self.assertIn("os: [ubuntu-latest]", workflow)
        self.assertNotIn("windows-latest", workflow)
        self.assertNotIn("macos-latest", workflow)
        coverage = workflow.split("  coverage:\n", 1)[1].split("  vulkan-runtime:", 1)[
            0
        ]
        self.assertEqual(coverage.count("--gate coverage"), 1)
        self.assert_single_shared_coverage_command()
        self.assertNotIn("run: cargo clean", coverage)
        self.assertNotIn("run: cargo nextest", coverage)
        self.assertIn("target/nextest/ci/junit.xml", coverage)
        self.assertIn("target/lcov.info", coverage)

    def test_coverage_skips_only_dense_sphere_replay_and_preserves_functional_suites(
        self,
    ) -> None:
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/ci.yml").read_text()
        coverage = workflow.split("  coverage:\n", 1)[1].split("  vulkan-runtime:", 1)[
            0
        ]
        selector = (
            "not test(=bvh_bug30655::BVH_PairDistanceTest__Bug30655_SphereDistance) & not test(=sketch_region_prism::owned_request::complete::cancellation_tests::every_complete_checkpoint_cancel_and_native_failure_is_terminal_and_reusable)"
        )
        argv = coverage_gate().commands[0].argv
        self.assertEqual(coverage.count("--gate coverage"), 1)
        self.assertEqual(argv.count("-E"), 1)
        self.assertEqual(argv[argv.index("-E") + 1], selector)
        functional = workflow.split("  coverage:\n", 1)[0]
        self.assertIn("cargo test-all --no-fail-fast", functional)
        self.assertNotIn("-E ", functional)
        self.assertEqual(
            rust_gates()[1].commands[0].argv, ("cargo", "test-all", "--no-fail-fast")
        )

    def assert_single_shared_coverage_command(self) -> None:
        """Assert the executable registry command, not an obsolete inline YAML copy."""
        gate = coverage_gate()
        self.assertEqual(len(gate.commands), 1)
        self.assertEqual(gate.commands[0].argv, EXPECTED_COVERAGE_ARGV)


if __name__ == "__main__":
    unittest.main()
