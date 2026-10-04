# Solid box offset and vertex closure gap

Pinned OCCT successfully offsets a `1 x 2 x 3` box as a solid. Its 26 faces include
six shifted planes, twelve edge-join cylinders, and eight spherical vertex closures.
The Rust skin builder currently lacks the final spherical closure stage. It returns a
named `Standard_NotImplemented` failure for this solid input so it cannot report the
partial 18-face shell as a completed solid offset. This is an explicit remaining gap.

OCCT source baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$occt/lin64/gcc/libi"
g++ -std=c++17 -I"$occt/include/opencascade" tools/oracle/tkoffset_standard_box_solid.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKOffset -lTKPrim -lTKTopAlgo -lTKBool \
  -lTKBRep -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_box_solid
/tmp/tkoffset_box_solid
```

The captured output is `tkoffset_standard_box_solid.expected`.
