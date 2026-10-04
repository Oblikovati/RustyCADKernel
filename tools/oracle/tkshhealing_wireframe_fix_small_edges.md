# Wireframe small-edge repair

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, optimized native modeling build.
The C++ and Rust fixtures compare null/no-op/compound handling, empty merge maps,
a mode-dropped singleton, repeated-edge and repeated-neighbor guards, multiple
disjoint face groups, nonmanifold-vertex context replacement, and positive
free-wire and face-wire small-edge repairs. The joined B-spline comparison
includes range, degree, pole count, endpoints and topology occurrence counts.

Implementation is partial. The observed guards skip pairs when the previous or
middle edge is marked repeated, or when the middle edge is topologically the same
as either neighbor. Separate face groups are processed in shape traversal order,
and joined-edge links are propagated to the caller maps. The broader nonmanifold
nonmanifold transfer's full representation/topology behavior, warning behavior,
linked-face shared-edge ordering, face-linked multi-edge drop behavior, and
periodic pcurve seam cases still need more source porting and oracle evidence.
For the current nonmanifold fixture native replaces the old vertex in context,
but the final wire has only its six endpoint occurrences and no internal child;
the Rust output matches that observed result. The single-edge face drop retains the native stale
`face_small` binding after the edge and face are removed. These fixtures do not
add legacy mappings or prove full API behavior.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_wireframe_fix_small_edges`
and `cargo test -p rustycad-tkshhealing --test shape_fix_wireframe_merge_branches`.
