# Spherical corner connector curve types

This fixture records the actual connector geometry that
`BRepOffset_Offset::Init(Vertex)` receives for the first vertex of a `1 x 2 x 3`
box offset. OCCT retains three circular 3D curves with radius `0.5`; their sphere
p-curves are two degree-7 B-splines with four knots and one line. The native sphere
face area is `0.39269908185860564`.

The rail builder now recognizes a parameter-plane line on an elementary analytic
pipe surface and stores the corresponding surface iso as the exact 3D curve. That
preserves the circle passed into the sphere projection; approximating the iso with
`BRepLib::BuildCurve3d` creates a B-spline instead and changes the quadrature inputs
to `BRepGProp_Face`. Rust checks the 3D circle, p-curve basis types and degrees, and
the full box area at `1e-12` against the native value.

OCCT baseline: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
occt=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$occt/lin64/gcc/libi"
g++ -std=c++17 -I"$occt/include/opencascade" tools/oracle/tkoffset_sphere_connector_curves.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKOffset -lTKPrim -lTKBO -lTKShHealing \
  -lTKTopAlgo -lTKGeomAlgo -lTKGeomBase -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkoffset_sphere_connector_curves
/tmp/tkoffset_sphere_connector_curves
cargo test -p rustycad-tkoffset --test standard_offset_sphere_boundaries
cargo test -p rustycad-tkoffset --test standard_offset_check_input \
  multi_face_solid_offset_builds_native_corner_spheres_and_history
```

Captured native output is `tkoffset_sphere_connector_curves.expected`.
