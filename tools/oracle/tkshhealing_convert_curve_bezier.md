# ShapeUpgrade curve-to-Bezier oracle

Pinned OCCT source: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

This probe calls `ShapeUpgrade_ConvertCurve3dToBezier` and
`ShapeUpgrade_ConvertCurve2dToBezier` directly. It records status flags, the
mutated split values, native split parameters, and the dynamic types and endpoint
coordinates of built result curves. These focused behavior fixtures do not
represent legacy GTest mappings.

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_convert_curve_bezier.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKG2d -lTKG3d -lTKMath -lTKernel \
  -o /tmp/rustycad-bezier-oracle
/tmp/rustycad-bezier-oracle
cargo test -p rustycad-tkshhealing --test shape_upgrade_convert_curve_bezier
```

The rows cover 3D line mode, recursive conversion through a trimmed line,
full-range and trimmed Beziers, circle mode and generated arcs, plus 2D line,
linear-Bezier recognition, trimmed-Bezier, and circle-arc builds. Advancing the
local Bezier parameter after each split interval is visible in the second output
curve of the line and Bezier cases.
