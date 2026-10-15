# ADR 0068: Public planar regions, extrusion and OperationResult

Status: implemented on a worktree branch (2026-10-15); integration and release are the
coordinator's.

## Context

The application's "first cube" slice (Oblikovati epic #3563, kernel region #10) calls a
prism-specific adapter that accepts only selected straight segments and exposes
`BRepPrimAPI_MakePrism` through a diagnostic snapshot. The owned annular work (ADRs
0058-0065) and the general loop region proof (ADR 0066 part C) are crate-private and
publish no body for general profiles. The application needs one public, released seam:
solved sketch curves to validated planar faces, a general extrusion, and an immutable
result with history, findings and achieved tolerance.

## Decision

The seam is the public module `rustycad_tkbo::profile_extrusion`, re-exported as
`rustycad_kernel::modeling`. It lives in TKBO because the certification it depends on
(ADR 0066) is crate-private there; the facade keeps the application's import path stable.

### Input and resolution

`SketchProfile { plane, vertices, curves }`: caller vertex ids and curve ids; lines and
circular arcs end at caller vertices, circles have none. Curves join only through shared
vertex ids, never by proximity. `Ellipse` and `Spline` are accepted as input variants and
declined by name (`UnsupportedCurve`).

`Resolution` keeps the tolerances distinct and has no default: `weld` (distinct vertices
closer than it, and curves shorter than it, decline), `sew` (largest admitted gap between
an arc end and its circle carrier), `angular` (plane frame orthogonality, minimum arc
sweep, extrusion direction against the plane), `display` (chordal and angular deflection,
carried to the consumer and never used for modelling) and `cap`: `ToleranceCap::NoLimit`,
`Report(limit)` (finding) or `Refuse(limit)` (decline).

### build_planar_regions(profile, resolution) -> PlanarRegionResult

One path for every profile, in order:

1. Admission of every caller value.
2. Loop assembly from the vertex graph: a vertex with one curve end is `OpenProfile`, more
   than two is `BranchingProfile`. Loops are ordered by their first curve in caller order.
3. Certification by the ADR 0066 general loop region proof, unchanged. Each curve becomes
   proper Bezier pieces: a line one degree-1 piece; an arc or circle quarter-turn pieces.
   Axis-aligned quarters use the exact stereographic form (corner on the diagonal,
   weights `(1, 1, 2)`), which keeps the exact census and area arithmetic short (an
   annulus certifies in 0.2 s instead of 30 s with `cos(pi/4)` weights in debug builds);
   other pieces use weights `(1, cos half, 1)`. Authored vertices are the exact piece ends
   and each split point is computed once. Each loop is one authored loop and one selected
   region, seeded half way from its first use's midpoint to the nearest inward boundary
   crossing. The seed is only a hypothesis: the proof locates it exactly, and the result
   must name the loop as outer loop and exactly its child loops as holes, otherwise the
   operation declines `IncompleteCertification`. An unexpected contact maps back to the
   two caller curves (`SelfIntersecting`).
4. Native construction: one vertex per caller vertex (tolerance raised to cover the sew
   gap of arc ends), one edge per caller curve on a `Geom_Line` or `Geom_Circle` carrier,
   shared by every face that uses it; outer wires counterclockwise about the plane normal,
   holes clockwise, orientation from the closed-form signed loop area.
5. Ordered validation per face, then the tolerance stage against the cap.

Every closed loop bounds one region: the loop minus its direct children. A circle in a
circle therefore yields the ring and the inner disc; the caller picks which to extrude.
`PlanarRegionSet::curve_edges()` maps every caller curve to exactly one native edge and the
regions it bounds.

Area: the certified area of the region's Bezier boundary, checked against the closed form.
Line-only regions are represented exactly and keep `MeasureKind::CertifiedBound`. Arc
pieces carry rounded split points and weights, so those regions add a representation
estimate and are tagged `MeasureKind::ClosedForm` (an estimate, not a proven enclosure).

### extrude(region, direction, options) -> Result<OperationResult, OperationDecline>

A linear sweep along the world vector `direction` (`BRepSweep_Prism` through
`BRepPrimAPIMakePrism` with copy, internal to the operation). The face is copied, so the
input region is never modified; a test checks that no result sub-shape is the same as an
input sub-shape and that input tolerances are unchanged. The direction must be finite,
longer than `weld` and outside the plane by more than `angular`.

### OperationResult

Immutable and owned, `Send + Sync` (asserted at compile time): the solid; its faces, edges
and vertices indexed once in deterministic native traversal order; `OperationHistory`;
`ValidationReport`; findings; `AchievedTolerance` (largest stored vertex, edge and face
tolerance); the volume `Measure` (area times height, exact for an affine sweep, error
propagated from the area); and the resolution. `detached_solid()` returns a deep copy with
fresh geometry for callers that need to modify it.

History records are `(source, role, relation, result)`, sorted, so equal inputs give equal
histories: both caps are `Generated` from `Region`; each `Curve` generates one `SideFace`, a
`StartEdge` and an `EndEdge`; each `Vertex` (or the seam vertex of a full circle, attributed
to its curve) generates a `LateralEdge`, a `StartVertex` and an `EndVertex`. Every result
face, edge and vertex must have exactly one record; a missing, shared or extra image is an
`IncompleteHistory` decline, never a partial result. `deleted` is explicit and empty for an
extrusion.

### Validation and self-intersection evidence

Topology and geometry use the shared ADR 0014 stages (`BRepCheck`). The shared native
self-check (`BOPAlgo_CheckerSI`) still runs and any interference declines, but by design it
always reports `PairCompletionUnobserved` and never certifies completeness. The
self-intersection pass therefore rests on a named certificate in
`ValidationReport::self_intersection_evidence`: `RegionContactCertificate` (the ADR 0066
proof) for faces, `AffineSweepOfCertifiedRegion` for the solid (the sweep
`(p, t) -> p + t d` of a certified region with `d` outside the plane is injective).

### Declines

`OperationDecline { operation, stage, cause }`, with `class()` (InvalidInput, Unsupported,
InvalidGeometry, Incomplete, OverBudget, Native). Causes name entities and values, e.g.
`OpenProfile { vertex, uses }`, `SelfIntersecting { curves, point }`,
`ToleranceOverBudget { achieved, cap }`. No operation panics on caller input.

## Reuse and what is not reused

Reused unchanged: the ADR 0066 profile preparation and loop region proof (a crate-level
entry `certify_profile` was added beside it), the ADR 0014 ordered validation stages, the
native builders and `BRepPrimAPI_MakePrism` history. The annular frozen projection,
history V2 scopes, semantic names and the one-call annular operation (ADRs 0058-0065) are
bound to the two-loop domain and the partition candidates; this seam builds faces from the
caller curves instead of binding partition candidates. Generalizing the frozen projection
and semantic naming to N loops, and binding partition candidates to regions, remain open
under Kernel #10.

## Evidence

`crates/tkbo/tests/profile_extrusion` (28 tests, under 1 s in debug): rectangle (box,
V8 E12 F6), clockwise rectangle, circle (cylinder, V2 E3 F3 with seam history), annulus
(ring with one hole and inner disc; tube with V - E + F - R = 0, genus 1), slot of two
lines and two tangent half-circles, two disjoint rectangles, square with a circular hole,
a circular segment with a general (non-quarter) arc, an oblique extrusion on a tilted
plane. Oracles: closed-form areas and volumes, independent adaptive Gauss-Kronrod face
area and solid volume (`BRepGProp` with `eps`, no tessellation), Euler-Poincare with rings,
exactly one side face per curve, both caps from the region, one parent per result entity,
sorted history, identical results across calls and threads, input isolation. Declines:
open, branching, bow-tie crossing (curves and crossing point named), crossing circles,
ellipse, empty profile, NaN resolution, arc radius mismatch, refused tolerance cap (region
and solid), report cap finding, invalid directions. Rustdoc examples on the module entry
points and the facade.

## Limits

Follow-ups (2026-10-15): ADR 0081 replaces `Measure`/`MeasureKind` with the shared
`Measured` (`ClosedForm` arc areas are now `RepresentationEnclosure`); ADR 0082 makes
`OperationResult::validation()` the public `validate_body` report; ADR 0083 admits
ellipses, elliptic arcs and clamped B-splines; ADR 0084 replaces the loop proof with the
plane graph proof, admitting shared edges and loops touching at a vertex. The items below
describe the state at this ADR.

- Loops must be disjoint (no shared edges between adjacent selected faces, no touching
  loops): the ADR 0066 proof refuses them. Tangent contacts between different carriers are
  refused as incomplete.
- Ellipses and splines are declined by name; the proof admits rational and polynomial
  Bezier edges, so adding them needs only their pieces, published carrier and history.
- The certified area of arc regions is of the Bezier representation; the representation
  error is an estimate (`ClosedForm`).
- Semantic names, persistent naming across edits and the application's adoption (parameter
  edits, save/reopen, undo/redo) are not part of this seam; history gives the evaluator the
  source-to-result map they need.
- This is additive public API; no version bump is made here (the coordinator releases).
