# Private prism outward and trim orientation, behavior version 1

## Decision and scope

For Kernel#10, add one opt-in private orientation inspection after the existing
fresh single-Jordan polynomial embedding theorem (ADR0029). Reuse its current
source/copy coefficients, exact winding, regularity, complete actual chart domains,
trim identities, all parents and exact nonzero affine Jacobian. No new geometry
evaluator, winding engine, topology traversal or endpoint check is introduced.

`inspect_prism_orientation` takes the original bridge, sources and receipt,
explicit orientation/embedding versions and unchanged regularity policy, and a
cancellation callback. Its immutable owned result retains the embedding evidence,
the winding derivation, three outward face rows and six induced trim rows. It
leaves completion, coverage, gaps, measurements and history untouched. Missing
prerequisites return typed failure/incompleteness, never a partial successful table.

## Distinct orientation operands

Let U,V be the exact placed source-support columns, E the captured extrusion, and
J=det(U,V,E). The existing exact dyadic J is reused without normalization or a
rounded normal. The actual cap chart identities prove the same columns and exact
translation. Actual cumulative TopAbs face/trim orientations come from the
body-context keys retained by the theorem, not native returned-image orientations.
Forward is +1, Reversed is -1; Internal/External are refused with the affected key.
Parent orientation is already composed and must not be multiplied a second time.

Let f be the selected candidate face-facing sign, r the captured boundary traversal
sign, w the fresh classified faced winding, and q the winding for increasing stored
curve parameter. Existing wire assembly applies r to the native contribution;
candidate `facing_winding` then applies f. Therefore w=f*r*q and q=f*r*w.
The admitted single full-range Jordan component has w=+1.

The result deliberately names this provenance `DerivedFromFreshFacedWinding`.
The shared binder reruns the exact classifier, compares the selected component and
windings, and binds its metadata to current coefficients and actual traversal.
Its result does not retain that fresh classifier's native contribution as a
separate proof object. No such independent retention is claimed. The derivation is
valid only with the theorem's single segment, full [0,1] range, exact closure,
Jordan domain and source/copy correspondence hypotheses.

## Outward normals and induced trims

For native tangent (a,b), set T=aU+bV and L=-bU+aV. The universal identity

    (T cross E) dot (q L) = -q J (a*a+b*b)

shows that sign(qJ)*(T cross E) is outward. Regularity supplies a*a+b*b>0.
The actual stored side uses v=-h*s, h>0, and E=-h*D_world, hence its parameter
normal is -(T cross E)/h. There is no requirement for C1 closure at the authored
seam; one-sided regular chart normals and the seam quotient are retained.

| Chart | Required cumulative face sign |
| --- | --- |
| First cap | -sign(J) |
| Last cap | sign(J) |
| Generated side | -q*sign(J) |

Cap edge sign e must satisfy e*q=o, where o is its cumulative face sign.
The stored side rectangle is u in [0,1], v in [-h,0]; its positive parameter-domain
boundary has native trim signs First=-1, Last=+1, Start=+1, End=-1.
The two seam curves use increasing native parameter with v=-lambda.
Each actual cumulative trim sign must equal this sign multiplied by o.
The two seam occurrences retain distinct authored use keys and parents.

These checks concern orientation relative to actual chart domains. They neither
duplicate nor replace ADR0033's combinatorial wire closure, opposite edge uses,
vertex links or shell connectivity. Signed volume alone is not orientation proof.

## Binding, errors and replay

The operation calls the existing embedding entry once. After its bounded sign
checks, a final callback is followed by the existing callback-free fresh binding
comparison. Every used source, orientation, interval, chart/trim coefficient and
history operand remains tied to the captured embedding. No cache or global state
is added. Success and failure types have compile-time Send/Sync checks; legacy
failure causes remain diagnostics rather than published isolated bodies.

Typed errors retain unchanged requested options, phase, original source when
available, and boxed prerequisite causes or face/trim entities with actual and
expected orientations. Existing body diagnostics are not converted into an exact
validation pass. A changed source, callback mutation, unsupported orientation,
wrong face sign or wrong trim sign prevents a successful orientation report.

The prior embedding-v1 portable receipt was captured from base f4cd79ae2 before
registering the new module and pinned in a separate baseline commit. Old embedding
and batch entry behavior remains selectable. This is an additive private behavior
version, not a public API migration or replacement; no production deletion is due.
Kernel#10 completion still requires the ordered topology/geometry/self-intersection/
tolerance composition, frozen naming/history/ownership binding, public contract and
full issue corpus. Orientation evidence does not authorize release.

## Independent expectations and verification

The authored cubic poles (0,0),(4,0),(0,4),(0,0) have native Green integral +12/5;
reversing poles gives -12/5. Reversing an edge occurrence alone leaves those
stored-parameter signs unchanged. Actual pipeline tests independently vary poles,
source traversal and extrusion sign. Isolated face-facing/traversal tables exercise
f=-1 without claiming that a modified proof operand is a fresh admitted operation.
Isolated flips cover every face and all six trims; full-entry mutation tests
separately reject stale geometry and cumulative body contexts.

The retained offline Jacobian requests verify the inward-dot polynomial identity
for arbitrary U,V,E, and the stored side normal rational identity. An independent
integer verifier checks all eight f/r/q combinations, the rectangle orientation
table and 11,808 nonsingular integer bases, including 377,856 side-sign and 47,232
cap-sign cases. Singular cases remain outside orientation admission.
Raw request/result bytes, the initial transport-schema rejection, exact verification
script and provenance accompany final evidence. No native OCCT oracle or tessellation
is used. Source-bookended Rust 1.99 tests, doctests, lint/format, process/thread replay
and ordinary-host sanitizer outcomes are recorded separately after source freeze.
