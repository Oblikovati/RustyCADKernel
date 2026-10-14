# Cumulative pcurve import V2 verification

Frozen source: `7a2b9e388d1432ce00ac8317d867246545365f38`. Rust 1.99.0, Linux host, build jobs 2.
The machine was shared with other work; timings are observations, not benchmarks.

- TKBRep all targets/all features: **1674 passed**, 0 failed, 0 ignored.
- TKBO all targets/all features: **1112 passed**, 0 failed, 0 ignored.
- Both doctest suites: **249 passed**, 0 failed, 0 ignored.
- Clippy all targets/all features with `-D warnings`: passed.
- Workspace formatting and explicit included test-module formatting: passed.
- Two separately executed process outputs and their strict JSON payloads were
  byte for byte equal. Eight independent operation threads also matched the
  owned snapshot in the ordinary test suite.
- All **15648** tracked crate/build-input hashes matched before and
  after checks, and again during receipt recovery. The V1 fixture remains
  `f54f5bdc7b770cfeb81d2d053650f2393824ec47946892dfdcf0eaf5171ee55a` from baseline `adcb2c311`.

## Finalizer recovery

The original orchestrator returned 1 after every command passed: its parser
expected JSON at line start, whereas libtest prefixes the test name. The raw
outputs already had identical bytes. `initial-finalizer-receipt.json` and
`initial-finalizer-raw.tar.gz` preserve that failed finalizer result. Recovery
strictly extracts one expected JSON object from each retained output, verifies
both complete output bytes and payload bytes, and rehashes every original source
input. The recovered `verification.json` records the method and original hashes.
No source or tests changed for this recovery.

`raw-verification.tar.gz` includes commands, logs, source bookends, original
runner, recovery program and diagnosis logs. Post-run hashes of executed test
artifacts are recorded; no pre-build binary provenance is inferred. Earlier
diagnostic runs were not frozen acceptance runs.

## Boundaries

V2 applies only to newly extracted definitions. Existing definitions are reused,
so migration requires a fresh/rebuilt graph and audit. The graph remains mutable
and alias-bearing; import success is not ordered body validation. Polygon and
triangulation lookup are unchanged. No live OCCT, GPU, application acceptance,
performance, full API/test parity or immutable publication is claimed.

ThreadSanitizer was not executed for this slice and its acceptance is incomplete;
the existing host startup limitation is recorded in
`../native-display-lifecycle-v1-2026-10-14/TSAN.md`. Ordinary thread/process replay
and the policy enum's compile-time Send/Sync check are narrower evidence.
