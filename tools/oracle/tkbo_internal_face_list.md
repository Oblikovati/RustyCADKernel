# Internal-face list classification

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
160 box/thin-box rows cover empty, two-face, repeated four/six-face and identical
two-face lists, four target orientations and every first-face edge.
Replay: `cargo test -p rustycad-tkbo --test face_off`.

The translation preserves ordered pairing, the native fence of every opposite
candidate (not only the selected face), retained selected-face state between
iterations, and early return for either inside or ambiguous classification.
The input list is unchanged.

The expanded native probe segfaulted (exit 139) for kind=1 (cylinder), edge=1,
mode=2 (four repeated supports), orientation=0. That seam supplies no opposite
candidate in FindFacePairs and leads to null selected-face use. Running this
probe with an argument reproduces that observation; it is excluded from the
passing fixture and is not claimed as parity. Odd/unbalanced lists, distinct
non-manifold supports, arbitrary locations and complete legacy/API parity remain
unproven. No native failure skip is added to the legacy acceptance ledger.
