# BVH pair selector

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
36 native 3D fixtures compare exact traversal order and sorted mapped pairs for
shared/distinct BVHs, positional same-tree filtering, repeated selection,
clear/reselect, offsets and duplicate element IDs. The node rejection metric
remains unchanged. Replay: `cargo test -p rustycad-tkbo --test pair_selector`.

Shared inputs use one mutable Rust borrow and the same built tree for both
traversal inputs. Distinct inputs borrow both sets; a failure building the
second tree restores the first. Native SetSame filters internal box positions,
not the IDs mapped through Element; the fixture uses descending and duplicate
IDs to distinguish these rules. Selection appends until Clear is called.

Partial: only the 3D specialization is ported; custom builders, varied layouts,
allocation failures and full legacy/API parity remain unproven. Direct access
without sets returns a Rust Standard_NullObject error rather than reproducing
native undefined pointer access. The absent-set regression is Rust-only.

## Native 2D specialization compile failure

A GCC 13.3 C++17 probe instantiating the declared BOPTools_Box2dPairSelector alias
fails against the pinned source. RejectNode hardcodes BVH_Box<double,3> and
attempts to construct it from NCollection_Vec2<double> corners; no matching
constructor exists. The minimal reproducer is
`tkbo_pair_selector_2d_compile_failure.cxx`. This records an upstream template
failure domain; it neither deletes the API obligation nor constitutes a passing
runtime parity test.
