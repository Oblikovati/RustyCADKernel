"""Context capture scoping preserves all eligible real translation units."""

from __future__ import annotations

import unittest
from pathlib import Path

from retry_occt_header_context import CompileContext, contexts_for_failed_toolkits


class RetryScopeTests(unittest.TestCase):
    def test_filters_only_toolkits_that_matching_contexts_already_rejects(self) -> None:
        contexts = [
            CompileContext(Path(path), Path("/build"), ())
            for path in [
                "/occt/src/FoundationClasses/TKernel/Standard/a.cxx",
                "/occt/src/ModelingAlgorithms/TKBO/BOPAlgo/a.cxx",
                "/outside/src/FoundationClasses/TKernel/Standard/a.cxx",
                "/occt/src/short.cxx",
            ]
        ]
        selected = contexts_for_failed_toolkits(
            contexts, [{"toolkit": "TKernel"}], Path("/occt/src")
        )
        self.assertEqual(selected, contexts[:1])
        self.assertFalse(contexts_for_failed_toolkits(contexts, [], Path("/occt/src")))
        self.assertEqual(
            contexts_for_failed_toolkits(
                contexts,
                [{"toolkit": "TKernel"}, {"toolkit": "TKBO"}],
                Path("/occt/src"),
            ),
            contexts[:2],
        )


if __name__ == "__main__":
    unittest.main()
