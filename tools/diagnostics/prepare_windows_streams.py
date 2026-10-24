"""Materialize controlled Git blobs so Windows checkout conversion cannot alter evidence."""
import json
from pathlib import Path
import subprocess

prefix = "tools/diagnostics/windows_document_streams"
for suffix in [".rs", ".lock", "_fix.patch", "_fix.json"]:
    relative = prefix + suffix
    payload = subprocess.check_output(["git", "show", "HEAD:" + relative])
    Path(relative).write_bytes(payload)
manifest = json.loads(Path(prefix + "_fix.json").read_text())
sdk = Path("target/document-access-sdk")
for relative in manifest["files"]:
    path = Path(relative)
    if path.is_absolute() or ".." in path.parts:
        raise ValueError(f"unexpected source path {relative}; expected a repository-relative file")
    result = subprocess.run(["git", "-C", str(sdk), "show", "HEAD:" + relative], capture_output=True)
    if result.returncode == 0:
        (sdk / path).write_bytes(result.stdout)
