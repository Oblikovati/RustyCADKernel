# Sampled wire representation gaps

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The native fixture records coincident and offset representations, unequal
parameter ranges, reversed edge orientation, and a gap below precision. Each
case then removes the 3D curve to check retained metrics on failure, adds a
second edge to check aggregate DONE/FAIL flags and maximum distance, and clears
statuses. Rust compares all fields exactly.

Rows: `single_found single_distance failed_found FAIL1 retained_distance
aggregate_found aggregate_DONE1 aggregate_FAIL1 aggregate_distance cleared_OK
cleared_distance`.

```sh
native_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
g++ -std=c++17 -O2 -I"$native_build/include/opencascade" \
  tools/oracle/tkshhealing_wire_curve_gaps.cxx \
  -L"$native_build/lin64/gcc/libi" -Wl,-rpath,"$native_build/lin64/gcc/libi" \
  -lTKShHealing -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o target/api-proof/wire_curve_gaps
./target/api-proof/wire_curve_gaps
cargo test -p rustycad-tkshhealing --test shape_analysis_wire_curve_gaps
```

These checks establish these branch observations, not full toolkit parity.

## Combined edge consistency

The four `consistency_*` rows run `CheckEdgeCurves` on a single edge with
coincident representations, offset 3D geometry, unequal ranges, or no 3D curve.
Endpoint vertices are explicitly constructed. Rows encode `found DONE1..DONE7
FAIL1..FAIL7 last_DONE1 last_FAIL1 minimum3d minimum2d cleared_OK`.
These validate the seven check categories and native last-check state retention.
