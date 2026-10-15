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

## Conic and spline sources

`native_curve_cases.py` writes `native-curve-cases.csv`: 76 ordinary-range cases
with a bounded curve source (ADR 0067): a full circle, a counterclockwise and a
clockwise ray arc, and a rotated ellipse, each against three lines, two circles, a
ray arc, an ellipse and two Go spline recipes; and every Go fit and control recipe
as a source against a segment, a circle, an arc, an ellipse and another recipe
shifted by `(0.3125, 0.1875)`. The shift keeps spline pairs off their shared fit
stations, whose exact coincidence depends on fit residuals of order 1e-16 (an
unshifted pair gave 9 native points where both the kernel and the exact Fraction
oracle of TKMath certify 7: the extras are a third point at the shared origin and
the source end (1, -2), where the exact curves do not meet).

The same `native_oracle.cpp` replays them: a source field holding a curve spec is
built like a target. `occt-curves-8.0.0.csv` appends, after the segment count,
the end points of each coincidence segment. OCCT 8.0.0 reports five circle/circle
crossings in four cases of this corpus as tolerance-length coincidence segments instead of
points (a native quirk kept as recorded); the Rust test reads each such segment as
one location at its middle with half its length as extra slack. Certified
contacts are compared on the segment domain, so arcs and splines are bounded as
natively; counts must agree and every native location must lie within the
certified error, the segment radius and 1e-8 of a distinct contact. The mapped
Rust intersector is compared the same way.

```sh
python3 crates/tkgeomalgo/tests/sketch_line_incidences/native_curve_cases.py \
  > /tmp/native-curve-cases.csv
cmp /tmp/native-curve-cases.csv crates/tkgeomalgo/tests/sketch_line_incidences/native-curve-cases.csv
/tmp/incidence-oracle crates/tkgeomalgo/tests/sketch_line_incidences/native-curve-cases.csv \
  > /tmp/incidences-occt-curves.csv
cmp /tmp/incidences-occt-curves.csv crates/tkgeomalgo/tests/sketch_line_incidences/occt-curves-8.0.0.csv
cargo test --locked -p rustycad-tkgeomalgo --test toolkit sketch_line_incidences::native_curves
```

## Periodic splines and offsets

`native_offset_cases.py` writes `native-offset-cases.csv`: 108 ordinary-range cases
(ADR 0080). A segment, a circle and a (3, 4) ellipse source meet offsets of two
segments, a circle, a counterclockwise and a clockwise arc, an axis-aligned and a
(3, 4) ellipse and Go fit recipes (the segment every recipe at distances 0.125 and
-0.1; the conics the three open recipes at 0.125, since a conic against a closed
recipe's offset gives a degree-20 norm with 2,700-bit coefficients that takes about
15 s per span to isolate in a debug build); offset
sources (segment, circle, both arcs, both ellipses, three fit recipes) meet lines,
a circle, an ellipse, an arc and a segment offset; a native periodic cubic meets
lines and conics as a source and a target. `native_oracle.cpp` builds an offset as
`Geom2d_OffsetCurve` of the untrimmed basis trimmed like the basis, with clockwise
arcs on a clockwise circle so that the offset side follows the travel direction,
and a periodic spec as a periodic `Geom2d_BSplineCurve`. The offset of an
unbounded line is left out: OCCT 8.0.0 refuses its infinite domain in the
intersector ("Domaine sur courbe incorrect"). A source coincident with its target
is left out too: the kernel names it `UncertifiedOverlap`.

The Rust test certifies each case on the segment domain with a 1e-12 world error
limit: counts agree, and every native point lies within the certified error plus
1e-8 of a distinct contact. No native coincidence zone occurs. The same check runs
against the mapped Rust `InterCurveCurve` on the same mapped curves.

```sh
python3 crates/tkgeomalgo/tests/sketch_line_incidences/native_offset_cases.py \
  > /tmp/native-offset-cases.csv
cmp /tmp/native-offset-cases.csv crates/tkgeomalgo/tests/sketch_line_incidences/native-offset-cases.csv
/tmp/incidence-oracle crates/tkgeomalgo/tests/sketch_line_incidences/native-offset-cases.csv \
  > /tmp/incidences-occt-offsets.csv
cmp /tmp/incidences-occt-offsets.csv crates/tkgeomalgo/tests/sketch_line_incidences/occt-offsets-8.0.0.csv
cargo test --locked -p rustycad-tkgeomalgo --test toolkit sketch_line_incidences::native_offsets
```
