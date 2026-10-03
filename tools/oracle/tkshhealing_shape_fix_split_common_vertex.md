# ShapeFix_SplitCommonVertex oracle

Built and run against the pinned OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The fixture compares the source's orientation-sensitive `TopoDS_Shape::operator==` test,
face-local orientation-insensitive replacement map, discarded result in `Init`, and warning
sent through the hidden null `ShapeFix_Root::myShape`.

```sh
native_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$native_build/lin64/gcc/libi"
g++ -std=c++17 -I"$native_build/include/opencascade" \
  tools/oracle/tkshhealing_shape_fix_split_common_vertex.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_shape_fix_split_common_vertex
/tmp/tkshhealing_shape_fix_split_common_vertex
```

Expected output is in `tkshhealing_shape_fix_split_common_vertex.expected`.
