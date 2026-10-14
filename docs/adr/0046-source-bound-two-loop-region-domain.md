# ADR 0046: Source-bound two-loop selected region domain

- Status: proposed implementation; no annular body acceptance.
- Baseline: `80c1074511e0bc32220cc2a3899f5aa10a048bdc`.
- Scope: Kernel #10 prerequisite, after the existing single-boundary operation.
- Behavior: additive private `TwoLoopRegionBehavior::NestedPolynomialV2`;
  the V1 `RegionCertificate`, callers, classification and replay remain selectable unchanged.

## Decision

Use the existing source-bound pair/self census, exact directed winding classifier,
candidate builder, copied-member observations and captured geometry. Add one private
owning inspection at `sketch_region_prism::region_certificate::two_loop`. This is
a planar domain proof, not a completed prism, solid, topology certificate or public
request interface. No alternate evaluator, tessellated decision or retry path exists.

The admitted inputs are exactly two distinct full-range stored polynomial Bézier
Jordan boundaries in the same certified plane chart. Each actual self domain must
be complete and have only its exact native closure contact. The actual cross domain
must be certified complete empty, including exceptional fibres and rejected roots.
An incomplete report or missing domain never means disjoint.

The caller's outer/inner source roles are hints. Certify a nonzero winding of the
outer boundary about the inner boundary's bound native closure image; the witness
must equal that image in the common certified chart, without independently rounded
projection. Disjoint Jordan boundaries plus this witness prove strict containment.
Reversal preserves nesting but reverses the recorded winding sign. No assumption
of positive native traversal is permitted. A private inner-interior witness must
also have exact winding ±1 in the inner boundary. A separate material seed must
have winding ±1 in the outer boundary and zero in the inner boundary. The witness
bits, signed conclusions, policies and achieved work are retained. Future public
orchestration must obtain and check these witnesses using the existing pipeline;
application callers are not required to prove private intermediates.

## Interface checkpoint

`inspect_two_loop_region(request, progress)` returns `Inspected`, `Incomplete`,
`Failed` or `Cancelled`. The request borrows the actual
`PairedContactTransaction`, original `ArrangementSource` slice, explicit
operation/source/candidate/wire identity bindings and requested limits.

Operation-local occurrence keys are internal adapter inputs. They must not become
obligations of a future public analytic request. Caller-authored nonzero candidate
and wire IDs are lookup bindings, never names derived from order, coordinates,
parents alone or pointer values. Require an exact bijection over all actual
generated candidates and directed wire uses. The selected candidate hint must
have the proved outer and inner boundaries in the required facing traversal.
Every other actual candidate retains its explicit disposition and membership.

The immutable owned result has borrowed read views and contains:

- source coefficients, range, placement, support and directed closure operands;
- requested/effective source, pair and winding settings and completed work;
- complete per-self and cross-domain certificate projections;
- the exact nesting witness, its native role and signed winding evidence;
- full named source/candidate/wire/edge/endpoint membership with parent multiplicity;
- selected/rejected dispositions and explicit exclusions from body validity.

No Shape, geometry Handle, Report, registry, cache or lock is retained. Raw owner
identities are lookup-only and do not enter portable bytes. A source revision label
is provenance only: identical labels cannot conceal different captured values.

A callback-free `rebind_two_loop_region(request, certificate)` reprojects actual
current geometry, contexts, observations and proof operands and compares complete
owned source/domain/candidate replay and rechecks current legacy history consistency.
Complete composed copy/deletion history is explicitly excluded. The inspection
invokes this rebind after the last external callback. Concurrent
mutation through aliases is outside this operation-local capture contract.

## Bounds and failure policy

Retain the existing census's exact source and pair work accounting. The new winding
work has an explicit operation budget; charge completed work on failure and refusal.
A checked retained-operand budget applies before cloning nested coefficient,
certificate, ancestry, parent and membership rows. Limits are inclusive; no
truncation, implicit enlargement or wraparound is allowed. Structural units count
borrowed validation rosters and separately retained projection rows at their
consumption boundaries; they are not bytes or a total instruction quota. Exact
integer limb storage is not separately capped. Current evaluator coefficient
preflight is repeated after the plane-rank callback and before pair recapture.
Endpoint/chart admission uses exact zero residual, independently of later body
tolerance acceptance.

Missing source/actual membership, stale captures, unsupported representations,
unproved equality, incomplete coverage and exhausted work are typed incompleteness.
Native computation failures and cancellation retain their classes and causes.
Failure entities use authored IDs when bound. Owner-bearing legacy diagnostics
remain available separately if an exact owned projection is unavailable; an
unsupported payload cannot be replaced by a count or erased from success.

## Ownership and invocation boundary

The owned certificate is compiler-checked Send + Sync and read views take no lock.
`TwoLoopRegionProgress: Send` and a compile-time assertion make the private
`TwoLoopWork` progress/budget context Send. Synchronous input inspection borrows
`PairedContactTransaction` and its retained native-owner graph; this change makes
no transferability claim for that borrowed graph or owner-bearing local refusals.
Independent operations run on separate threads; this is distinct from transferring
native owners between threads and from sanitizer acceptance. No unsafe marker is
introduced. Full public operation/thread-safety acceptance remains a separate gate.

## Independent corpus and acceptance

Pin the existing V1 region/frozen/history replays before structural changes. Reuse
the current nested-rectangle candidate fixture as a construction baseline. The new
polynomial fixture uses the existing cubic and an independently proved strict
homothety; exact derivative/product responses and request schemas are retained in
the evidence directory. Mathematical derivation remains separate from kernel
observations.

Required cases include reversed traversal, role reversal, disjoint unnested loops,
touching/crossing/coincident loops, missing or duplicated domains/areas/wires,
same-parent distinct identities, same-label geometry edits, final callback mutation,
tiny/exact/one-short budgets, success/failure/cancellation isolation, compile-time
Send/Sync and deterministic process/thread bytes. No live OCCT oracle is used.

## Migration and deletion gate

This is an additive private proof domain, not a breaking released API change.
Within the Kernel #10 milestone, compose this proof into the existing current-chart,
embedding, complete-history, topology, measurements and ordered publication pipeline.
Delete any temporary proof-test adapter after the first complete named annular
operation passes that corpus. Retire overlapping V1-only proof glue only through a
recorded migration retaining historical replay; do not keep a second modelling engine.

Annular face topology requires explicitly bound nesting/domain evidence. Neither
wire count nor raw V-E+F alone supplies it. Geometry, self-intersection, tolerance,
history and measurement coverage remain outstanding until their ordered composer
accepts exactly the same source binding.

