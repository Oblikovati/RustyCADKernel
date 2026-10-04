# Single face through `BRepOffset_MakeOffset`

This probe exercises native `MakeOffsetShape` for a forward plane with positive and
negative offsets, a reversed plane with a positive offset, and a quarter-cylinder
face with a positive offset. The source is pinned to OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`.

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
or a quarter-cylinder face with Arc joins, uniform face processing, and no caps,
thickening, or intersection repair. The plane uses a topology-preserving
translation. The cylinder uses the existing regular offset-face builder, including
source vertex orientations when it attaches generated boundary vertices. Shared
edge tubes, multiple faces, unverified non-planar surfaces, and other construction
options remain outside this validated slice. This is a real `MakeOffsetShape`
slice; it does not complete the OCC28131 regression.
