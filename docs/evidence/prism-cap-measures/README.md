# Source-bound prism cap and signed-volume measurements

Source revision: `9584ca4388245ed6ac111aec32642c6e797752ff`, based on frozen `a42ab0646`.
ADR0027 implements an additive prerequisite for issue #10's analytic
region-to-solid operation. It uses the existing polynomial integral, exact
metric arithmetic, classifier, Jordan certificate, copy/sweep history and
actual-chart proof pipeline. No body is published or promoted to valid.

## World-area arithmetic

Native Green integral I has plane-coordinate area units. The world-plane
measure is `sqrt(I² S)`, where S is the exact Gram determinant of the two placed
support columns. I, columns and S remain exact until the final directed square
root. No stored floating axis is assumed exactly unit; no intermediate rounded
norm or area can overflow/underflow and contaminate a finite final result.

The separate immutable `CertifiedPolynomialPlaneMeasure` reports a nonnegative
adjacent-binary64 enclosure, deterministic lower endpoint, upward-rounded width,
original/effective finite world-area error cap, and certified algebraic evidence.
It does not change the rational-only promise of `CertifiedPolynomialMeasure`.
A zero or singular scalar is not a valid region. Open/multiply traced boundaries
have conditional scalar expressions, not certified region area.

Only polynomial Bézier/affine-line occurrences over full native `[0,1]` are
admitted, with explicit reversal, degree at most 16 and at most 64 occurrences.
Weights, including equal weights, are explicitly unsupported. No policy clamp,
tolerance widening, estimated-error substitution, sampled acceptance, faceting
or retry ladder is introduced. Cancellation is polled before/after the shared
fixed-step square-root helper. Typed failures preserve underlying curve indices,
metric causes, options, exact squared-value diagnostics and achieved error.

## Source-bound composition

The private inspection is separate from PrismReceipt and frozen publication.
It freshly runs the existing classifier for current seed/root options, verifies
the selected identity and winding, then recaptures current original/census,
both Jordan candidate domains, copy, closure points and complete source history.
It computes and retains the current exact nonzero affine Jacobian; mutable
retained status or determinant rows cannot substitute for this proof.

Actual body contexts and every copied-query image parent are checked. Exact
parent multiplicities/dispositions are replayed through the existing copy and
generated history composition. Actual body output relations are recomputed.
The admitted single-boundary domain requires exactly one First cap, Last cap and
Generated side, each with unique history and complete current trim coverage.
Each actual chart is recaptured and reproved with the existing exact identity
engine. The side proof retains its positive height and exact agreement of stored
direction times height with requested extrusion; normalization equality is never
presumed.

The actual cap pcurves and support/placement coefficients supply cap measures.
The copied oriented native boundary and exact extrusion determinant supply the
signed sweep integral, with no area rounding before multiplication. After all
callbacks, a callback-free binding pass rechecks current source/candidate/copy,
closure-point directed contexts, body/history, chart coefficients, trim identities
and expectation operands. Any observed edit leaves the inspection incomplete.
Concurrent writes to legacy mutable Shape aliases are not authorized by this
operation-local binding. Successful reports contain only owned scalar captures,
immutable Location keys and operation-local identities, not Shape/geometry aliases.
These identities are not persistent document names.

The completed subset contains two cap measures and one signed sweep measure.
Side-area binding and total-surface-area composition remain explicitly pending,
although the independent side math API is integrated. Remaining ordered body,
self-intersection, tolerance, achieved operation error and publication checks
stay incomplete. Scalar measurement does not establish final-body validity.

## Independent expectations and checks

For poles `(0,0),(4,0),(0,4),(0,0)`, independent Green integration gives I=12/5.
Exact QQ augmented-Gram determinants independently give S=338 for columns
`(2,3,5),(7,11,13)` and
`81129638414606685298668707040461 / 81129638414606681695789005144064 > 1`
for stored binary64 columns `(.6,0,.8),(0,1,0)`. The latter's rounded floating
squared length is 1. Full oracle requests/results are retained. Tests compare
squared enclosure endpoints with those exact values and pin the binary64
neighbors of 12/5 and 24/5 independently.

Generated tests cover scaled/reversed boundaries, reflected/translated supports,
negative sweeps, oblique placement, near-cancelling Gram products, singular and
extreme/subnormal cases, and unchanged explicit error budgets. Source, copied
geometry, pcurves, evaluator overrides, parent multiplicity, missing/extra/
reparented body/history and late callback edits are refused. Every observed
cancellation poll is exercised; retry is deterministic. Source/candidate/copy
geometry and attached mesh caches survive successful, failed and cancelled
measurement inspections.

On Rust 1.99.0 with two build jobs:

- 20 focused plane-measure tests and 28 source-bound composition tests pass.
- 4,318 affected TKBO/TKMath all-target/all-feature tests pass.
- 88 doctests, Clippy with denied warnings, and formatting pass.
- 216 standalone extracted predicate tests pass without OCCT.
- Two independent processes emit byte-identical scalar receipts. Eight independent
 operation workers × four rebuilds match serial receipts; plane arithmetic also
 exercises eight workers × 32 reports.
- All frozen command source bookends match. 145 new functions across 20 files pass
 the 4–20 physical-line audit; all files remain below 500 lines.

```sh
cargo test -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features --locked
cargo test -p rustycad-tkbo -p rustycad-tkmath --doc --all-features --locked
cargo clippy -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
cargo fmt -p rustycad-tkbo -p rustycad-tkmath --check
```

## Retained attempts and provenance

The shared sqrt pair originates in 19d6c14fd and was first cherry-picked as
56313554b. Rebasing onto the integrated side branch correctly removed this
already-applied dependency. Exact support-column extraction preserved the
Jacobian fixture bytes and all 7 before/after tests, including the original
three-poll cancellation/error order. Structural chart access/owned-scalar Clone
came from reviewed frozen-agent equivalents f57d069bb/5afe541f7, originally
6ab9bb8e6/756793d39; no frozen projection WIP is a dependency.

The archive preserves initial diagnostic-only attempts. The first composition
run exposed an incorrect assumption that cap parents all move to modified.
The existing copy composition retains generated face-construction ancestry;
the corrected check replays that existing composition with exact per-disposition
multisets. A cap trim range mutation was initially expected to fail at a later
chart stage. It changes the shared copied edge range: the regression now pins
actual `[0,.5]`, frozen `[0,1]`, unchanged pole and the precise earlier
`pre-sweep/current copied geometry` refusal. Topology mutation fixtures initially
hit `TopoDS_FrozenShape`; the test now explicitly unlocks the operation-local
fixture before its intentional edit. The function audit also caught an empty
split test; the final missing-image case asserts the exact unattributed-member
refusal. These earlier runs are not frozen acceptance.

The frozen base a42ab0646 was amended upstream to 42740916a only to correct the
regularity receipt's candidate-file count 19 to 18. This branch did not rewrite
source during verification; integration must retain main's corrected evidence.

## Sanitizer, exclusions and retained bytes

Ordinary-host ThreadSanitizer acceptance remains incomplete. The isolated
instrumented standalone build succeeded with nightly-2026-09-25 and build-std,
but the test process exited with SIGSEGV before any test began (Cargo exit 101).
The log establishes neither a passing test nor a race conclusion. Source hashes
match before/after; exact compiler, command and environment changes are retained.

No host protection or instrumentation was masked, disabled or excluded. No
unsafe Send/Sync implementation, global mutable algorithm state, native OCCT
run or new native capture was added. The unchanged native fixture in extraction
is historical replay only. No application acceptance, healthy body, total area,
release migration/deletion gate or previous-release accepted-workload benchmark
is claimed.

`runs.tar.gz` contains final exact commands/logs/hash bookends, source/corpus
extraction equality, standalone source+lock+authored support+historical fixture,
independent oracle payloads/results, initial attempt logs, process receipt bytes
and audit scripts. Build artifacts are excluded. Extracted replay:

```sh
cargo test --manifest-path final/extracted/Cargo.toml --offline --locked --lib
```

`receipt.json` records source/dependency revisions, options, platform, oracle
kinds, executed checks, exclusions and sanitizer status; `SHA256SUMS` seals this
set. The legacy scalar and batch-only prism paths remain available unchanged.
