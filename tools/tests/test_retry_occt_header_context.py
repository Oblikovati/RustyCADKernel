"""Tests for source-backed retries of failed OCCT header probes."""

from __future__ import annotations

import pathlib
import tempfile
import unittest

from retry_occt_header_context import (
    CompileContext,
    artifact_reference,
    include_directives,
    include_roots,
    index_source_includes,
    matching_contexts,
    resolves_to_header,
)


class RetryHeaderContextTest(unittest.TestCase):
    def test_artifact_reference_is_relative_to_retry_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            output = root / "artifacts" / "probe.jsonl"
            output.parent.mkdir()
            output.write_text("{}\n")
            self.assertEqual(artifact_reference(output, root), "artifacts/probe.jsonl")

    def test_include_roots_separate_quote_and_system_search_paths(self) -> None:
        roots = include_roots(
            (
                "-iquote",
                "/project/quote",
                "-I/project/include",
                "-isystem",
                "/usr/include",
            )
        )
        self.assertEqual([str(path) for path in roots[0]], ["/project/quote"])
        self.assertEqual([str(path) for path in roots[1]], ["/project/include"])
        self.assertEqual([str(path) for path in roots[2]], ["/usr/include"])

    def test_resolves_direct_include_against_compiler_search_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            header = root / "include" / "Part.hxx"
            source = root / "src" / "Part.cxx"
            header.parent.mkdir()
            source.parent.mkdir()
            header.write_text("class Part {};\n")
            source.write_text("#include <Part.hxx>\n")
            self.assertEqual(include_directives(source), [("<", "Part.hxx")])
            found, name = resolves_to_header(source, (f"-I{header.parent}",), header)
        self.assertTrue(found)
        self.assertEqual(name, "Part.hxx")

    def test_index_follows_generated_wrapper_to_exact_occt_source_header(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source_root = root / "OCCT" / "src"
            source_dir = source_root / "Module" / "TKThing" / "Package"
            source_dir.mkdir(parents=True)
            header = source_dir / "Thing.hxx"
            unit = source_dir / "Thing.cxx"
            wrapper_dir = root / "build" / "include"
            wrapper_dir.mkdir(parents=True)
            wrapper = wrapper_dir / "Thing.hxx"
            header.write_text("class Thing {};\n")
            wrapper.write_text(f'#include "{header}"\n')
            unit.write_text("#include <Thing.hxx>\n")
            context = CompileContext(unit, root, (f"-I{wrapper_dir}",))
            index = index_source_includes([context], source_root)
            matches = matching_contexts(header, "TKThing", index, source_root)
        self.assertEqual(len(matches), 1)
        self.assertEqual(matches[0], (context, "Thing.hxx"))


if __name__ == "__main__":
    unittest.main()
