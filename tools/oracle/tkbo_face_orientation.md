# TKBO face orientation oracle

Pinned source: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Compile `tkbo_face_orientation.cxx` with the installed OCCT headers and link
TKBO, TKTopAlgo, TKBRep, TKGeomAlgo, TKGeomBase, TKG3d, TKG2d, TKMath and TKernel.
Its output is `tkbo_face_orientation.expected`.

The 240 cases combine five supports, shared versus independent surface handles
versus a boundaryless split face, and all sixteen face-orientation pairs.
They compare both face/shape IsSplitToReverse return values and error outputs,
and the shared-boundary Sense result. A separate Rust test covers unsupported
shape dispatch error 100 from the native switch. Existing edge-overload evidence
is recorded separately in the split-edge oracle fixtures.

The same-surface shortcut compares all orientation values directly; geometric
normal comparison only reverses normals for TopAbs_REVERSED. The Sense search
preserves the first face's last visited edge if every edge is degenerate or a
seam. No match in the second face returns zero before normal evaluation.

These API mappings remain partial: located geometry, general splines, projection
and singular-normal failure cases, successful boundary fallback after hatcher
failure, and exceptional output state need additional evidence. This fixture
adds no legacy-test declaration mapping.

`tkbo_orientation_warning.cxx` adds six optional-report success/failure cases.
Two repeated calls preserve two separate warnings, each storing a compound of
split then original shape. Rust verifies counts, message keys and child identity
and order. The wrapper remains partial with its underlying orientation checks.
