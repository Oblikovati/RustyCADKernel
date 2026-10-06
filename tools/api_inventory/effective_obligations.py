"""Build a separate access-qualified obligation union without changing native visibility."""

from __future__ import annotations

from api_inventory.inputs import JSONObject, object_rows, text_field
from api_inventory.merge import declaration_identity

NATIVE_FACTS = (
    "kind",
    "qualified_name",
    "signature",
    "usr",
    "identity_signature",
    "template_signature",
    "access",
    "ancestor_access",
    "api_visible",
    "exclusion_reason",
    "source",
    "parent_usr",
)


def exact_declaration_index(rows: list[JSONObject]) -> dict[str, JSONObject]:
    """Reject duplicate or invented native IDs; e.g. `exact_declaration_index(declarations)`."""
    indexed: dict[str, JSONObject] = {}
    for row in rows:
        identifier = text_field(row, "id")
        if identifier in indexed or identifier != declaration_identity(row):
            raise ValueError(
                f"native identity {identifier}: expected unique exact declaration ID"
            )
        indexed[identifier] = row
    return indexed


def eligible_edge_targets(edge: JSONObject) -> list[tuple[JSONObject, str]]:
    """Separate inferred records from public member access; e.g. `eligible_edge_targets(edge)`."""
    if edge.get("status") != "private_record_members_recorded":
        return []
    target = edge.get("target_declaration")
    if not isinstance(target, dict):
        raise TypeError(f"target {target!r}: expected recorded private declaration")
    result = [(target, "inferred_record")]
    for member in object_rows(edge.get("members"), "edge members"):
        declaration = member.get("declaration")
        if member.get("access_basis") == "inferred_type_public_member" and isinstance(
            declaration, dict
        ):
            result.append((declaration, "inferred_public_member"))
    return result


def exposure_routes(edges: list[JSONObject]) -> dict[str, list[JSONObject]]:
    """Keep every eligible producer route; e.g. `exposure_routes(returned_edges)`."""
    routes: dict[str, list[JSONObject]] = {}
    for edge in edges:
        for declaration, role in eligible_edge_targets(edge):
            identifier = text_field(declaration, "id")
            exact_declaration_index([declaration])
            route: JSONObject = {
                "edge_id": text_field(edge, "id"),
                "role": role,
                "producer_id": nested_declaration_id(edge, "producer_declaration"),
                "seed_id": nested_declaration_id(edge, "seed_declaration"),
            }
            if route not in routes.setdefault(identifier, []):
                routes[identifier].append(route)
    return routes


def nested_declaration_id(edge: JSONObject, field: str) -> str:
    """Authenticate embedded native identities; e.g. `nested_declaration_id(edge, 'seed_declaration')`."""
    declaration = edge.get(field)
    if not isinstance(declaration, dict):
        raise TypeError(f"{field}={declaration!r}: expected declaration object")
    identifier = text_field(declaration, "id")
    if identifier != declaration_identity(declaration):
        raise ValueError(f"{field}={identifier}: expected exact recomputed identity")
    return identifier


def require_access_contract(
    contract: JSONObject, declaration: JSONObject, routes: list[JSONObject]
) -> None:
    """Require a scoped demonstrated expression; e.g. `require_access_contract(proof, declaration, routes)`."""
    for field in ("qualified_name", "signature", "kind"):
        if contract.get(field) != declaration.get(field):
            raise ValueError(
                f"access contract {field}={contract.get(field)!r}: expected {declaration.get(field)!r}"
            )
    if contract.get("role") not in {route["role"] for route in routes}:
        raise ValueError(
            f"access role {contract.get('role')!r}: expected eligible returned-type route"
        )
    require_native_access_role(contract, declaration)
    require_access_limits(contract)


def require_native_access_role(contract: JSONObject, declaration: JSONObject) -> None:
    """Do not turn private members into public operations; e.g. `require_native_access_role(proof, native)`."""
    role = contract.get("role")
    if role == "inferred_public_member" and declaration.get("access") != "public":
        raise ValueError(
            f"native access {declaration.get('access')!r}: expected public member access"
        )
    if role == "inferred_record" and declaration.get("kind") != "CXXRecord":
        raise ValueError(
            f"native kind {declaration.get('kind')!r}: expected inferred CXXRecord"
        )


def require_access_limits(contract: JSONObject) -> None:
    """Keep demonstrated operations bounded; e.g. `require_access_limits(contract)`."""
    if (
        not contract.get("demonstrated_operations")
        or contract.get("private_name_accessible") is not False
    ):
        raise ValueError(
            f"access contract {contract!r}: expected bounded operations and inaccessible private name"
        )
    if contract.get("implementation_credit") is not False:
        raise ValueError(
            f"implementation credit {contract.get('implementation_credit')!r}: expected false"
        )


def obligation_row(declaration: JSONObject, basis: str) -> JSONObject:
    """Preserve original access facts under an independent obligation basis; e.g. `obligation_row(row, 'primary')`."""
    return {
        "native_id": text_field(declaration, "id"),
        "basis": [basis],
        "native_facts": {field: declaration.get(field) for field in NATIVE_FACTS},
        "implementation_credit": False,
    }


def effective_obligation_union(
    declarations: list[JSONObject], edges: list[JSONObject], contracts: list[JSONObject]
) -> list[JSONObject]:
    """Union exact proven obligations with primary candidates; e.g. `effective_obligation_union(rows, edges, proofs)`."""
    native = exact_declaration_index(declarations)
    obligations = {
        key: obligation_row(row, "primary")
        for key, row in native.items()
        if row.get("api_visible") is True
    }
    add_access_demonstrations(native, exposure_routes(edges), contracts, obligations)
    return [obligations[key] for key in sorted(obligations)]


def add_access_demonstrations(
    native: dict[str, JSONObject],
    routes: dict[str, list[JSONObject]],
    contracts: list[JSONObject],
    obligations: dict[str, JSONObject],
) -> None:
    """Deduplicate exact obligations while preserving every access route; e.g. `add_access_demonstrations(native, routes, proofs, union)`."""
    seen: set[str] = set()
    for contract in contracts:
        identifier = text_field(contract, "native_id")
        if identifier in seen or identifier not in native or identifier not in routes:
            raise ValueError(
                f"access ID {identifier}: expected one known eligible exact declaration"
            )
        require_access_contract(contract, native[identifier], routes[identifier])
        row = inferred_obligation(obligations, native[identifier])
        row["access_demonstration"] = contract
        row["exposure_routes"] = routes[identifier]
        seen.add(identifier)


def inferred_obligation(
    obligations: dict[str, JSONObject], native: JSONObject
) -> JSONObject:
    """Retain both bases if a demonstrated row was already primary; e.g. `inferred_obligation(union, native)`."""
    identifier = text_field(native, "id")
    row = obligations.setdefault(
        identifier, obligation_row(native, "demonstrated_inferred_access")
    )
    basis = row["basis"]
    if isinstance(basis, list) and "demonstrated_inferred_access" not in basis:
        basis.append("demonstrated_inferred_access")
    return row
