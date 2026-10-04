# Face relations and edge occurrences

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkbo_face_relations.cxx` with installed OCCT headers and link TKBO,
TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
Default output is `tkbo_face_domain.expected`; `lookup` emits
`tkbo_edge_on_face.expected`.

150 same-domain cases combine five supports, shared/independent faces, two
translated locations, a boundaryless first face, three fuzz values and two
boundary-edge tolerance scales. The algorithm samples one interior point and
uses the first face's maximum non-degenerate edge tolerance to raise both face
tolerances. It does not independently establish mathematical surface equality.

160 edge lookup cases combine five supports, same versus foreign edge identity,
and every edge/face orientation pair. Both GetEdgeOff and GetEdgeOnFace retain
the caller's sentinel on failure. The former finds the opposite orientation
using a recursive explorer; the latter searches exactly two direct-child levels.

Rows remain partial for more complex nesting, holes, spline domains, exceptional
states and other location transformations. These fixtures add no legacy mapping.
