"""Source-TU definitions do not create API candidates without header sites."""

import pathlib
import tempfile
import unittest

from api_inventory.merge import (
    InventoryRecords,
    consume_event,
    declaration_identity,
    exclude_implementation_only_declarations,
    exclude_implementation_only_macros,
)


class SourceTranslationUnitTest(unittest.TestCase):
    def declaration(self, root: pathlib.Path, path: str, usr: str) -> dict:
        source = str(root / path)
        return {
            "event": "declaration",
            "usr": usr,
            "kind": "Function",
            "signature": "void (int)",
            "source": source,
            "qualified_name": usr,
            "api_visible": True,
            "expansion_location": {"file": source, "line": 1, "column": 1},
        }

    def test_header_declaration_survives_while_cxx_only_entity_is_excluded(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            records = InventoryRecords()
            public = self.declaration(root, "include/api.hxx", "public")
            definition = self.declaration(root, "src/api.cxx", "public")
            internal = self.declaration(root, "src/internal.cxx", "internal")
            consume_event(records, public, "api.hxx", root)
            consume_event(records, definition, "api.cxx", root)
            consume_event(records, internal, "internal.cxx", root)
            excluded = exclude_implementation_only_declarations(records)
        self.assertEqual(excluded, 1)
        self.assertTrue(records.declarations[declaration_identity(public)]["api_visible"])
        hidden = records.declarations[declaration_identity(internal)]
        self.assertFalse(hidden["api_visible"])
        self.assertEqual(hidden["exclusion_reason"], "implementation_only_no_header_site")

    def test_source_only_macros_remain_as_excluded_evidence(self) -> None:
        records = InventoryRecords()
        consume_event(
            records,
            {
                "event": "macro",
                "source": "src/private.cxx",
                "name": "PRIVATE",
                "api_candidate": True,
                "location": {},
            },
            "private.cxx",
            pathlib.Path("/"),
        )
        self.assertEqual(exclude_implementation_only_macros(records), 1)
        macro = next(iter(records.macros.values()))
        self.assertFalse(macro["api_candidate"])
        self.assertEqual(
            macro["exclusion_reason"], "implementation_only_no_header_site"
        )


if __name__ == "__main__":
    unittest.main()
