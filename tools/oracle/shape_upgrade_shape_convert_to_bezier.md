# ShapeUpgrade_ShapeConvertToBezier oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The probe compares all ten mode defaults and setter results. It also runs a
bounded plane face through no conversion, surface conversion, 3D line
conversion, p-curve conversion, and the combined two-pass conversion. A full
cylinder patch checks circle conversion, seam p-curve conversion, and the
topology change. A mixed Bezier/non-Bezier seam with a non-unit range checks
`GeomLib::SameRange`, p-curve ordering, and endpoint values. Output counts
include Bezier 3D curves, Bezier p-curves, and Bezier surfaces.

```sh
build=target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/shape_upgrade_shape_convert_to_bezier.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKShHealing -lTKTopAlgo -lTKBRep \
  -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/shape_upgrade_shape_convert_to_bezier
/tmp/shape_upgrade_shape_convert_to_bezier
cargo test -p rustycad-tkshhealing --test shape_upgrade_shape_convert_to_bezier
```

Known limits: conic policy combinations and nested/reentrant `Perform` calls
are not yet covered by the differential fixture.
