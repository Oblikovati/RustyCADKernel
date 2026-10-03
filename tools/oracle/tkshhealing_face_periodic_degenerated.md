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
