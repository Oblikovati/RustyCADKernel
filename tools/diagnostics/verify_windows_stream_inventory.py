"""Check the complete native compiler delta without admitting its baseline."""
from collections import Counter
import hashlib
import json
from pathlib import Path

sdk = Path("target/document-access-sdk")
report = json.loads((sdk / "target/unsafe-inventory/latest.json").read_text())
baseline = json.loads((sdk / "docs/features/safe-toolkits/compiler-baselines/x86_64-pc-windows-msvc.json").read_text())
manifest = json.loads(Path("tools/diagnostics/windows_document_streams_fix.json").read_text())
assert report["target"] == "x86_64-pc-windows-msvc"
assert "host: x86_64-pc-windows-msvc" in report["provenance"]["compiler"]
assert "--workspace --all-targets --target x86_64-pc-windows-msvc" in report["provenance"]["command"]
assert report["provenance"]["compiler"].splitlines()[0] == baseline["compiler"]
key = "crates/martensite-tkernel/src/osd/document_file/windows_streams.rs"
old = {json.dumps(row["location"], sort_keys=True): row["emitted_primary_spans"] for row in baseline["summary"]}
new = {json.dumps(row["location"], sort_keys=True): row["emitted_primary_spans"] for row in report["diagnostics"]["workspace_summary"]}
added = set(new) - set(old)
assert len(added) == 1 and not (set(old) - set(new))
assert all(new[location] == count for location, count in old.items())
location = json.loads(next(iter(added)))
assert location == {"diagnostic": "usage of an `unsafe` block", "expansion_depth": 0, "package": "workspace:martensite-tkernel@0.3.6", "source": key, "target": "martensite_tkernel", "target_kinds": ["lib"]}
assert new[next(iter(added))] == 6
sites = Counter()
for diagnostic in report["diagnostics"]["workspace"]:
    for span in diagnostic["spans"]:
        if span["is_primary"] and span["file_name"] == key:
            assert diagnostic["message"] == "usage of an `unsafe` block" and span["expansion"] is None
            sites[(span["line_start"], span["column_start"], span["byte_start"], span["byte_end"]) ] += 1
assert len(sites) == 3 and set(sites.values()) == {2}
assert hashlib.sha256((sdk / key).read_bytes()).hexdigest() == manifest["files"][key]
result = {"scope": "Native Windows workspace all-targets compiler collection; baseline comparison intentionally failed before admission", "equivalent_product_candidate": manifest["fix"], "native_host": "x86_64-pc-windows-msvc", "added_groups": 1, "added_primary_spans": 6, "unique_reviewed_system_call_blocks": 3, "all_existing_groups_unchanged": True, "sites": [{"line": site[0], "column": site[1], "byte_start": site[2], "byte_end": site[3], "emissions": count} for site, count in sorted(sites.items())]}
Path("target/runtime-oracle/native-inventory-delta.json").write_text(json.dumps(result, indent=2) + "\n")
