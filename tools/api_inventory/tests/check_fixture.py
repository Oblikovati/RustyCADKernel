#!/usr/bin/env python3
import json
import pathlib
import subprocess
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from api_inventory.merge import InventoryRecords, merge_declaration
from api_inventory.inputs import JSONObject
from api_inventory.template_identity import normalize_template_signature


def run_extract(executable: str, root: pathlib.Path, header: pathlib.Path):
    command = [
        executable,
        "--source-root", str(root),
        "--main-header", str(header),
        "--", "-std=c++17", "-I", str(root / "include"),
    ]
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    events = [json.loads(line) for line in result.stdout.splitlines()]
    return result, events


def one(events, event_type: str, **fields):
    matches = [event for event in events if event.get("event") == event_type]
    for key, value in fields.items():
        matches = [event for event in matches if event.get(key) == value]
    assert len(matches) == 1, (event_type, fields, len(matches))
    return matches[0]


def check_declarations(events):
    declarations = [event for event in events if event.get("event") == "declaration"]
    assert all(event["source"] for event in declarations)
    assert one(events, "declaration", qualified_name="fixture::PublicType::protected_field")["api_visible"]
    assert one(events, "declaration", qualified_name="fixture::PublicType::private_field")["exclusion_reason"] == "private_access"
    assert one(events, "declaration", qualified_name="fixture::PrivateOuter::SecretOuter::HiddenNested")["exclusion_reason"] == "private_ancestor"
    one(events, "declaration", qualified_name="fixture::Box", kind="ClassTemplate")
    assert one(events, "declaration", qualified_name="fixture::Box", kind="CXXRecord")["exclusion_reason"] == "template_owned_declaration"
    assert one(events, "declaration", qualified_name="fixture::scaled")["api_visible"]
    template_function = one(events, "declaration", qualified_name="fixture::identity", kind="FunctionTemplate")
    assert template_function["parameter_types"] == ["type-parameter-0-0"]
    assert template_function["written_parameter_types"] == ["T"]
    assert "(" in template_function["signature"]
    assert one(events, "declaration", qualified_name="LocalAlias")["exclusion_reason"] == "local_scope"
    assert one(events, "declaration", qualified_name="LocalRecord")["exclusion_reason"] == "local_scope"
    assert one(events, "declaration", qualified_name="local_value")["exclusion_reason"] == "local_variable"
    check_template_parameter_metadata(events, declarations, template_function)
    assert one(events, "declaration", kind="StaticAssert")["exclusion_reason"] == "static_assert"
    overloads = [event for event in declarations if event.get("qualified_name") == "fixture::PublicType::overload"]
    assert len(overloads) == 2 and overloads[0]["usr"] != overloads[1]["usr"]
    sfinae_overloads = [event for event in declarations if event.get("qualified_name") == "fixture::SFINAEOverloads::IsValidIn" and event.get("kind") == "FunctionTemplate"]
    assert len(sfinae_overloads) == 2
    assert sfinae_overloads[0]["usr"] == sfinae_overloads[1]["usr"]
    assert len({event["signature"] for event in sfinae_overloads}) == 2
    assert len({event["template_signature"] for event in sfinae_overloads}) == 1
    unresolved_using = one(events, "declaration", qualified_name="fixture::DependentValueBase::inherited_value")
    assert unresolved_using["kind"] == "UnresolvedUsingValue"
    assert unresolved_using["api_visible"]
    assert unresolved_using["api_resolution"] == "unresolved_using_value_target"
    assert unresolved_using["callable_status"] == "unknown"
    redeclared = [event for event in declarations if event.get("qualified_name") == "fixture::redeclared_value"]
    assert len(redeclared) == 2
    assert len({event["identity_signature"] for event in redeclared}) == 1
    default_rows = [event for event in declarations if event.get("qualified_name") == "fixture::redeclared_default"]
    assert len(default_rows) == 2
    default_observations = [entry for event in default_rows for entry in event["default_arguments"]]
    assert any(entry["state"] == "explicit" and entry["expression"] == "7" for entry in default_observations)
    assert any(entry["state"] == "inherited" for entry in default_observations)
    inherited = one(events, "declaration", qualified_name="fixture::BaseDefaults::inherited_default")
    assert inherited["default_arguments"][0]["state"] == "explicit"
    assert inherited["default_arguments"][0]["expression"] == "11"
    named_templates = [event for event in declarations if event.get("qualified_name") == "fixture::named_template" and event.get("kind") == "FunctionTemplate"]
    assert len(named_templates) == 2
    assert len({event["identity_signature"] for event in named_templates}) == 1
    assert len({event["template_signature"] for event in named_templates}) == 1
    class_templates = [event for event in declarations if event.get("qualified_name") == "fixture::class_template_parameter" and event.get("kind") == "FunctionTemplate"]
    assert len(class_templates) == 2
    assert len({event["identity_signature"] for event in class_templates}) == 1
    assert len({event["template_signature"] for event in class_templates}) == 1
    default_templates = [event for event in declarations if event.get("qualified_name") == "fixture::default_template_parameter" and event.get("kind") == "FunctionTemplate"]
    assert len(default_templates) == 2
    assert default_templates[0]["template_signature"] != default_templates[1]["template_signature"]
    generated = one(events, "declaration", qualified_name="fixture::PublicType::from_macro")
    assert generated["macro_generated"]
    assert generated["spelling_location"] != generated["expansion_location"]


def check_template_parameter_metadata(
    events: list[JSONObject], declarations: list[JSONObject], template_function: JSONObject
) -> None:
    parameters = [
        event for event in declarations
        if event.get("parent_usr") == template_function["usr"]
        and event.get("kind") == "TemplateTypeParm"
    ]
    assert len(parameters) == 1
    assert parameters[0]["exclusion_reason"] == "template_parameter_metadata"
    check_class_template_parameters(events, declarations)


def check_class_template_parameters(
    events: list[JSONObject], declarations: list[JSONObject]
) -> None:
    box = one(events, "declaration", qualified_name="fixture::Box", kind="ClassTemplate")
    box_parameters = [
        event for event in declarations
        if event.get("parent_usr") == box["usr"]
        and event.get("kind") in {"TemplateTypeParm", "NonTypeTemplateParm"}
    ]
    assert {event["kind"] for event in box_parameters} == {"TemplateTypeParm", "NonTypeTemplateParm"}
    assert all(event["exclusion_reason"] == "template_parameter_metadata" for event in box_parameters)
    check_template_wrapper_signatures(events, box)


def check_template_wrapper_signatures(events: list[JSONObject], box: JSONObject) -> None:
    assert box["api_visible"] and "unsigned int __template_param_1 = 2" in box["template_signature"]
    pack = one(events, "declaration", qualified_name="fixture::Pack", kind="ClassTemplate")
    assert "typename ...__template_param_0" in pack["template_signature"]
    nested = one(events, "declaration", qualified_name="fixture::TemplateTemplateBox", kind="ClassTemplate")
    assert "template <typename> typename __template_param_0" in nested["template_signature"]
    constrained = one(events, "declaration", qualified_name="fixture::AssociatedTypeBox", kind="ClassTemplate")
    assert "value_type" in constrained["template_signature"]
    alias = one(events, "declaration", qualified_name="fixture::ElementAlias", kind="TypeAliasTemplate")
    assert alias["api_visible"] and "__template_param_0" in alias["template_signature"]
    assert one(events, "declaration", qualified_name="fixture::Scalar")["api_visible"]
    assert one(events, "declaration", qualified_name="fixture::CountAlias")["api_visible"]


def check_merged_template_metadata(events) -> None:
    plain = dict(one(events, "declaration", qualified_name="fixture::scaled"))
    assert plain["template_signature"] == ""
    records = InventoryRecords()
    merge_declaration(records, plain, "Fixture.hxx")
    assert plain["stable_identity"] and "api_resolution" not in plain
    unresolved = dict(one(events, "declaration", kind="UnresolvedUsingValue"))
    assert unresolved["template_signature"] == ""
    merge_declaration(records, unresolved, "Fixture.hxx")
    assert unresolved["api_resolution"] == "unresolved_using_value_target"
    templates = [
        event["template_signature"]
        for event in events
        if event.get("event") == "declaration"
        and event.get("kind") in {"FunctionTemplate", "ClassTemplate", "TypeAliasTemplate"}
    ]
    results = [normalize_template_signature(item) for item in templates]
    assert results and all(item.supported for item in results)


def check_preprocessor(events, root: pathlib.Path):
    guard = one(events, "macro", name="_Fixture_HeaderFile")
    assert guard["macro_category"] == "include_guard" and not guard["api_candidate"]
    assert one(events, "macro", name="FIXTURE_SCALE")["api_candidate"]
    fallback = one(events, "macro", name="M_PI")
    assert fallback["macro_category"] == "source_macro" and fallback["api_candidate"]
    skipped = one(events, "skipped_range")
    assert skipped["start"]["line"] < skipped["end"]["line"]
    for fragment in ("Fixture.lxx", "Fixture.pxx", "Fixture.gxx"):
        path = str((root / "include" / fragment).resolve())
        one(events, "header_seen", source=path, emit_declarations=True)
        assert any(row.get("source") == path for row in events if row.get("event") == "declaration")


def check_parse_failure(executable: str):
    with tempfile.TemporaryDirectory() as temporary:
        root = pathlib.Path(temporary)
        header = root / "Broken.hxx"
        header.write_text("#error fixture parse error\n")
        result, events = run_extract(executable, root, header)
    assert result.returncode != 0
    assert any(event.get("event") == "diagnostic" and event.get("severity") == "error" for event in events)


def check_external_main_header(executable: str):
    with tempfile.TemporaryDirectory() as temporary:
        root = pathlib.Path(temporary) / "source"
        root.mkdir()
        header = pathlib.Path(temporary) / "generated.hxx"
        header.write_text("struct GeneratedMainType {};\n")
        result, events = run_extract(executable, root, header)
    assert result.returncode == 0, result.stderr
    candidate = one(events, "declaration", qualified_name="GeneratedMainType")
    assert candidate["api_visible"] and candidate["source"] == str(header.resolve())


def main():
    executable = sys.argv[1]
    fixture_root = pathlib.Path(sys.argv[2]).resolve() / "source"
    header = fixture_root / "include" / "Fixture.hxx"
    result, events = run_extract(executable, fixture_root, header)
    assert result.returncode == 0, result.stderr
    check_declarations(events)
    check_merged_template_metadata(events)
    check_preprocessor(events, fixture_root)
    summary = one(events, "summary")
    assert summary["complete_for_emit_scope"]
    assert not summary["unhandled_decl_kinds"]
    check_parse_failure(executable)
    check_external_main_header(executable)


if __name__ == "__main__":
    main()
