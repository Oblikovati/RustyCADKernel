"""Do not remove active API scope because only a parameter or implementation is obsolete."""

import unittest

from api_retirement_markers import (
    comment_status_label,
    enum_alias_group,
    upstream_marker_status,
)


class UpstreamRetirementMarkerTest(unittest.TestCase):
    def test_explicit_group_and_embedded_api_sentence(self) -> None:
        for text in (
            "//! @name deprecated methods",
            "//! @name obsolete methods",
            "//! @name sub-intensity management (deprecated)",
            "//! This constructor calls perform. This constructor is deprecated.",
        ):
            self.assertIsNotNone(comment_status_label(text, "CXXMethod"))
        self.assertIsNone(
            comment_status_label(
                "//! This model emulates obsolete calculations.", "CXXMethod"
            )
        )

    def test_documented_disabled_attribute_and_retired_flag(self) -> None:
        self.assertEqual(
            comment_status_label(
                '// Standard_DEPRECATED("Deprecated method, use replacement")',
                "CXXMethod",
            ),
            "deprecated",
        )
        self.assertEqual(
            comment_status_label(
                "//!< environment flag (obsolete, to be removed)", "EnumConstant"
            ),
            "obsolete",
        )

    def test_enum_group_is_bounded_and_explicitly_reset(self) -> None:
        lines = [
            "// obsolete aliases",
            "enum Mode {",
            " Modern,",
            " // obsolete aliases",
            " Old,",
            " // active aliases",
            " New,",
            "};",
        ]
        self.assertIsNone(enum_alias_group(lines, 2, 3))
        self.assertEqual(
            enum_alias_group(lines, 2, 5), (4, "obsolete", " // obsolete aliases")
        )
        self.assertIsNone(enum_alias_group(lines, 2, 7))

    def test_enum_group_close_and_invalid_interval(self) -> None:
        lines = [
            "enum Mode {",
            "//! @name deprecated aliases",
            "Old,",
            "//! @}",
            "New,",
            "};",
        ]
        self.assertEqual(
            enum_alias_group(lines, 1, 3),
            (2, "deprecated", "//! @name deprecated aliases"),
        )
        self.assertIsNone(enum_alias_group(lines, 1, 5))
        with self.assertRaisesRegex(ValueError, "expected 1 <= enum <= member"):
            enum_alias_group(lines, 0, 3)

    def test_formal_attribute_is_explicit_even_without_message(self) -> None:
        marker = {"kind": "deprecated_attribute", "text": ""}
        self.assertEqual(upstream_marker_status(marker, "CXXMethod"), "deprecated")

    def test_explicit_api_documentation_labels_are_recognized(self) -> None:
        for label in [
            "//! Obsolete class.",
            "//! This is obsolete method for compatibility.",
            "/** This method is now obsolete. */",
            "//! An obsolete field.",
        ]:
            with self.subTest(label=label):
                self.assertEqual(comment_status_label(label, "CXXMethod"), "obsolete")

    def test_doxygen_deprecation_and_enum_trailing_labels(self) -> None:
        self.assertEqual(
            comment_status_label("//! @deprecated Use active().", "CXXMethod"),
            "deprecated",
        )
        self.assertEqual(
            comment_status_label("//! \\deprecated Use active().", "CXXMethod"),
            "deprecated",
        )
        self.assertEqual(
            comment_status_label("//!< display vertices (obsolete)", "EnumConstant"),
            "obsolete",
        )
        self.assertIsNone(
            comment_status_label("//! argument values (obsolete)", "CXXMethod")
        )

    def test_parameters_cache_states_and_old_techniques_do_not_retire_api(self) -> None:
        for label in [
            "//! @param mode obsolete argument for compatibility.",
            "//! Obsolete argument for compatibility.",
            "//! Deprecated parameter value.",
            "//! Marks cached bounding box as obsolete.",
            "//! Returns the limit for obsolete fixed-function rendering.",
            "//! Uses deprecated helper internally.",
        ]:
            with self.subTest(label=label):
                self.assertIsNone(comment_status_label(label, "CXXMethod"))

    def test_unknown_marker_and_malformed_documentation_fail_closed(self) -> None:
        self.assertIsNone(upstream_marker_status({"kind": "unknown"}, "CXXMethod"))
        with self.assertRaisesRegex(ValueError, "text=None; expected a string"):
            upstream_marker_status(
                {"kind": "documentation_candidate", "text": None}, "CXXMethod"
            )
