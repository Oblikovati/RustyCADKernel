# Same-body prism proof join V1: frozen candidate evidence

Source: `afb12e5ccc0119344c6aecd9157cddd6fc4be751`, reconciled with reviewed main
`d59cbce3cff316151890483233defb673269213e`. This receipt applies to that exact
source, before the later requested structural extraction.

## Executed scope

Rust 1.99.0, two Cargo build jobs, isolated worktree target, Linux platform and
full compiler fingerprint in `receipt.json`. All **15,871** tracked crate and
workspace/configuration input hashes match before/after.

- TKBO all targets/all features: **1,384 passed**.
- TKBO doctests: **35 passed**; documentation generation passed.
- All-target/all-feature Clippy with `-D warnings`, workspace formatter check: passed.
- Two independent processes: complete joined proof **193,080 bytes**, frozen V1
  **89,921 bytes**, complete measures **207 bytes**, cap subset **319 bytes**;
  each pair equal. Complete join's eight worker operations also match serial bytes.
- Owned containment V1 fixture remains unchanged. Legacy diagnostics, incomplete
  coverage and body completion remain unchanged.

The full test command is:

```sh
CARGO_BUILD_JOBS=2 cargo +1.99.0 test -p rustycad-tkbo --all-targets --all-features --locked -- --test-threads=1
```

The command list, timings, exact replay hashes and compiler/platform details are
in `receipt.json`; compressed logs, input hashes and complete replay artifacts
are retained. The as-run scripts preserve the actual checkout path. The initial
standalone complete-measure selector used a file name instead of the registered
module name and executed zero tests. Its output assertion rejected that run;
the corrected exact selectors each executed one test. No zero-test run receives
runtime credit. Both scripts and the failed selector log are preserved.

## Independent acceptance evidence

The join checks actual-context/authored-name correspondence, three faces, six
trimmed-domain self/pair conclusions, all raw body vertex uses and a bijection of
all twelve semantic closure contexts. It retains parent multiplicities before
frozen set projection. Independent raw-point tests use local x=1 translated by
2^53: rounded world x remains 2^53 although exact transport is 2^53+1. Initial raw
probes detect hidden-rounding changes before callbacks can inherit old evidence.

Tests cover typed missing/malformed/unsupported admission, wrong contact and
source names, changed chart/range/extrusion operands, parent multiplicities,
input/output isolation, final callback edits of original coefficients, raw body
and source points, and cap trims. Cancellation at every observed callback is
followed by successful byte-identical reuse. Original query maps, bounds and
options remain owned; parameter map/root-width encoding preserves bits without
admitting invalid values. No new geometry or intersection evaluator is added.

## Limits

TKMath/TKBRep sources are unchanged; their standalone suites were not rerun in
this TKBO-only gate. No new OCCT execution, GPU, native oracle or release
benchmark was performed. Complete history, outward orientation, ordered
validation, exact represented-geometry tolerance and operation publication are
separate gates; this result does not complete Kernel #10 or establish a valid body.

ThreadSanitizer remains incomplete under the retained host startup limitation
in `docs/evidence/native-display-lifecycle-v1-2026-10-14/TSAN.md`. No bypass or new
TSan claim is made. Compile-time Send/Sync and ordinary concurrency/replay tests
do not establish complete thread-safety acceptance. Development logs include
preflight compile/Clippy failures and corrections; only the frozen matrix above
is this revision's full gate.
