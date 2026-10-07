"""Scan Rust sources for the registrations rustdoc cannot show.

rustdoc does not index `impl RuntimeTyped for dyn Trait`, and it never records which
parent a descriptor names, so `impl_transient!` / `impl_transient_interface!` calls are
read from the source itself. Exception factories (`Failure::with_type("Name", ...)`) are
read the same way to prove which C++ exception class a Rust function constructs.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
from typing import Protocol

FACTORY = re.compile(
    r"pub fn (?P<function>\w+)\(\s*(?P<argument>\w+)\s*:\s*&str\s*\)\s*->\s*Failure\s*\{\s*"
    r"Failure::with_type\(\s*\"(?P<name>\w+)\"\s*,\s*(?P=argument)\s*,\s*\"\"\s*\)\s*,?\s*\}"
)


@dataclass(frozen=True)
class TransientRegistration:
    """One descriptor registration; `rust_type` is the macro's spelling (`dyn Curve`)."""

    rust_type: str
    cxx_name: str
    parent_type: str
    interface: bool
    file: str
    line: int
    macro: str = "impl_transient"


@dataclass(frozen=True)
class ExceptionFactory:
    """A `fn name(message: &str) -> Failure` that builds exception `cxx_name`."""

    function: str
    cxx_name: str
    file: str


class RustSourceTree(Protocol):
    def rust_files(self) -> list[tuple[str, str]]: ...

    def text(self, relative: str) -> str | None: ...


class WorkspaceRustSources:
    """List `crates/*/src/**/*.rs` as (relative path, text); deterministic order."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.files: list[tuple[str, str]] | None = None
        self.by_path: dict[str, str] | None = None

    def rust_files(self) -> list[tuple[str, str]]:
        if self.files is None:
            paths = sorted((self.root / "crates").glob("*/src/**/*.rs"))
            self.files = [
                (path.relative_to(self.root).as_posix(), path.read_text(encoding="utf-8"))
                for path in paths
            ]
        return self.files

    def text(self, relative: str) -> str | None:
        if self.by_path is None:
            self.by_path = dict(self.rust_files())
        return self.by_path.get(relative)


def clone_is_derived(tree: RustSourceTree, source_file: str, type_name: str) -> bool:
    """True when `type_name` gets Clone from `#[derive(..Clone..)]`, not a manual impl.

    Example: `clone_is_derived(tree, 'crates/a/src/x.rs', 'Point')`.
    """
    text = tree.text(source_file) or ""
    manual = re.search(rf"\bimpl\b[^{{;]*\bClone for {re.escape(type_name)}\b", text)
    derive = re.search(
        rf"#\[derive\([^)]*\bClone\b[^)]*\)\](?:\s*#\[[^\]]*\]|\s*///[^\n]*)*\s*"
        rf"pub(?:\([^)]*\))? (?:struct|enum) {re.escape(type_name)}\b",
        text,
    )
    return manual is None and derive is not None


def exception_parent_pairs(tree: RustSourceTree) -> set[tuple[str, str]]:
    """`("Child", "Parent")` literals of the Rust exception hierarchy tables."""
    pairs = re.compile(r"\(\s*\"(\w+)\"\s*,\s*\"(\w+)\"\s*\)")
    return {
        (match.group(1), match.group(2))
        for _, text in tree.rust_files()
        if "HIERARCHY" in text or "_PARENT" in text
        for match in pairs.finditer(text)
    }


def pick_registration(
    registrations: list[TransientRegistration], interface: bool
) -> TransientRegistration | None:
    """The registration of the wanted shape (trait or struct), else the first one."""
    matching = [item for item in registrations if item.interface == interface]
    chosen = matching or registrations
    return chosen[0] if chosen else None


def squeeze(text: str) -> str:
    return re.sub(r"\s+", " ", text.strip())


def type_leaf(spelling: str) -> str:
    """`type_leaf('dyn crate::geom::Curve') == 'Curve'`; generics are not resolved."""
    return spelling.removeprefix("dyn ").rsplit("::", 1)[-1].strip()


def exception_factories(tree: RustSourceTree) -> list[ExceptionFactory]:
    """Factories whose whole body is `Failure::with_type("Name", message, "")`."""
    return [
        ExceptionFactory(match.group("function"), match.group("name"), relative)
        for relative, text in tree.rust_files()
        for match in FACTORY.finditer(text)
    ]
