# Periodic cone apex boundary

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Eight native rows cover apex above/below the loop, reversed wire and face,
partial angular span, absent p-curve, multiple wires and context replacement.
Comparisons include return value, retained Face identity, Result wire/edge counts,
unchanged status, orientations, context recording, degenerated edge tolerance,
range and p-curve endpoints (absolute tolerance 1e-12 for floating values).

The public FixPeriodicDegenerated method adds a second wire and updates Result,
while retaining Face until context application. The earlier Perform cone path
continues with seam construction; shared apex construction is reused by both.
The public method does not invent missing p-curves, unlike the preceding wire
repair stage of Perform. Broader placed/transformed and failure domains remain
unproven. No legacy declarations are added by these differential fixtures.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_face_periodic_degenerated --test shape_fix_face_test`.

Eight aggregate rows additionally call Perform with FixWireMode=0. They compare
both cone signs, wire/face reversal, partial/missing/multiple p-curves and context.
The missing-pcurve case caught an incorrect helper choice: native FixMissingSeam
calls BRepTools::UVBounds, whose fallback uses surface bounds; ShapeAnalysis's
bounds helper raises on the empty curve box. The port now calls BRepTools too.
Source audit also corrected collapsed-domain expansion to test the substituted
bound, as OCCT does, instead of the original infinite bound.
