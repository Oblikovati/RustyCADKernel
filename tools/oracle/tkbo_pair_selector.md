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
