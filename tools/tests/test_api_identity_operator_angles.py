"""Exact C++ operator selectors may contain namespace-qualified template types."""

from __future__ import annotations

import unittest

from api_identity import parse_upstream_id


class OperatorAngleParsingTest(unittest.TestCase):
    def test_less_and_greater_skip_only_the_operator_token_angles(self) -> None:
        for symbol in ("<", ">", "<<", ">>"):
            with self.subTest(symbol=symbol):
                identity = (
                    f"C::operator{symbol}(const std::vector<X::Value>&) const"
                )
                parsed = parse_upstream_id(identity)
                expected = (
                    f"C::operator{symbol}",
                    ["const std::vector<X::Value>&"],
                    "const",
                )
                self.assertEqual(parsed, expected)

    def test_conversion_and_call_operators_keep_their_normal_signatures(self) -> None:
        conversion = parse_upstream_id(
            "C::operator std::basic_string_view<char>() const"
        )
        call = parse_upstream_id("C::operator()(const std::array<int,3>&) const")
        self.assertEqual(
            conversion, ("C::operator std::basic_string_view<char>", [], "const")
        )
        self.assertEqual(call, ("C::operator()", ["const std::array<int,3>&"], "const"))

    def test_missing_parameter_list_remains_a_name_only_identity(self) -> None:
        self.assertEqual(parse_upstream_id("C::operator<"), ("C::operator<", None, ""))

    def test_unclosed_parameter_template_does_not_guess_signature_start(self) -> None:
        malformed = "C::operator<(const std::vector<int>& const"
        self.assertIsNone(parse_upstream_id(malformed))
