# Closed three-face cylinder shell offset baseline

This fixture sends the lateral cylinder face and both planar caps through the native
skin/Arc maker as one shell. It checks that both shared boundaries produce toroidal
patches and that the five-face result remains a valid shell. This covers multiple
shared edges in a single connected component; the Rust implementation still supports
only the explicitly tested planar/cylinder analytic surface pair combinations.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$occt/lin64/gcc/libi"
g++ -std=c++17 -I"$occt/include/opencascade" \
  tools/oracle/tkoffset_standard_cylinder_closed_shell.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKOffset -lTKPrim -lTKTopAlgo -lTKBool \
  -lTKBRep -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_cylinder_closed_shell
/tmp/tkoffset_cylinder_closed_shell
```

The captured output is `tkoffset_standard_cylinder_closed_shell.expected`.
