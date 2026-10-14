"""Named captured-I/O fakes verify strict repeated stencil corpus gates."""

import unittest
import json
from pathlib import Path
from vulkan_ordered_stencil_cases import OrderedStencilCase
from vulkan_stencil_matrix import (
    StencilPrograms,
    stencil_environment,
    stencil_command,
    stencil_prefix,
    check_repeat,
    compare_stencil_case,
    validate_stencil_identities,
    execute_stencil_case,
)


class FakeStencilCaptureIO:
    def __init__(self) -> None:
        self.files: dict[Path, bytes] = {}
        self.commands: list[list[str]] = []
        self.exit_code = 0

    def read(self, path: Path) -> bytes:
        if path not in self.files:
            raise FileNotFoundError(path)
        return self.files[path]

    def run(self, argv: list[str], environment: dict[str, str], log: Path) -> int:
        self.commands.append(argv)
        prefix = argv[-2]
        for suffix in ("rgba", "depth", "stencil-state", "stencil-trace", "draw-trace"):
            self.files[Path(prefix + "." + suffix)] = (
                bytes(16384) if suffix in ("rgba", "depth") else b"state"
            )
        self.files[Path(prefix + ".stencil-trace")] = json.dumps(
            {
                "framebuffer": 1,
                "attachment_kind": 5890,
                "attachment_internal_format": 35056,
                "attachment_stencil_size": 8,
                "attachment_component_type": 35863,
            }
        ).encode()
        self.files[log] = b"Mesa 25.2.8 llvmpipe 128 bits"
        return self.exit_code


def fake_programs() -> StencilPrograms:
    return StencilPrograms(
        Path("/root"),
        Path("/native"),
        Path("/rust"),
        Path("/mesa"),
        Path("/validation"),
        Path("/captures"),
    )


class StencilMatrixTests(unittest.TestCase):
    def test_explicit_profiles_pin_cohort_and_remove_version_overrides(self) -> None:
        env = stencil_environment(
            fake_programs(),
            "software",
            {"MESA_GL_VERSION_OVERRIDE": "9", "GALLIUM_DRIVER": "softpipe"},
        )
        self.assertNotIn("MESA_GL_VERSION_OVERRIDE", env)
        self.assertNotIn("GALLIUM_DRIVER", env)
        self.assertEqual(env["LP_NATIVE_VECTOR_WIDTH"], "128")
        self.assertEqual(
            env["VK_DRIVER_FILES"], "/mesa/usr/share/vulkan/icd.d/lvp_icd.json"
        )

    def test_native_and_rust_commands_bind_original_case_seed_and_wrapper(self) -> None:
        case = OrderedStencilCase("point", "weighted", 129)
        prefix = stencil_prefix(fake_programs(), "software", case, "rust", 2)
        command = stencil_command(fake_programs(), "rust", case, prefix)
        self.assertIn("/root/tools/with_vulkan_validation.py", command)
        self.assertEqual(command[-4:], ["point", "weighted", str(prefix), "129"])
        self.assertEqual(
            stencil_command(fake_programs(), "native", case, prefix)[0], "/native"
        )

    def test_complete_case_executes_four_actual_requests_and_strict_comparison(
        self,
    ) -> None:
        io = FakeStencilCaptureIO()
        result = execute_stencil_case(
            fake_programs(),
            "software",
            OrderedStencilCase("enable", "weighted", 1),
            {},
            io,
        )
        self.assertTrue(result["complete"])
        self.assertEqual(len(io.commands), 4)
        self.assertEqual(result["maximum_depth_delta"], 0.0)

    def test_process_failure_is_preserved_and_never_becomes_complete(self) -> None:
        io = FakeStencilCaptureIO()
        io.exit_code = 7
        result = execute_stencil_case(
            fake_programs(),
            "software",
            OrderedStencilCase("enable", "weighted", 1),
            {},
            io,
        )
        self.assertFalse(result["complete"])
        self.assertEqual(len(io.commands), 4)

    def test_missing_or_different_repeat_bytes_are_not_accepted(self) -> None:
        io = FakeStencilCaptureIO()
        io.files[Path("/first.rgba")] = b"1"
        io.files[Path("/second.rgba")] = b"2"
        with self.assertRaisesRegex(ValueError, "expected identical rgba"):
            check_repeat(io, Path("/first"), Path("/second"), ("rgba",))

    def test_exact_color_and_state_gates_reject_a_changed_result(self) -> None:
        io = FakeStencilCaptureIO()
        io.run(["/native", "/first", "1"], {}, Path("/first.log"))
        io.run(["/rust", "/second", "1"], {}, Path("/second.log"))
        io.files[Path("/second.stencil-state")] = b"changed"
        with self.assertRaisesRegex(ValueError, "exact native context state"):
            compare_stencil_case(io, Path("/first"), Path("/second"), "case")

    def test_wrong_cohort_or_substituted_software_device_is_rejected(self) -> None:
        io = FakeStencilCaptureIO()
        io.files[Path("/first.log")] = b"Mesa 26.1 llvmpipe"
        io.files[Path("/second.log")] = b"Mesa 25.2.8 AMD"
        with self.assertRaisesRegex(ValueError, "actual Mesa25.2.8"):
            validate_stencil_identities(io, Path("/first"), Path("/second"), "software")
