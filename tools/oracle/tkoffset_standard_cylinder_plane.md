# Adjacent cylinder and planar cap offset baseline

The fixture extracts a lateral cylinder face and one planar cap from a native cylinder,
then adds them to an open shell with their source orientations. It runs the pinned
`BRepOffset_MakeOffset` skin/Arc operation and checks the resulting quilt, validity,
face history, shared-edge generated image, and convex offset interval. The exact
`GeomFill_Pipe::KPartT4` path produces a toroidal connector; the source offset-face
boundary on the cylinder is an exact circular 3D curve obtained from its V-isoparametric
p-curve.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$occt/lin64/gcc/libi"
g++ -std=c++17 -I"$occt/include/opencascade" \
  tools/oracle/tkoffset_standard_cylinder_plane.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKOffset -lTKPrim -lTKTopAlgo -lTKBool \
  -lTKBRep -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_cylinder_plane
/tmp/tkoffset_cylinder_plane
```

The captured output is `tkoffset_standard_cylinder_plane.expected`.
