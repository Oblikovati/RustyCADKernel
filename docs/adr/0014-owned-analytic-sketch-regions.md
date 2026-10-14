# Owned analytic sketch regions

Kernel [issue #10](https://github.com/Oblikovati/RustyCADKernel/issues/10) owns
the application's analytic sketch-region construction contract. Implement this
as a Martensite extension over the existing interference, split, arrangement
and history stages. Keep mapped `BOPAlgoBuilder::CheckData` and its two-argument
minimum unchanged; the extension admits nonempty input through `BuilderHooks`,
including one closed analytic curve. Empty input is a typed decline.

Use one general pipeline for all admitted representations. Circle-specific
construction, sampled model replacements and application-local arrangement
engines would bypass shared incidences and history. Conditioned routes belong
inside that pipeline and require independent acceptance evidence.

Accept immutable analytic inputs with stable source identities, a plane,
separate tolerance quantities and cancellation. Own copied geometry and
topology before operations that can add representations or change caches;
`Clone`, `Arc` and non-destructive General Fuse do not establish isolation.
Compose copy identity relations with arrangement and prism history. Verify
caller isolation after success, failure, cancellation and source mutation.

Return completion, source-specific typed diagnostics, ordered validation
coverage, complete parent relations and requested/effective/measured error
together. Completed empty, declined, failed, cancelled and incomplete results
remain distinguishable. Missing or inconclusive required validation cannot
produce a successful body. Required validation proceeds through topology,
geometry consistency, self-intersection and tolerance consistency, using the
declared body kind. Unknown achieved error remains unknown.

Use exact or filtered topology signs independently of metric incidence limits.
Certify pruning, root completeness, construction error and candidate order;
reuse each shared incidence. Point residuals alone are insufficient. Boundary
and ambiguous seeds require explicit outcomes, with no first-match selection.
The fixture-specific classifier and sampled `BRepTopAdaptorFClass2d` path do
not satisfy this extension contract.

The first end-to-end acceptance workload is the existing closed cubic with
poles `(0,0), (4,0), (0,4), (0,0)`, seed `(1,1)` and prism vector `(0,0,2)`.
Its independent per-face receipt expects volume `4.8` and three prism faces.
Keep the single/multiple/empty admission cases and all existing native
compatibility fixtures. Extend acceptance to arcs, splines, intersections,
nesting, touching, near tangency, self-intersections, scales, planes, tolerance
limits, complete edge/vertex history and deterministic ordering. Prove a valid
named solid whose references survive the consuming edit and round-trip.

Issue #10 is also the deletion task for the application's diagnostic-only
construction boundary. Replace it within the same application R3 milestone
after the full corpus and a tested owner release pass, followed by application
parameter edits, save/reopen, undo/redo and presentation acceptance. Earlier native OCCT captures remain historical replay evidence only. This decision supplies no production OCCT or
Go fallback and does not authorize publication.

At kernel `fd992c41f`, `tests/exact_region_prism.rs` is diagnostic only. Its
ownership regression records an input representation count changing from one
to two; explicit geometry copying proves only the tested success and subsequent
source-mutation case. These tests seed the corpus, not production completion,
ordered validation, naming or full isolation. No API completion credit follows
from this ADR.


## Actual sweep coefficient correspondence, diagnostic version 1

The existing copied-selection/sweep/history pipeline captures the isolated
source support, boundary curves, pcurves, ranges and endpoint occurrence keys
before sweep. Actual output planes, extrusion bases, directions, placements,
pcurves and 3D trims are then captured independently through explicit history
contexts. A later shared-source/output edit cannot redefine the expectation.

The admitted correspondence domain is planar caps with full native `[0,1]`
polynomial or positive-weight rational Bézier boundaries and stored linear
extrusion sides with standard evaluators. Cap affine coefficients, translated
3D boundaries and native pcurves must match exactly. Side basis coefficients,
stored direction multiplied by trim height, all four rectangular pcurves and
3D trims must match exactly, including both occurrences of a closed-edge seam.
Stored placement chains remain separate to avoid rounded composition. Exact
homogeneous polynomial identities reuse the existing metric coefficient kernel.
A normalized direction/height product that differs from the requested extrusion
is incomplete; there is no tolerance relaxation or canonical-surface retry.

Every face/trim keeps the existing source and output occurrence keys and all
original parents. Current body containment must agree with retained history;
missing, additional or reparented faces cannot inherit old proofs. Scalar
observations describe the captured revision, not later mutable geometry.
Compatibility occurrence wrappers remain private and thread-confined.

This is coefficient correspondence, not a side-surface regularity theorem or
publication gate. Region/source correspondence, domain injectivity and complete
ordered solid validation still govern the single complete operation. The
combined prism gap and unknown achieved error remain explicit; no successful
body, replacement deletion or application acceptance follows from these proofs.
Existing behavior remains available; this additive diagnostic version introduces
no new release or migration gate. Issue #10 retains its existing deletion task
and full-operation corpus requirement. See the immutable
[surface chart evidence](../evidence/prism-surface-charts/README.md).

## Source-bound Jordan candidate certificates, version 1

The opt-in private `retain_source_bound_prism` entry borrows the completed
contact/copy receipts and current original sources. An explicitly temporary
candidate view enters the existing winding, deep-copy, sweep and history
transaction. The original outcomes, statuses and histories remain intact;
cloned compatibility handles supply metadata and do not establish isolation.
The prior batch-only entry remains selectable for replay.

Version 1 discharges only `Construction`, `SharedIncidenceReuse`,
`CandidateHistoryCoverage` and `SearchCompleteness`, together, when all of the
following current hypotheses hold:

- One full native `[0,1]` standard-evaluator Bézier boundary has exact zero
  original/plane/copied/basis residuals. Polynomial and positive-weight rational
  representations use the same path. Current source identities, coefficients,
  ranges and placements must still match the original captures. Alternate 3D
  and pcurve evaluators cannot be certified from their unchanged stored poles.
- The existing complete source-bound self-contact provider covers its full
  off-diagonal domain and reports exactly the joint endpoint pair `(0,1)`.
  Missing search, failed work or any additional contact leaves this entry
  incomplete. Continuity and absence of all other coincidences give a Jordan
  boundary; no new classifier or shape-recognition branch is introduced.
- Actual generation contains exactly two directed face domains for that
  boundary, each one wire and one edge with two opposite occurrences of the
  same physical endpoint. Both faces, every descendant and all parent and copy
  dispositions must agree with the retained generated contexts. The existing
  exact winding query selects `+1`; the complementary traversal has `-1`.
  Traversal, selected winding and both captured face identities are retained.
  Candidate order supplies no identity.
- Fresh source-domain, partition-span, image, generated-parent and candidate
  history inspections agree. Existing contact transport observations are
  rebound through actual copy links. Multiple native history queries may reach
  the same directed context: query multiplicity is checked against a fresh
  execution of the existing observer, while the distinct directed context set
  contains exactly four endpoint occurrences across the two faces.
- The actual isolated copy matches both current candidate coefficients and its
  retained pre-sweep snapshot, with a bijection of composed parent relations.
  The declared seed plane matches the actual support. The existing exact,
  nonzero support/sweep Jacobian establishes a nondegenerate affine embedding.
  Zero leaves this hypothesis incomplete; it does not invalidate every planar
  region operation.

The certificate is attached only after the existing transaction reaches its
incomplete diagnostic result. Failed hypotheses, cancellation and retries keep
all four gaps present; recertification first restores them. Mutated copied or
source geometry cannot inherit a previous certificate. The private receipt
retains compatibility occurrence keys and remains thread-confined. Captured
scalar evidence describes its recorded revision and does not authorize a later
mutated shape. Independent workers create their own compatibility geometry and
exchange only portable scalar receipt bytes.

This is a bounded progression toward the single representative operation. It
still publishes no healthy body. `SelfIntersection`, `Tolerance`, combined prism
injectivity/surface-chart coverage, ordered solid checks and achieved-error
coverage remain explicit. Conditional integral APIs are a separate dependency;
no area/volume or accepted-operation benchmark is claimed here. The existing
issue #10 deletion task, R3 migration limit and full-operation corpus gate remain
in force. Existing native fixtures are historical replay evidence only; none
are regenerated and no OCCT process is used as an oracle.
