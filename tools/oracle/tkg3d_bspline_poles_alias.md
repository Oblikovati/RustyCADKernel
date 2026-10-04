# TKG3d B-spline pole reference alias

Pinned source: OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`Geom_BSplineCurve::Poles()` and `Geom_BSplineSurface::Poles()` return const
references to the object's internal arrays. This probe retains each reference
across a `SetPole` call and prints the updated point.

```sh
native_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
c++ -std=c++17 -I"$native_build/include/opencascade" \
  tools/oracle/tkg3d_bspline_poles_alias.cxx \
  -L"$native_build/lin64/gcc/libi" \
  -Wl,-rpath,"$native_build/lin64/gcc/libi" \
  -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkg3d_bspline_poles_alias
/tmp/tkg3d_bspline_poles_alias
```

Expected native output:

```text
curve 5 6 7
surface 5 6 7
```

The Rust `poles()` methods return owned snapshots. The new `poles_array` methods
provide the corresponding read-only values without copying for the duration of
a callback that holds the state read lock. Rust callers must not mutate the
curve or surface from inside that callback. Safe Rust cannot preserve the
native reference across a later mutation, so this remains a deliberate scoped
borrowing translation rather than identical reference-lifetime behavior.
