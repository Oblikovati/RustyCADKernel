# Martensite original-source and parameter conformance

This ADR0014 prerequisite certifies supplied analytic representations. It does
not certify contact identity, mapped splitting, candidate coverage, a healthy
region or a solid. Existing missing-operation obligations remain unchanged.

## Capture and ownership

`capture_boundary_inputs` admits callers, snapshots every actual cumulative
original edge occurrence, then performs the existing one-fresh-aggregate
`BRepBuilderAPICopy(true,true)`. Original snapshots use the same complete
`OriginalMemberIdentity` as copy history: source key, physical topology identity,
location and orientation. Identity addresses never recover original objects.
The retained coefficients, native trim and separate placement factors are read
from each actual source before copying or plane pcurve construction.

Every copied segment retains its actual copied curve, stored pcurve, support
plane coefficients and the unchanged candidate controls/native trim/basis trim.
Original member keys bind each proof to the real copy/history parents. Original,
copied and stored native ranges must agree; the output trim is never assumed to
be the original source range. SameParameter is a prerequisite, not proof of
conformance. Unsupported analytic bases, absent curves, missing pcurves and
unproved parameter bindings remain typed incomplete causes.

The inspection owns coefficient snapshots and occurrence metadata, without
keeping mutable geometry/topology handles as proof. Repeating inspection runs
fresh computations and charges actual work; there is no global cache or claimed
zero-work reuse. Later source or copy mutation cannot rewrite retained scalar
snapshots. Source and construction snapshots remain available after input drop.

## Three independent checks

For every original parent, inspection checks:

1. Original placed 3D curve against the actual stored pcurve lifted through the
   supplied support plane and location, on the complete original native trim.
2. Actual copied placed 3D curve against that same stored planar representation.
3. Original 3D curve against the unchanged captured boundary basis.

A limit exceedance or incomplete stage stops that parent's remaining stages.
Other parents retain their own attributed results. Requested and effective
spatial limits are equal. The original-plane and construction limits are
separate, and neither substitutes for parameter width or achieved incidence
world error. Bounds prove the requested inequality, not a tight measured maximum.

## Exact transport

The legacy same-parameter metric APIs and candidate line capture stay unchanged.
The new `TransportedResidualQuery` carries independent maps for each curve.
`MetricParameterMap::Endpoints([a,b])` represents exactly `a+(b-a)*u`; stored
binary64 `a` and `b` are converted separately before subtraction. Homogeneous
integer power coefficients undergo exact substitution, with one common scale
for every coordinate. No origin, span or placement product is rounded first.
The existing threshold/Sturm/sign engine supplies the proof; no new solver or
curve approximation is introduced.

A stored line retains native origin/direction. The captured candidate line still
uses the native rounded endpoint evaluations and basis `[0,1]`. Conformance
therefore measures that real construction error rather than replacing or
snapping the endpoints. Bézier controls and positive rational weights retain
their native basis and actual full trim.

`native_parameter_membership` maps exact dyadic root bounds into native
parameters, then compares the actual binary64 mapped parameter as an exact
number. It never divides by a rounded span or snaps a split. Reversed maps order
the resulting bounds independently of topological orientation. A singleton or
membership result alone cannot establish algebraic contact identity; the native
incident-edge/generated-vertex/history and complete incidence obligations remain.

## Work and failures

The new metric terminal reports completed subdivisions even after cancellation
or exhausted isolation. It uses the shared `isolate_accounted` event, emitted
only after both child cells were stored at the original increment point.
Inspection subtracts completed work from the requested aggregate budget across
stages and parents. Polynomial preparation, cancellation polls and sign failures
preserve their original typed causes; no failed-work refund or inferred count is
used. Legacy metric query poll/error order is preserved by its no-op observer.

## Boundaries

The currently admitted concrete line and polynomial/positive-rational Bézier
representations are covered. Other analytic bases remain explicit unsupported
capture causes, not line or faceted fallbacks. Degenerate maps can certify a
constant supplied representation but cannot imply a unique contact. Invalid
parameter evidence or transported Bézier domains are incomplete. Full general
region acceptance still requires exhaustive pair/self domains, actual mapped
partition correspondence, candidate coverage, validation and tolerance contracts.
