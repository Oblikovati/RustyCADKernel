"""Constructor narrowing must preserve actual parent, kind, order and ambiguity."""

from collections.abc import Iterator
import unittest

from api_inventory.inputs import JSONValue
from api_template_selectors import (
    constructors_for_template,
    template_constructor_name_groups,
)
from api_rust_candidates import requested_kind_paths


class UnrelatedConstructorRowsFake(list[dict[str, JSONValue]]):
    def __iter__(self) -> Iterator[dict[str, JSONValue]]:
        """Reject unrelated declaration scans; `iter(rows)` reports a regression."""
        operation = "unrelated declarations"
        raise AssertionError(
            f"operation={operation!r}; expected constructor name groups only"
        )


def constructor_lookup_row(
    name: str, parent: str = "owner", kind: str = "CXXConstructor"
) -> dict[str, JSONValue]:
    """Build a compiler declaration fixture; `constructor_lookup_row("Buffer::Buffer")`."""
    return {"qualified_name": name, "parent_usr": parent, "kind": kind}


class TemplateConstructorLookupTests(unittest.TestCase):
    def test_matching_names_still_require_actual_parent_and_constructor_kind(
        self,
    ) -> None:
        plain = constructor_lookup_row("ns::Buffer::Buffer")
        generic = constructor_lookup_row(
            "ns::Buffer::Buffer<T>", kind="FunctionTemplate"
        )
        index = {
            "ns::Buffer::Buffer": [
                plain,
                constructor_lookup_row("ns::Buffer::Buffer", "other"),
            ],
            "ns::Buffer::Buffer<T>": [
                generic,
                constructor_lookup_row("ns::Buffer::Buffer<T>", kind="CXXMethod"),
            ],
            "ns::Buffer::Resize": UnrelatedConstructorRowsFake(
                [constructor_lookup_row("ns::Buffer::Resize")]
            ),
        }
        templates = [{"usr": "owner", "qualified_name": "ns::Buffer"}]
        self.assertEqual(constructors_for_template(templates, index), [plain, generic])

    def test_duplicate_actual_constructors_remain_ambiguous_in_original_order(
        self,
    ) -> None:
        first = constructor_lookup_row("Buffer::Buffer")
        second = {**first, "id": "second"}
        index = {"Buffer::Buffer": [first, second]}
        templates = [{"usr": "owner", "qualified_name": "Buffer"}]
        self.assertEqual(constructors_for_template(templates, index), [first, second])

    def test_missing_or_invalid_owners_cannot_select_unrelated_rows(self) -> None:
        index = {"Buffer::Buffer": UnrelatedConstructorRowsFake([])}
        self.assertEqual(template_constructor_name_groups(index, {}), [])
        self.assertEqual(template_constructor_name_groups(index, {"bad": None}), [])
        self.assertEqual(
            template_constructor_name_groups(index, {"owner": "Absent"}), []
        )

    def test_kind_targets_do_not_decide_paired_target_validity(self) -> None:
        self.assertEqual(
            requested_kind_paths({"rust_item": "crate::first"}), {"crate::first"}
        )
        row: dict[str, JSONValue] = {"rust_items": ["crate::second", None, 3]}
        self.assertEqual(requested_kind_paths(row), {"crate::second"})
        self.assertEqual(requested_kind_paths({}), set())
