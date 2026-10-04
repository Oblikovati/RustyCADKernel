# BRepOffset_Inter3d stored intersection state

Pinned OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The probe exercises `FaceInter` on intersecting unbounded plane faces, then checks
the resulting symmetric done state, touched faces, unique new edge map, both
`BRepAlgo_AsDes` descendant lists, section parameter range, endpoint coordinates,
and the two face-specific edge orientations. It also checks repeated explicit
`SetDone`.

The Rust `FaceInter` slice matches this unbounded plane case through the existing
`IntTools_FaceFace`, section-edge builder, `OrientSection` vector rule, and `Store`
state. Trimmed faces and shared subshapes return `Standard_NotImplemented`; they
remain on the native PaveFiller/common-topology branches and are not claimed.

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

Rust covers the native `SetDone`/`IsDone` state and reproduces the private `Store`
state transition with explicit section-edge lists. `FaceInter` itself is still
unimplemented, so this slice does not close the finite trimmed-plane offset gap.
