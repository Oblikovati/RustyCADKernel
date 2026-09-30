# TKTopAlgo port status

**In progress; toolkit acceptance is not complete.** `crates/tktopalgo`
(`rustycad-tktopalgo`) is the port of the ModelingAlgorithms toolkit TKTopAlgo. It
depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep and TKGeomAlgo ports,
as the native toolkit does (its EXTERNLIB). Each of the 16 OCCT packages is one module
of the crate, declared up front in `src/lib.rs`; a module whose package is not ported
yet holds only its documentation. No package is ported in full yet.

`brep_builder_api::MakeVertex` is a **stub**, not the port of
`BRepBuilderAPI_MakeVertex`: a minimal vertex builder that the TKernel test
`crates/tkernel/tests/toploc_location.rs` and the TKernel geometry probe use. It has
no `api` row, and the port of BRepBuilderAPI replaces it.

Generic C++ code (the `.gxx`, `.pxx` and `.lxx` templates that OCCT instantiates per
type) is ported once as a Rust generic and instantiated where OCCT instantiates it, as
[topalgo-design.md](topalgo-design.md) section 1 fixes.

A package may start when the packages it includes are ported (the lower toolkits are
ported):

| Level | Packages |
| --- | --- |
| 1 | BRepApprox, BRepClass, Bisector, MAT |
| 2 | BRepTopAdaptor, MAT2d |
| 3 | IntCurvesFace |
| 4 | BRepBndLib, BRepBuilderAPI, BRepCheck, BRepClass3d, BRepGProp, BRepLib |
| 5 | BRepExtrema, BRepIntCurveSurface, BRepMAT2d |

The levels come from the `#include` lines of each package. The level-4 packages
include one another in a cycle (for example BRepLib includes BRepCheck and
BRepCheck_Edge includes BRepLib; BRepBuilderAPI includes BRepLib, which includes
BRepBndLib, which includes BRepBuilderAPI), so they are ported class by class in an
order that follows the includes of each file.

The toolkit test inventory maps these suites (0 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |

## Verification

`python3 tools/verify_toolkit.py TKTopAlgo --jobs 16 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, runs them and the
Rust targets, checks exact runtime names, then runs `tools/verify_tktopalgo_numeric.py`.
While no suite is mapped it runs the numeric verifier alone and reports 0 mapped
declarations. Evidence is in `target/oracle-tktopalgo/`. CI runs it like the TKGeomAlgo
verifier. The shared native build already builds and links `libTKTopAlgo` (and
`libTKPrim`, which some unchanged suites use to build their shapes).

## Numeric comparison policy

`tools/verify_tktopalgo_numeric.py` uses the shared driver `tools/toolkit_numeric.py`.
It builds `tools/oracle/tktopalgo_probe.cxx` (a table of operations over
`tktopalgo_probe_<package>.cxx`, linked with libTKTopAlgo and the toolkits below it)
and its twin `crates/tktopalgo/examples/numeric_probe.rs` (one module per probe file
under `examples/tp_probe`), runs both on `tools/tktopalgo_corpus.py` (one
`tktopalgo_corpus_<package>.py` per package) and requires identical tokens: doubles as
IEEE bits (NaN by classification), exception types by name. Both probes reuse the
TKGeomAlgo reader and printers (`tkgeomalgo_probe_common.cxx` and
`ga_probe/common.rs`), so the token stream is the same. The corpus runs in 64
round-robin shards (`tools/probe_shards.py`), so every request prints exactly one line
and never relies on an earlier request. `tools/coverage_report.py` replays the saved
corpus under coverage.

BRepBuilderAPI (stub): `tp_make_vertex` compares the vertex of the stub `MakeVertex`
with `BRepBuilderAPI_MakeVertex` (its point and its tolerance, `BRepLib::Precision()`)
on 27 points (signed zeros, tiny, huge and decimal coordinates, 20 seeded random
points). It proves the pipeline; it is not evidence for a BRepBuilderAPI port.

Bisector: the `tp_bisec_*` requests (455, `tools/tktopalgo_corpus_bisector.py` over the
curve helpers of `tools/tktopalgo_corpus_curves.py`) compare `Bisector_Bisec` on corners
of every pair of lines, arcs, an ellipse, a parabola and B-splines (both sides, the three
join types, on and off the curve, with the vectors taken from the curves as MAT2d takes
them), a curve or a point and a point, and two points; every `Bisector_BisecAna` overload
with its members; `Bisector_BisecCC` and `Bisector_BisecPC` with their polygons, values,
derivatives, extensions, `ChangeGuide`, `Copy` and `Dump`; `Bisector_Inter` on the corner
bisectors of straight, curved and B-spline triangles (neighbour curve-curve bisectors
included); the two functions, the polygon and `IsConvex`. The Rust side reads the curves
with the TKGeomAlgo reader `ga_probe/curve2d.rs`.

## Preserved native behaviour

Each ported package adds its own subsection.

### Bisector

- `Bisector_BisecPC::Values` has no case for `N = 3`: `EvalD3` (and `DN(U, 3)`) returns the
  bisected point `P` itself (the initial value of its output) and null derivatives inside
  the bisector's range.
  `Bisector_BisecCC::Values` computes the first derivative only, so `EvalD2` and `EvalD3`
  return null higher derivatives; with a polygon of one point it computes the extension,
  then goes on to the iterative value.
- The distance `Bisector_BisecCC::Extension` returns is not squared, unlike the other
  paths of `ValueAndDist`. A path of `ValueAndDist` or `ValueByInt` that writes no
  parameter on the second curve keeps the caller's (uninitialised) variable natively; the
  port's outputs start at 0, and `Perform` keeps the parameters of the previous call, as
  its native locals do.
- `new Bisector_BisecPC(Cu, P, Side, UMin, UMax)` does not set the shift, `DistMax` or the
  empty flag: they are 0, 0 and false from the cleared native allocation, as in the port.
  `Bisector_BisecCC::Copy` does not copy the shift and the current interval (always 0).
- `Bisector_BisecAna::SetTrim(Cu)` does nothing (its body is commented out). A
  `Bisector_BisecAna` is periodic when its conic is, but its period is the inherited
  `Last - First` of the trimmed conic, which a `Geom2d_TrimmedCurve` built on it uses to
  adjust its bounds.
- `Perform` of the algorithmic bisectors appends to the existing intervals (and polygon)
  instead of resetting them. On a second `Bisector_BisecPC::Perform` over a periodic curve,
  the merge of the first and last intervals reads the period of the bisector itself, which
  is not periodic: Standard_NoSuchObject after the two intervals were removed.
- `Bisector_Inter::Perform` tests the extension at the end of the second bisector against
  the interval count of the first one, and reads the first `NB` slots of the part arrays,
  whose slots of intervals outside the domain are null handles: a native crash when an
  interval before the domain was skipped, Standard_NullObject in the port.
- `TestExtension` of `Bisector_BisecCC` compares the tangent of the first curve without
  normalizing it at the last end of the second curve.
- A circle-line bisector trimmed at the apex of its parabola ends at `-0.0` (the native
  `-secondparameter`).
- `Bisector_BisecAna::Perform` prints "Not yet implemented" on std::cout for curves that
  are not circles or lines; the port does not print (the probe mutes std::cout).
- A `Bisector_BisecAna` without curve (a `Perform` that selects no solution, as the curve
  of concentric arcs through a point off both) crashes natively on any member; in the port
  the null handle dereference panics. The corpus leaves those two requests out.

Deviations: `FirstParameter`/`LastParameter` of a `Bisector_BisecCC` or
`Bisector_BisecPC` without interval raise Standard_NoSuchObject natively; the `Geom2d_Curve`
members of the port cannot fail and answer NaN. `Bisector_PolyBis` raises
Standard_OutOfRange for a 30th point or an index outside its 30 slots where the native
array overruns. Math functions (`Bisector_FunctionH`, `Bisector_FunctionInter`) answer
`false` on an evaluation failure and give it through `raised()`, where the native
exception leaves the solver.
