# Compiler API inventory snapshot

This directory contains reproducible evidence for the pinned OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871` and the Rust source snapshot identified
inside `rust-api.json.gz`. Full API and test parity are **not complete**.

- `completeness.json`: small crosswalk summary, per-toolkit counts and proof gaps.
- `api-crosswalk.json.gz`: every ledger row resolution and unclaimed C++ declaration.
- `rust-api.json.gz`: reachable public Rust paths, signatures and source hashes.
- `occt-manifest.json`: compiler input fingerprints, artifact hashes and counts.
- `occt-declarations.jsonl.gz`: Clang declaration identities, dispositions and sites.
- Other `occt-*.jsonl.gz` tables: macros, headers, diagnostics, conflicting identities,
  skipped preprocessor ranges, contributing files, and source-context retries.

The initial scan attempted all 5,905 standalone headers available in the Linux
configuration. It parsed 5,639; authentic OCCT translation-unit retries recovered
260 more. All 384 inline fragments were observed. Six header probes
remain unresolved and another 28 distinct source headers lack a configured
owning toolkit. Original failures are preserved even when a context retry works.

The observed surface contains 110,134 public/protected declaration candidates
and 6,465 macro candidates. Macro bodies, inactive platform/configuration
branches, unknown configurations, and unresolved identities require additional
work. The declaration count is a preliminary candidate count: the current
visibility filter still includes internal helpers, so it is neither a validated
lower bound nor a denominator for API coverage. Shared package ownership makes
per-toolkit totals overlap.

## Reproduce

Build the pinned extractor as described in
[`tools/api_inventory/README.md`](../../tools/api_inventory/README.md). Use the
pinned OCCT checkout and an exported CMake compilation database:

```sh
python3 tools/capture_occt_inventory.py --source /path/to/OCCT \
  --database /path/to/build/compile_commands.json \
  --extractor target/api_inventory/occt-api-extract
python3 tools/retry_occt_header_context.py --source /path/to/OCCT \
  --database /path/to/build/compile_commands.json \
  --extractor target/api_inventory/occt-api-extract \
  --capture target/api-proof/occt-inventory/capture-manifest.json \
  --output target/api-proof/context-retries
python3 tools/merge_occt_inventory.py --capture target/api-proof/occt-inventory \
  --context-retries target/api-proof/context-retries/context-retry-manifest.json
python3 tools/capture_rustdoc_json.py capture
python3 tools/normalize_rustdoc_inventory.py
python3 tools/api_coverage_report.py
python3 tools/publish_api_snapshot.py
PYTHONPATH=tools python3 -m unittest discover -s tools/tests
```

To examine the checked-in crosswalk without generating compiler artifacts:

```sh
python3 tools/api_coverage_report.py --rust porting/api-inventory/rust-api.json.gz
```

This last command checks correspondence against the recorded Rust snapshot.
After changing Rust source, recapture and normalize before treating the result
as current. Freshness checks also bind the Rust normalizer and crosswalk tool
implementations by content hash; changing those tools requires regenerating
the affected stages. Captured absolute build paths identify the original environment;
content hashes and source-relative declaration paths are the portable identity.

## Legacy test obligations

`legacy-test-summary.json` and `legacy-tests.jsonl.gz` independently account for
all 8,648 GoogleTest declaration candidates: 7,398 have exact Rust mappings and
1,250 remain unmapped. There are no duplicate or unknown mappings in this
snapshot. All 16,698 DRAW candidates and 18,309 regression/support files are
listed as discovered obligations.

The legacy snapshot does not consume runtime verifier evidence, so its rows
say `unverified` even when another gate has run a suite successfully. It never
turns a `translated` ledger status into a green runtime result. Regenerate with:

```sh
python3 tools/publish_legacy_test_inventory.py
```

Audit an existing TKernel runtime report with the pinned source checkout:

```sh
python3 tools/legacy_runtime_evidence.py target/oracle-tkernel/report.json \
  --occt-source /path/to/OCCT
```

This check rejects missing or changed input hashes and a mismatched or dirty OCCT
checkout. Aggregate reports remain `aggregate_only` even when their inputs match;
individual declaration results require an explicit runtime-instance join. The
assessor never grants runtime proof. New TKernel reports also fingerprint the
verifier's local helper modules.

## Paired runtime evidence

`tkernel-runtime-report.json.gz` retains the complete verifier report and its input
hashes at Rust revision `432e74bc`. `tkernel-runtime-declarations.json.gz` records
the exact declaration/instance join: 1,315 declarations passed, one declaration
was an approved upstream skip, and 7,332 were unverified by this TKernel run.
There were zero unmapped runtime cases or mapping anomalies. The run executed
1,521 passing native instances and 201,890 differential cases. These counts do
not prove toolkit or global completion. Later source changes require a rerun.

The verifier now compares input fingerprints before and after execution and
rechecks the pinned OCCT checkout. To regenerate the paired evidence:

```sh
python3 tools/verify_tkernel.py --source /path/to/OCCT --jobs 2
python3 tools/join_legacy_runtime_evidence.py target/oracle-tkernel/report.json \
  --occt-source /path/to/OCCT --output target/api-proof/tkernel-joined-runtime.json
```

The current API crosswalk resolves 16,304 implemented declaration claims; 10,673
ledger rows remain unresolved. It is not a behavioral coverage percentage.
