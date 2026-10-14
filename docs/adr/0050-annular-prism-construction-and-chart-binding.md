# ADR 0050: Annular prism construction and actual chart binding

- Status: private binding V2 implemented; review and integration gates pending; no annular body acceptance.
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

The additive `bind_annular_prism_v2` connects the reviewed source-bound two-loop region
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

## Private binding V2 contract

`AnnularPrismBinding` owns the complete two-loop certificate and the actual
source-keyed selected/copy correspondence, raw native closure operands, four
captured/proven charts, twelve complete directed trim contexts, signed nonzero
exact sweep Jacobian, and requested/effective sweep operands. All constructors
are private. Source/role/output keys include every ancestor and child slot;
repeated Generated roles and seam definitions are never inferred from order.
Copy correspondence preserves and reflects physical definition sharing.

Every chart and trim is explicitly joined to one full source/output/image-role
native relation. Two history views are retained separately: all kinded parent
multiplicities, and the captured flat sequence. The latter follows the existing
`all_parents` behavior: preserved, modified, generated traversal with exact
first-occurrence deduplication. Its sequence is mapped one parent at a time so
canonical class sorting cannot change its order. Both views are replayed using
authored source/native-role keys, never owner addresses.

Every external callback is forwarded in its original order. Source-domain
rebind and final selected/copy/chart rebind are required; no external callback
occurs after the final Commit checkpoint. Guarding is placed before actual
native materialization, not repeatedly during arithmetic on owned operands.
The existing V1 chart pipeline receives a no-op guard, with complete byte and
callback behavior preserved. Cancellation and callback failures retain their
original typed causes and operation/stage attribution. No partial binding is
returned on refusal.

## Bounded materialization policy

The admitted two-loop construction has fixed traversal envelopes: selected and
copied roots have maximum ancestor depth 3 and at most 9 directed uses each;
the swept root has maximum depth 5 and at most 48 directed uses. The existing
history preflight visitor performs this admission, including cycle and native
iterator failures. Its additive census counts actual directed uses and ancestor
elements in the same walk. Envelopes bound the declared subset; they supply no
geometry identity, topology certificate, or inferred annularity.

The requested `maximum_retained_units` remains unchanged. One cumulative
counter reserves canonical directed-context rows and their ancestor/slot rows,
nested borrowed history rows and parent occurrences, geometric descriptors and
polynomial poles, and the original-parent mapping roster/output rows. Reused
views of canonical contexts are not recounted as distinct logical contexts.
These are structural operand units, not bytes, allocations, or elapsed work.
Original kinded parent occurrences and captured flat singleton mapping rows are
charged separately because both are retained. Checked overflow is a typed
incomplete outcome. Exact cap and one-short fixtures pin the shared mapper;
no-limit behavior is introduced nowhere.

Before native coefficient capture, raw stored curve/pcurve/extrusion basis
counts use the existing two-loop polynomial pole admission and original
`maximum_poles`. Unsupported or rational representations remain incomplete.
Hierarchy, nested history, and representation admission run before initial
capture, after external stage callbacks, before each actual face capture, after
the final chart callback before full context/parent capture, and during final
callback-free rebind. Mutations that grow a copied curve, pcurve, extrusion
basis, copy hierarchy, or swept hierarchy are refused before the next relevant
native materialization. The earlier source-domain budgets remain separate and
are retained in the owned region certificate.

## Remaining authority and migration gate

This binding is a private prerequisite. It does not complete self-intersection,
weighted-domain topology, raw geometry/tolerance conformance, complete composed
history, integration, stable naming, public body ownership, or ordered
publication. Those consumers must use the same complete binding and rebind live
inputs after their final callbacks. Effective spatial/parameter settings remain
`None` when the retained diagnostic construction did not apply them; requested
limits and unknown achieved error are preserved without invention.

V1 single-boundary entrypoints and all prior replay receipts remain selectable.
At the Kernel#10 complete-annular-operation gate, retire the temporary raw
four-chart diagnostic test adapter in favor of the fully bound actual fixture,
after corpus/replay comparison. No replacement geometry or history engine is
introduced by this migration. The host ThreadSanitizer startup failure remains
an explicit unfulfilled instrumentation gate, not a waived acceptance claim.
