# BRepOffset::Surface native oracle probe

This standalone probe records `BRepOffset::Surface` outputs from the pinned OCCT
build. Its cases mirror the Rust surface-offset tests: plane, cylinder, cone,
sphere, ring torus, spindle torus, trimmed cylinder, and a planar Bezier surface.
Each analytic case runs with direct and indirect `gp_Ax3` orientation. The probe
prints status, dynamic type, bounds, a D0 sample at `(0.3, 0.4)`, and analytic
radii/angles where available. It also exercises `allowC0=true` on the Bezier
fallback.

The fixed offsets include `1`, `-2`, `-3`, `±1e-7`, plus offsets around the
zero-radius threshold for cylinders, spheres, tori, and cones. The near-threshold
values use `Precision::Confusion()` and factors `0.999` / `1.001` to sample both
sides of OCCT's comparisons.

The checked-in output fixture was produced from OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871` and is stored at
`crates/tkoffset/tests/fixtures/offset_surface.txt`.

Build against the same OCCT headers and library output used by the Rust port:

```sh
c++ -std=c++17 -I<occt-build>/include/opencascade \
  tools/oracle/tkoffset_surface_probe.cxx \
  -L<occt-build>/lin64/gcc/libi \
  -Wl,-rpath,<occt-build>/lin64/gcc/libi \
  -lTKOffset -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o tkoffset_surface_probe
./tkoffset_surface_probe > tkoffset_surface_probe.txt
```

The oracle deliberately preserves quirks in the pinned `BRepOffset.cxx`:
unsupported surface kinds use `Geom_OffsetSurface`; a degenerate analytic
cylinder/sphere/torus also falls back to `Geom_OffsetSurface`; a torus with
`MinorRadius >= MajorRadius` skips the analytic path without changing status;
and the torus reversed branch sets status but still falls back. The cone path
uses zero as its radius threshold and reverses a negative result while keeping
status `Good`.

The Rust replay is `cargo test --locked -p rustycad-tkoffset --test
 offset_surface_oracle` (join the wrapped command into one line). It checks all
131 records, including every recorded output property, and requires exact finite
floating-point bits. The integration worktree regenerated the fixture with the
command above and compared it byte-for-byte before accepting the Rust results.
This focused oracle is supplemental evidence, not a translated legacy GoogleTest
or a claim of complete TKOffset behavior.
