"""Observe libraries loaded by the actual diagnostic child, without guessing from ldd."""

from __future__ import annotations
import hashlib
import json
from pathlib import Path
import subprocess


def library_map_paths(snapshot: str) -> set[Path]:
    """Select mapped shared objects; `library_map_paths(proc_maps)` excludes anonymous mappings."""
    paths = (mapped_library_path(line) for line in snapshot.splitlines())
    return {path for path in paths if path is not None}


def mapped_library_path(line: str) -> Path | None:
    """Keep only complete live library mappings; `mapped_library_path(proc_map_line)`."""
    fields = line.split(maxsplit=5)
    if len(fields) != 6:
        return None
    name = fields[5]
    if not name.startswith("/") or ".so" not in name or name.endswith(" (deleted)"):
        return None
    return Path(name)


def sample_process_maps(pid: int) -> set[Path]:
    """Read only the owned diagnostic child; `sample_process_maps(child.pid)`."""
    try:
        return library_map_paths(Path(f"/proc/{pid}/maps").read_text())
    except (FileNotFoundError, ProcessLookupError):
        return set()


def complete_observed_process(
    child: subprocess.Popen[str],
) -> tuple[str, str, set[Path]]:
    """Drain pipes while observing actual loaded libraries; `complete_observed_process(child)`."""
    libraries: set[Path] = set()
    while True:
        libraries.update(sample_process_maps(child.pid))
        try:
            stdout, stderr = child.communicate(timeout=0.02)
            return stdout, stderr, libraries
        except subprocess.TimeoutExpired:
            continue


def run_observed_command(
    arguments: list[str], root: Path, output: Path
) -> subprocess.CompletedProcess[str]:
    """Retain actual maps and exit, including red tests; `run_observed_command(args, root, prefix)`."""
    child = subprocess.Popen(
        arguments, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
    )
    stdout, stderr, libraries = complete_observed_process(child)
    record_mapped_libraries(child.pid, libraries, output)
    return subprocess.CompletedProcess(arguments, child.returncode, stdout, stderr)


def record_mapped_libraries(pid: int, libraries: set[Path], output: Path) -> None:
    """Identify observed bytes and explicit missing paths; `record_mapped_libraries(pid, libraries, prefix)`."""
    hashes = {
        str(path): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in sorted(libraries)
        if path.is_file()
    }
    payload = {
        "pid": pid,
        "actual_mapped_libraries": hashes,
        "missing_mapped_libraries": [
            str(p) for p in sorted(libraries) if not p.is_file()
        ],
    }
    output.with_suffix(".mapped-libraries.json").write_text(
        json.dumps(payload, indent=2)
    )
