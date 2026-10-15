# Kernel #10 release acceptance matrix

Status: **incomplete; release condition unsatisfied**. Audit date: 2026-10-14.
Source baseline: `bb5f065290ddbe0059384970b1d2ca3fefa9d118` on local `main`.
This is a requirement/evidence map, not a capability approval. Active branches
are identified separately; their existence, review or test totals do not close
an owning requirement. No implementation, tests, native oracle, publication or
remote mutation was performed by this audit.

## Authority and source precedence

1. [Kernel issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10)
   owns all seven requirements below. It is open, has no assigned milestone and
   has no comments at the read time. The authenticated kernel milestones API
   returned an empty list; no Q1–X2/T1 assignment is inferred.
2. [ADR0014](adr/0014-owned-analytic-sketch-regions.md) specifies the shared
   pipeline, initial cubic operation, expanded corpus and application R3
   deletion gate. Its earlier progress paragraphs describe historical states;
   later constituent evidence below advances those states without granting
   complete operation authority.
3. The current root [AGENTS.md](../AGENTS.md), including the replacement
   operation/ownership/verification rules, governs implementation. Current root
   [blockers](application-blockers.md) and [priority list](porting-plan.md#current-priorities)
   put Kernel #10 first. Those two user-edited files were read, not modified.
4. The [conditional release authorization](evidence/kernel-10-release-authorization-2026-10-14.json)
   permits release **after** the complete Kernel #10 capability and owner gates.
   It supersedes the issue's older no-push wording for that release; ordinary
   development remains local. It does not waive general profiles or naming.
5. [Application epic #3563](https://github.com/Oblikovati/Oblikovati/issues/3563)
   is open and unassigned to a milestone. Its linked
   [R3/R5 plan](https://github.com/Oblikovati/Oblikovati/blob/rust/architecture/rust-port/milestones.md)
   and [K02/K03/K04/K11 ledger](https://github.com/Oblikovati/Oblikovati/blob/rust/architecture/rust-port/martensite.md)
   require parameter-driven valid solids, provenance and persistent references.
   R3 includes edit/recompute, save/reopen, undo/redo, deleted and ambiguous
   references. R5 requires the complete baseline feature corpus. K02 also covers
   revolve; that broader family is not silently added to Kernel #10's extrusion
   requirement, nor declared complete by this work.

The API reads, upstream blob identities, local authority hashes and candidate
snapshots are recorded in the small [audit receipt](evidence/kernel-10-release-acceptance-2026-10-14.json).
Older OCCT comparisons remain attributed historical fixtures. Their retention
is required; new OCCT execution is neither an oracle nor a completion requirement
under the replacement rules. Go sources are comparison provenance to audit for
known defects, never a production fallback or copied sampled classification policy.

## Canonical milestone repository supplement

The Q1–X2/T1 milestone plan in AGENTS belongs to the separate
[Oblikovati/Martensite repository](https://github.com/Oblikovati/Martensite/milestones).
It is not absent merely because this checkout's `Oblikovati/RustyCADKernel`
remote has no milestones. Root rechecked the live milestone list and the three
C1 issues on 2026-10-14; the [supplemental source receipt](evidence/kernel-10-milestone-supplement-2026-10-14/receipt.json)
retains those responses. The earlier audit receipt describes its original
`57754d864` document snapshot; this supplement extends that authority inventory.

- [Martensite#11](https://github.com/Oblikovati/Martensite/issues/11) requires a
  real-consumer operation result with owned revision, completion, validation,
  history and diagnostics; incomplete history or validation forbids strict success.
- [Martensite#12](https://github.com/Oblikovati/Martensite/issues/12) requires
  requested/effective/achieved quality and method, bounded integration with its
  error, and explicit inconclusive analytic refusal. Tessellation cannot close it.
- [Martensite#13](https://github.com/Oblikovati/Martensite/issues/13) specifies a
  parameterized **sketch with a hole**, prism, dependent reference, edit,
  undo/redo, save/reopen and display. Per-face surface types, loop counts and
  closed-form areas must pass through the new result. Production and diagnostics
  use one implementation; whole-body volume alone cannot pass.

These are existing canonical gates, not new milestone assignments for
RustyCADKernel#10. The single-Jordan polynomial operation in E7/E8 is a necessary
bounded prerequisite. It does not satisfy the with-hole operation, consumer
workflow or C1 completion. Repository-qualified issue numbers are required:
`Martensite#10` is a different issue from `RustyCADKernel#10`.

## Evidence vocabulary and source locations

- **Integrated bounded proof:** accepted source/evidence for a stated private
  domain. It does not mean complete region construction or public API readiness.
- **Candidate:** committed branch source with the stated checks; main integration
  and its exact combined gate are separate.
- **WIP:** uncommitted work or a proposed interface. Owner-reported focused tests
  are observations, not a sealed acceptance receipt.
- **Open:** no complete requirement-level acceptance was found in the inspected
  source/evidence. This does not assert that no related primitive exists.

Source paths below are relative to `crates/tkbo/src/`. `sketch_region`,
`sketch_region_candidates`, `sketch_region_prism` and its publication candidates
are private at the baseline; see [lib.rs](../crates/tkbo/src/lib.rs). An inner
`pub` or `pub(crate)` declaration is not an application-facing exported contract.
Candidate-only files can be inspected with `git show <revision>:<path>` after
integration; they are not falsely linked as files present at this baseline.

## Integration supplement at `4cf1da12b`

The original audit and E1–E8 observations below retain their recorded revisions.
The following subsequent root checks advance three bounded constituents on
local `main`; they do not close an issue-level requirement by themselves.

| Constituent | Integrated source and root evidence | Verified scope |
| --- | --- | --- |
| E4 exact conformance and tolerance | Source `bb5f06529`, [root receipt](evidence/prism-exact-tolerance-integration-2026-10-14/INTEGRATED.md), evidence commit `75f2f717d` | 6,323 tests +302 documentation tests across TKMath/TKBRep/TKBO; 38 candidate files exact; six process outputs preserve complete V2 conformance/tolerance and older frozen/inspection bytes. |
| E5 joined named measures | Source `d596b9b0b`, [root receipt](evidence/prism-joined-measures-integration-2026-10-14/INTEGRATED.md), evidence commit `f621c83c8` | 1,479 tests +35 documentation tests; all 1,438 frozen candidate passing test names retained; eight process outputs preserve named measures, legacy scalar and exact-tolerance bytes. The sweep integral remains signed. |
| E6 ordered validation | Source `12faf0730`, [root receipt](evidence/ordered-prism-validation-integration-2026-10-14/INTEGRATED.md), evidence commit `4cf1da12b` | 1,517 tests +35 documentation tests; 30 candidate files exact; all 1,498 frozen candidate passing names retained; twelve process outputs preserve six full receipts, including ordered validation. |

Each root check also passed strict Clippy, formatting, warning-denied generated
documentation and unchanged source bookends. These totals describe their own
revisions and overlapping suites; they must not be summed as distinct coverage.
Completed worktrees and branches were deleted after integration and checks.

Complete history (E7) and the single complete publication operation (E8) are
still candidates at this supplement. Their focused tests, internal success
types and independent review do not constitute main integration or a public
operation. Owned caller requests, semantic naming, owned refusal projection,
general profiles, the with-hole consumer workflow and owner release checks
remain open. ThreadSanitizer acceptance remains incomplete.

## Complete private operation integration supplement

The subsequent actual-main gates advance the private representative operation:

| Constituent | Integrated source and root evidence | Verified scope |
| --- | --- | --- |
| Semantic naming reservations | Source `4c9511d80`, root evidence `52d794b06` | 1,531 tests +35 doctests; full literal naming bytes reproduced in two processes. Reservations do not prove actual construction correspondence or persistent split identity. |
| E7 complete history | Source `4d02ab813`, integrated source `c88ca1760`, [root receipt](evidence/prism-complete-history-integration-2026-10-14/INTEGRATED.md) | 1,598 tests +35 doctests; six process outputs preserve full history, lineage and frozen bytes. Complete original/intermediate query coverage is bound to the one actual private operation. |
| E8 complete publication | Source `3e8bb7dc3`, integrated source `80c107451`, [root receipt](evidence/prism-publication-integration-2026-10-14/INTEGRATED.md) | 1,628 tests +35 doctests; one composer returns bound ordered validation, complete history, per-face measurements and the unchanged signed sweep integral. The terminal callback precedes callback-free source/history checks. |

These overlapping test totals describe separate revisions. They are not distinct
coverage counts. Each root matrix includes strict Clippy and formatting; its
supplement records full process replay and generated-documentation checks.

The complete private operation is still a single-Jordan polynomial domain.
Owned caller construction, actual semantic correspondence, public owned outcomes,
general profiles and holes, persistent edit/persistence references, sanitizer and
release-package acceptance remain open. Neither this supplement nor the private
`CompleteNamedPrismV1` status closes Kernel #10 or authorizes an early release.

## Owned boundary and physical-volume integration supplement

The subsequent actual-main gates add two bounded prerequisites:

| Constituent | Integrated source and root evidence | Verified scope |
| --- | --- | --- |
| Conditional physical volume | Source `805e33928`, integrated source `53147a45c`, [root receipt](evidence/prism-joined-volume-integration-2026-10-14/INTEGRATED.md) | 1,641 tests +35 doctests; exact orientation transports the signed enclosure and representative without changing certified error or policy. Six process outputs preserve volume, measures and publication. |
| Owned request preparation and refusal projection | Sources `06a9f7e71` / `893373d1e`, integrated source `ff70563b9`, [root receipt](evidence/owned-prism-boundary-integration-2026-10-14/INTEGRATED.md) | 1,689 tests +35 doctests; owned coefficients construct native endpoint identities independently of traversal. Supported diagnostic projections own their values; unsupported retained payloads remain explicitly incomplete. Twenty-six process outputs preserve thirteen full receipts. |

Both matrices passed strict Clippy, formatting and warning-denied generated
documentation with unchanged source bookends. Their overlapping totals describe
their own revisions. Completed worktrees and branches were deleted after checks.
The preparation candidate's 3,696 observed cancellation prefixes remain covered
by its identical integrated test; no portable whole-preparation replay is claimed.

Actual semantic correspondence, portable body publication, complete owned
diagnostics, general profiles including holes, persistent consumer references
and release-package acceptance remain open. The [sanitizer diagnosis](evidence/prism-tsan-startup-diagnosis-2026-10-14/README.md)
records startup failure on both installed nightly runtimes. The injected host
library is observed, but its causal responsibility is unproved. No actual
operation sanitizer pass or host-security workaround is claimed.

## Actual semantic resolver and portable body integration supplement

Source `fd98c89fd`, evidence `4e75cb9da`:
[combined root receipt](evidence/prism-resolver-body-integration-2026-10-14/INTEGRATED.md).
The actual sealed invocation now binds authored names to the constructed operation,
retains reservation coverage, and projects the accepted private publication into
an immutable portable body. Root ran 1,931 tests and 35 doctests plus strict lint,
formatting and documentation. Eighteen processes reproduced nine complete
candidate payloads; source hashes and candidate passing names were preserved.
Both completed worktrees and branches were deleted after checks.

This advances actual single-boundary semantic correspondence and owned body
reads. It does not close the public one-call outcome, complete owned failures,
general profiles or the sketch-with-hole consumer workflow. At that checkpoint,
annular binding, embedding and named topology remained development candidates.
The supplements below advance their individual gates; complete ordered
validation, measurements, history and publication remain required.

## Publication diagnostics and annular binding/embedding supplement

The bounded sealed publication diagnostic adapter is integrated at `e04266c24`,
with evidence and deleted-checkout receipt `6a51fe8f2`:
[publication diagnostics](evidence/prism-publication-diagnostic-integration-2026-10-14/INTEGRATED.md).
Root passed 1,963 tests and 35 doctests, strict lint/fmt/docs, and thirty fresh
process checks against fifteen retained replay records. Exact source/request/
policy associations and typed native/scalar causes are owned where supported.
Unsupported nested families retain their actual original carrier plus explicit
gaps. This does not close the complete owned failure boundary.

Actual annular binding and embedding are integrated at `949990450`, evidence
`275b064c6`: [combined receipt](evidence/prism-annular-binding-embedding-integration-2026-10-14/INTEGRATED.md).
Root passed 2,040 tests and 35 doctests, lint/fmt/docs, and forty-six process
checks against twenty-three retained full replay records. Both candidates and
all previous main passing test names remain; overlapping counts are not added.
All 16,319 source/configuration hashes were unchanged. Both completed worktrees
and merged branches were deleted after the clean/ancestor/process audit.

This evidence covers the actual nested polynomial pair, source/copy membership,
four charts, twelve trims, complete classed and captured-flat parent association,
regularity limits, four parameter domains and ten embedding incidence claims.
Final callback-free recapture prevents issuance after a stale source mutation.
The reviewed same-role cap-rim operand-swap regression is retained with its red
and green logs; valid nominal bytes did not change.

The public general-profile operation, complete annular ordered validity/history/
measures/publication, owned failure families and application edit/undo/reopen/
display workflow remain open. These private prerequisites grant no release
approval. ThreadSanitizer and complete owner package/benchmark acceptance remain
incomplete under the existing authority and exclusions.

## Every issue requirement

| Issue item | Concrete implementation / executed evidence | Precise remaining acceptance |
| --- | --- | --- |
| **1. Owning ADR, deletion gate, preserve mapped minimum, general route** | ADR0014 owns deletion in application R3. `sketch_region/arrangement.rs::NonemptyArrangement` uses `BuilderHooks::check_data`; `arrange_owned_sources` reuses filler, builder, copy and history. Existing mapped and historical fixtures remain selectable. | Record the eventual application replacement and deletion against the full corpus and exact released version. This item supplies architecture/private admission credit, not permission to delete the diagnostic today. |
| **2. Immutable analytic inputs, IDs, plane, explicit policy/cancel; single/multiple/empty** | `ArrangementSource`, `RegionSourceId`, `arrange_owned_sources`; `PrismRequest`, `retain_source_bound_prism`; explicit copies and source revision checks. Private admission/isolation tests and E1 below. | Public owned analytic request DTO and operation context; no borrowed mutable `Shape` contract as the final boundary. Prove single/multiple/empty outcomes through that contract. Current complete-certificate path admits one full-range Jordan Bézier boundary, not arbitrary profiles. |
| **3. Complete status/typed source diagnostics/ordered coverage/error/history/isolation** | Private receipts distinguish incomplete, no-body, failure and cancellation. E2–E6 supply owned joined operands, exact orientation, conformance, tolerance, scalar bounds and ordered-validation candidate. History/publication are WIP E7–E8. | One returned operation outcome containing all facts, completed empty distinct from decline/failure/incomplete, immutable owned public failures, exhaustive original/intermediate/output history, complete copy composition, final source/history rebind and success/failure/cancel/mutation isolation. Legacy `PrismReceipt` contains mutable aliases and has no completed-body variant. |
| **4. Exact/filtered bounded-region and seed classification; explicit boundary/ambiguity** | ADR0009/0010 contributions/wire winding and `sketch_region_candidates/classification.rs::classify_candidates` share the exact solver. Boundary and multiple-interior outcomes do not select first match. E1 certifies a tightly bounded Jordan source/candidate correspondence. | Carry exact root/pruning completeness, construction errors, shared contacts and source revision binding through every admitted profile and split. General intersections, nesting, touching and ambiguous regions must pass the full operation, not only the supplied-curve winding query. Sampled `BRepTopAdaptorFClass2d` remains inadmissible as exact authority. |
| **5. Full correctness corpus** | Historical native region fixtures, exact classification tests, original cubic and generated local reversal/scale/extrusion cases; E1–E6 preserve independent expectations and failed attempts. | The operation-level corpus table below remains open. Primitive success and historical eight-case cardinality experiments do not prove every required representation/interaction. Retain all existing per-face assets and independently extend correctness evidence without new native captures. |
| **6. Region → prism → valid named solid, including one closed curve; persistent split identity** | E2 authenticates caller-authored names against actual faces/trims/parents; E3 proves outward/trim directions; topology/embedding are bounded constituent proofs. E7/E8 target a first private complete unsplit polynomial operation. | Complete that operation with geometry/validation/history/measures bound to identical bytes, then prove a public named solid. Caller-supplied names and parent sets alone do not establish persistent split identity. A consumed reference must survive parameter edits and file round-trip, with deleted/ambiguous references explicit. |
| **7. Tested owner release and exact app adoption; no fallback** | Conditional release authorization exists; [owner workflow](releases.md) defines sealed packages, exact tested revision/tag and extracted-package checks. Existing v0.2.7 is a prerequisite release, not the completed region extension. | Meet all preceding rows, semver/changelog review, full committed-source owner gate, immutable release/tag/package manifests, exact application pin, then replace the diagnostic with production `sketch.profiles`/`features.add(extrude)` integration and consumer tests. No sibling-source, Go or OCCT runtime fallback. |

## Evidence ledger: capability and revision, not accumulated test credit

| ID | API / scope | Evidence actually recorded | Limit |
| --- | --- | --- | --- |
| **E1** | `region_certificate::retain_source_bound_prism`; original/copy/construction/contact/candidate source binding | [Region integration](evidence/region-certificates/INTEGRATED.md), source `8d4009c02`: TKMath/TKBO 4,209 tests +86 docs, lint/fmt and exact source bookends. [Winding integration](evidence/region-winding-solver-integration-2026-10-14/receipt.json) records solver/winding reuse. | Four candidate obligations discharged only under current exact hypotheses; whole body remains incomplete. One full native range, standard evaluator, no extra contacts, complete current two-directed-face universe. Positive-weight rational support in some constituent checks does not imply complete rational-prism acceptance. |
| **E2** | `frozen::proof_join::{compose_prism_proof,rebind_joined_prism_proof}`; owned common source, copied charts, original closure queries, names/parents | [Join integration](evidence/prism-proof-join-integration-2026-10-14/INTEGRATED.md), source `31fa1b8b2`: 1,384 TKBO +35 docs, lint/fmt/docs, two-process full 193,080-byte join equality. | Same-body immutable proof input, not full original history, public request, current validity or stable edit identity. |
| **E3** | `frozen::joined_orientation::inspect_joined_prism_orientation`; pure 3-face/6-trim directions | [Orientation integration](evidence/prism-joined-orientation-integration-2026-10-14/INTEGRATED.md), source `0d2bef9d9`, receipt `072e785e8`: 1,413 +35 docs and two exact 215,177-byte replays; independent stored-pole/traversal/extrusion cases. | Full join-bound conditional directions. Ordinary-host TSan crashed before tests; sanitizer acceptance incomplete. |
| **E4** | `exact_tolerance::{certify_joined_prism_conformance,certify_joined_prism_tolerances}`; exact represented endpoints/closures/trims plus stored metadata policy | [Frozen receipt](evidence/prism-exact-tolerance-v2-2026-10-14/final/receipt.json), source `378fc0db8`: 6,292 tests +302 docs/lint/fmt. [Reconciliation](evidence/prism-exact-tolerance-v2-2026-10-14/reconciliation/receipt.json), `e9890ebc8`: 41 focused +21 orientation and exact replay. Source merged at baseline `bb5f06529`; final root combined gate was in progress at audit start. | Exact represented-geometry proof and metadata observations remain separate quantities. No full operation, general approximation-error promise or sanitizer pass. |
| **E5** | `measurements::inspect_complete_prism_measurements`; conditional cap/signed sweep/side/total; joined wrapper `inspect_joined_prism_measures` in candidate | [Integrated math/measure receipt](evidence/prism-complete-measures/INTEGRATED.md), `1978079a4`: 4,397 +88 docs, independent Fraction/isqrt fixture regeneration. Joined candidate `64c6fa30b` adds eight domain cases; prior `20703816f` records 50 legacy/14 joined tests, 718 cancellation cutoffs and unchanged full replay. Later reconciliation is active. | Native-plane integral is not world area: cap uses exact placed-column Gram factor; sweep is signed exact determinant multiplication; side uses certified chart extrusion. Scalar completion cannot establish valid body. Joined wrapper full/main gate is separate. |
| **E6** | `frozen::ordered_validation::validate_joined_prism`, ADR0039; exact topology → geometry → SI → tolerance on one join | Reviewed prefix `dec0d4566`: recorded 26 focused tests and strict Clippy. Typed stage ledger, conformance reuse and complete SI domain/pair correspondence; later reconciliation `1fbbe02bf` was observed, not credited with the prefix's tests automatically. | Validation-only certificate. Full source-frozen/main gate pending at observation; no history/measures/live rebind/publication. Historical checker failure remains recorded and is not relabeled as an exact pass. |
| **E7** | `frozen::complete_history`, candidate branch `kernel/prism-complete-history`; original/copy/partition/candidate/selection/sweep/query census | Committed structural prefix `52cd40d4c`; uncommitted behavior. Owner reports seven actual-operation tests for complete unsplit chains and retained empty/false native queries. | Operation/rebind wrapper, adversarial corpus, final checks and ADR0037 pending at observation. One unsplit polynomial curve; no certified persistent split or edit mapping. `ComposedDeletionCoverage` remains in legacy receipt. |
| **E8** | `sketch_region_prism::publication`, ADR0041 candidate; planned one join + validation/history/measures + final rebind | Committed `97400f5f0` with baseline/progress and reviewed dependencies. Owner reports four baseline/progress tests and unchanged original join bytes. Live adapters under implementation. | No complete entry/success constructor at observation. Await reviewed history operation. Initial refusals may retain legacy mutable diagnostics; public owned failure projection remains required. |

E5–E8 identify active local commits/paths, not available released APIs. Their
candidate receipts must be retained when root integrates; temporary target logs
are not durable final acceptance evidence. Where earlier receipts list a later
completed prerequisite as open, read that statement at its recorded revision.

## Required operation corpus

Each row needs input, expected outcome, independent oracle, per-face/trim/history
checks, policy/error results and exact source revision. A complete status for one
row must not imply completion of the others.

| Domain / behavior | Current evidence scope | Missing full-contract proof |
| --- | --- | --- |
| Single closed polynomial cubic | Exact Jordan, charts, embedding, orientation, conformance and bounded measures; baseline seed `(1,1)`, extrusion `(0,0,2)`, volume `4.8`, three faces. | Finish E7/E8 and public complete named-solid/edit round-trip gate. |
| Single circle and ellipse; arcs | Historical nonempty-builder/analytic fixtures; some rational Bézier primitives. | Full admissible analytic representation/domain conversion, certified contacts/trims/classification and complete operation. Polynomial-only measures/theorem do not cover all conics. |
| Multiple curves / split intersections | Shared arrangement/partition/contact and source relation prerequisites. | Complete split construction/root-error/search evidence, per-output parent composition and persistent split identity, then named-solid operation corpus. |
| Nested loops / holes / multiple regions | Supplied-wire winding and explicit boundary/ambiguous selection machinery. | Complete region topology/nesting and sweep validation for each selected component; no silent one-wire restriction as release scope. |
| Touching / near tangent / coincident / self-intersecting | Independent primitive root/contact tests and typed incomplete outcomes. | Certified complete contact census and intended operation outcome for each case; ambiguity/refusal is explicit where required, never arbitrary selection. |
| Splines / rational / periodic domains | Existing spline constructors and exact polynomial/rational primitives are prerequisites. | Full supported representation/trim coverage, domain decomposition, certified search/construction/measure bounds and shared pipeline acceptance. |
| Scales / planes / placements / direction / traversal | Exact placed-support Jacobian/chart identities; generated scalar scales and orientation reversals. | Operation corpus across applicable scales, oblique/translated planes, sign changes and extreme/degenerate requests with unchanged limits. Exact normalization mismatch remains incomplete. |
| Empty / multiple / boundary / ambiguous seeds | Private cardinality admission and explicit selection outcomes. | Public typed decline vs completed empty vs failed/incomplete/cancelled, stable source diagnostics and isolation. |
| Budgets / cancellation / reuse / source mutation | Many constituent cutoffs, exact bit policies, owned replay and final rebind tests. | Unified public operation at every checkpoint, current revision/history/policy rebinding, no partial successful solid. |
| Naming / deterministic order | Explicit authored names and full process replay; no coordinate/order-derived IDs. | Reference consumption across actual edits/splits, save/reopen and undo/redo; deleted/ambiguous outcomes; no address or root-rank persistence. |

## Canonical contract cross-check

The following groups map the current AGENTS/ADR0014 contracts to their remaining
operation-level gate. They are not optional because some primitives already pass.

| Contract group | Existing seam | Acceptance still required |
| --- | --- | --- |
| Completion + diagnostics + validation + history + achieved error together | E8 planned assembler; E1–E7 constituents | One coherent typed outcome; no invalid/partial geometry on success; completed empty distinct from failure/decline; affected source identities on all refusals. |
| Ordered, body-kind-specific, explicit exact coverage | E3/E4/E6 and frozen topology/embedding | Full ordered gate for publication; local changed-entity checks need explicit contracts before being offered. Missing/inconclusive evidence stays incomplete. |
| Complete history incl copy/healing | E7 + existing maker/copy relations | Deleted/merged/created/split/derived relations with all parents and full query coverage. No implicit healing; if healing is introduced, copy plus identity remap and composed history are required. Document naming remains application-owned. |
| Shared general pipeline, exact topology vs metric error | Shared arrangement, roots, winding, chart and incidence code | No special-shape retries, sampled/faceted acceptance, second classifier or fallback. Complete certified pruning/root selection; shared incidence computed once and reused across consumers. |
| Numerical policy and reported evidence | E4/E5; explicit spatial/parameter/root and metadata policies | Preserve requested/effective/achieved quantities separately, certified/sample/unknown tags; distinguish angular/modelling/approximation quantities where applicable. Report/refuse/no-limit explicit; no widened limits or invented aggregate error. |
| Immutable ownership and revisions | E2/E3/E4 owned snapshots and complete replay bytes | Public inputs/results/errors and all reachable geometry/topology/cache/mesh ownership; lock-free reads; operation context `Send`; no global/thread-local algorithm state. Source/options/policy keyed reuse and all outcome isolation. |
| Concurrency and deterministic identity | Existing Send/Sync assertions, independent workers, repeated/two-process replay | Repeat unified operation stress and sanitizer on supported instrumented runner. Current ordinary-host TSan startup failures are incomplete, not waivers. Any new unsafe Send/Sync requires safety reasoning and Miri/loom. |
| Independent correctness and per-face evidence | Closed forms, exact/rational interval oracles, reviewed fixtures | Generated degenerate operation corpus, every face/trim/tolerance/history, no volume-only acceptance. Review each changed baseline with reason/version; no weakened tests. |
| Performance and distribution | Owner packaging workflow; timing logs are observations | Identical **accepted** workloads versus previous release, time and peak memory; if previous release cannot perform the operation, record that limitation and compare supported overlaps separately. Measure copy/allocation/lock costs before storage redesign. Full extracted-package tests without OCCT available. |
| Versioning, migration and cleanup | Additive private V1/V2 seams; ADR0014 R3 deletion task | Public semantic-version/changelog decision; earlier behavior replay until recorded migration; delete superseded diagnostic paths only after full corpus + released consumer gate, within the recorded milestone. No milestone reset to defer deletion. |

## Next dependency-ready work after the complete-operation gate

First finish and integrate E6–E8 on one exact source, including all final rebind,
history, independent result, failure/cancellation and replay tests. That completes
only the representative private operation gate.

The next smallest slice is an **owned application-facing request and outcome
boundary** over that existing operation: analytic coefficient inputs with stable
source/directed-use IDs, plane and separate policies; internally build/copy the
mapped geometry; return immutable typed status/diagnostics/current coverage,
history and achieved errors. Project legacy failure payloads into owned source
identities and causes instead of exporting mutable diagnostic `Shape` aliases.
Reuse the existing assembler; do not expose its stack of private receipts as the
application's obligation or add a parallel arrangement engine.

Prove the already admitted polynomial operation through that boundary with
single/multiple/empty admission, all isolation outcomes and extracted-package
compilation. Unsupported domains must stay explicit **open requirements**, not a
redefinition of release scope. Next extend the same pipeline's certified split
and multi-boundary coverage for the required analytic/profile corpus, and prove
actual reference continuation across the consuming edit and persistence with the
application owner. Do not claim that an unsplit authored key proves split naming.

Release remains gated on every issue row and the owner package workflow; consumer
adoption then closes the separate application replacement/acceptance work.

### Named annular topology integration supplement

Local main `3cdfa7535` integrates the sealed four-face, six-wire, twelve-trim
annular topology candidate. Root checks passed 2,063 tests and 35 doctests,
strict lint, formatting and docs, plus fifty independent replay processes.
See [integration evidence](evidence/prism-annular-frozen-topology-integration-2026-10-14/INTEGRATED.md).
This is a private prerequisite; the remaining release requirements above still apply.


### Paused batch integration supplement (2026-10-14)

Local main source `0bae73e49` integrates annular exact conformance/tolerance,
conditional four-face measures, ordered annular validation, shared history helpers
and actual census fixtures, plus owned post-publication projection refusals. The
complete private single-boundary owned operation was integrated at `5b7366168`.

The [combined root evidence](evidence/kernel-10-batch-integration-2026-10-14/INTEGRATED.md)
records **5,384 TKMath/TKBO tests and 89 doctests**, strict lint, formatting and
warning-denied docs, plus **70 exact-binary replay processes**. All prior-main and
candidate tests remain qualified by package/target/name, and all 16,451 compiled
source/configuration hashes are unchanged. Counts overlap earlier revisions and
must not be added as distinct coverage. Completed trees and branches were deleted.

**Complete annular history remains unimplemented**: only its shared helpers and
native census fixtures landed. Annular publication, physical-volume certification,
owned public inputs/outcomes and complete failure families, general profiles,
consumer edit/undo/reopen/display, sanitizer and owner package/performance gates
remain incomplete. These constituents do not satisfy Kernel #10 or authorize release.

The user requested finishing the current batch and a handoff. Broader work is
paused; [the handoff](handoff-kernel-10-2026-10-14.json) records exact revisions,
remaining work and preserved older worktrees. No follow-on porting task, PR, push
or release was performed during this wrap-up.

### Annular history, publication and owned preparation supplement (2026-10-15)

A worktree branch (not yet integrated or root-gated) adds, on top of `6f158ea74`:

- ADR0058 V2 complete annular history issuer: six original directed uses
  continued through source-specific copies and native partition queries,
  continuation-derived partition parents, all three authored candidates with
  sealed dispositions, captured flat parents bound to native sweep rows, two
  separate intermediate wire retirements, certified-empty original deletion.
- ADR0061 named annular publication: one join, ordered validation, V2 history
  and four-face measures bound to one source; terminal callback before the
  callback-free source rebind and history reprojection. Closed-form cap areas
  (9/5) and sweep magnitude (18/5) lie inside the certified enclosures.
- ADR0062 owned annular preparation from owned coefficients through the shared
  census/partition/copy/contact/prism pipelines; it publishes end to end.
- ADR0056 follow-up: complete publication history findings are owned.

Still open: physical-volume certification and semantic naming for the
annulus, owned projection of annular publication/preparation causes and of
resolution/preparation/partial-transaction families, a public owned outcome,
general analytic profiles beyond two nested closed polynomial loops,
persistent split identity, consumer edit/undo/reopen/display, sanitizer and
owner release/package/performance gates. Nothing here closes Kernel #10.

### Annular refusals, volume, names and one-call operation supplement (2026-10-15)

A worktree branch on top of `58e06355f` (not yet integrated or root-gated) adds:

- ADR0062 follow-up: owned annular preparation refusal. Admission, role and
  stage causes keep typed operands; the full request is encoded by
  `Martensite.OwnedAnnularPrismRequest/v1`. Retained partial transactions and
  nested census/contact/region-prism/stage-completion families stay explicit
  gaps, so post-construction refusals are not yet complete.
- ADR0061 follow-up: owned annular publication refusal. Version, binding,
  history cancellation, all callback stops, moved validation and measure child
  failures (scalar causes with every partial certificate), history findings
  without local owners and owned-operand join causes are complete. Borrowed
  join families and nested numerical validation/measure failures stay explicit
  gaps with the original retained.
- ADR0063: annular physical volume from the certified material-normalized sweep
  and the exact Jacobian sign of the same join; closed form 18/5 for every
  orientation variant, both signs exercised, unchanged certified error.
- ADR0064: production semantic names for the annulus (role-derived tokens plus a
  caller dictionary) replacing test fixtures on the production path.
- ADR0065: `run_owned_annular_prism` one-call owned outcome (crate-private):
  caller-named faces with certified areas, volume, validation coverage, history
  counts and receipts, or a typed refusal with stage, class, stop and gaps.

Still open: owned projection of resolution (single-boundary resolver), region,
binding, naming and embedding refusals, retained partial transactions,
legacy/lineage/frozen/geometry history families and nested numerical families;
a public (crate-exported) facade, withheld while ADR0014's private gating
stands; a portable annular body view; general analytic profiles (circles and
other rational curves need certified rational integration and a rational
region proof; more than two loops and several regions need an N-loop region
proof; naming that survives splits); consumer edit/undo/redo/save/reopen/
display; ThreadSanitizer; owner release/package/performance gates.
