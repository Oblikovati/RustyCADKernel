"""Keep slice and fixed-array trait arguments distinct and losslessly rendered."""

from __future__ import annotations

import unittest

from rustdoc_generic_arguments import display_type, trait_arguments


class RustdocArrayArgumentsTest(unittest.TestCase):
    def test_slice_argument_retains_element_type(self) -> None:
        element = {"primitive": "u16"}
        self.assertEqual(display_type({"slice": element}, {}), "[u16]")
        self.assertEqual(display_type({"slice": {"generic": "T"}}, {}), "[T]")

    def test_fixed_array_retains_const_parameter_or_literal(self) -> None:
        for length in ("N", "CAPACITY", "_Size", "0", "16"):
            with self.subTest(length=length):
                argument = {"array": {"type": {"primitive": "u8"}, "len": length}}
                self.assertEqual(display_type(argument, {}), f"[u8; {length}]")

    def test_nested_arrays_keep_each_dimension(self) -> None:
        inner = {"array": {"type": {"generic": "T"}, "len": "3"}}
        outer = {"array": {"type": inner, "len": "4"}}
        self.assertEqual(display_type(outer, {}), "[[T; 3]; 4]")

    def test_borrowed_array_trait_argument_preserves_mutability(self) -> None:
        array = {"array": {"type": {"primitive": "u8"}, "len": "N"}}
        reference = {
            "borrowed_ref": {"type": array, "lifetime": "'a", "is_mutable": True}
        }
        arguments = {
            "angle_bracketed": {"args": [{"type": reference}], "constraints": []}
        }
        self.assertEqual(trait_arguments(arguments, {}), "<&'a mut [u8; N]>")

    def test_unsupported_lengths_remain_unresolved(self) -> None:
        for length in (None, 3, "", "_", "N + 1", "{ N }", "A::SIZE", "-1", "3usize"):
            with self.subTest(length=length):
                argument = {"array": {"type": {"primitive": "u8"}, "len": length}}
                self.assertIsNone(display_type(argument, {}))

    def test_malformed_arrays_and_unsupported_elements_stay_unresolved(self) -> None:
        for argument in (
            {"array": []},
            {"array": {"len": "N"}},
            {"slice": {"unsupported": "T"}},
            {"array": {"type": {"unsupported": "T"}, "len": "3"}},
        ):
            with self.subTest(argument=argument):
                self.assertIsNone(display_type(argument, {}))
