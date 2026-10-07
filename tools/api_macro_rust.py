"""Query the normalized rustdoc inventory for the trait impls that carry C++ special members.

Paths follow the inventory convention `crate::path::Type::<trait path>::item`, e.g.
`rustycad_tkg2d::geom2d::Geom2dLine::<core::clone::Clone>::clone`.
"""

from __future__ import annotations

from collections import defaultdict
import json

from api_inventory.inputs import JSONObject, JSONValue

RUNTIME_TYPED = "rustycad_tkernel::standard::RuntimeTyped"
TRANSIENT = "rustycad_tkernel::standard::Transient"
CLONE = "core::clone::Clone"
DROP = "core::ops::drop::Drop"
TYPE_KINDS = {"struct", "enum", "union", "trait", "type_alias"}
FACADE_CRATE = "rustycad_kernel"


def trait_item(owner: str, trait: str, item: str) -> str:
    """Inventory path of a trait item; `trait_item('c::T', CLONE, 'clone')`."""
    return f"{owner}::<{trait}>::{item}"


class RustInventoryIndex:
    """Path lookups over one normalized inventory; `RustInventoryIndex(doc).kinds(path)`."""

    def __init__(self, document: JSONObject) -> None:
        self.by_path: dict[str, list[JSONObject]] = defaultdict(list)
        crates = document.get("crates")
        for crate in crates if isinstance(crates, list) else []:
            for item in crate.get("declarations", []) if isinstance(crate, dict) else []:
                if isinstance(item, dict) and isinstance(item.get("path"), str):
                    self.by_path[item["path"]].append(item)
        self.type_name_owners = self._runtime_type_names()
        self.trait_items: dict[tuple[str, str], list[str]] = defaultdict(list)
        for path in sorted(self.by_path):
            owner, separator, rest = path.partition("::<")
            if separator and ">::" in rest:
                self.trait_items[(owner, rest.rsplit(">::", 1)[1])].append(path)

    def dynamic_type_item(self, owner: str) -> str | None:
        """The `dynamic_type` that answers C++ DynamicType for `owner`.

        `Transient::dynamic_type` is preferred; a class family with its own virtual
        root (alerts implement `message::Alert::dynamic_type`) qualifies when it is the
        only project trait that declares the method for the type.
        """
        preferred = trait_item(owner, TRANSIENT, "dynamic_type")
        if self.unique(preferred):
            return preferred
        found = [
            path
            for path in self.trait_items.get((owner, "dynamic_type"), [])
            if path.partition("::<")[2].startswith("rustycad_tk") and self.unique(path)
        ]
        return found[0] if len(found) == 1 else None

    def kinds(self, path: str) -> list[str]:
        """Declaration kinds at one path; an empty list when the path is absent."""
        return [str(item.get("kind")) for item in self.by_path.get(path, [])]

    def unique(self, path: str) -> bool:
        """True when exactly one declaration answers `path`, as the crosswalk requires."""
        return len(self.by_path.get(path, [])) == 1

    def is_type(self, path: str) -> bool:
        kinds = self.kinds(path)
        return len(kinds) == 1 and kinds[0] in TYPE_KINDS

    def has_trait_item(self, owner: str, trait: str, item: str) -> bool:
        return self.unique(trait_item(owner, trait, item))

    def _runtime_type_names(self) -> dict[str, list[str]]:
        """Map each `RuntimeTyped::TYPE_NAME` string literal to its non-facade owners."""
        owners: dict[str, list[str]] = defaultdict(list)
        suffix = f"::<{RUNTIME_TYPED}>::TYPE_NAME"
        for path, items in sorted(self.by_path.items()):
            name = literal_value(items[0].get("signature")) if len(items) == 1 else None
            if path.endswith(suffix) and name and not path.startswith(FACADE_CRATE + "::"):
                owners[name].append(path[: -len(suffix)])
        return dict(owners)


def literal_value(signature: JSONValue) -> str | None:
    """Decode a string-literal const value; `'"Geom2d_Line"'` -> `Geom2d_Line`."""
    value = signature.get("value") if isinstance(signature, dict) else None
    if not isinstance(value, str) or not value.startswith('"'):
        return None
    decoded = json.loads(value)
    return decoded if isinstance(decoded, str) else None


def crate_of(path: str) -> str:
    """`crate_of('rustycad_tkg2d::geom2d::X') == 'rustycad_tkg2d'`."""
    return path.split("::", 1)[0]


def preferred_path(paths: list[str], crate: str) -> str | None:
    """Pick one public path deterministically, preferring the toolkit's own crate."""
    own = sorted((len(path), path) for path in paths if crate_of(path) == crate)
    if own:
        return own[0][1]
    others = sorted({crate_of(path) for path in paths})
    if len(others) != 1:
        return None
    return sorted((len(path), path) for path in paths)[0][1]
