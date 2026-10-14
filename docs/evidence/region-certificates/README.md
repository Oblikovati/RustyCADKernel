# Source-bound Jordan region certificate, version 1

Final source revision: `874b704379dfb3cc93d297d0014fe8ca9ee58908`, following
`fbeffb302` on base `b6e39c42a339908eee2fd2c9e5b8a99d2dbff9cc`.
This is an ADR0014 / issue #10 prerequisite in Martensite's existing region,
candidate selection, copy, sweep and history pipeline. It publishes no valid
region or solid. The original batch-only entry remains available for replay.

## Certificate hypotheses

The opt-in entry borrows the original bridge receipt. A temporary metadata view
feeds the existing pipeline; actual isolation is established by its deep copy
and source/copy mutation checks, not by cloned handles. Original outcomes,
observations, gaps and history remain intact.

Four candidate obligations can be discharged together only when all of these
current hypotheses hold:

- **Construction:** one full native `[0,1]` standard polynomial or positive-weight
  rational Bézier boundary has exact current original/census/candidate/copy
  coefficients, ranges, placements and pcurves. The declared seed plane agrees
  with the actual support. Original, census, actual partition, both directed
  candidates and prism-copy vertex coordinates are freshly read. The existing
  exact point residual engine proves zero error at the exact closure singleton.
  Nonzero residuals, including values below the earlier spatial allowance,
  remain incomplete. Requested spatial limit, separate zero certificate limit,
  root policy, measured bound and completed work remain explicit.
- **SharedIncidenceReuse:** the exhaustive source-bound self-contact census has
  exactly the closure pair `t=0,u=1`. Native parameters, report/root identities,
  physical vertex reuse and every directed copy/query context agree. Each actual
  physical vertex is measured once; all its directed contexts reuse that report.
  Parameter, spatial and identity checks remain distinct.
- **CandidateHistoryCoverage:** actual face/wire/edge containment agrees with
  captured generated descendants; fresh source/image/endpoint/history binding
  has no faults. Actual deep-copy relations compose every original parent and
  are bijective with selected occurrences. Identity comparisons retain exact
  multiplicities, so a duplicate cannot replace an omitted identity.
- **SearchCompleteness:** the existing exact census exhausts the whole admitted
  self domain, with no other contacts, failed work or unresolved searches. The
  actual generated universe has both complementary directed faces, each with
  one wire and one boundary. The existing winding pipeline freshly selects the
  bounded component with winding `+1` and complementary winding `-1`, preserving
  actual face orientation and boundary traversal. Neither order nor proximity
  supplies an identity. A current nonzero support/sweep Jacobian establishes the
  nondegenerate affine embedding needed by this entry.

Zero Jacobian leaves this bounded entry incomplete; it does not declare every
planar region invalid. Missing/extra/reparented geometry, ambiguous parents,
alternate evaluators, changed original/copy geometry, cancellation and reuse
cannot retain the four certificates. Recertification restores all four gaps
before attempting the hypotheses again.

Scalar evidence has compile-time `Send`/`Sync` checks. `Location` is immutable;
earlier comments attributing thread confinement to it were stale and are
corrected. Occurrence addresses are operation-local keys, not stable document
names. The enclosing `PrismReceipt` still contains mutable `Shape` aliases and
is not an immutable published body. Independent workers build their own
geometry and exchange only scalar receipt text.

## Independent expectations and executed checks

The representative polynomial has
`x(t)=12t(1-t)^2`, `y(t)=12t²(1-t)`. Its positive interior coordinates and
strictly increasing ratio `y/x=t/(1-t)` independently establish interior
injectivity, with only the common endpoint at the origin. Weights `[1,2,2,1]`
retain the same ratio and a positive denominator. Scaling, reversed traversal,
reordered candidates, retracing/open/extra-contact examples, source and copied
evaluator overrides, mutations, missing evidence, cancellation and recovery
exercise the same general path.

With pinned Rust 1.99.0 on the platform in [receipt.json](receipt.json):

- **40 focused region tests** pass, including all three isolated review regressions.
- **4,162 tests** pass across all TKBO/TKMath targets and features.
- **84 doctests**, Clippy with warnings denied and formatting pass.
- **138 independently extracted predicate tests** pass. The standalone
  source tree is byte-equal to this revision and requires no OCCT execution.
  An unchanged native fixture is retained strictly as historical replay.
- Two independent processes emit byte-equal scalar receipts, including actual
  captured closure points. The replay test also compares eight workers × four
  fresh operation-local rebuilds with serial output.
- Source hashes match before and after every frozen verification command.
  The function audit covers 151 functions: the 20-file region module plus the
  two narrow existing-engine point-query helpers. Every audited function is
  4–20 physical lines after formatting; all changed Rust files are below 500 lines.

```sh
cargo test -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features --locked
cargo test -p rustycad-tkbo -p rustycad-tkmath --doc --all-features --locked
cargo clippy -p rustycad-tkbo -p rustycad-tkmath --all-targets --all-features --locked -- -D warnings
cargo fmt -p rustycad-tkbo -p rustycad-tkmath --check
```

## Preserved failed attempts

The archive keeps all initial attempt logs. Early uncommitted working-tree
attempts are not presented as frozen revision acceptance. The first strict
closure-context assertion expected four raw query rows; the existing bridge
actually issues eight queries covering four distinct directed contexts. The
correction reuses the existing observer, compares exact query multiplicities,
and separately checks the four distinct contexts. No new incidence is invented.
An early test-builder compile failure and Clippy's large-error-variant refusal
are retained, along with the corrected boxed refusal result.

Review found two multiplicity holes and stale point conformance at `fbeffb302`.
The first regression run passed 31 and failed one: composed parents admitted
`[A,A]` against `[A,B]`. Full-entry tests for duplicated descendants and a large
vertex displacement declined at other gates, so they did not isolate those
predicate defects. The next run passed 29 and failed all three isolated cases:
`generated_universe` admitted the duplicated descendant; composed parents
admitted the duplicate; and recertification of an already retained receipt
cleared all four gaps after its actual shared vertex moved. These failures are
preserved separately from the passing corrected runs. The final implementation
uses one exact multiset helper and fresh source-bound point reports.

The old full verification pipeline at `fbeffb302` was deliberately stopped
during compilation before source edits for the review corrections. Its partial
log and interruption record are diagnostic-only, not frozen acceptance.
Only the final corrected revision has the complete successful verification and
matching source bookends recorded here.

## Remaining gates and retained bytes

`SelfIntersection`, `Tolerance`, combined prism injectivity/surface-chart
coverage, ordered solid validation and achieved-error coverage remain explicit.
No healthy body, immutable publication, area/volume acceptance, replacement
migration/deletion, application acceptance or previous-release accepted-workload
benchmark is claimed. The independently integrated conditional integral API was
not pulled into this bounded branch.

Ordinary-host ThreadSanitizer acceptance remains incomplete. This slice made
no new sanitizer run or unsafe trait implementation. The prior ordinary-host
surface-chart attempt failed before test output, as recorded in that evidence;
it is not a pass. No host protection or instrumentation was masked, disabled
or excluded. A supported instrumented runner is still needed.

`runs.tar.gz` retains all attempt logs, exact commands and exit codes, source
bookends, function audit and script, portable process bytes, extraction equality
hashes, and the complete standalone predicate package with Cargo.lock, authored
support corpus and unchanged historical fixture. Build artifacts are excluded.
After extraction, replay with Rust 1.99.0:

```sh
cargo test --manifest-path final/extracted/Cargo.toml --offline --locked --lib
```

The process comparison uses the `{"operation":"martensite.region-certificate"`
JSON result line, excluding harness timing and process-local addresses.
`receipt.json` records revisions, options, platform, oracle kinds and exclusions;
`SHA256SUMS` seals this evidence set.
