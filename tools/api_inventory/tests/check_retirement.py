"""Check formal/inherited Clang deprecation and attached API documentation."""

from __future__ import annotations
import json
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from api_retirement_markers import upstream_marker_status

_EXPECTED: dict[str | None, tuple[str, ...]] = {
    "deprecated": (
        "retiredFunction",
        "RetiredRecord::inheritedMember",
        "RetiredTemplate::inheritedMember",
        "RetiredMode",
    ),
    "obsolete": (
        "ObsoleteRecord::inheritedMember",
        "ActiveRecord::obsoleteMethod",
        "ObsoleteMode",
    ),
    None: (
        "activeFunction",
        "ActiveMode",
        "ActiveRecord::activeWithOldParameter",
        "ActiveRecord::invalidateBounds",
        "ActiveTemplateParameter",
    ),
}


def fixture_events(
    executable: str, root: Path, filtered: bool = False
) -> list[dict[str, object]]:
    """Extract compiler evidence; e.g. fixture_events(extractor, source)."""
    command = [
        executable,
        "--source-root",
        str(root),
        "--main-header",
        str(root / "Retirement.hxx"),
    ]
    if filtered:
        command.append("--retirement-markers-only")
    command.extend(["--", "-std=c++17", "-fparse-all-comments"])
    result = subprocess.run(command, capture_output=True, text=True, check=True)
    return [json.loads(line) for line in result.stdout.splitlines()]


def declaration_index(events: list[dict[str, object]]) -> dict[str, dict[str, object]]:
    """Index named fixture APIs; e.g. declaration_index(fixture_events(extractor, source))."""
    return {
        str(event["qualified_name"]): event
        for event in events
        if event.get("event") == "declaration"
    }


def assert_status(event: dict[str, object], expected: str | None) -> None:
    """Check attached markers; e.g. assert_status(declaration, 'deprecated')."""
    markers = event.get("upstream_retirement_markers", [])
    assert isinstance(markers, list), markers
    statuses = {
        upstream_marker_status(marker, str(event["kind"])) for marker in markers
    }
    statuses.discard(None)
    assert expected in statuses if expected else not statuses, event["qualified_name"]


def check_filtered_capture(executable: str, root: Path) -> None:
    """Keep supplemental capture distinct from API coverage; e.g. check_filtered_capture(extractor, source)."""
    events = fixture_events(executable, root, True)
    summary = next(event for event in events if event.get("event") == "summary")
    assert summary["retirement_markers_only"] and not summary["complete_for_emit_scope"]
    declarations = declaration_index(events)
    assert "retirement_fixture::activeFunction" not in declarations
    assert "retirement_fixture::ActiveMode" in declarations
    assert_status(declarations["retirement_fixture::retiredFunction"], "deprecated")


def check_fixture(executable: str, root: Path) -> None:
    """Verify inheritance and negative controls; e.g. check_fixture(extractor, source)."""
    declarations = declaration_index(fixture_events(executable, root))
    for expected, names in _EXPECTED.items():
        for name in names:
            assert_status(declarations["retirement_fixture::" + name], expected)
    check_filtered_capture(executable, root)


if __name__ == "__main__":
    check_fixture(sys.argv[1], Path(sys.argv[2]))
