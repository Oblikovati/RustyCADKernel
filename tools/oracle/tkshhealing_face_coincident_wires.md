# Coincident face wires

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Six native rows cover the single-wire early return, two removable wires,
a distinct copied edge, an INTERNAL wire, reversed face orientation and
context replacement. Rust compares return value, child count, orientation,
identity, untouched Result, untouched DONE status and context recording.

The face-level intersection delegate shares the ported IntersectionTool and
retains partial-intersection limitations. Its single-wire early return is also
covered. These additional comparisons are not legacy test declarations.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_face_coincident_wires`.
