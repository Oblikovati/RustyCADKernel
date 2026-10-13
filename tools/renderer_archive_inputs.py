"""Fingerprint source, shader, fixture and normalized manifest inputs on both build roots."""

from __future__ import annotations
import json
from pathlib import Path
from typing import Protocol
from diagnose_release_renderer import digest_file, RendererDiagnosticProcess

INPUT_SUFFIXES = {
    "source": {".rs"},
    "shader": {".spv", ".frag", ".vert", ".glsl"},
    "fixture": {".rgba", ".depth"},
    "manifest": {".toml", ".lock"},
}


class RendererInputStore(Protocol):
    def paths(self, root: Path) -> list[Path]: ...
    def digest(self, path: Path) -> str: ...


class FileRendererInputStore:
    def paths(self, root: Path) -> list[Path]:
        """Select actual build inputs without target caches; `store.paths(root)`."""
        paths = [path for path in (root / "crates").rglob("*") if path.is_file()]
        paths.extend(
            root / name for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml")
        )
        return [path for path in paths if path.is_file()]

    def digest(self, path: Path) -> str:
        """Hash the selected file bytes; `store.digest(path)`."""
        return digest_file(path)


def renderer_input_hashes(
    root: Path, store: RendererInputStore | None = None
) -> dict[str, dict[str, str]]:
    """Preserve comparable relative inputs; `renderer_input_hashes(source_root, input_store)`."""
    selected = store or FileRendererInputStore()
    paths = selected.paths(root)
    return {
        kind: {
            str(path.relative_to(root)): selected.digest(path)
            for path in sorted(paths)
            if matches_input_kind(path, kind)
        }
        for kind in INPUT_SUFFIXES
    }


def matches_input_kind(path: Path, kind: str) -> bool:
    """Retain complete fixture inputs and original manifests; `matches_input_kind(path, 'fixture')`."""
    if kind not in INPUT_SUFFIXES:
        raise ValueError(
            f"received input kind={kind!r}; expected one of {tuple(INPUT_SUFFIXES)}"
        )
    if kind == "fixture":
        return "fixtures" in path.parts
    if kind == "manifest" and path.name == "Cargo.toml.orig":
        return True
    return path.suffix in INPUT_SUFFIXES[kind]


def record_renderer_inputs(
    probe: RendererDiagnosticProcess, label: str
) -> dict[str, dict[str, str]]:
    """Retain an exact input bookend; `record_renderer_inputs(probe, 'inputs-before')`."""
    inputs = renderer_input_hashes(probe.root)
    (probe.output / f"{label}.json").write_text(json.dumps(inputs, sort_keys=True))
    return inputs


def input_differences(
    left: dict[str, str], right: dict[str, str]
) -> dict[str, list[str]]:
    """Separate missing files from changed bytes; `input_differences(source, stage)`."""
    return {
        "missing_from_stage": sorted(left.keys() - right.keys()),
        "extra_in_stage": sorted(right.keys() - left.keys()),
        "changed": sorted(
            name for name in left.keys() & right.keys() if left[name] != right[name]
        ),
    }


def require_input_bookend(
    before: dict[str, dict[str, str]], after: dict[str, dict[str, str]]
) -> None:
    """Reject input mutation rather than certifying stale captures; `require_input_bookend(before, after)`."""
    if before != after:
        changed = {
            kind: input_differences(before.get(kind, {}), after.get(kind, {}))
            for kind in before.keys() | after.keys()
        }
        raise ValueError(
            f"received input mutation={changed}; expected unchanged source/shader/fixture/manifest inputs"
        )


def record_feature_graph(probe: RendererDiagnosticProcess) -> None:
    """Keep actual resolved default and all-feature graphs; `record_feature_graph(probe)`."""
    for label, features in (("default", []), ("all", ["--all-features"])):
        record_metadata_features(probe, label, features)
        record_selected_feature_tree(probe, label, features)
    record_feature_hashes(probe)


def record_metadata_features(
    probe: RendererDiagnosticProcess, label: str, features: list[str]
) -> None:
    """Retain complete resolved workspace metadata; `record_metadata_features(probe, 'default', [])`."""
    result = probe.run(
        f"features-{label}",
        ["cargo", "metadata", "--locked", "--format-version=1", *features],
    )
    if result.returncode:
        raise ValueError(
            f"received metadata exit={result.returncode}; expected Cargo feature graph"
        )
    (probe.output / f"features-{label}.json").write_text(result.stdout)


def record_selected_feature_tree(
    probe: RendererDiagnosticProcess, label: str, features: list[str]
) -> None:
    """Keep the selected Vulkan target's feature tree; `record_selected_feature_tree(probe, 'default', [])`."""
    arguments = [
        "cargo",
        "tree",
        "--locked",
        "-p",
        "martensite-vulkan",
        "-e",
        "features",
        *features,
    ]
    result = probe.run(f"feature-tree-{label}", arguments)
    if result.returncode:
        raise ValueError(
            f"received feature tree exit={result.returncode}; expected selected Vulkan feature graph"
        )


def record_feature_hashes(probe: RendererDiagnosticProcess) -> None:
    """Fingerprint observed feature graphs without discarding raw paths; `record_feature_hashes(probe)`."""
    names = [f"features-{label}.json" for label in ("default", "all")]
    names.extend(f"feature-tree-{label}.stdout" for label in ("default", "all"))
    hashes = {
        name: digest_file(probe.output / name)
        for name in names
        if (probe.output / name).is_file()
    }
    (probe.output / "feature-graphs-sha256.json").write_text(
        json.dumps(hashes, indent=2)
    )
