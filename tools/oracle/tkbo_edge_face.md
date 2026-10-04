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

Rust replay: `cargo test -p rustycad-tkbo --test int_tools_edge_face`.
The independent check-touch path uses the existing `ExtCS` port. The no-extrema
exact-intersector fallback, `DistanceFunction` special cases for cylinder/cone/
torus, `CheckTouchVertex`, and line/cylinder or circle/plane postprocessing are
still unported and must remain explicit parity gaps.
