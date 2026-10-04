# OCC28131 standard offset native probe

This probe captures the `BRepOffset_Offset::Init` face branch and the complete
standard `BRepOffset_MakeOffset` call for the legacy OCC28131 input. It separates
the face initializer from the later sphere joins and shell repair, so a passing
face-stage test does not imply the complete legacy declaration is ported.

The native fixture uses OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` and the
`target/occt-modeling-full` build. Build and capture it with:

```sh
c++ -std=c++17 -I<occt-build>/include/opencascade \
  tools/oracle/tkoffset_standard_offset_occ28131.cxx \
  -L<occt-build>/lin64/gcc/libi -Wl,-rpath,<occt-build>/lin64/gcc/libi \
  -lTKOffset -lTKFeat -lTKFillet -lTKBool -lTKBO -lTKPrim -lTKShHealing \
  -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKBRep -lTKG3d -lTKG2d -lTKMath \
  -lTKernel -o /tmp/tkoffset-standard-occ28131-face
/tmp/tkoffset-standard-occ28131-face
```

The captured output is `tkoffset_standard_offset_occ28131.expected.txt`. The Rust
face-stage regression is `cargo test -p rustycad-tkoffset --test
occ28131_offset_face`. The separate legacy GTest is
`GeomFill_BSplineCurvesTest.OCC28131_StandardOffsetOfFilledFace`.
