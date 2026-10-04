# Single planar face through `BRepOffset_MakeOffset`

This probe exercises native `MakeOffsetShape` for a forward face with positive and
negative offsets, and for a reversed face with a positive offset. The source is
pinned to OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Compile and compare from the RustyCADKernel repository root:

```sh
c++ -std=c++17 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkoffset_make_offset_single_face.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKOffset -lTKFeat -lTKFillet -lTKBool -lTKBO -lTKPrim \
  -lTKShHealing -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKBRep \
  -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset-make-offset-single-face
/tmp/tkoffset-make-offset-single-face | diff -u \
  tools/oracle/tkoffset_make_offset_single_face.expected -
```

The Rust implementation currently supports a single identity-located planar face
with Arc joins, uniform face processing, and no caps, thickening, or intersection
repair. It offsets by a topology-preserving translation, then builds the single
open shell. Shared-edge tubes, multiple faces, non-planar surfaces, and other
construction options return explicit `Standard_NotImplemented` errors. This is a
real `MakeOffsetShape` slice; it does not complete the OCC28131 regression.
