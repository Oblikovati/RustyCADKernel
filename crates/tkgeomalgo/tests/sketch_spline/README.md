# Sketch spline construction oracles

These fixtures support extension ADR 0011 and kernel issue #9. They are retained
Rust test resources; no Go or OCCT executable is needed to run the Rust tests.
Sources and capture utilities carry GPL-2.0-only attribution.

`go-fit-oracle.csv` is a lossless numeric projection of the application's
`crates/martensite/tests/planar_spline/go-fit-oracle.json`, SHA-256
`cbc7412797c03f41272dfa622ea30b65aae8fc4f53fcb48345460e7a16e2d014`.
That corpus was captured with Go 1.27.0 from application revision
`aa8297d1b0ac33c024f19e54821536d76e8a447a`. Cases 0–2 are open and
3–5 are closed, with centripetal, chord-length, and uniform spacing in order.
The source points are `(0,0), (0.2,1), (2,1.5), (3,-0.5), (1,-2)`.
Rows retain degree/counts, poles, full knots, stations, and all 135 analytic
position/first-derivative samples. The fixture's expected values do not come
from the new Rust constructor.

`control_oracle.go` captures open/closed five-point control curves using that
same pinned Go revision and the actual control constructors. The open knot
vector is the five-point `controlCurveFor` policy from `curvesample_spline.go`.
Its 34 sample rows are in `go-control-oracle.csv`.

Each Go CSV row is `case,kind,index,values...`. The `shape` row values are degree,
pole count, full-knot count, and fit-station count. `pole` rows contain x/y;
`knot` and `station` rows contain one scalar. `sample` rows contain parameter,
x/y, and first-derivative x/y. Fit-station count is zero for control intent.

`occt_oracle.cpp` independently evaluates those Go recipes through OCCT 8.0.0
`Geom2d_BSplineCurve::D2`. Output rows are case, parameter, x/y, first-derivative
x/y, and second-derivative x/y. The fit and control outputs are retained in
`occt-8.0.0.csv` and `occt-control-8.0.0.csv`. This checks analytic evaluation of
the intended recipe; it does not claim OCCT's interpolation policy equals Go's.

To reproduce from the kernel root, select a Go work file that uses the pinned
application module and replaces `oblikovati.org/api` with the API `go` module:

```sh
GOWORK=/absolute/path/to/go.work go run \
  crates/tkgeomalgo/tests/sketch_spline/control_oracle.go > /tmp/control.csv
cmp /tmp/control.csv crates/tkgeomalgo/tests/sketch_spline/go-control-oracle.csv
g++ -std=c++17 -I../occt-install/include/opencascade \
  crates/tkgeomalgo/tests/sketch_spline/occt_oracle.cpp \
  -L../occt-install/lib -Wl,-rpath-link,../occt-install/lib \
  -lTKG2d -lTKMath -lTKernel -o /tmp/sketch-spline-oracle
LD_LIBRARY_PATH=../occt-install/lib /tmp/sketch-spline-oracle \
  crates/tkgeomalgo/tests/sketch_spline/go-fit-oracle.csv > /tmp/fit.csv
cmp /tmp/fit.csv crates/tkgeomalgo/tests/sketch_spline/occt-8.0.0.csv
LD_LIBRARY_PATH=../occt-install/lib /tmp/sketch-spline-oracle \
  crates/tkgeomalgo/tests/sketch_spline/go-control-oracle.csv > /tmp/control-d2.csv
cmp /tmp/control-d2.csv crates/tkgeomalgo/tests/sketch_spline/occt-control-8.0.0.csv
```

The Rust tests compare these independent values with a bound of
`1024 * f64::EPSILON * max(abs(expected), 1)`. Runtime acceptance uses separate
caller-supplied fit and seam limits; the fixture comparison bound is not a
production tolerance policy. The constructor uses the existing native LU
solver, not Go's Gauss–Jordan implementation, so bitwise solver parity is not
claimed.

## Active handle corpus

`handle_oracle.go` calls the actual pinned Go `SplineHandles.Activate`,
`TangentDirection`, `Weight`, and `EntityPolyline` methods. `go-handles.csv`
contains six cases (three station policies, open/closed), 33 station rows and
219 sampled positions. Handles at source indices 0 and 2 change direction and
weight; a coincident endpoint at index 4 retains its natural derivative.
Station rows are `case,station,index,u,x,y,dx,dy`; sample rows are
`case,sample,index,u,x,y`. Go's sampled points are oracle observations only;
production construction and evaluation remain analytic.

`handle_occt_oracle.cpp` builds separate OCCT 8.0.0 cubic Bezier curves from
those source positions and effective derivatives. It does not use Martensite's
output poles. `occt-handles-8.0.0.csv` contains 243 rows of
`case,span,u,x,y,dx,dy,ddx,ddy`, including both sides of each join. Derivatives
are converted from local Bezier parameter to normalized global parameter.
The same explicit floating-point fixture bound above applies.

```sh
GOWORK=/absolute/path/to/go.work go run \
  crates/tkgeomalgo/tests/sketch_spline/handle_oracle.go > /tmp/handles.csv
cmp /tmp/handles.csv crates/tkgeomalgo/tests/sketch_spline/go-handles.csv
g++ -std=c++17 -I../occt-install/include/opencascade \
  crates/tkgeomalgo/tests/sketch_spline/handle_occt_oracle.cpp \
  -L../occt-install/lib -Wl,-rpath-link,../occt-install/lib \
  -lTKG2d -lTKMath -lTKernel -o /tmp/handle-oracle
LD_LIBRARY_PATH=../occt-install/lib /tmp/handle-oracle /tmp/handles.csv \
  > /tmp/handles-d2.csv
cmp /tmp/handles-d2.csv crates/tkgeomalgo/tests/sketch_spline/occt-handles-8.0.0.csv
```
