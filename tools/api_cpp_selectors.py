"""Explicit ledger pins that pick one C++ declaration among same-name declarations.

A qualified name with parameters cannot tell apart a class template from its
specializations, an Enum from a Typedef of the same name, two typedefs in two headers,
a using-declaration from the method it shares a name with, or overloads that differ
only in a SFINAE return type. A row names the declaration it claims with these fields:

- `cpp_selector`: an object whose keys each must equal the declaration field exactly:
  `kind` (Clang cursor kind), `source` (OCCT header path), `usr` (Clang USR, which
  spells template arguments) and `signature` (declaration signature, whitespace
  normalized). Example: `{"kind": "ClassTemplateSpecialization",
  "usr": "c:@N@std@S@hash>#$@S@TopoDS_Shape"}`.
- `cpp_declaration_id`: the inventory `id` of the declaration (the macro row
  generator writes it); the crosswalk accepts only that declaration.

The crosswalk never guesses: a pin that admits zero or several declarations leaves the
row unresolved.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from api_identity import norm_cpp
from api_inventory.inputs import JSONValue

SELECTOR_KEYS = frozenset({"kind", "source", "usr", "signature"})
INVALID_PIN = "invalid_cpp_selector"


@dataclass(frozen=True)
class CppPin:
    """Declaration filter a row declares; `CppPin(None, {"kind": "Enum"}).admits(row)`."""

    declaration_id: str | None
    selector: dict[str, str] = field(default_factory=dict)

    def admits(self, declaration: dict[str, JSONValue]) -> bool:
        """Accept a declaration only when every pinned field matches it exactly."""
        if self.declaration_id is not None and declaration.get("id") != self.declaration_id:
            return False
        return all(
            selector_field_matches(key, value, declaration)
            for key, value in self.selector.items()
        )


def selector_field_matches(
    key: str, value: str, declaration: dict[str, JSONValue]
) -> bool:
    """Compare one pinned field; signatures compare after whitespace normalization."""
    if key != "signature":
        return declaration.get(key) == value
    spellings = [declaration.get("signature")]
    variants = declaration.get("signature_variants")
    if isinstance(variants, list):
        spellings.extend(variants)
    wanted = norm_cpp(value)
    return any(isinstance(text, str) and norm_cpp(text) == wanted for text in spellings)


def row_cpp_pin(row: dict[str, JSONValue]) -> tuple[CppPin | None, str | None]:
    """Read a row's pin; e.g. `row_cpp_pin({"cpp_selector": {"kind": "Enum"}})`.

    Returns `(None, None)` for an unpinned row and `(None, error)` for a malformed pin.
    """
    if "cpp_selector" not in row and "cpp_declaration_id" not in row:
        return None, None
    identity = row.get("cpp_declaration_id")
    if "cpp_declaration_id" in row and (not isinstance(identity, str) or not identity):
        return None, f"cpp_declaration_id {identity!r}: expected a non-empty string"
    selector, error = parsed_selector(row.get("cpp_selector", {}))
    if selector is None:
        return None, error
    return CppPin(identity if isinstance(identity, str) else None, selector), None


def parsed_selector(value: JSONValue) -> tuple[dict[str, str] | None, str | None]:
    """Validate the selector object shape; unknown keys are an error, never ignored."""
    expected = f"an object with string fields among {sorted(SELECTOR_KEYS)}"
    if not isinstance(value, dict):
        return None, f"cpp_selector {value!r}: expected {expected}"
    unknown = set(value) - SELECTOR_KEYS
    blank = [key for key, item in value.items() if not isinstance(item, str) or not item]
    if unknown or blank:
        return None, f"cpp_selector {value!r}: expected {expected}"
    return {key: str(item) for key, item in value.items()}, None


def pinned_candidates(
    pin: CppPin | None, candidates: list[dict[str, JSONValue]]
) -> list[dict[str, JSONValue]]:
    """Keep the declarations a pin admits; an absent pin keeps all of them."""
    if pin is None:
        return candidates
    return [row for row in candidates if pin.admits(row)]
