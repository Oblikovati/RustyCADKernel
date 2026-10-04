# IntTools_EdgeFace bounded line/plane oracle

Oracle source is OCCT `IntTools_EdgeFace.cxx` at the pinned source revision. The
native probe exercises the ordinary BeanFace route for a crossing line and the
quick coincidence route for a line contained in a bounded plane. Its full
precision output is in `tkbo_edge_face.expected`.

The crossing case specifically checks `MakeType` calling `CheckTouch`: the
common-part interval starts at `0.99999985`, while Extrema refines the vertex
parameter to exactly `1.0`. Keeping the interval midpoint (`0.999999925`) is a
behavioral mismatch. The test also checks the bounding points taken from the
interval endpoints. The quick route returns the selected edge interval and
native default bounding points.

To rebuild against the pinned local OCCT build:

```sh
g++ -std=c++17 -O0 -g0 -I"$OCCT_BUILD/include/opencascade" \
  tools/oracle/tkbo_edge_face.cxx -L"$OCCT_BUILD/lin64/gcc/libi" \
  -Wl,-rpath,"$OCCT_BUILD/lin64/gcc/libi" \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo -lTKBRep \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/tkbo_edge_face
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/libi" /tmp/tkbo_edge_face
```

Rust replay: `cargo test -p rustycad-tkbo --test int_tools_edge_face` now covers
all five public probe cases: crossing, quick coincidence, line/cylinder tangency,
circle/plane tangency, and a translated line/cylinder tangency. The tangent replay checks every emitted field against
the native output with absolute numeric tolerance `1e-12`; it is not a bitwise
proof. It verifies one cylinder common part and two circle seam common parts,
including the native minimum-distance value of `2` in the circle case.

Additional unit regressions exercise the exact-intersection fallback on a
line/plane crossing and `CheckTouchVertex` refinement. Those remain helper
fixtures. The touch stages now construct bounded, transformed geometric adaptors as in
the native source. CheckTouch uses face bounds while CheckTouchVertex uses the
underlying surface bounds; a regression checks this distinction. Arbitrary
surface/curve domains and additional fallback cases remain proof gaps. These supplemental fixtures do not
map additional legacy declarations or establish full EdgeFace parity.

`tkbo_edge_face_state.cxx` adds seven native state snapshots, replayed by
`int_tools_edge_face_state.rs`: default state, repeated successful `Perform`,
a degenerated edge, a geometry-free edge, negative fuzzy input, and recovery
through quick coincidence with a reset context. The comparison checks all 49
numeric state fields exactly. Native results accumulate across repeated calls;
invalid edge checks retain prior completion and results. The degenerated-edge
case finishes with error 3 because the subsequent geometric check overrides
error 2. This is additional lifecycle evidence, not a new legacy declaration.
