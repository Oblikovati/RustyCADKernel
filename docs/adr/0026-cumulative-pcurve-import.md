# Stored pcurve import: cumulative occurrence behavior V2

## Decision

Add `populate::PCurveImportBehavior::CumulativeOccurrenceV2` to the existing
BRepGraph importer. The default `LegacyLocalV1` and all existing entrypoints
retain their behavior. Public `Options` structs keep their existing layout.
New `perform_with_behavior`, `append_with_behavior`,
`append_flattened_with_behavior`, and
`ShapesViewMut::add_with_options_and_behavior` pass an operation-local selection
through the same traversal, extraction and registration pipeline.

This is a prerequisite for RustyCADKernel issue #10's current analytic-region
blocker, and the broader Martensite C1 / issue #13 sketch-to-prism consumer.
It does not close either issue or C1/C2 acceptance.

## Observed defect and representation contract

The actual retained closed cubic prism has three faces and six directed trim
uses. Its graph import returned Success, but Audit reported a missing Curve2D
representation on the **last cap**. Retained sweep history identifies the cap;
it is not the side seam. Importing that cap alone reproduces the missing use.
A single authored planar face, one line edge and a stored line pcurve reproduce
the defect when the face is translated and `create_auto_product` is false.
Identity placement and auto-product normalization mask the defect.

Face extraction traverses local wire and edge children. Its source surface
location includes the face location, whereas V1 compares that location to only
the local edge location. The stored representation's relative key therefore
fails to match. A probe applying only the cumulative source face placement to
the lookup edge restores both the minimized case and the actual prism audit.

For V2, let F, W and E be the source face, wire and edge occurrence locations.
The lookup edge has location F W E. For a stored face-support location S, the
representation key is (F W E)^-1 (F S) = E^-1 W^-1 S. Location composition uses
existing TopLoc operations in this order; no matrix/coordinate matching or new
identity inference is introduced. Edge orientation continues selecting the first
or second stored seam pcurve. Only pcurve lookup uses this cumulative edge;
definition geometry, identity, incidence order, ranges, graph placements,
polygon-on-surface and polygon-on-triangulation lookup remain unchanged.

## Reuse, completion and ownership boundaries

The selection applies when this call **extracts a new definition**. Existing
bound definitions are reused unchanged. A V2 append into a graph with V1
missing pcurves does not repair them and is not a graph-wide V2 certificate.
Migration requires a fresh graph (or `perform_with_behavior` clearing the old
one), V2 import, then the required audit and downstream geometry checks. Tests
pin mixed-version reuse and successful explicit rebuild separately.

The importer keeps its existing BuildStatus/AddResult and original typed
Failure causes. Success means import status, not ordered validation or a valid
published body. BRepGraph still owns mutable storage, caches and shared geometry
handles; this change supplies no immutable publication contract. Cancellation
is not added. The downstream frozen projection must own captured values and
retain its incomplete validation disposition.

## Independent expectations and replay

The owned V1 text fixture predates the structural refactor and remains byte for
byte unchanged. Authored planar/seam fixtures specify exact pcurve coefficients,
ranges and directed incidence; their intentionally partial faces are diagnostic
inputs, not accepted modelling regions. Nested transforms independently require
F(W(E(t,0,0))) = (-1,t,2) and UV = (-3,t). Seam uses require u=0 and u=2*pi,
v=t, with opposite coedge directions and range [0,2]. The existing complete
closed cubic prism supplies the original end-to-end graph audit regression.

Checks include reversal, duplicate source occurrences with distinct shell
placements and compound reference sharing, original typed failure preservation,
source coefficient/range/tolerance observations, null append, and independent
thread/process replay. No live OCCT, mesh sampling or widened tolerance is used.
These checks do not prove complete model validity, performance, application
acceptance, all platforms, or full API/test parity.

## Migration and deletion gate

`PCURVE-IMPORT-V2-MIGRATE` belongs to the **Martensite C1 milestone gate** under
Martensite issue #13, with RustyCADKernel issue #10 as the current consumer.
Before changing the default, rebuild and audit the admitted analytic corpus:
placed and nested faces/wires/edges, reversed/seam uses, repeated occurrences,
all supported analytic representations, and the complete sketch-to-prism
consumer. Record requested behavior, source revision and executed acceptance.
Then explicitly migrate callers, document replay retention and the default
change, apply semantic versioning, and remove the superseded production V1
selection within that same milestone. Do not extend the overlap into another
milestone. V1 fixtures remain reviewed historical evidence.

This slice does not change the default, publish a package, or claim that the
migration/deletion gate passed. Polygon/mesh placement requires a separate
contract and evidence; it is outside this behavior version.
