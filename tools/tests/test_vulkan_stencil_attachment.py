"""Independent observed-storage shape checks keep absent/default information honest."""

import unittest
from vulkan_stencil_attachment import validate_attachment


def attachment_record(kind: int, storage: tuple[int, int, int]) -> dict[str, int]:
    return dict(
        framebuffer=0 if kind == 33304 else 1,
        attachment_kind=kind,
        attachment_internal_format=storage[0],
        attachment_stencil_size=storage[1],
        attachment_component_type=storage[2],
    )


class StencilAttachmentTests(unittest.TestCase):
    def test_actual_combined_texture_and_renderbuffer_storage(self) -> None:
        for kind in (5890, 36161):
            validate_attachment(attachment_record(kind, (35056, 8, 35863)))

    def test_missing_attachment_and_unspecified_drawable_are_distinct(self) -> None:
        validate_attachment(attachment_record(0, (-1, -1, -1)))
        validate_attachment(attachment_record(33304, (-1, 8, 35863)))

    def test_fabricated_drawable_format_and_missing_stencil_bits_reject(self) -> None:
        for kind, storage in ((33304, (35056, 8, 35863)), (5890, (35056, 0, 35863))):
            with self.assertRaisesRegex(ValueError, "expected"):
                validate_attachment(attachment_record(kind, storage))

    def test_unknown_storage_kind_and_wrong_drawable_binding_reject(self) -> None:
        with self.assertRaisesRegex(ValueError, "kind=123"):
            validate_attachment(attachment_record(123, (1, 1, 1)))
        row = attachment_record(33304, (-1, 8, 35863))
        row["framebuffer"] = 2
        with self.assertRaisesRegex(ValueError, "framebuffer=2"):
            validate_attachment(row)
