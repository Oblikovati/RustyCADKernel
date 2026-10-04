# `BRepOffset_Offset::Init(Path, Edge1, Edge2, Offset, FirstEdge, LastEdge, ...)`

This native differential fixture exercises caller-supplied connector edges whose vertices are
shared with the two 3D rails. The first supplied edge is intentionally created in reversed
endpoint order. OCCT reverses its occurrence and p-curve/range, attaches the face p-curve,
builds its 3D curve, then resets SameRange/SameParameter before recomputing them with the
caller tolerance. The result preserves both supplied edge identities and is valid.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
occtlib="$occt/lin64/gcc/libi"
g++ -std=c++17 -I"$occt/include/opencascade" \
  tools/oracle/tkoffset_offset_rails_with_connectors.cxx \
  -L"$occtlib" -Wl,-rpath,"$occtlib" \
  -lTKOffset -lTKFillet -lTKPrim -lTKTopAlgo -lTKBRep -lTKGeomAlgo \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_offset_rails_with_connectors
/tmp/tkoffset_offset_rails_with_connectors
```

Native output is recorded in `tkoffset_offset_rails_with_connectors.expected`. Rust coverage is
in `crates/tkoffset/tests/offset_rails.rs`. P-curve-only rails, nonidentity locations,
degenerated circular rails, closed paths, and failed/mismatched supplied-edge cases remain
outside this fixture.
