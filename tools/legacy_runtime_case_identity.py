"""Declaration-bound identity keys for paired native and Rust runtime cases."""

from api_inventory.inputs import JSONObject

CaseIdentity = tuple[str, str, str, str, str, str, str]


def case_identity(row: JSONObject) -> CaseIdentity:
    """Key both runtime cases and their declaration; e.g. `case_identity(result)`."""
    return tuple(
        string_field(row, field)
        for field in (
            "toolkit",
            "source",
            "upstream_id",
            "native_case",
            "rust_package",
            "rust_target",
            "rust_test",
        )
    )


def string_field(row: JSONObject, field: str) -> str:
    """Require one nonempty identity field; e.g. `string_field(row, 'source')`."""
    value = row.get(field)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field}: expected nonempty string, got {value!r}")
    return value
