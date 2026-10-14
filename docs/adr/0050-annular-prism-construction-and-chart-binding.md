# ADR 0050: Annular prism construction and actual chart binding

- Status: construction baseline in progress; no annular body acceptance.
- Baseline: `0a429b1af`; reviewed two-loop fixture dependency `94e47df87`.
- Scope: RustyCADKernel#10 prerequisite; complete operation remains outstanding.

## Decision

Reuse `retain_selected_prism` with the actual selected two-loop candidate. The
existing whole-face copier and prism builder remain the only construction path.
The test seam is the retained diagnostic operation and its actual chart/history
inspection. No OCCT oracle, tessellation, fallback or inferred region certificate
is involved. The preliminary two-loop fixture supplies a candidate universe;
its selection alone does not certify an annular region or solid.

The independently authored combinatorial expectation is two cap faces, each with
an outer and an inner closed rim, and two generated side faces. Each closed source
edge gives two physical rim edges, one physical seam and two physical vertices.
Thus the body has four faces, six wires, six edges and four vertices. Cap trims
contribute four directed edge uses, side trims eight: twelve coedges in total.
Each physical seam must retain both directed uses and both stored pcurves.
Counts supplement exact actual context/history and per-trim checks; they do not
supply semantic identity or prove manifoldness.

## Structural extraction

After pinning this original-path result, extract the existing actual chart/history
capture mechanics independently of V1 single-boundary admission. Preserve the
V1 admission order, callbacks, causes and complete replay bytes. Plural consumers
must bind charts by exact source, role and full output context. The V1 role-only
chart replay matcher is not a plural matcher and will remain V1-specific.

## Pending composition

A later additive binding must connect the reviewed source-bound two-loop region
certificate to the actual selected face, prism copy, cap wires and generated
side charts, retaining parent multiplicity. It cannot infer annularity from wire
counts. The embedding theorem, weighted face-domain topology, exact conformance,
tolerance policy, complete history, bounded measurements and ordered publication
must all consume the same bound operands before any successful body is issued.

## Verification and migration

Pin independent incidence and full trim expectations; preserve every actual
refusal. Run the existing single-boundary replay corpus before and after any
structural extraction, including independent processes. Keep Rust 1.99.0, two
build jobs and exact source/evidence hashes. No new thread-safety claim is made
by this diagnostic fixture; the existing host TSan startup limitation remains.

Within the Kernel#10 milestone, consume the shared chart mechanics in the first
complete annular operation and remove temporary duplicate test adapters. A new
behavior version is required for plural certificate admission. Historical V1
admission and replay remain selectable until an explicit recorded migration.
