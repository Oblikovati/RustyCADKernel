#!/usr/bin/env python3
"""Enumerate pinned legacy renderer sources and controls without claiming implementation parity."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

PIN = "3d097a0328e71b826377d4814ab05ec3c3d23871"
PARAMS = "src/Visualization/TKService/Graphic3d/Graphic3d_RenderingParams.hxx"
ROOTS = (
    "src/Visualization/TKOpenGl",
    "src/Visualization/TKOpenGles",
    "src/Visualization/TKService/Graphic3d",
    "src/Visualization/TKService/Shaders",
    "src/Visualization/TKV3d/V3d",
    "src/Draw/TKOpenGlTest",
    "tests/opengl",
    "tests/v3d",
)


def rendering_controls(source: str) -> list[dict[str, str | int]]:
    """Extract pinned RenderingParams fields and source lines; e.g. rendering_controls(header)."""
    result = []
    section = ""
    for line, text in enumerate(source.splitlines(), 1):
        if "public: //! @name" in text:
            section = text.split("@name", 1)[1].strip()
        match = re.match(r"\s+([\w:<>]+(?:\s+int)?)\s+(\w+)\s*;", text)
        if match and section:
            result.append(
                {
                    "name": match[2],
                    "type": match[1],
                    "section": section,
                    "source": PARAMS,
                    "line": line,
                    "status": "pending",
                }
            )
    return result


def source_manifest(source: Path) -> list[dict[str, str]]:
    """Retain every candidate source/regression file for capability review; e.g. source_manifest(occt)."""
    paths = sorted(
        {p for root in ROOTS for p in (source / root).rglob("*") if p.is_file()}
    )
    return [
        {
            "path": str(p.relative_to(source)),
            "sha256": hashlib.sha256(p.read_bytes()).hexdigest(),
            "review_status": "pending",
        }
        for p in paths
    ]


def baseline(source: Path) -> dict[str, object]:
    """Require a pinned clean reference before enumerating; e.g. baseline(occt)."""
    require_pinned_source(source)
    return {
        "schema": 1,
        "upstream_revision": PIN,
        "implementation": "Vulkan 1.3",
        "scope": "legacy OCCT OpenGL capabilities; no additional renderer features",
        "review_status": "enumerated candidates; semantic capability audit remains open",
        "roots": ROOTS,
        "rendering_controls": rendering_controls((source / PARAMS).read_text()),
        "files": source_manifest(source),
    }


def require_pinned_source(source: Path) -> None:
    """Reject changed upstream inputs; e.g. require_pinned_source(occt)."""
    revision = subprocess.check_output(
        ["git", "-C", str(source), "rev-parse", "HEAD"], text=True
    ).strip()
    dirty = subprocess.check_output(
        ["git", "-C", str(source), "status", "--porcelain", "--untracked-files=no"],
        text=True,
    )
    if revision != PIN or dirty:
        raise ValueError(
            f"received {revision=} {dirty=}; expected clean upstream {PIN}"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    inventory = baseline(args.source)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(inventory, indent=2) + "\n")
    print(
        f"Legacy renderer candidates: {len(inventory['files'])} files, {len(inventory['rendering_controls'])} rendering controls; review pending"
    )


if __name__ == "__main__":
    main()
