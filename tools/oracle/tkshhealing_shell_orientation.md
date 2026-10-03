# Shell repair oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The adjacent C++ fixture emits 18 rows consumed by `shape_fix_shell.rs`.

Orientation rows compare return value, shell count, DONE2/FAIL status, result
shape kind, face count, input/result shell closed flag, and source face identities
and orientations in traversal order. Cases include boxes, flipped and duplicate
faces, disconnected components, multiply shared boundaries, nonmanifold merging,
and inconsistent face orientation. Perform rows compare face and orientation
modes, cancellation, retained status after Init, context creation and empty input.

The implementation preserves native ordering and state quirks, including retained
status, cumulative multiple-edge counts, sticky merging flags, and nonmanifold
merging that can return false despite changing the result. Closed-candidate
containment and broader geometric/error domains need additional differential
coverage. No legacy declarations are mapped by these additional tests.

Run Rust comparisons with `cargo test -p rustycad-tkshhealing --test shape_fix_shell`.
Build the fixture against the pinned native modeling build with TKShHealing,
TKPrim, TKTopAlgo, TKBRep, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel libraries.
