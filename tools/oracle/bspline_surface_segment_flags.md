# B-spline surface protected segment flags

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The protected `Geom_BSplineSurface::segment` overload takes independent U/V
flags. Public `Segment` always passes `(true, true)`, while `CheckAndSegment`
derives each flag from whether its bounds moved. Rust previously exposed only
those two operations. `segment_with_flags` forwards the raw bounds, epsilons,
and independent flags to the existing translation.

The flags control knot insertion, periodic-origin changes, and endpoint
multiplicities independently; both directions still trim their knot and pole
ranges. OCCT consequently reports reversed bounds for a direction with a false
flag in this fixture. The probe runs all four flag combinations after attaching
an evaluation representation; all outputs, including knot multiplicities,
pole counts, bounds and cleared representation state, are pinned in
`bspline_surface_segment_flags.expected`. The probe also primes the resolution
cache before each call and records the invalidated state.

```sh
c++ -std=c++17 -O0 -g0 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/bspline_surface_segment_flags.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKG3d -lTKGeomBase -lTKMath -lTKernel \
  -o /tmp/bspline-surface-segment-flags
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/bspline-surface-segment-flags
```

Rust comparisons are in `crates/tkg3d/tests/geom_bspline_surface_segment_flags.rs`.
The public wrapper intentionally preserves the protected operation's raw
contract: it forwards provided epsilons and does not add the range validation
used by the higher-level `segment` method. The probe covers valid ordered bounds.
