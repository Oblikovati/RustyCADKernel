# B-spline `updateKnots` protected operation

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The native wrappers derive from `Geom2d_BSplineCurve` and `Geom_BSplineCurve`
to expose their protected `updateKnots()` for direct verification. Each probe
constructs an independent evaluation representation, primes the resolution
cache, calls the protected method, and reports knot distribution, continuity,
periodicity, flat knot sequence, evaluation-representation presence, and cache
validity. The 2D case is nonuniform C0; the 3D case is periodic uniform C2.

Native output is pinned in `bspline_update_knots.expected`. Both curves retain
their evaluation representation and knot metadata, while `MaxDerivInvOk`
changes from 1 to 0 after the operation. The Rust method is exposed as a safe
callable counterpart and forwards to the existing shared curve-core algorithm.

Reproduce against the installed OCCT libraries:

```sh
c++ -std=c++17 -O0 -g0 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/bspline_update_knots.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKG2d -lTKG3d -lTKGeomBase -lTKMath -lTKernel \
  -o /tmp/bspline-update-knots
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  /tmp/bspline-update-knots
```
