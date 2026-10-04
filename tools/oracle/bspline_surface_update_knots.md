# B-spline surface protected knot refresh

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`Geom_BSplineSurface::updateUKnots` and `updateVKnots` are protected void
operations. The native derived probe exposes both on a nonuniform C0 U vector
and a periodic uniform C2 V vector, primes `Resolution`, and attaches a plane
evaluation representation. It confirms each call refreshes its selected knot
metadata, invalidates the shared resolution cache, and leaves the evaluation
representation and opposite-direction metadata intact. Rust exposes safe
counterparts forwarding to the existing direction-specific state refresh.

Both methods are void natively; the Rust wrappers return `Result` to report
allocation/state failures from the underlying refresh. The valid constructed
surface domain is tested; fabricated corrupt private knot arrays are outside
the probe's defined input domain. The protected `segment` method is already
ported and exposed as `GeomBSplineSurface::segment` in Rust, so this change
does not claim additional segment behavior.

Reproduce with the installed pinned OCCT build:

```sh
c++ -std=c++17 -O0 -g0 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/bspline_surface_update_knots.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKG3d -lTKGeomBase -lTKMath -lTKernel \
  -o /tmp/bspline-surface-update-knots
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/bspline-surface-update-knots
```

The output is pinned in `bspline_surface_update_knots.expected`. Rust
regressions are in `crates/tkg3d/tests/geom_bspline_surface_update_knots.rs`.
