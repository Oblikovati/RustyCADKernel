# BeanFace adaptive range traversal

Oracle: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The access-only observer calls private ComputeRangeFromStartPoint on the unchanged
linked implementation and reads its marked range set. The 168 rows cover four
line/plane relationships, zero/blocked/already-classified neighbours, seven seed
positions (including outside the interval), and both traversal directions.

Every row compares interval count, order, flags and endpoints. The port uses the
existing Extrema_GenLocateExtPS algorithm, native strict distance test, adaptive
step limits, neighbour transitions, stagnation check and native loop counter.
This helper does not yet activate the unfinished general Perform path.

Limitations: curved/freeform supports, failed local extrema fallback, exceptional
parameters and allocator faults remain unproven. No public C++ or legacy test
coverage is claimed for these private-method comparisons.

Build with the pinned native modeling headers/libraries as for tkbo_bean_distance.
Rust replay: `cargo test -p rustycad-tkbo --lib native_adaptive_walk`.
