# Native and owned-spline incidence fixtures

These fixtures support ADR 0060. `native_cases.py` writes `native-cases.csv`:
38 ordinary-range cases of an infinite source line against circles (crossing,
tangent and missing), a counterclockwise quarter arc, an ellipse with a skew
axis, crossing/endpoint/parallel segments, and every retained Go fit and control
spline recipe from `../sketch_spline` crossed by three lines.

`native_oracle.cpp` replays the corpus with OCCT 8.0.0
`Geom2dAPI_InterCurveCurve` at tolerance 1e-9 and writes `occt-8.0.0.csv`
(`id;count;x,y|...;segments`, binary64 bit patterns). Arcs are trimmed at the
`atan2` angles of their sector rays, as an application would build them.

The Rust tests certify each case on the supporting line with a 1e-12 world error
limit and require the same contact count and, for every native point, a distinct
certified contact within its achieved error plus 1e-8. The slack reflects that
native points are rounded solver output, not certified roots. The same check runs
against the mapped Rust `InterCurveCurve`, and against 40 cases built from the
owned sketch constructor: three fit spacings and control intent, open and closed,
plus open and closed curves with an active derivative handle (triple knots).
No native count mismatch or overlap was observed in this ordinary corpus; the
generated ulp-level cases in TKMath use the exact Fraction oracle instead.

```sh
python3 crates/tkgeomalgo/tests/sketch_line_incidences/native_cases.py \
  > /tmp/native-cases.csv
cmp /tmp/native-cases.csv crates/tkgeomalgo/tests/sketch_line_incidences/native-cases.csv
g++ -std=c++17 -I../occt-install/include/opencascade \
  crates/tkgeomalgo/tests/sketch_line_incidences/native_oracle.cpp \
  -L../occt-install/lib -Wl,--disable-new-dtags,-rpath,../occt-install/lib \
  -lTKGeomAlgo -lTKG2d -lTKGeomBase -lTKG3d -lTKMath -lTKernel -o /tmp/incidence-oracle
/tmp/incidence-oracle crates/tkgeomalgo/tests/sketch_line_incidences/native-cases.csv \
  > /tmp/incidences-occt.csv
cmp /tmp/incidences-occt.csv crates/tkgeomalgo/tests/sketch_line_incidences/occt-8.0.0.csv
cargo test --locked -p rustycad-tkgeomalgo --test toolkit sketch_line_incidences
```
