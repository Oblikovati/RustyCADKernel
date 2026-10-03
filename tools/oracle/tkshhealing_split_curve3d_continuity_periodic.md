# Periodic 3D continuity splitter supplemental oracle

Oracle revision: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The safe fixture probe covers a degree-two periodic five-pole curve, a valid one-knot
removal that leaves a constructor-valid periodic curve, separate direct knot removals,
sequential removal through three knots, and a zigzag with multiple failed removals.
The Rust test compares those observations with `shape_upgrade_split_curve3d_continuity_periodic.expected`.

Build against the pinned OCCT installation:

```sh
g++ -std=c++17 tools/oracle/tkshhealing_split_curve3d_continuity_periodic.cxx \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKShHealing -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tksh-continuity-periodic-probe
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-install/lib:/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib \
  /tmp/tksh-continuity-periodic-probe
```

The separate `*_periodic_undefined.cxx` probe records OCCT's result when repeated
`RemoveKnot` calls reduce this curve to two knots and one pole. OCCT's constructor
rejects fewer than two poles (`Geom_BSplineCurve.cxx`, lines 74–76), but the native
remove operation commits that state. Its subsequent periodic `KnotSequence` prefix
uses a rebased pointer and reaches `pkn[0]` (`BSplCLib.cxx`, lines 499–526) for
one-based two-knot arrays. That is an out-of-bounds native read, so the captured row
is recorded as a known non-parity observation, not as behavior the Rust port should
emulate. The Rust regression names this divergence and asserts its current outcome:
two spline knots remain and the split values become `[0, 5, 5]` after cache rebuilding
reports an out-of-range failure. It does not count as green native parity.

Build and run that native-only observation separately:

```sh
g++ -std=c++17 tools/oracle/tkshhealing_split_curve3d_continuity_periodic_undefined.cxx \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKShHealing -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tksh-continuity-periodic-undefined-probe
LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-install/lib:/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib \
  /tmp/tksh-continuity-periodic-undefined-probe
```
