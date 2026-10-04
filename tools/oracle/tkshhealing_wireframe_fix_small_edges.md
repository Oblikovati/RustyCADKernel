# Wireframe small-edge repair

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`, optimized native modeling build.
The C++ and Rust fixtures compare null/no-op/compound handling, empty merge maps,
and a positive free-wire small-edge repair. The joined B-spline comparison includes
range, degree, pole count, endpoints and topology occurrence counts.

Implementation is partial. Face-linked edges, pcurves/seams, drop and angle modes,
multiple groups, nonmanifold transfer and warnings require further source porting
and oracle evidence. Explicit errors preserve these gaps in the current port;
these fixtures do not add legacy mappings or prove full API behavior.

Run `cargo test -p rustycad-tkshhealing --test shape_fix_wireframe_fix_small_edges`.
