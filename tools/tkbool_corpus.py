"""Deterministic request corpus for the TKBool numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkbool_probe.cxx and
crates/tkbool/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkbool_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkbool_requests(requests: list[str]) -> None:
    """Append every TKBool request family in a fixed order."""
    # BRepFill_Generator (tkbool_corpus_brepfill_generator.py)
    __import__('tkbool_corpus_brepfill_generator').append_generator_requests(requests)
    # The BRepFill package functions (tkbool_corpus_brepfill_package.py)
    __import__('tkbool_corpus_brepfill_package').append_package_requests(requests)
    # BRepFill_CompatibleWires (tkbool_corpus_brepfill_compatible.py)
    __import__('tkbool_corpus_brepfill_compatible').append_compatible_requests(requests)
    # BRepFill_ShapeLaw and BRepFill_NSections (tkbool_corpus_brepfill_section_law.py)
    __import__('tkbool_corpus_brepfill_section_law').append_section_law_requests(requests)
    # The BRepFill location laws (tkbool_corpus_brepfill_location_law.py)
    __import__('tkbool_corpus_brepfill_location_law').append_location_law_requests(requests)
    # BRepFill_Section and BRepFill_SectionPlacement (tkbool_corpus_brepfill_section.py)
    __import__('tkbool_corpus_brepfill_section').append_section_family_requests(requests)
    # BRepFill_Sweep and BRepFill_TrimShellCorner (tkbool_corpus_brepfill_sweep.py)
    __import__('tkbool_corpus_brepfill_sweep').append_sweep_requests(requests)
    # BRepFill_PipeShell (tkbool_corpus_brepfill_pipe_shell.py)
    __import__('tkbool_corpus_brepfill_pipe_shell').append_pipe_shell_requests(requests)
    # BRepFill_Pipe (tkbool_corpus_brepfill_pipe.py)
    __import__('tkbool_corpus_brepfill_pipe').append_pipe_requests(requests)
    # BRepFill_Draft (tkbool_corpus_brepfill_draft.py)
    __import__('tkbool_corpus_brepfill_draft').append_draft_requests(requests)
    # BRepFill_Filling (tkbool_corpus_brepfill_filling.py)
    __import__('tkbool_corpus_brepfill_filling').append_filling_requests(requests)
    # BRepFill_OffsetWire (tkbool_corpus_brepfill_offset_wire.py)
    __import__('tkbool_corpus_brepfill_offset_wire').append_offset_wire_requests(requests)
    # BRepFill_MultiLine, ApproxSeewing, TrimSurfaceTool (tkbool_corpus_brepfill_multi_line.py)
    __import__('tkbool_corpus_brepfill_multi_line').append_multi_line_requests(requests)
    # BRepFill_AdvancedEvolved (tkbool_corpus_brepfill_advanced_evolved.py)
    __import__('tkbool_corpus_brepfill_advanced_evolved').append_advanced_evolved_requests(requests)
    # BRepAlgo (tkbool_corpus_brepalgo.py)
    __import__('tkbool_corpus_brepalgo').append_brepalgo_requests(requests)
    # BRepAlgo_NormalProjection and BRepProj (tkbool_corpus_brepproj.py)
    __import__('tkbool_corpus_brepproj').append_brepproj_requests(requests)
    # TopOpeBRepTool_CurveTool (tkbool_corpus_topopebreptool_curve_tool.py)
    __import__('tkbool_corpus_topopebreptool_curve_tool').append_curve_tool_requests(requests)
    # TopOpeBRepDS (tkbool_corpus_topopebrepds.py)
    __import__('tkbool_corpus_topopebrepds').append_topopebrepds_requests(requests)
    __import__('tkbool_corpus_curve_explorer').append_curve_explorer_requests(requests)
    __import__('tkbool_corpus_point_explorer').append_point_explorer_requests(requests)
    __import__('tkbool_corpus_interference_tool').append_interference_tool_requests(requests)
    # TopOpeBRepTool shape helpers (tkbool_corpus_topopebreptool_shapes.py)
    __import__('tkbool_corpus_topopebreptool_shapes').append_topopebreptool_shapes_requests(requests)
    # TopOpeBRepDS data structure and build tool (tkbool_corpus_topopebrepds_build.py)
    __import__('tkbool_corpus_topopebrepds_build').append_topopebrepds_build_requests(requests)
    # TopOpeBRepTool classifiers (tkbool_corpus_topopebreptool_classifier.py)
    __import__('tkbool_corpus_topopebreptool_classifier').append_topopebreptool_classifier_requests(requests)
    # TopOpeBRepBuild face chain (tkbool_corpus_topopebrepbuild.py)
    __import__('tkbool_corpus_topopebrepbuild').append_topopebrepbuild_requests(requests)
    __import__('tkbool_corpus_brepfill_evolved').append_evolved_requests(requests)
    # TopOpeBRepTool TOOL, TOPOLOGY, transitions and regularization (tkbool_corpus_topopebreptool_tool.py)
    __import__('tkbool_corpus_topopebreptool_tool').append_topopebreptool_tool_requests(requests)
