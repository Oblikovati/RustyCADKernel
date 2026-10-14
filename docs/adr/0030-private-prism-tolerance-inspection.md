# Private prism tolerance inspection V1

## Decision and scope

Add a private, operation-local inspection for the admitted single-Jordan
polynomial prism. This is a prerequisite for RustyCADKernel issue #10 and the
Martensite C1 / issue #13 consumer. It does not publish a successful body or
complete C1/C2. `PrismLimits`, `OrderedCoverage`, their existing replay behavior,
and the compatibility `BRepTool` tolerance accessors remain unchanged.

The inspection records three different quantities:

1. Raw stored vertex, edge and face tolerance metadata, with its entity and
   complete directed contexts. Raw values must be finite and nonnegative.
2. The effective value returned by the existing compatibility accessor. That
   accessor applies a floor; it cannot validate the raw value. In particular,
   negative values and NaN can otherwise become an apparently finite value.
3. Independently certified world-space geometric residual bounds on captured
   coefficients and endpoint incidences. These bounds are not stored tolerances.

The caller explicitly chooses a finite nonnegative stored-metadata cap in
`Report` or `Refuse` mode, or explicitly requests `NoLimit`. There is no default
policy. An independent finite nonnegative world-spatial budget constrains metric
residuals. Neither budget modifies geometry, raises tolerances, lowers metadata,
or changes the other budget. A report-mode cap exceedance stays in the report;
it is never silently discarded. No-limit still validates raw metadata.

## Evidence and reuse

Use the existing history/context binding and owned chart replay from the cap
measurement prerequisite. Use the frozen projection's exact endpoint residual
helper after its reviewed integration, with one shared scalar implementation.
Preserve all six directed trim contexts and all endpoint contexts even when
physical incidence arithmetic is shared. Never recover identity from coordinate
matching or returned order. Recheck captures before returning evidence.

Unknown, absent, unsupported, cancelled or exhausted metric obligations stay
incomplete. A zero chart identity is evidence about those coefficients, not a
blanket statement about the body. Missing incidence coverage prevents any
complete residual aggregate. Keep original typed causes and affected entities.

## Remaining tolerance semantics

This decision imposes no inferred vertex/edge/face tolerance hierarchy. Raw
metadata caps compare stored scalar quantities only. They are not a claim that
those quantities are world-space uncertainty. Location scaling and tolerance
transport must receive their own explicit policy and corpus evidence before the
ordered tolerance-consistency stage can pass. A transform builder scaling a
tolerance and an occurrence carrying a placement are different existing paths.

## Independent fixtures

Pin the authored closed cubic with extrusion `[0,0,2]`: exact chart and endpoint
residuals are zero. Raising only stored metadata to `1/8` must keep those metric
residuals zero while a metadata cap of `1/16` reports or refuses. Moving only an
output vertex by `1/8` must fail exact endpoint equality while unchanged chart
coefficients cannot conceal the discrepancy. Exercise invalid raw NaN, negative
and infinite values; missing/repeated incidences; placement and reversal; source
mutation; cancellation; and deterministic process replay. Existing exact planar
`1/8` residual tests retain their independent threshold expectations.

## Migration and deletion gate

`PRISM-TOLERANCE-INSPECTION-V1-COMPOSE` belongs to the Martensite C1 milestone
gate. Before composition, validate the admitted analytic corpus, settle placement
scaling and metadata-consistency semantics, and integrate tolerance after the
required topology, geometry and self-intersection stages. Record the behavior
version, explicit migration and replay disposition. Replace temporary private
inspection plumbing within that milestone after the complete operation gate
passes; do not extend overlapping paths into a later milestone. Until then the
existing incomplete validation outcome remains selectable and unchanged.

This slice makes no performance, general geometry, thread-safety, application,
full API-parity or full ordered-validation claim.
