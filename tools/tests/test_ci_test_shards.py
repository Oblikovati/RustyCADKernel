"""Keep every Martensite package tested while bounding hosted build footprints."""

import unittest
import subprocess
from pathlib import Path
from ci_test_shards import (
    PackageWeight,
    shard_packages,
    shard_command,
    metadata_packages,
    run_shard,
)


class CiTestShardTests(unittest.TestCase):
    def test_disk_regression_large_workspace_is_partitioned_without_omissions(
        self,
    ) -> None:
        packages = [
            PackageWeight(f"package-{i}", weight)
            for i, weight in enumerate(
                [429, 251, 160, 113, 92, 86, 73, 64, 48, 48, 44, 37, 37, 36, 31]
            )
        ]
        shards = shard_packages(packages, 6)
        self.assertEqual(
            sorted(p.name for s in shards for p in s), sorted(p.name for p in packages)
        )
        self.assertTrue(all(sum(p.targets for p in s) <= 429 for s in shards))
        self.assertEqual(shards, shard_packages(list(reversed(packages)), 6))

    def test_invalid_shard_counts_fail_instead_of_skipping_tests(self) -> None:
        for count in [0, -1, 3]:
            with self.subTest(count=count), self.assertRaisesRegex(
                ValueError, str(count)
            ):
                shard_packages([PackageWeight("a", 1), PackageWeight("b", 1)], count)

    def test_all_runners_select_exact_packages_and_keep_features_and_lock(self) -> None:
        for runner in ["test", "nextest", "coverage"]:
            command = shard_command(
                runner, [PackageWeight("a", 1), PackageWeight("b", 2)]
            )
            self.assertIn("--all-features", command)
            self.assertIn("--locked", command)
            self.assertEqual(command[-4:], ["--package", "a", "--package", "b"])
            self.assertNotIn("--workspace", command)
        self.assertEqual(
            shard_command("test", [PackageWeight("a", 1)])[:2], ["cargo", "test"]
        )
        self.assertIn("ci", shard_command("nextest", [PackageWeight("a", 1)]))
        self.assertIn(
            "target/lcov.info", shard_command("coverage", [PackageWeight("a", 1)])
        )

    def test_ordinary_tests_collect_all_target_failures(self) -> None:
        command = shard_command("test", [PackageWeight("a", 1)])
        self.assertIn("--no-fail-fast", command)

    def test_empty_shard_and_unknown_runner_are_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "empty"):
            shard_command("test", [])
        with self.assertRaisesRegex(ValueError, "unknown"):
            shard_command("unknown", [PackageWeight("a", 1)])


class FakeCargo:
    def __init__(self, packages: list[PackageWeight]) -> None:
        self.packages = packages
        self.commands: list[list[str]] = []

    def metadata(self, root: Path) -> list[PackageWeight]:
        return self.packages

    def run(self, command: list[str], root: Path) -> None:
        self.commands.append(command)


class CiTestShardExecutionTests(unittest.TestCase):
    def test_metadata_ignores_dependencies_and_includes_targetless_members(
        self,
    ) -> None:
        metadata = {
            "workspace_members": ["a", "b"],
            "packages": [
                {"id": "a", "name": "kernel", "targets": [{}, {}]},
                {"id": "b", "name": "facade", "targets": []},
                {"id": "external", "name": "dependency", "targets": [{}]},
            ],
        }
        self.assertEqual(
            metadata_packages(metadata),
            [PackageWeight("kernel", 2), PackageWeight("facade", 1)],
        )

    def test_every_shard_invokes_cargo_with_its_partition(self) -> None:
        cargo = FakeCargo(
            [PackageWeight("a", 4), PackageWeight("b", 3), PackageWeight("c", 1)]
        )
        run_shard(Path("."), 0, 2, "test", cargo)
        run_shard(Path("."), 1, 2, "test", cargo)
        self.assertEqual(
            cargo.commands[0],
            ["cargo", "test", "--no-fail-fast", "--all-features", "--locked", "--package", "a"],
        )
        self.assertEqual(cargo.commands[1][-4:], ["--package", "b", "--package", "c"])

    def test_invalid_indices_never_launch_cargo(self) -> None:
        cargo = FakeCargo([PackageWeight("a", 1)])
        for index in [-1, 1]:
            with self.assertRaisesRegex(ValueError, str(index)):
                run_shard(Path("."), index, 1, "test", cargo)
        self.assertEqual(cargo.commands, [])


class FailingCargo(FakeCargo):
    def run(self, command: list[str], root: Path) -> None:
        raise subprocess.CalledProcessError(101, command)


class CiTestFailureTests(unittest.TestCase):
    def test_failed_cargo_tests_fail_the_shard(self) -> None:
        cargo = FailingCargo([PackageWeight("a", 1)])
        with self.assertRaises(subprocess.CalledProcessError):
            run_shard(Path("."), 0, 1, "test", cargo)
