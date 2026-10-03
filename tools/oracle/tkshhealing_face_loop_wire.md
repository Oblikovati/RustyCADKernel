# Face loop reconstruction

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, native optimized modeling build.

Eight rows exercise a simple triangle, two loops sharing a junction, three/four
spokes, parallel edges, preloaded output, and missing p-curves on a cylinder.
Each row records return value, DONE7, output count, then each output wire's edge
count, Closed flag and ordered oriented endpoint XY coordinates. Fixture vertices
have integral coordinates, so those comparisons are exact.

The cylindrical case returns false after appending two wires. Closed graph loops
can return true without DONE7; that flag is set by path joining. Output sequences
are appended to, and the no-loop early return preserves existing entries.
Status is updated after each completed join, preserving partial mutations if a
later step fails. Face and Result are not replaced by this method.

Seam/small-edge paths and broader curved, located and exceptional inputs still
need differential coverage. This fixture adds no legacy-test mapping and does
not establish full face-repair parity.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_face_loop_wire`.
