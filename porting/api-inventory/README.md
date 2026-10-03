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
hashes at clean execution-time Rust revision `f8e39c45`. `tkernel-runtime-declarations.json.gz` records
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

The published API crosswalk at Rust revision `d61f7ade` resolves 20,301 implemented declaration claims; 6,902
ledger rows remain unresolved. It is not a behavioral coverage percentage.
This refresh includes 283 corrected TKernel `noexcept` selectors for existing
claims, one protected removal-status mapping, and four wire singularity-check
mappings. The selector corrections are audited in
[`tkernel-noexcept-selector-audit.json`](../tkernel-noexcept-selector-audit.json).

### TKMath paired run

`tkmath-runtime-report.json.gz` and `tkmath-runtime-declarations.json.gz` retain
execution evidence from clean Rust revision `9ba5c46c`: all 2,023 mapped native
cases and their 2,023 Rust counterparts passed, with no native skips. The
report pins execution-time Git identity, source hashes, the native executable,
CMake configuration, and 20 resolved OCCT/GoogleTest shared libraries. The
join records execution and join revisions separately. Other declarations
remain unverified by this run; full API and toolkit completion remain false.

```sh
python3 tools/verify_tkmath_legacy.py --source /path/to/OCCT \
  --build-dir /path/to/occt-modeling-full --jobs 2
python3 tools/join_legacy_runtime_evidence.py target/oracle-tkmath-legacy/report.json \
  --occt-source /path/to/OCCT --output target/api-proof/tkmath-joined-runtime.json
```

Both retained receipts now record execution-time Git provenance. They remain
historical receipts after later source changes; rerun the verifier to assess those changes.

### TKG2d paired run

`tkg2d-runtime-report.json.gz` and `tkg2d-runtime-declarations.json.gz` retain
523 passing native/Rust pairs from clean execution revision `5644d059`. No
cases were skipped and the declaration join has no mapping anomalies. This
receipt covers every CMake-listed TKG2d GoogleTest declaration at the pinned
source revision; it does not prove API, DRAW, toolkit or global completion.

```sh
python3 tools/verify_tkmath_legacy.py --toolkit TKG2d --source /path/to/OCCT \
  --build-dir /path/to/occt-modeling-full --jobs 2
```

### TKG3d paired run

`tkg3d-runtime-report.json.gz` and `tkg3d-runtime-declarations.json.gz` retain
794 exact native/Rust pairs from clean execution revision `1696fa2f`. All passed,
with zero skips, unmatched runtime cases or mapping anomalies. The runner retains
Rust package identity because the mappings span `rustycad-tkg3d` and
`rustycad-tkgeomalgo`. The global inventory still has 7,854 unverified declarations
in this scoped receipt; toolkit and global proof flags remain false.

```sh
python3 tools/verify_tkmath_legacy.py --toolkit TKG3d --source /path/to/OCCT \
  --build-dir /path/to/occt-modeling-full --jobs 2
python3 tools/join_legacy_runtime_evidence.py target/oracle-tkg3d-legacy/report.json \
  --occt-source /path/to/OCCT --output target/api-proof/tkg3d-joined-runtime.json
```

The source revision and all hashed runtime inputs are retained in each receipt.
These historical receipts do not jointly establish a current whole-kernel pass.

### TKGeomBase native failure

The paired verifier now supports all 839 mapped TKGeomBase declarations. Its first
strict run at `65fed4cd` returned 838 native passes and one exact-zero assertion
failure in `ExtremaPC2d_HyperbolaTest.SingletonInvalidAndValueAccessor`. No successful
paired receipt was generated. The unchanged single test passes at `-O0` and at
`-O2 -fno-builtin-sinh -fno-builtin-cosh`, but fails at the native build's `-O2`.
See [reproduction and limitations](../../docs/tkgeombase-native-runtime-blocker.md).
