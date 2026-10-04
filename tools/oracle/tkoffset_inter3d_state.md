# BRepOffset_Inter3d stored intersection state

Pinned OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The probe exercises `FaceInter` on intersecting unbounded planes and a bounded
plane pair. For each it checks the symmetric done state, touched faces, unique
new edge map, both `BRepAlgo_AsDes` descendant lists, section parameter range,
endpoint coordinates, and face-specific edge orientations. It also checks
repeated explicit `SetDone`.

The Rust `FaceInter` slice matches unbounded, rectangular, triangular, and
straight-edged hole fixtures through `IntTools_FaceFace`, boundary event
clipping/classification, section-edge construction, the native `OrientSection`
vector rule, and `Store`. The triangle clips to `[-1, 1]`; the hole produces two
edges with ranges `[-3, -1]` and `[1, 3]`. Shared-subshape, non-plane, curved
boundary, and coincident-boundary branches remain explicitly unsupported;
general PaveFiller behavior is not claimed.

Build against the pinned native modeling install:

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$occt/lin64/gcc/libi"
g++ -std=c++17 -I"$occt/include/opencascade" \
  tools/oracle/tkoffset_inter3d_state.cxx -L"$lib" -Wl,-rpath,"$lib" \
  -lTKOffset -lTKPrim -lTKTopAlgo -lTKBool -lTKBRep -lTKGeomAlgo \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset-inter3d-state
/tmp/tkoffset-inter3d-state
```

Rust covers native `SetDone`/`IsDone`, the private `Store` transition, and the
two listed plane intersection fixtures. This does not yet close the negative
adjacent-face offset path: its boundary trimming and shared-topology behavior
still require the remaining `BRepOffset_Tool::Inter3D`/PaveFiller branches.
