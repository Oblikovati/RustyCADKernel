"""Compile access controls and check inferred private-return-type inventory edges."""

from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))
from api_inventory.inputs import JSONObject
from api_inventory.merge import declaration_identity


def capture(executable: str, root: pathlib.Path) -> list[JSONObject]:
    """Extract the focused fixture; e.g. `capture(extractor, fixture_root)`."""
    command = [
        executable,
        "--source-root",
        str(root),
        "--main-header",
        str(root / "include/ReturnedPrivate.hxx"),
        "--",
        "-std=c++17",
    ]
    result = subprocess.run(command, capture_output=True, text=True, check=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def compile_control(root: pathlib.Path, expression: str, succeeds: bool) -> None:
    """Execute positive controls and reject ill-formed ones with private/protected diagnostics."""
    with tempfile.TemporaryDirectory() as temporary:
        source = pathlib.Path(temporary) / "control.cxx"
        output = pathlib.Path(temporary) / "control"
        source.write_text('#include "ReturnedPrivate.hxx"\n' + expression + "\n")
        result = subprocess.run(
            [
                "c++",
                "-std=c++17",
                "-I",
                str(root / "include"),
                str(source),
                "-o",
                str(output),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert (result.returncode == 0) == succeeds, result.stderr
        if succeeds:
            subprocess.run([str(output)], check=True)
        else:
            assert (
                "private" in result.stderr
                or "protected" in result.stderr
                or "final" in result.stderr
            ), result.stderr


def native_controls(root: pathlib.Path) -> None:
    """Prove access through inferred/aliased types while inaccessible names stay rejected."""
    positive = """
using returned_fixture::Owner;
using Hidden = decltype(Owner{}.byValue());
struct HiddenChild : Hidden { using Hidden::protectedValue; };
int main() {
  Owner owner;
  returned_fixture::DerivedOwner derived;
  auto value = owner.byValue();
  Owner::Alias alias;
  decltype(value)::PublicAlias child;
  return value.read() != 5 || owner.byPointer()->read() != 5 ||
    owner.byReference().read() != 5 || owner.byConstReference().read() != 5 ||
    owner.byAlias().read() != 5 || derived.protectedGetter().read() != 5 ||
    HiddenChild{}.protectedValue() != 7 || child.leaf() != 9 || alias.read() != 5;
}"""
    compile_control(root, positive, True)
    compile_control(
        root,
        "using Final = decltype(returned_fixture::Owner{}.finalValue()); "
        "struct Bad : Final { using Final::blocked; }; int main() {}",
        False,
    )
    for expression in [
        "returned_fixture::Owner::Hidden value;",
        "auto v = owner.byValue(); v.secret();",
        "auto v = owner.byValue(); v.protectedValue();",
        "owner.privateGetter();",
        "owner.protectedGetter();",
    ]:
        compile_control(
            root,
            "int main() { returned_fixture::Owner owner; " + expression + " }",
            False,
        )


def edges_for(events: list[JSONObject], suffix: str) -> list[JSONObject]:
    return [
        event
        for event in events
        if event.get("event") == "returned_type_reachability"
        and event["seed_declaration"]["qualified_name"]
        == "returned_fixture::Owner::" + suffix
    ]


def check_edges(events: list[JSONObject]) -> None:
    """Check exact IDs, access categories, wrappers, transitive edges and withheld templates."""
    declarations = {
        declaration_identity(row): row
        for row in events
        if row.get("event") == "declaration"
    }
    edges = edges_for(events, "byValue")
    hidden = next(
        edge
        for edge in edges
        if edge.get("status") == "private_record_members_recorded"
        and edge["target_declaration"]["qualified_name"].endswith("::Hidden")
    )
    members = {
        row["declaration"]["qualified_name"].split("::")[-1]: row
        for row in hidden["members"]
    }
    assert members["read"]["access_basis"] == "inferred_type_public_member"
    assert members["secret"]["access_basis"] == "private_member_inaccessible"
    assert members["protectedValue"]["access_basis"] == "requires_derived_inferred_type"
    assert any(edge.get("status") == "cycle_or_repeated_target" for edge in edges)
    assert any(
        edge["target_declaration"]["qualified_name"].endswith("::Leaf")
        for edge in edges
    )
    for row in hidden["members"]:
        assert declaration_identity(row["declaration"]) in declarations
        assert row["declaration"]["api_visible"] is False
    assert hidden["private_type_name_accessible"] is False
    assert all(
        edge["coverage_credit"] is False and edge["reachability_complete"] is False
        for edge in edges
    )


def check_paths(events: list[JSONObject]) -> None:
    """Check return forms, protected producers, aliases and explicitly unsupported templates."""
    for name, wrapper in [
        ("byPointer", "pointer"),
        ("byReference", "lvalue_reference"),
        ("byConstReference", "lvalue_reference"),
    ]:
        assert any(
            edge["type_wrappers"] == [wrapper] for edge in edges_for(events, name)
        )
    assert edges_for(events, "Alias") and edges_for(events, "byAlias")
    assert edges_for(events, "protectedGetter") and not edges_for(
        events, "privateGetter"
    )
    for name in ["concreteTemplate", "dependentTemplate", "unknownTemplate"]:
        assert (
            edges_for(events, name)[0]["status"]
            == "unsupported_template_or_dependent_type"
        )
        assert "members" not in edges_for(events, name)[0]
    final = edges_for(events, "finalValue")[0]
    assert any(
        row["access_basis"] == "protected_member_of_final_type"
        for row in final["members"]
    )
    summary = next(row for row in events if row.get("event") == "summary")
    assert summary["returned_type_reachability_events"] == sum(
        row.get("event") == "returned_type_reachability" for row in events
    )


def main() -> None:
    """Run with extractor and fixture source root arguments."""
    root = pathlib.Path(sys.argv[2]) / "source"
    events = capture(sys.argv[1], root)
    check_edges(events)
    check_paths(events)
    native_controls(root)
    print(
        "Returned private-type inventory edges and seven native access controls passed"
    )


if __name__ == "__main__":
    main()
