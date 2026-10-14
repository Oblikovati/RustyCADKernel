# Complete conditional prism measurements

Source `9e64161bbdc4825af1cc8340e1edf337e2c9c2bf`, based on integrated current-binding main `83eccdbcc`.
Decision: [ADR0031](../../adr/0031-complete-conditional-prism-measurements.md).
This additive private entry completes current-bound cap, side, total surface and
signed sweep scalar evidence for the admitted single Jordan polynomial prism.
It does not validate or publish a body. The earlier subset remains selectable;
its original portable receipt is pinned by an owned fixture.

## Existing geometry and arithmetic pipeline

The entry captures one shared `CurrentPrismBinding` and reuses the original cap
and signed-sweep arithmetic. The unique actual Generated side chart supplies its
stored polynomial basis and affine placement. The existing chart proof retains
positive height h, stored direction D and placement P and establishes exact
`E=-h linear(P)D`, together with complete four-role trim coverage over
`[0,1] x [-h,0]`. The unchanged side-integral API receives the actual placed basis
and captured E. There is no intermediate rounded height, norm or length product,
new scaled-vector API, new classifier or second geometric validation engine.

The complete result owns the shared snapshot and the unchanged component
reports. Original source/candidate/copied coefficients, exact Jacobian, current
winding/traversal, actual surface coefficients, direction/height/placement,
closure points, all trim expectations and operation-local history identities
remain attached to the shared snapshot. All external callbacks finish before
shared callback-free rebind. Tests edit original/copy/side geometry at the side
engine's final callback and the outer final callback and require fresh refusal.
This does not authorize concurrent mutation of legacy shape aliases.

## Exact endpoint sum and distinct budgets

The general `sum_nonnegative_enclosures` operation accepts finite ordered
nonnegative endpoints with an explicit finite nonnegative full-width cap. It
owns the original component endpoint bits/order and adds their exact dyadic
lower and upper values. Only the final lower/upper sums are rounded outward.
The lower endpoint is the representative; the upward-rounded exact published
width includes child uncertainty and final endpoint rounding. Empty input
completes as zero; malformed intervals, overflow, cancellation and excessive
width remain typed failures retaining operation, policy and offending values.

`EnclosureSumEvidence::CertifiedConditionalEndpointArithmetic` expressly tags
conditional arithmetic, not input geometric truth. The complete prism entry
retains each cap/side/signed-sweep report unchanged and applies a separate total
area cap. No budget allocation, clamps, retries or tolerance widening occur.
All world-area budgets use squared-coordinate units, distinct from signed-volume
units and topology/spatial/parameter tolerances.

Default cap/signed-volume limits are 1e-12, side area 1e-5 with 4096 maximum
subdivisions, and total area 1e-4. Degree maximum is 16; sum count cap is 64.
Mutation/replay tests explicitly request side error 0.01 and total error 0.02;
the independent numerical fixture tests use the original defaults. These
coarser test policies are recorded inputs, not hidden runtime clamps.

## Independent representative and invariant evidence

For the owned cubic `(0,0),(4,0),(0,4),(0,0)` and extrusion `(0,0,2)`, side area is
`integral_0^1 24 sqrt(p(t)) dt`, with
`p=18t^4-36t^3+26t²-8t+1=1/9+18(t-1/3)^2(t-2/3)^2`.
The retained Jacobian QQ factor request/result independently reconstructs p-1/9.
The offline generator also checks the exact derivative-norm power identity,
the positive factor identity and derivative Bernstein coefficients. These give
`p>=1/9`, `|p'|<=28/3`, `|p''|<=56`, hence `|f''|<=16128`.
Composite midpoint error is at most `672/N²`.

The generator uses N=131072, exact integer sqrt brackets on a 2^-80 grid and
exact rational accumulated endpoints plus analytic remainder. It imports no
production arithmetic or native kernel. The side fixture is enclosed by
`[11.521280478003524, 11.521280556234622]`; adding exact two-cap area 24/5 gives
`[16.321280478003523, 16.321280556234623]`. Tests require the entire independent
oracle interval inside the returned production interval. The offline generator
and exact JSON/TSV endpoints are retained; ordinary tests only read fixture bytes.
Regeneration reproduces both files byte for byte.

Additional checks cover half-ulp endpoint addition, subnormal sums, finite-range
overflow, malformed components, zero/invalid/error budgets, completed empty
sums, exact-order independence, generated scale/reversal/negative sweeps,
source/copy/side edits, precise direction mismatch, missing history, source and
candidate mesh-cache isolation, scalar work exhaustion and cancellation at new
composition boundaries. Original subset replay stays unchanged. Four independent
operation workers match a serial portable scalar receipt; sum arithmetic also
uses eight workers x 32 independent reports. Public scalar types and complete
owned inspection payloads have compile-time Send/Sync assertions.

## Frozen checks

Pinned Rust 1.99.0, two build jobs:

- 14 focused sum/independent-fixture tests and 17 complete-entry tests pass.
- The owned subset-v1 baseline passes unchanged before and after composition.
- 4,362 TKBO/TKMath all-target/all-feature tests and 88 doctests pass.
- Clippy with denied warnings and Rust formatting pass.
- 230 standalone predicate tests pass without OCCT dependencies.
- Two processes emit byte-identical portable scalar receipts; this excludes
  process-local source addresses and is not a frozen body serialization claim.
- Every command's before/after tracked source hashes match.
- 99 functions across 14 new Rust files and one offline generator satisfy the
  4–20 physical-line gate; every new file is below 500 lines.

```sh
cargo test -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features --locked
cargo test -p rustycad-tkbo -p rustycad-tkmath --doc --all-features --locked
cargo clippy -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
cargo fmt -p rustycad-tkbo -p rustycad-tkmath --check
```

## Provenance, earlier attempts and exclusions

The reviewed shared binding extraction `5685d1c0c` was authorized and picked as
`9c6969142`; rebase onto `83eccdbcc` dropped the equivalent dependency. The sole
rebase conflict was the region-certificate export tail: main's embedding baseline
module and the additive full-measure exports were both preserved. No binding
implementation was changed by this task. `receipt.json` records pre/post-rebase
source commit provenance.

Earlier attempts are retained as diagnostic-only. The first sum run passed ten
tests with missing field-doc warnings; fields were documented and scope/fixture
checks added. Initial complete-entry compilation exposed test-module path and
Debug-bound mistakes, corrected without changing accepted geometry. Two later
compiler jobs were intentionally stopped after source edits to avoid treating
mixed revisions as frozen acceptance. A rebased compile caught one remaining
test-only helper path, fixed before final source freeze. The first source-frozen full tests/doctests passed, but Clippy refused the new
176-byte error payload. A separate structural commit boxed diagnostic payloads,
retaining all source keys and causes. The preserved pre-boxing all-target test
executable emitted a portable receipt before any test rebuild; its hash and build
log provenance are retained. Rebuilt post-boxing output matches those bytes.
The complete corrected matrix was then rerun with new source bookends. No
numerical baseline or rejection stage was weakened.

Ordinary-host ThreadSanitizer acceptance remains incomplete. The isolated
instrumented build succeeded, but its process exited with SIGSEGV before any test began
(Cargo exit 101). The log establishes neither a passing test nor a race
conclusion. Exact compiler, command, environment and matching source bookends
are retained. No instrumented complete-operation pass is claimed.

No host protections or instrumentation were masked, disabled or excluded.
No native OCCT run/new capture, faceted fallback, sampled modelling acceptance,
unsafe Send/Sync implementation, release benchmark, application acceptance,
ordered valid-body check or immutable body publication is claimed. Legacy
private failure diagnostics may still contain mutable shape aliases; they are
not immutable successful results. Component source keys remain operation-local.

`runs.tar.gz` retains source bookends, commands/logs, independently generated
fixtures and generator, exact Jacobian request/result, source/corpus extraction,
standalone lockfile and historical replay fixture, portable receipt bytes and
all earlier attempts. Build artifacts are excluded. Extracted replay:

```sh
cargo test --manifest-path final/extracted/Cargo.toml --offline --locked --lib
```

`SHA256SUMS` seals the README, machine-readable receipt and archive.
