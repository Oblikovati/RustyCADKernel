# Actual prism surface correspondence, version 1

Code revision: `1389bc35e1181d2b0ea412e19fe7c21280a0014b`. This bounded issue
#10 / ADR0014 prerequisite runs inside the existing copied-selection, sweep and
history pipeline. It does not publish a completed region or solid.

## Proven correspondence and limits

The admitted domain uses planar caps, full native `[0,1]` polynomial or
positive-weight rational Bézier boundaries, stored linear extrusion sides and
standard evaluators. Before sweep, owned scalar snapshots freeze the copied
support, 3D curves, stored pcurves, ranges and endpoint occurrence keys. Output
coefficients are captured independently after sweep through existing history.

Exact homogeneous polynomial identities check the complete cap affine maps,
cap translation and trims, side basis and actual direction × trim height, and
all four side pcurves and 3D boundary curves. Both occurrences of the closed
cubic seam are checked. Placement chains stay separate to preserve exact
stored coefficients. The oblique vector `(1,1,2)` demonstrates that normalized
direction/height rounding can fail exact correspondence; that result remains
incomplete without widened tolerances or a replacement surface path.

Existing source/context identities and every original parent remain attached.
Current body containment is revisited: changed bodies, missing history and
orphaned retained faces cannot inherit old proofs. Captured scalar evidence is
historical after later mutation and never authorizes the modified shape.

This proves coefficient correspondence, not boundary regularity, source/region
construction completeness, self-intersection freedom or solid validity. The
combined `PrismInjectivityAndSurfaceCharts` gap, ordered validation gaps and
unknown achieved error remain explicit. No successful body, migration gate,
replacement deletion, area/volume bound or application acceptance is claimed.

## Executed verification

With Rust 1.99.0 on the platform recorded in [receipt.json](receipt.json):

- The committed TKBO/TKMath all-target/all-feature run passed **4,122 tests**.
- All **84 doctests**, Clippy with warnings denied and formatting passed.
- The additions comprise 20 bridge tests and nine scalar identity tests.
- An independently extracted predicate source tree passed **138 tests**.
  All extracted source bytes match the committed tree. It builds without any
  OCCT dependency or execution; one unchanged native fixture replay is retained
  strictly as historical evidence, alongside the owned algebraic expectations.
- Two independent scalar-corpus processes produced identical result bytes.
  Each compares 64 generated cases with eight threads and fresh serial reuse.
- Two independent chart-proof processes produced identical proof bytes. Each
  checks independent surface/trim expectations, then compares eight workers ×
  five operation-local rebuilds with serial results. Compatibility geometry is
  created within each worker; only owned proof text crosses the thread boundary.
- Source isolation is checked after ordinary, cancelled and failed attempts.
  Other regressions cover interior edits preserving endpoints, edited direction,
  cap support co-mutation, native trim range edits, missing history, changed body
  containment, all coincident-input parents, both seam occurrences, reversed and
  located inputs, rational weights and unsupported straight-side canonicalization.
- Generated zero, negative and nonfinite heights, missing/duplicate seam roles,
  all scalar cancellation polls, collapsed maps, finite extreme scales and typed
  invalid weights retain explicit outcomes.

The source scan records 156 new functions across 21 files, each function 4–20
physical lines after formatting and every file below 500 lines. Compile-time
`Send`/`Sync` checks cover the public scalar API and private scalar coefficient/
proof types. Occurrence-containing diagnostic wrappers remain thread-confined;
there is no new unsafe trait implementation or full-result thread-safety claim.

```sh
cargo test -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features --locked
cargo test -p rustycad-tkbo -p rustycad-tkmath --doc --all-features --locked
cargo clippy -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
cargo fmt -p rustycad-tkbo -p rustycad-tkmath --check
```

## Preserved initial failures

`runs.tar.gz` includes the initial working-tree failure logs. Those early
attempts were not committed; they are not presented as a pinned earlier
algorithm revision.

The first attempt incorrectly expected the last cap's raw plane origin to be
translated. The actual cap keeps origin zero and stores translation in its
occurrence placement. More seriously, comparing against live post-sweep copied
support admitted a shared source/output edit. The correction freezes source
coefficients before sweep. The regression now proves that both shared live
supports moved while the retained source origin stayed zero, and the exact cap
identity rejects the edit. No equality or limit was relaxed.

A later regression expected an edited copied curve to fail at `SideBasis`.
Inspection showed that the surface basis is independently copied, while the
actual 3D trim shares the edited curve. The test now pins the frozen pole and
source occurrence identity, unchanged basis pole `4`, edited trim pole `6`, and
the precise `TrimSpaceCurve` rejection. The original failure log is retained.
The initial Clippy large-variant failure and the successful boxed-payload replay
are also retained.

## Ordinary-host ThreadSanitizer remains incomplete

The extracted test built with nightly `2026-09-25`, `-Zsanitizer=thread`,
`-Zbuild-std` and target `x86_64-unknown-linux-gnu`. Its executable received
SIGSEGV before test output; Cargo exited 101. The cause was not established.
This is not a sanitizer pass. No host protection was masked or disabled and no
altered-environment run was attempted. A supported instrumented runner remains
necessary for ordinary-host TSan acceptance.

## Retained artifacts and replay

`runs.tar.gz` contains every attempt log, process output bytes, command records,
function-line audit, extraction equality report, and the complete standalone
`extracted/` package with source, lockfile, authored workload support and unchanged
historical fixture bytes. Build products are intentionally excluded. After
extracting the archive, use Rust 1.99.0 and run:

```sh
cargo test --manifest-path extracted/Cargo.toml --offline --lib
cargo test --manifest-path extracted/Cargo.toml --offline --lib curve_identity_tests::generated_identity_corpus_is_deterministic_across_threads_and_reuse -- --nocapture
```

The recorded process comparison uses the `identity-corpus-v1:` or
`chart-proof-v1:` result lines, excluding test-harness timing. Source/receipt
hashes and exclusions are in `receipt.json`; `SHA256SUMS` seals this evidence
set. No previous-release accepted-operation benchmark exists for this slice;
incidental all-target benchmark hooks do not supply that missing acceptance.
