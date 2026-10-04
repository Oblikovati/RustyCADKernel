# BVH box selector

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
30 native cases cover 2D and 3D BVH sets, containing/intersecting/disjoint/point/
invalid query boxes, repeat-selection accumulation, Clear retaining the query
and set, node rejection metrics, and exact ordered element IDs.
Replay: `cargo test -p rustycad-tkbo --test box_selector`.

The selector implements the existing source-derived TKMath BVH traversal traits.
A mutable borrow represents the native set pointer and lets Select build a dirty
tree at the native call boundary. Direct access through an absent set returns
Standard_NullObject in Rust; native null-pointer access is undefined. The Rust
regression for that error is not claimed as native parity.

Partial: 4D and arbitrary tree layouts, custom builders, allocator/exhaustion,
failure behavior, and full legacy/API parity remain unproven.
