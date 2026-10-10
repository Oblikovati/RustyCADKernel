#!/usr/bin/env python3
"""Diagnose persistence captures at explicit 8-bit native subpixel precision.

This additional control does not replace or change the raw verify_vulkan gate.
"""

from __future__ import annotations

import argparse
from dataclasses import asdict
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile

from verify_vulkan import (
    CASES,
    PERSISTENCE_MODES,
    ROOT,
    build_reference,
    check_reference,
    run,
    source_hashes,
)
from vulkan_capture_artifacts import CaptureStore, FileCaptureStore, export_comparison
from vulkan_profiles import CommandRunner, VulkanProfile, probe_profile


def persistence_arguments(label: str) -> tuple[list[str], str]:
    """Decode signed offsets without ambiguous splitting; e.g. persistence_arguments('48-96-1--3-pers-zoom')."""
    match = re.fullmatch(r"(\d+)-(\d+)-([01])-(-?\d+)-(pers-.+)", label)
    if match is None or match[5] not in PERSISTENCE_MODES:
        raise ValueError(
            f"received capture label {label}; expected a known persistence case"
        )
    return list(match.groups()[:4]), match[5]


def persistence_rows(rows: list[dict[str, object]]) -> list[dict[str, object]]:
    """Require the complete persistence matrix; e.g. persistence_rows(profile['cases'])."""
    expected = {
        "-".join(map(str, (*case, mode)))
        for case in CASES
        for mode in PERSISTENCE_MODES
    }
    selected = [
        row
        for row in rows
        if isinstance(row.get("case"), str) and row["case"] in expected
    ]
    if len(selected) != len(expected) or {row["case"] for row in selected} != expected:
        raise ValueError(
            f"received {len(selected)} persistence cases; expected {len(expected)} unique cases"
        )
    return selected


def require_renderer_sources(
    recorded: dict[str, str], current: dict[str, str]
) -> dict[str, str]:
    """Reject stale renderer captures; e.g. require_renderer_sources(receipt['source_hashes'], now)."""
    inputs = {
        key: value
        for key, value in current.items()
        if key.startswith("crates/") or key in ("Cargo.toml", "Cargo.lock")
    }
    old = {
        key: value
        for key, value in recorded.items()
        if key.startswith("crates/") or key in ("Cargo.toml", "Cargo.lock")
    }
    changed = [
        key for key in inputs.keys() | old.keys() if inputs.get(key) != old.get(key)
    ]
    if not inputs or changed:
        raise ValueError(
            f"received changed renderer inputs {changed}; expected the receipt's exact renderer sources"
        )
    return inputs


def require_profile(recorded: dict[str, object], current: VulkanProfile) -> None:
    """Prevent comparing cached captures from another driver; e.g. require_profile(row, probe)."""
    if (
        recorded["device"] != asdict(current.device)
        or recorded["driver_info"] != current.probe
    ):
        raise ValueError(
            f"received profile {current.name} with changed device/driver; expected the recorded exact profile"
        )


def require_control_output(output: str, original: str) -> None:
    """Require the control and unchanged native driver identity; e.g. require_control_output(stdout, original)."""
    expected = original + "\nMARTENSITE_RASTER_CONTROL=8"
    if output != expected:
        raise ValueError(
            f"received native output {output}; expected original identity plus raster8 marker"
        )


def buffer_fingerprints(prefix: Path, store: CaptureStore) -> dict[str, str]:
    """Bind the actual reused buffers to the control result; e.g. buffer_fingerprints(prefix, store)."""
    return {
        suffix: hashlib.sha256(store.read(Path(f"{prefix}.{suffix}"))).hexdigest()
        for suffix in ("rgba", "depth")
    }


def capture_control(
    native: Path,
    folder: Path,
    profile: VulkanProfile,
    row: dict[str, object],
    runner: CommandRunner,
    store: CaptureStore,
) -> dict[str, object]:
    """Render the precision control against unchanged Vulkan bytes; e.g. capture_control(gl, folder, profile, row)."""
    arguments, mode = persistence_arguments(row["case"])
    control, ported = folder / f"gl-raster8-{row['case']}", Path(row["ported_capture"])
    for suffix in ("rgba", "depth"):
        store.write(Path(f"{control}.{suffix}"), b"")
    output = runner(
        [str(native), *arguments, str(control), mode, "raster8"], profile.environment
    )
    require_control_output(output, row["opengl_runtime"])
    result = export_comparison(
        row["case"],
        int(arguments[0]),
        int(arguments[1]),
        control,
        ported,
        store,
    )
    result.update(
        raw_native_status=row["status"],
        opengl_runtime=output,
        ported_buffer_sha256=buffer_fingerprints(ported, store),
        control_buffer_sha256=buffer_fingerprints(control, store),
    )
    return result


def profile_controls(
    recorded: dict[str, object], native: Path, folder: Path
) -> dict[str, object]:
    """Check every persistence state on the recorded device; e.g. profile_controls(profile, gl, folder)."""
    profile = probe_profile(
        recorded["profile"],
        dict(os.environ),
        list(Path("/usr/share/vulkan/icd.d").glob("*.json")),
        run,
    )
    require_profile(recorded, profile)
    destination = folder / profile.name
    destination.mkdir(parents=True, exist_ok=True)
    cases = [
        capture_control(native, destination, profile, row, run, FileCaptureStore())
        for row in persistence_rows(recorded["cases"])
    ]
    return {
        "profile": profile.name,
        "device": asdict(profile.device),
        "driver_info": profile.probe,
        "status": (
            "passed" if all(row["status"] == "passed" for row in cases) else "failed"
        ),
        "cases": cases,
    }


def verify_controls(arguments: argparse.Namespace) -> dict[str, object]:
    """Require pinned source and source-bound original captures; e.g. verify_controls(args)."""
    original = json.loads(arguments.receipt.read_text())
    current = source_hashes()
    renderer = require_renderer_sources(original["source_hashes"], current)
    check_reference(arguments.source, arguments.build, dict(os.environ))
    with tempfile.TemporaryDirectory(prefix="martensite-raster-control-") as directory:
        native = build_reference(
            arguments.build.resolve(), Path(directory), dict(os.environ)
        )
        profiles = [
            profile_controls(row, native, arguments.captures.resolve())
            for row in original["profiles"]
        ]
    if {row["profile"] for row in profiles} != {"host", "lavapipe"} or len(
        profiles
    ) != 2:
        raise ValueError(
            "received incomplete profiles; expected both host and lavapipe controls"
        )
    return {
        "scope": "Additional 8-bit native raster precision control only; raw acceptance gate is unchanged",
        "status": (
            "passed" if all(row["status"] == "passed" for row in profiles) else "failed"
        ),
        "source_receipt_sha256": hashlib.sha256(
            arguments.receipt.read_bytes()
        ).hexdigest(),
        "renderer_sources_sha256": hashlib.sha256(
            json.dumps(renderer, sort_keys=True).encode()
        ).hexdigest(),
        "control_source_hashes": {
            key: value for key, value in current.items() if key.startswith("tools/")
        },
        "profiles": profiles,
    }


def main() -> int:
    """Publish a separate control receipt; e.g. python3 tools/verify_vulkan_raster_precision.py --help."""
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "build", "receipt", "output", "captures"):
        parser.add_argument(f"--{name}", type=Path, required=True)
    arguments = parser.parse_args()
    result = verify_controls(arguments)
    result["control_source_hashes"][str(Path(__file__).relative_to(ROOT))] = (
        hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    )
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    arguments.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"status": result["status"], "scope": result["scope"]}))
    return 0 if result["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
