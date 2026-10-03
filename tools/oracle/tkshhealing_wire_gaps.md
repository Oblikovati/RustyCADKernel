# Boundary gap oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Thirteen cases cover coincident/below-tolerance/gapped boundaries, reversed
orientation, cylinder surface resolution, missing 3D/2D curves, absent face,
unloaded analyzer, NaN precision, curve removal after measurements, and implicit
planar p-curves. Each row records CheckGap3d and CheckGap2d results, DONE1/FAIL1
and minimum/maximum metrics at indices 2, 1, 0 and -1, then repeats index 2 after
optional curve removal. Finally it records both aggregates and status reset.

Native positive comparisons leave DONE unset for NaN precision. This fixture
caught and fixes the Rust negative-guard inversion that previously flagged NaN.
Curve-removal cases verify failed individual checks retain prior measurements,
while aggregate maxima exclude failures and end at zero when every check fails.
All recorded values compare exactly. Fixtures supplement legacy declarations.

```sh
native_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O2 -I"$native_build/include/opencascade" \
  tools/oracle/tkshhealing_wire_gaps.cxx -L"$native_build/lin64/gcc/libi" \
  -Wl,-rpath,"$native_build/lin64/gcc/libi" \
  -lTKShHealing -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o target/api-proof/tkshhealing-wire-gaps
target/api-proof/tkshhealing-wire-gaps
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_gaps
```
