# `Geom_BezierSurface::init` oracle

The derived fixture exposes the protected `init` overload. It compares constructor
validation with direct reinitialization, rationality and resolution changes, evaluation
representation clearing, null-weight initialization, and partial state after a dimension
exception. The 3x2 poles and 2x3 weights have the same total element count; OCCT checks
both array extents and raises `Standard_DimensionMismatch` after resizing/copying poles.
The post-failure probe reads only initialized fields, never the resized weight cells.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Header SHA-256: `f2f8352da27bbfdb7b5c86a1852fd19a119fcb3dc6662c2be0e7b04de73fa8c2`.
Implementation SHA-256: `31313464caa8ff9080924d862861b72bd39225be0a96ea1a56c9facd60c4e864`.
Compiler declaration ID: `442159fd3109881bebba2c78a2f05043a1a65f135894dcbfac8ee9d3aa152f84`.

Build from the repository root with the pinned native model build:

```sh
occt=target/occt-modeling-full
c++ -std=c++17 -O2 -I"$occt/include/opencascade" \
  tools/oracle/tkg3d_bezier_surface_init.cxx \
  -L"$occt/lin64/gcc/libi" -Wl,-rpath,"$occt/lin64/gcc/libi" \
  -lTKG3d -lTKMath -lTKernel -o /tmp/tkg3d_bezier_surface_init_probe
/tmp/tkg3d_bezier_surface_init_probe > /tmp/tkg3d_bezier_surface_init.out
diff -u tools/oracle/tkg3d_bezier_surface_init.expected \
  /tmp/tkg3d_bezier_surface_init.out
```

The native library also crashed when a nonrational 2x2 instance was reinitialized with
weighted 2x2 data. This sequence is recorded as an observed native failure domain; it is
not treated as defined behavior for the Rust port. Constructor-only rejection of zero
weights and direct-init acceptance are covered separately with a safe 3x2 resize.
The same case is reproducible with `/tmp/tkg3d_bezier_surface_init_probe same-count-crash`;
it emits `before-same-count-init` and exits with status 139.
