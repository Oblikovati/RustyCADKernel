# TKGeomAlgo port status

**In progress; toolkit acceptance is not complete.** `crates/tkgeomalgo`
(`rustycad-tkgeomalgo`) is the port of the ModelingAlgorithms toolkit TKGeomAlgo. It
depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase and TKBRep ports, as the
native toolkit does (its EXTERNLIB). Each of the 33 OCCT packages is one module of the
crate, declared up front in `src/lib.rs`; a module whose package is not ported yet
holds only its documentation. No package is ported in full yet.

Generic C++ code (the `.gxx` and `.pxx` templates that OCCT instantiates per type) is
ported once as a Rust generic and instantiated where OCCT instantiates it, as
[geomalgo-design.md](geomalgo-design.md) fixes.

A package may start when the packages it includes are ported:

| Level | Packages |
| --- | --- |
| 1 | AppBlend, FairCurve, GccEnt, GccInt, Hatch, IntImp, IntRes2d, IntSurf, Intf, Law, LocalAnalysis, NLPlate, Plate, TopTrans |
| 2 | GccAna, HatchGen, IntCurveSurface, IntImpParGen, IntWalk, TopClass |
| 3 | IntCurve, IntPolyh, IntStart |
| 4 | Geom2dInt |
| 5 | Geom2dAPI, Geom2dGcc, Geom2dHatch |
| 6 | IntPatch, then ApproxInt and GeomInt, then GeomFill and GeomAPI, then GeomPlate |

The toolkit test inventory maps these suites (0 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| IntSurf_LineOn2S_Test.cxx | int_surf_line_on2s | 3 | pass | pass |
| IntSurf_Quadric_Test.cxx | int_surf_quadric | 1 | pass | pass |
| Plate_Plate_Test.cxx | plate_plate | 2 | pass | pass |

## Verification

`python3 tools/verify_toolkit.py TKGeomAlgo --jobs 16 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, runs them and the
Rust targets, checks exact runtime names, then runs `tools/verify_tkgeomalgo_numeric.py`.
While no suite is mapped it runs the numeric verifier alone and reports 0 mapped
declarations. Evidence is in `target/oracle-tkgeomalgo/`. CI runs it like the TKBRep
verifier.

## Numeric comparison policy

`tools/verify_tkgeomalgo_numeric.py` uses the shared driver `tools/toolkit_numeric.py`.
It builds `tools/oracle/tkgeomalgo_probe.cxx` (a table of operations over
`tkgeomalgo_probe_<package>.cxx`, linked with libTKGeomAlgo and the toolkits below it)
and its twin `crates/tkgeomalgo/examples/numeric_probe.rs` (one module per probe file
under `examples/ga_probe`), runs both on `tools/tkgeomalgo_corpus.py` (one
`tkgeomalgo_corpus_<package>.py` per package) and requires identical tokens: doubles as
IEEE bits (NaN by classification), exception types by name. The corpus runs in 64
round-robin shards (`tools/probe_shards.py`), so every request prints exactly one line
and never relies on an earlier request. `tools/coverage_report.py` replays the saved
corpus under coverage.

The numeric corpus has 54 requests.

Intf: `intf_triangle` compares `Intf::PlaneEquation` and `Intf::Contain` on 54
triangles: both orientations, coincident and collinear vertices, a normal that
underflows below `gp::Resolution`, huge coordinates, points on edges and vertices,
outside and off the plane, and 40 seeded random cases.

IntSurf: 334 requests. `intsurf_quadric_point` and `intsurf_quadric_uv` evaluate every
`IntSurf_Quadric` member and `IntSurf_QuadricTool::Tolerance` for the five quadric
kinds (including a cylinder and a cone of radius 0) and the undefined quadric, on
direct, indirect and oblique frames, at the frame origin, on the axis, on the torus
core circle and at seeded random points; `intsurf_transition` drives
`IntSurf::MakeTransition` through its four branches and `intsurf_same` compares
`IntSurf_PntOn2S::IsSame` with and without the 2d tolerance.

Law: 266 requests. `law_bspline` builds scalar B-spline laws (clamped, rational,
periodic, piecewise Bezier, uniform, C0, quintic and invalid data) and compares every
query and evaluator at the knots, inside, at the ends and outside the range;
`law_edit` applies each editing member (degree elevation, knot insertion and removal,
reversal, segmentation, periodicity, origin, weights, knots, poles,
MovePointAndTangent, Copy) to five laws; `law_function` drives the constant, linear,
S, B-spline, interpolated and composite laws through every `Law_Function` member for
every `GeomAbs_Shape`; `law_mix` covers `MixBnd`, `MixTgt`, `Scale`, `ScaleCub`,
`Reparametrize` (on a circle adaptor) and `Law_BSplineKnotSplitting`;
`law_interpolate` covers `Law_Interpolate` with and without parameters and tangents.

Plate: 106 requests. `plate_solve` loads pinpoint (values and derivatives up to order
2), linear XYZ and linear scalar constraints, solves orders 1 to 10 (the three `SolveTI`
systems and the refused orders, anisotropies, empty and degenerate boxes) and compares
`IsDone`, `Continuity`, `UVBox`, `Evaluate`, four `EvaluateDerivative`s, `CoefPol` and
`UVConstraints` for the plate, its copy and an assigned plate. `plate_constraint`
builds the line, plane, global translation, sampled curve, GtoC (G1/G2/G3, with and
without projection) and FreeGtoC (G1/G2/G3, incremental loads, orientations and the
degenerate normals) constraints, compares their content and the plate that loads them.

LocalAnalysis: 406 requests. `la_curve` compares every query of
`LocalAnalysis_CurveContinuity` and its `Dump` text for the seven `GeomAbs_Shape` orders
and two tolerance sets on joined and broken pairs of lines, circles and Bezier curves
(tangent and reversed joins, curvature jumps, null first and second derivatives,
undefined tangents, straight pieces); `la_surface` does the same for
`LocalAnalysis_SurfaceContinuity` at points, on two 2d lines and through
`ComputeAnalysis`, on planes, cylinders, spheres and tori (tangent and broken joins,
poles, umbilics).

FairCurve: 178 requests. `fc_law` evaluates `FairCurve_BattenLaw` before and after its
setters, with heights that turn negative. `fc_density` compares the tension, sagging
and jerk densities with their gradients and packed Hessians (derivative orders 0 to 2,
with and without the free-sliding value, uniform tension, a vanishing law) on a Bezier
and a two-span B-spline of degree 9. `fc_energy` compares `Variable`, `Value`,
`Values` and the Hessian of `FairCurve_EnergyOfBatten` and `FairCurve_EnergyOfMVC` for
every pair of end-constraint orders, at the initial and at perturbed variables (the
clamped and the infinite sliding), and minimizes them with `FairCurve_Newton`.
`fc_batten` computes batten and minimal-variation curves (end angles that need several
homotopy steps, free and imposed slidings, end curvatures, physical ratios, null
heights, invalid heights and ratios, confused points), moves their end points and
computes again, comparing every getter, the curve and the `Dump` text.

## Preserved native behaviour

- `Intf::PlaneEquation` of a degenerate triangle (normal shorter than
  `gp::Resolution`) returns the undivided normal and a polar distance of 0.

### IntSurf

- `IntSurf_Transition::SetValue(Tangent, Type)` and `SetValue()` change only the type
  (and tangent flag): the situation and opposite flag of an earlier TOUCH transition
  stay. `IntSurf::MakeTransition` writes its results with them, so the Rust
  `make_transition` updates two `&mut IntSurfTransition` in place instead of returning
  fresh values.
- `IntSurf_Quadric::Normale(U, V)` of a cone is the unnormalized `D1u ^ D1v` (null when
  `|D1u| < 1e-7`) and ignores the frame handedness, unlike every other normal of the
  class. The plane constructor leaves the axis line at its default (the Z axis).
- `IntSurf_Quadric::D1`, `ValAndGrad` and `Parameters` of the undefined quadric leave
  their output arguments unchanged; the Rust members return `None` there.
  `Normale(P)` raises `Standard_ConstructionError` on a cylinder axis and at a sphere
  centre (the `gp_Dir` of a null vector), and so does `Normale(U, V)` on a cylinder of
  radius 0.
- `IntSurf_LineOn2S::Split` gives the new line the default allocator, not the
  allocator of the split line, and keeps the (possibly stale) boxes of the split line.
- A default-constructed `IntSurf_PathPoint` holds a null parameter sequence that
  `Value2d`, `AddUV`, `Multiplicity` and `Parameters` dereference natively; the port
  returns `Standard_NullObject` there instead of crashing. A copy of a path point
  shares the parameter sequence, as the native copy of the handle does.

### Law

- `Law_BSpline` is an older copy of `Geom_BSplineCurve` with real poles, and keeps its
  differences: `DN` evaluates at the given parameter while `D0` to `D3` normalize a
  periodic parameter first; `LocateU` tests the given, not the normalized, parameter
  against the first and last knots; `IncreaseMultiplicity` inserts with `Add = false`.
  `InsertKnots` and `RemoveKnot` natively pass the stored knots and multiplicities as
  their own outputs when the knot count does not change; the port writes fresh arrays,
  which gives the same result because BSplCLib reads each entry before rewriting it
  and `NbPoles` forces equal first and last multiplicities on a periodic law.
- `Law_BSpline::MovePointAndTangent` on a periodic law makes BSplCLib write past the
  new pole array (undefined behaviour natively); the port raises
  `Standard_OutOfRange`. The `Local*` evaluators pass knot indices to the flat-knot
  `LocateParameter`; when the located flat index exceeds the knot count,
  `BSplCLib::FlatIndex` reads past the multiplicities natively and the port raises
  `Standard_OutOfRange`. The corpus avoids both undefined cases.
- `Law_BSpFunc::NbIntervals` tests the first split knot against the range, but
  `Intervals` first overwrites it with the start index, so the two can disagree;
  `Intervals` writes `T(1..n+1)` at absolute indices, then `T(Lower)` and
  `T(Lower + n)`. `Law_Composite::Intervals` writes `T(1)`, `T(2)` and the following
  entries at absolute indices too, and `Law_Composite::Continuity` raises
  `Standard_NotImplemented`.
- `Law_Composite::ChangeElementaryLaw` and `ChangeLaws` return references natively;
  the port lends the current law and the list to a closure.
- `Law_Interpolate` builds chord-length parameters reading `Parameters(ii)` with the
  point index `ii` (correct for one-based points only), stores the tangent flags that
  `Load` receives before it checks their length, shares that flag array (Perform marks
  the end tangents it computes in it) and never reads its tolerance.
  `Law_Interpolate::ClearTangents` is declared but never defined at this commit, so its
  row is excluded.
- `rustycad_tkgeombase::geom2d_convert::split_knots` (the split table of the knot
  splittings) is public so that `Law_BSplineKnotSplitting` shares it.

### Plate

- `SolEm`'s derivative table (the closed forms of the fundamental solution for every
  `(IU, IV)` with `IU + IV <= 7`) is generated from the native switch into
  `plate/sol_em_terms.rs`: integer factors become the equal `f64` literals and the
  products keep their left-to-right order, so every value is bit-identical (the probe
  compares it through all the solves). `SolEm` keeps the native point cache
  (`Uold`, `Vold`, `U2`, `R`, `L`) behind a lock; a fresh cache holds `(1e20, 1e20)` with
  `R = 0`, so a first call at exactly that scaled point returns 0, as natively.
- `SolveTI2` allocates its right-hand side once, so the linear XYZ rows accumulate the X,
  then Y, then Z terms with `+=`; the port keeps the accumulation. `SolveTI` sets the
  order before it refuses an invalid request, so `CoefPol` of a refused plate reads a
  missing solution: the port raises `Standard_NullObject` (a null dereference natively).
  `IsDone` is true after `Init`, as the native test documents.
- `Plate_Plate::UVBox` (and so `SolveTI`) reads the first pinpoint constraint: without
  one the port raises `Standard_OutOfRange`, as the native checked sequence access.
  Derivative orders past the ten scale factors `ddu`, `ddv` (undefined natively) raise
  `Standard_OutOfRange`.
- The linear constraints share their arrays through handles, so `SetPPC` and `SetCoeff`
  on a copy (or on the constraint after `Load`) are seen by the loaded plate; a
  default-constructed linear constraint has no arrays and `Load` raises
  `Standard_NullObject`. `Plate_SampledCurveConstraint` builds its arrays before it
  raises `Standard_DimensionMismatch` for `n > m`.
- `Plate_GtoCConstraint` with `D3` and `nP` ignores `nP`, as natively. The GtoC and
  FreeGtoC constructors stop with the constraints built so far on a degenerate or
  orthogonal normal; a FreeGtoC incremental load between parallel normals raises
  `Standard_ConstructionError` (the null rotation axis). `GetPPC` and `LSC` beyond their
  fixed arrays raise `Standard_OutOfRange` (undefined natively). The G1/G2/G3 blocks,
  which the native constructors repeat verbatim, are written once in `gtoc_math.rs`.

### LocalAnalysis

- `StatusError` is uninitialized natively until an analysis fails, and
  `ContinuityStatus` of a surface analysis built from tolerances only is uninitialized
  until `ComputeAnalysis`: the port returns `None` and `Standard_NoSuchObject` there.
  The surface analysis on two 2d curves fails without a reason when the parameter is
  outside either curve, so its `Dump` prints only its first line.
- The two `Dump` reports keep their native texts, including the two spaces of
  "one of the  second derivatives is null", the blank line after the surface G1 values
  and the reasons each report omits (TangentNotDefined for surfaces,
  CurvatureNotDefined for curves). Reals are written as the default `std::ostream`
  (`%g`).
- Every angle is folded into `[0, PI/2]` when it is measured, so the curve tests'
  second clause `|angle - PI| <= eps` never holds; it is kept, as natively. `IsG2` of a
  curve compares the fractional part of `(G2Angle + EpsG2) / PI` with `EpsG2`.

### FairCurve

- The densities keep copies of the batten law: the energies' own `SetSliding` reaches
  no density, and only the tension receives the sliding length (`SetLengthSliding`), so
  the sagging and jerk heights use the sliding of the step's law. The uniform tension
  of the minimal-variation energy freezes the middle height at construction, whatever
  its sign.
- `FairCurve_EnergyOfBatten::Compute` records a failed sagging integral as
  `Ok = false` and goes on; a later interval can set it back. The minimal-variation
  energy returns at the first failed integral.
- The sliding guard of both energies reports `InfiniteSliding` beyond ten times the
  original length and clamps it below one hundredth; the clamped length is what
  `Variable` writes back.
- `FairCurve_Energy::Hessian1` reads the intermediate values `Vect(5)` and `Vect(6)`
  for the start G2 term and `Vect(jj)` for the end one; the jerk Hessian omits the
  `ProduitV` and `HNumRho` terms in its XiXj and YiYj entries and mixes the (ii, ii-1)
  and (ii-1, ii) orders in its diagonal XiYi entry. All are kept.
- `SlidingOfReference` tests the requested (`New`) angle of the opposite end, not its
  argument, before halving it for an unconstrained end.
- After a successful minimization the batten keeps the poles of the last energy
  evaluation (not of the returned location), and reloads the variables from the energy
  before the convergence and infinite-sliding checks. A failed minimization returns
  false with the energy status as the code, or true with `NullHeight` when the law
  vanishes at an end.
- `FairCurve_MinimalVariation` does not check its physical ratio at construction; the
  first `Compute` raises the `Standard_DomainError` of `FairCurve_EnergyOfMVC`. NaN
  passes both checks.
- `Dump` pads only the old value to 7 characters (`o.width(7)`), writes flags as 0/1
  and reals as the default `std::ostream`.
- The FairCurve sources define `No_Standard_RangeError` and `No_Standard_OutOfRange`:
  an index out of range is undefined natively; the port panics there. The knot
  refinement computes its native `int` products in `i64` (their overflow is undefined
  natively and unreachable with finite angles).
- The batten constructor passes a 2-entry weight array to a non-rational
  `BSplCLib::IncreaseDegree`, which never writes it; the port passes none.
