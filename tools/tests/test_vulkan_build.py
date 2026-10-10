"""A shared Cargo target must not substitute another checkout's renderer executable."""

import unittest
from pathlib import Path
from vulkan_build import build_vulkan_examples, vulkan_build_environment


class FakeVulkanCargo:
    def __init__(self) -> None:
        self.calls: list[tuple[list[str], dict[str, str]]] = []

    def __call__(self, command: list[str], environment: dict[str, str]) -> str:
        self.calls.append((command, environment.copy()))
        return "all checks passed"


class VulkanBuildTests(unittest.TestCase):
    def test_inherited_shared_target_cannot_choose_a_foreign_worktree_binary(
        self,
    ) -> None:
        workspace = Path("/workspace/Martensite")
        environment = {"CARGO_TARGET_DIR": "/shared/target", "DISPLAY": ":1"}
        cargo = FakeVulkanCargo()
        target, tests = build_vulkan_examples(workspace, environment, cargo)
        expected = workspace / "target/vulkan-harness"
        self.assertEqual(target, expected)
        self.assertEqual(tests, "all checks passed")
        self.assertEqual(environment["CARGO_TARGET_DIR"], "/shared/target")
        for command, received in cargo.calls:
            self.assertEqual(received["CARGO_TARGET_DIR"], str(expected), command)
            self.assertEqual(received["DISPLAY"], ":1")
        self.assertTrue(any("--release" in command for command, _ in cargo.calls))

    def test_each_checkout_owns_a_separate_cache(self) -> None:
        first = vulkan_build_environment(Path("/first"), {})
        second = vulkan_build_environment(Path("/second"), first)
        self.assertEqual(first["CARGO_TARGET_DIR"], "/first/target/vulkan-harness")
        self.assertEqual(second["CARGO_TARGET_DIR"], "/second/target/vulkan-harness")
