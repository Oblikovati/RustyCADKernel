# Compound dimensions and shell closure

Pinned source: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkbo_shape_utilities.cxx` with the OCCT installation headers and link
TKBO, TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
Default output is `tkbo_shape_utilities.expected`; argument `shell` produces
`tkbo_open_shell.expected`.

Forty compound cases cover all eight concrete topological types, empty and
nested compounds, repeated leaves with different orientations, uniform and
mixed dimensions, all parent orientations, and repeated calls with a retained
identity fence. Compare dimensions, leaf counts and ordered leaf type/orientation.
Existing list content is appended to; the fence ignores orientation as native.

Thirty-two shell cases cover empty shells, one face, repeated faces (two and
three occurrences), Internal faces, mixed manifold/Internal occurrences,
degenerated edges and disconnected faces, with every shell orientation.
The predicate counts manifold face occurrences, not unique faces, and checks
for exactly one occurrence. It ignores degenerated edges.

The API rows remain partial pending located/shared-geometry arrangements,
exceptional input states, and wider topology stress cases. These fixtures are
supplemental evidence and introduce no legacy-test mapping.
