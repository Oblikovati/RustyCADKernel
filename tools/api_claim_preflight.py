"""Verify Martensite claim fragments with the production crosswalk resolver."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass

from api_crosswalk import match_api_row
from api_inventory.inputs import JSONObject, JSONValue, object_rows, text_field
from api_rust_kind import RustCrateIndex


@dataclass(frozen=True)
class ClaimFragmentPreflight:
    """Inject compiler indexes and source hashing; `preflight.verify_fragment(rows)`."""

    native_by_name: dict[str, list[JSONObject]]
    rust_by_crate: RustCrateIndex
    known_sources: set[str]
    source_digest: Callable[[str], str]

    def verify_fragment(self, fragment: JSONValue) -> list[JSONObject]:
        """Check array or api-object claims; `preflight.verify_fragment({'api': rows})`."""
        supplied = fragment.get("api") if isinstance(fragment, dict) else fragment
        claims = object_rows(supplied, "claim fragment")
        results: list[JSONObject] = []
        seen: set[str] = set()
        for claim in claims:
            result = self.verify_row(claim)
            native = object_rows(result["cpp_matches"], "resolved C++ declarations")
            identity = text_field(native[0], "id")
            if identity in seen:
                raise ValueError(
                    f"cpp_id={identity!r}; expected one claim per declaration"
                )
            seen.add(identity)
            results.append(result)
        return results

    def verify_row(self, claim: JSONObject) -> JSONObject:
        """Resolve actual declaration names and selectors; `preflight.verify_row(claim)`."""
        expected = claim.get("status")
        if expected not in {"implemented", "partial"}:
            raise ValueError(f"status={expected!r}; expected implemented or partial")
        source = text_field(claim, "source")
        if source not in self.known_sources:
            raise ValueError(f"source={source!r}; expected a captured native header")
        result = match_api_row(
            claim, self.native_by_name, self.rust_by_crate, self.known_sources
        )
        self._require_binding(claim, result)
        self._require_native_source(claim, result)
        for body in object_rows(result["rust_matches"], "resolved Rust declarations"):
            self._require_source(body)
        return result

    def _require_native_source(self, claim: JSONObject, result: JSONObject) -> None:
        native = object_rows(result["cpp_matches"], "resolved C++ declarations")[0]
        actual, expected = native["source"], claim["source"]
        if actual == expected:
            return
        raise ValueError(
            f"native source={actual!r}; expected claim header {expected!r}"
        )

    def _require_binding(self, claim: JSONObject, result: JSONObject) -> None:
        cpp, rust = result["cpp_status"], result["rust_status"]
        resolution, expected = result["resolution"], claim["status"]
        if cpp == rust == "unique" and resolution == expected:
            return
        # e1ef59476's Group rows showed why a valid ID/hash alone cannot grant credit.
        raise ValueError(
            f"claim={claim.get('upstream_id')!r}, cpp={cpp!r}, rust={rust!r}, "
            f"resolution={resolution!r}; expected unique declarations and {expected!r}"
        )

    def _require_source(self, body: JSONObject) -> None:
        filename = text_field(body, "source_file")
        recorded = text_field(body, "source_sha256")
        actual = self.source_digest(filename)
        if actual == recorded:
            return
        raise ValueError(
            f"source={filename!r}, sha256={actual!r}; expected captured hash {recorded!r}"
        )
