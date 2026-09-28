"""Expand explicitly paired native/Rust cases without losing declaration identity."""


def runtime_pairs(row):
    """A regular declaration has one case; typed declarations list every expansion."""
    if "runtime_cases" not in row:
        return [(row["upstream_id"], row["rust_test"])]
    expansions = row["runtime_cases"]
    if not expansions:
        raise ValueError(f"Empty runtime expansion for {row['upstream_id']}")
    pairs = [(case["upstream_case"], case["rust_test"]) for case in expansions]
    if len({native for native, _ in pairs}) != len(pairs):
        raise ValueError(f"Repeated native expansion for {row['upstream_id']}")
    if len({rust for _, rust in pairs}) != len(pairs):
        raise ValueError(f"Repeated Rust expansion for {row['upstream_id']}")
    return pairs


def mapped_runtime_sets(mappings):
    """Require a distinct Rust test for each distinct native runtime identity."""
    native_cases = set()
    rust_cases = set()
    for row in mappings:
        for native, rust in runtime_pairs(row):
            rust_identity = (row["rust_target"], rust)
            if native in native_cases or rust_identity in rust_cases:
                raise ValueError(f"Duplicate mapped runtime identity: {native}, {rust_identity}")
            native_cases.add(native)
            rust_cases.add(rust_identity)
    return native_cases, rust_cases


def listed_runtime_cases(listing):
    """Parse GoogleTest listings, retaining typed suite indices before comments."""
    discovered = set()
    suite = None
    for raw_line in listing.splitlines():
        line = raw_line.split(" #", 1)[0].rstrip()
        if line and not line.startswith(" ") and line.endswith("."):
            suite = line
        elif line.startswith("  ") and suite:
            identity = suite + line.strip()
            if identity in discovered:
                raise ValueError(f"Repeated GoogleTest runtime identity: {identity}")
            discovered.add(identity)
    return discovered


# Explicit user choice, recorded in docs/tkernel-progress.md#occ23497-behavior-decision.
OCC23497 = "UnitsAPI_Test.FClassesBug_23497_CompositeUnit"
OCC23497_REASON = "TODO OCC23497: composite units are not supported by UnitsAPI."


def verified_native_skips(cases, mappings):
    """Reject failures and all skips except the exact recorded OCC23497 provenance."""
    approved = {row["upstream_id"]: row for row in mappings if row.get("status") == "upstream_skipped"}
    if set(approved) - {OCC23497}:
        raise ValueError("Unapproved upstream skip mapping")
    skipped = []
    for case in cases:
        identity = case.attrib['classname']+'.'+case.attrib['name']
        if case.find('failure') is not None or case.attrib.get('status') != 'run':
            raise ValueError(f"Failed or unexecuted native case: {identity}")
        marker = case.find('skipped')
        if marker is None:
            if identity in approved: raise ValueError(f"Native skip changed; review required: {identity}")
            continue
        row = approved.get(identity)
        reason_lines = marker.attrib.get('message', '').strip().splitlines()
        reason = reason_lines[-1] if reason_lines else ''
        if not row or reason != OCC23497_REASON or row.get('behavior_decision') != 'preserve_current':
            raise ValueError(f"Unexpected native skip: {identity}: {reason}")
        skipped.append({'upstream_id': identity, 'reason': reason,
                        'current_behavior_rust_test': row['rust_test'], 'legacy_assertions_passed': False})
    if {row['upstream_id'] for row in skipped} != set(approved):
        raise ValueError("Missing approved native skip observation")
    return skipped
