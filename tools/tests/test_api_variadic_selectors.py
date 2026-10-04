"""Variadic selectors must match the separate compiler ellipsis flag exactly."""

from __future__ import annotations

import unittest

from api_crosswalk import fixed_selector_parameters, parameter_match


class VariadicSelectorTests(unittest.TestCase):
    def test_printf_ellipsis_is_separate_from_named_parameters(self) -> None:
        declaration = {"parameter_types": ["const char *const"], "variadic": True}
        self.assertTrue(parameter_match(["const char*", "..."], declaration))
        self.assertFalse(parameter_match(["const char*"], declaration))
        self.assertFalse(parameter_match(["char*", "..."], declaration))

    def test_nonvariadic_overload_cannot_match_ellipsis(self) -> None:
        declaration = {"parameter_types": ["const char*"], "variadic": False}
        self.assertFalse(parameter_match(["const char*", "..."], declaration))
        self.assertTrue(parameter_match(["const char*"], declaration))

    def test_malformed_or_unrecorded_variadic_contract_stays_unresolved(self) -> None:
        self.assertIsNone(fixed_selector_parameters(["...", "int"], {"variadic": True}))
        self.assertIsNone(fixed_selector_parameters(["..."], {}))
        self.assertIsNone(fixed_selector_parameters(["..."], {"variadic": "true"}))
        self.assertEqual(fixed_selector_parameters(["..."], {"variadic": True}), [])


if __name__ == "__main__":
    unittest.main()
