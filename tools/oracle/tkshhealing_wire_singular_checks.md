# ShapeAnalysis_Wire singular-edge branch regressions

Pinned native source: OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`,
`ShapeAnalysis_Wire.cxx`, `CheckDegenerated(int, gp_Pnt2d&, gp_Pnt2d&)`.
The companion C++ fixture links the existing native modeling build; it does not
change OCCT or its tests. Each expected row records `found DONE1 DONE2 FAIL1 FAIL2`.

- An isolated cone-apex edge without a p-curve uses the singularity endpoints and
  succeeds with DONE2. It does not require neighboring p-curves.
- A closed 3D circle with a midpoint away from the apex is not an existing
  degenerate edge. The missing-edge branch succeeds with DONE1 instead. Native
  missing-p-curve FAIL1 flags are replaced by the final DONE1 status.
- Disconnected edge vertices cannot trigger the missing singular-edge branch,
  even when their supplied p-curves lie along the cone apex.

All three Rust regressions failed before the correction and pass after it.
The invalid degenerate-flag case additionally verifies FAIL2 and simultaneous
DONE1/FAIL2 closure flags. Each case additionally checks CheckClosed/StatusClosed, the boolean overload and bulk overload, retention
of aggregate statuses after clearing the wire, and explicit ClearStatuses reset.
These are branch fixtures, not a claim of exhaustive wire API or legacy parity.
The Rust translation also restores the separate minimum, maximum, and expanded
vertex tolerances, the previous-start-vertex check, index-based adjacency, and
native accumulation of FAIL2 when a candidate segment is not degenerate.

Reproduce from the repository root (use the configured native modeling build):

```sh
native_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O2 -I"$native_build/include/opencascade" \
  tools/oracle/tkshhealing_wire_singular_checks.cxx \
  -L"$native_build/lin64/gcc/libi" -Wl,-rpath,"$native_build/lin64/gcc/libi" \
  -lTKShHealing -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/wire-singular-native
/tmp/wire-singular-native
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_singular_checks
```
