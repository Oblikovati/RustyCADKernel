#!/usr/bin/env python3
"""Compile the PackedWordReader negative examples as a separate external Rust crate."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/tkernel/src/ncollection/packed_map/integer/native_reader.rs"


def compile_library() -> Path:
    command = ["cargo", "build", "--locked", "-p", "rustycad-tkernel", "--message-format=json"]
    build = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    if build.returncode:
        raise RuntimeError(f"cargo exit {build.returncode}: expected a successful library build\n{build.stderr}")
    artifacts: list[str] = []
    for line in build.stdout.splitlines():
        record = json.loads(line)
        if record.get("reason") == "compiler-artifact" and record["target"]["name"] == "rustycad_tkernel":
            artifacts.extend(name for name in record["filenames"] if name.endswith(".rlib"))
    if len(artifacts) != 1:
        raise RuntimeError(f"library artifacts {artifacts}: expected exactly one rlib")
    return Path(artifacts[0])


def source_examples() -> list[tuple[str, str]]:
    pattern = r"/// ```compile_fail,(E\d+)\n(.*?)/// ```"
    examples = []
    for code, block in re.findall(pattern, SOURCE.read_text(), re.DOTALL):
        body = "\n".join(line.removeprefix("/// ") for line in block.splitlines())
        examples.append((code, f"fn main() {{\n{body}\n}}\n"))
    if len(examples) != 3:
        raise RuntimeError(f"privacy examples {len(examples)}: expected exactly three")
    return examples


def error_codes(stderr: str) -> set[str]:
    codes: set[str] = set()
    for line in stderr.splitlines():
        record = json.loads(line)
        if record.get("level") == "error" and record.get("code") is not None:
            codes.add(record["code"]["code"])
    return codes


def compile_example(rlib: Path, folder: Path, index: int, example: tuple[str, str]) -> dict[str, object]:
    code, source = example
    path = folder / f"reader_privacy_{index}.rs"
    path.write_text(source)
    command = ["rustc", "--edition=2024", "--error-format=json", str(path), "--out-dir", str(folder),
               "--extern", f"rustycad_tkernel={rlib}", "-L", f"dependency={rlib.parent / 'deps'}"]
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    observed = error_codes(result.stderr)
    if result.returncode == 0 or observed != {code}:
        raise RuntimeError(f"privacy probe {index}: exit {result.returncode}, errors {observed}; expected only {code}")
    return {"index": index, "expected_error": code, "actual_errors": sorted(observed),
            "exit": result.returncode, "command": command, "source": source,
            "source_sha256": hashlib.sha256(source.encode()).hexdigest(), "diagnostics": result.stderr}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    rlib = compile_library()
    with tempfile.TemporaryDirectory(prefix="packed-word-reader-privacy-") as directory:
        rows = [compile_example(rlib, Path(directory), index, item)
                for index, item in enumerate(source_examples())]
    receipt = {"library": str(rlib), "library_sha256": hashlib.sha256(rlib.read_bytes()).hexdigest(), "probes": rows}
    args.output.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"privacy_probes": len(rows), "expected_rejections": len(rows)}))


if __name__ == "__main__":
    main()
