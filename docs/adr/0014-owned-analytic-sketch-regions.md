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
parameter edits, save/reopen, undo/redo and presentation acceptance. Native
OCCT remains a removable oracle. This decision supplies no production OCCT or
Go fallback and does not authorize publication.

At kernel `fd992c41f`, `tests/exact_region_prism.rs` is diagnostic only. Its
ownership regression records an input representation count changing from one
to two; explicit geometry copying proves only the tested success and subsequent
source-mutation case. These tests seed the corpus, not production completion,
ordered validation, naming or full isolation. No API completion credit follows
from this ADR.
