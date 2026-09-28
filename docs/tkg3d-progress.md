# TKG3d port status

**In progress; toolkit acceptance is not complete.** The `Geom` package is ported
in full in `crates/tkg3d` (`rustycad-tkg3d`), together with the GeomEval
evaluation descriptors and `GeomEval_RepUtils.pxx` that `Geom` includes, the
private helpers `Geom_OffsetCurveUtils.pxx`, `Geom_OffsetSurfaceUtils.pxx`,
`Geom_ExtrusionUtils.pxx`, `Geom_RevolutionUtils.pxx` and
`Geom_OsculatingSurface`, and the whole `AdvApprox` package, which
`Geom_OffsetSurface::UIso`/`VIso` need. The whole `GeomEval` package (the eleven
curves and surfaces), the whole `GeomHash` package (every public and private
hasher, mesh hashers included) and the `TopAbs` package are ported too. The other
TKG3d packages (`Adaptor3d`, `GeomAdaptor`, `GeomGridEval`) are not ported yet;
only the `GeomAdaptor_Surface` D1 path that `Geom_OsculatingSurface` samples with
is ported, next to that class. TKG3d depends on TKG2d as in OCCT: it reuses the
TKernel lock (`SharedState`), the array, dump, spline-data and `std::min`/`max`
helpers, the curve descriptor family (`define_rep_curve_descriptors!`), the
generic T-Bezier and AHT-Bezier basis evaluators of
`rustycad_tkg2d::geom2d_eval::eval_basis` (the GeomEval curves are the Geom2dEval
algorithms on `gp_XYZ`) and the Geom2dHash rounding, combining and dispatch
helpers.

The class mapping follows [geom-design.md](geom-design.md): abstract classes are
traits behind `Handle<dyn Trait>`, each object keeps its state behind one lock,
and native exceptions are `Failure` results.

The toolkit test inventory has 11 `Geom_*_Test.cxx`, 11 `GeomEval_*_Test.cxx` and
3 `GeomHash_*_Test.cxx` suites (TopAbs has none). 22 complete suites account for
438 mapped declarations, with matching native and Rust passes:

| Source | Rust target | Declarations |
| --- | --- | ---: |
| Geom_BSplineCurve_Test.cxx | geom_bspline_curve | 43 |
| Geom_BezierCurve_Test.cxx | geom_bezier_curve | 38 |
| Geom_BezierSurface_Test.cxx | geom_bezier_surface | 45 |
| Geom_Circle_Test.cxx | geom_circle | 8 |
| Geom_Line_Test.cxx | geom_line | 10 |
| Geom_OffsetCurve_Test.cxx | geom_offset_curve | 4 |
| Geom_Plane_Test.cxx | geom_plane | 8 |
| Geom_SurfaceEval_Test.cxx | geom_surface_eval | 9 |
| GeomEval_AHTBezierCurve_Test.cxx | geom_eval_aht_bezier_curve | 18 |
| GeomEval_AHTBezierSurface_Test.cxx | geom_eval_aht_bezier_surface | 14 |
| GeomEval_CircularHelicoidSurface_Test.cxx | geom_eval_circular_helicoid_surface | 14 |
| GeomEval_CircularHelixCurve_Test.cxx | geom_eval_circular_helix_curve | 14 |
| GeomEval_EllipsoidSurface_Test.cxx | geom_eval_ellipsoid_surface | 20 |
| GeomEval_HypParaboloidSurface_Test.cxx | geom_eval_hyp_paraboloid_surface | 19 |
| GeomEval_HyperboloidSurface_Test.cxx | geom_eval_hyperboloid_surface | 19 |
| GeomEval_ParaboloidSurface_Test.cxx | geom_eval_paraboloid_surface | 16 |
| GeomEval_SineWaveCurve_Test.cxx | geom_eval_sine_wave_curve | 11 |
| GeomEval_TBezierCurve_Test.cxx | geom_eval_tbezier_curve | 18 |
| GeomEval_TBezierSurface_Test.cxx | geom_eval_tbezier_surface | 17 |
| GeomHash_CurveHasher_Test.cxx | geom_hash_curve_hasher | 40 |
| GeomHash_MeshHasher_Test.cxx | geom_hash_mesh_hasher | 10 |
| GeomHash_SurfaceHasher_Test.cxx | geom_hash_surface_hasher | 43 |

Behaviour tests cover the other classes: `geom_behavior_surfaces.rs`,
`geom_behavior_bspline_surface*.rs`, `adv_approx_behavior.rs` and
`top_abs_behavior.rs`.

## Verification

`python3 tools/verify_toolkit.py TKG3d --jobs 8 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, runs them and
the Rust targets, checks exact runtime names, then runs
`tools/verify_tkg3d_numeric.py`. Evidence is in `target/oracle-tkg3d/`.

## Numeric comparison policy

`tools/verify_tkg3d_numeric.py` uses the shared driver `tools/toolkit_numeric.py`
(also used by TKG2d). It builds `tools/oracle/tkg3d_probe.cxx` (a table of
operations over `tkg3d_probe_*.cxx`) and its twin
`crates/tkg3d/examples/numeric_probe.rs`, runs both on the 2395 requests of
`tools/tkg3d_corpus.py` (one `tkg3d_corpus_*.py` module per probe file) and
requires identical bits for finite values, infinities and signed zeros, NaN by
classification, identical exception types and identical `DumpJson` text.

The corpus builds every curve and surface kind (valid, degenerate and invalid
data, nested trimmed and offset curves and surfaces) and checks for each one the
ranges, closure, periodicity, continuity, `IsCN*`, `EvalD0` to `EvalD3`, `EvalDN`
of every order (negative ones included), `Value`, the reversed parameters, the
iso curves, `TransformParameters`/`ParametricTransformation`, every
`Geom_Geometry` move and its copying twin, and `DumpJson`. Other requests cover the
point, direction, vector, axis placement and transformation classes; every conic
getter, setter and gp constructor; trimming; the offset-curve utilities directly
(first derivatives from 1 to 1e-160 reach every stable and unstable branch);
Bezier and B-spline curve and surface edition; elementary and swept surface
setters; AdvApprox with 1D/2D/3D subspaces, every continuity and both cutting
strategies; offset surfaces of every basis with both signs (equivalent surfaces,
AdvApprox isos, osculating surfaces along collapsed B-spline and Bezier
boundaries); and evaluation representations of all three kinds on every class
that accepts them, mapped ones with domains and `SwapUV` included.

The GeomEval curves and surfaces (kinds 20 and above of the curve and surface
specifications) go through the same summaries, plus their accessors, their copies,
`EvalDN` up to high orders, every constructor refusal and the refused members. The
GeomHash requests hash and compare sets of curves, surfaces, points, directions,
placements, polygons, polygons on triangulations and triangulations under nine
tolerance pairs (default, zero, tiny, huge), with aliases and null handles; hashes
are printed as integers and must match bit for bit, including the fallback on the
class name of GeomEval geometry. The TopAbs requests cover the orientation algebra,
every name and 68 `FromString` inputs (cases, NUL bytes, padding, non-ASCII).

## Preserved native behaviour

Curves:

- `Geom_Direction::Copy` normalizes the stored unit vector again through `gp_Dir`.
- `Geom_Axis2Placement::SetDirection` stores the new main direction before the
  projection of the old X direction can raise; `SetAxis` (inherited) changes the
  main axis only and keeps the X and Y directions.
- `Geom_Conic::Reverse` reverses the main direction of the frame (Geom2d reverses Y).
- `Geom_Hyperbola::Eccentricity` and `Parameter` raise Standard_ConstructionError
  for a null major radius; `Geom_Parabola::EvalD3` returns a null third derivative.
- `Geom_TrimmedCurve::SetTrim` stores the new bounds before it raises the out of
  range error, and `Copy` re-trims with the default `Sense` and `theAdjustPeriodic`.
- `Geom_OffsetCurve` copies its basis, folds nested offsets into one offset and
  direction (`PrevOff * V1 + Offset * V2`, through `gp_Dir::SetXYZ`), keeps the folded
  values when `SetBasisCurve` fails, multiplies the offset by the signed scale
  factor in `Transform` (Geom2d uses the absolute value) and returns the basis
  derivative above order 3.
- `Geom_OffsetCurveUtils::CalculateD3` subtracts the normal term of the second
  derivative in its low-magnitude branch, and, unlike Geom2d, its stable branch
  divides D2Ndir and D3Ndir by R first and multiplies by the offset last.
- B-spline curve: `EvalD0` to `EvalD3` normalize a periodic parameter and then pass
  the raw `U` to `LocateParameter`; `updateKnots` maps an interior multiplicity
  above the degree to C3; the copy keeps a stale `MaxDerivInv`; `IncreaseDegree`
  and `SetOrigin(Index)` clear the evaluation representation before their checks,
  `InsertKnots` only when poles are added, `RemoveKnot` and the Move members only on
  success, `SetKnot` only when the value changes and `SetWeight` always; the rational
  constructor with CheckRational = false stays rational; `MovePointAndTangent` makes
  a periodic curve non-periodic first; `LocalDN` names LocalD3 in its message;
  `PeriodicNormalization` uses a floor/nextafter closed form (an infinite parameter
  gives NaN instead of looping as in 2D).
- Bezier curve: closure compares with Precision::Confusion; the weight checks read
  `W(1..n)` whatever the lower bound; `InsertPoleAfter` refuses a curve of
  MaxDegree() poles; only `Transform` and the edits that rebuild the poles clear
  the evaluation representation; the copy resets the resolution cache to 0.

Surfaces:

- `Geom_Plane` reverses the X (U) and Y (V) directions of its frame; the other
  elementary surfaces reverse Y and the main direction; `Geom_ConicalSurface::VReverse`
  also negates the semi-angle.
- The cylinder and cone return a zero vector for `EvalDN` with Nv > 1 before calling
  ElSLib; the cylinder, cone and sphere compute `Coefficients` with their own
  formulas, the torus through `gp_Torus::Coefficients` (Standard_DimensionError for
  an array shorter than 35).
- The `Geom_ToroidalSurface` constructor accepts a minor radius not smaller than the
  major one, which the setters refuse.
- `Geom_SphericalSurface::UIso` is a circle trimmed to [-PI/2, PI/2], stored as
  [3PI/2, 5PI/2] by the periodic adjustment.
- Swept surfaces keep a private copy of the basis curve; `Copy` rebuilds the surface
  from the stored basis; every setter, reversal and `Transform` clears the evaluation
  representation; the extrusion's `IsCNu` and the revolution's `IsCNv` raise for
  N < 0 while the other direction is always true; `ParametricTransformation` of the
  extrusion is `TU * TV`; `Geom_SurfaceOfRevolution::Transform` also reverses U when
  `ScaleFactor * det(HVectorialPart) < 0`, `ReferencePlane` raises
  Standard_NotImplemented, and `VIso` normalizes the frame direction twice.
- `Geom_RevolutionUtils` zeroes D1U below Precision::SquareConfusion(); its
  `CalculateDN` cycles over `DerU % 4`.
- Bezier surface: `EvalD0` to `EvalD2` use span hints (1, 1) and `EvalD3`/`EvalDN`
  (0, 0), all passing the U knots as V knots; `EvalDN` tries the evaluation
  representation before checking the orders; `Increase` passes the U knots for the V
  elevation; only the reversals and `Transform` clear the evaluation representation;
  `InsertPole*Before(1)` raise Standard_OutOfRange through `After(0)`; inserts without
  weights into a rational surface add unit weights without recomputing the flags;
  `SetWeight` with a unit weight on a non-rational surface returns early;
  `SetWeightCol`/`Row` check only the length (Col) or the bounds (Row) and keep the
  weights written before a failure; `UIso`/`VIso` are rational only when the surface
  is U-/V-rational.
- B-spline surface: `EvalD0` normalizes periodic parameters and evaluates at index
  0, `EvalD1`/`EvalD2` locate the span with the raw parameters, `EvalD3`/`EvalDN`
  use them throughout; `PeriodicNormalization` uses a truncating `modf`;
  `InsertU/VKnots` store nothing when the pole count does not change;
  `SetU/VKnots` store while they check; `Weight()` of a non-rational surface is 1
  without an index check; `SetPole` has no index check of its own; the
  `SetU/VOrigin` messages name Geom_BSplineCurve; `IncreaseDegree` stores U before
  checking V; `MovePoint` evaluates through the evaluation representation and clears
  it only when a pole moves; `IsURational` means that the U isos are rational;
  `IsUClosed` compares the boundary isos with `IsEqual` and Precision::Confusion.
- Both spline surfaces and `Geom_RectangularTrimmedSurface` dump their base level as
  "Geom_Surface", because `Geom_BoundedSurface` inherits `Geom_Surface::DumpJson`.
- `Geom_RectangularTrimmedSurface::SetTrim` stores the U bounds before a V failure and
  the ordered bounds before an out of range failure; the one-direction constructor
  starts from zeroed fields (the OCCT allocator clears new objects); `Copy` returns a
  null handle when neither direction is trimmed.
- `Geom_OffsetSurface` sums the offsets of nested offsets, accepts a C0 basis when its
  profile (or middle U iso) and middle V iso are G1, keeps the summed offset and the
  continuity when `SetBasisSurface` fails, rebuilds the equivalent representation in
  `SetOffsetValue` but not after `UReverse`, `VReverse` or `Transform` (which multiplies
  the offset by the signed scale factor), transforms parameters through the basis and
  then again through the equivalent surface, and dumps the equivalent surface as
  "EquivSurface" (DumpFieldToName drops the "an" of `anEquivSurface`).
- `Geom_OsculatingSurface::Init` clears its flags whenever it does not complete,
  keeping the osculating surfaces already appended.
- AdvApprox: the odd Gauss-point branch of `SimpleApprox::Perform` never resets its
  running sum and overwrites the even coefficients; `Approximation` never reports
  error -1, so `HasResult` and `IsDone` agree; `PrepareConvert` zeroes the interval
  factors below 1e-5 (the PRO18577 guard); `PrefAndRec` copies its tables before it
  refuses a weight <= 1; a failed `SimpleApprox::Perform` keeps the previous tables.

GeomEval:

- No GeomEval curve or surface changes its parameterization: `Reverse`,
  `UReverse`/`VReverse`, `ReversedParameter`, `U/VReversedParameter`, `UIso`/`VIso`
  and `Transform` raise Standard_NotImplemented, so every move and `Reversed`,
  `Transformed` fails. For this reason the Geom traits return `Result` from the
  reversed parameters (as Geom2d does). The curves and the Bezier surfaces have no
  successful mutator and keep plain fields; the elementary surfaces keep their
  setters behind `SharedState`.
- The T-Bezier and AHT-Bezier curves run the Geom2dEval evaluators on `gp_XYZ`:
  equal weights still give a rational curve; the non-rational AHT curve stores one
  uninitialized weight (`myWeights(1,1)`, 0.0 in the port); the rational AHT
  `EvalD3` computes its first derivative as `(N1 - C*W1)/W0`, so it can differ in
  the last bits from `EvalD1`/`EvalD2`; the rational `EvalDN` of orders 1 to 3
  delegates to `EvalD1..D3`; the AHT `DumpJson` writes every pole under "Pole".
  `EvalDN` raises Standard_RangeError below order 1.
- `GeomEval_SineWaveCurve::EvalDN` and `GeomEval_CircularHelixCurve::EvalDN` take
  `sin`/`cos` of the angle shifted by `N*Pi/2` instead of rotating (sin, cos) by
  quarter turns as `EvalD1..D3` and Geom2dEval do, so `EvalDN(u, 1..3)` can differ
  in the last bits from `EvalD1..D3`. The helix accepts a zero or negative pitch.
- The elementary surfaces (ellipsoid, hyperboloid, paraboloid, hyperbolic
  paraboloid, circular helicoid) inherit CN continuity; `IsVClosed` and
  `IsVPeriodic` are false. `EvalDN` still adds a Z coefficient that is zero (the
  ellipsoid multiplies it by 0.0, the hyperboloid uses 0.0 when Nu > 0); the
  hyperbolic paraboloid builds its Z coefficient from 0.0 with `+=`, so a lone -0.0
  becomes +0.0. The helicoid refuses only an exact zero pitch (a negative pitch and
  NaN pass). The hyperboloid dumps its mode under the key `(int)myMode`.
- The T-Bezier and AHT-Bezier surfaces add the same tensor sums row by row, but the
  T-Bezier rounds the rational weight product as `(w * Bu) * Bv` and the AHT as
  `w * (Bu * Bv)`. The rational AHT `EvalD0` divides by W00 while `EvalD1` to
  `EvalD3` multiply by `1 / W00`, so the points can differ in the last bit. A purely
  algebraic non-rational AHT surface uses nested Horner schemes in `EvalD0..D3`
  (a special bicubic form when both degrees are 3), but not in `EvalDN`, where each
  order multiplies the previous basis value by the quotient `(k - d) / t` (exact
  factorials at `t == 0`). The rational T-Bezier `EvalDN` above total order 3 raises
  Geom_UndefinedDerivative. `GeomEval_AHTBezierSurface::IsUClosed`/`IsVClosed` are
  always false; the T-Bezier surface compares its boundary pole rows with
  Precision::Confusion, and a NaN distance does not make it open. The non-rational
  AHT surface stores a 1x1 uninitialized weight grid (0.0 in the port), the
  T-Bezier an empty one. Both dump their base level as "Geom_Surface".

GeomHash:

- Rounding is `static_cast<int64_t>(std::round(v / HashTolerance))` with the x86-64
  `cvttsd2si` result: NaN and out-of-range values give `INT64_MIN`, so a zero or tiny
  HashTolerance and huge coordinates still hash deterministically.
- `GeomHash_AxisPlacement` hashes the location, main and X directions of a `gp_Ax2`
  but not the Y direction; the elementary surfaces hash `Position().Ax2()`, which
  reverses the normal of a left-handed frame; the plane hash is the placement hash
  itself. The offset-curve hash combines the basis, the rounded offset, then the
  direction. The Bezier and B-spline surface hashes pack the rationality as
  `IsURational | IsVRational << 1`. A class without a dedicated hasher (the GeomEval
  geometry) hashes its type name and compares by handle identity.
- The mesh hashers default both tolerances to Precision::Computational(), hash each
  field into its own slot of a zero-initialized array (6, 11, 9 or 20 slots), and
  reject with `|a - b| > CompTolerance`, so a NaN difference counts as equal (the
  geometry hashers accept with `<=`, so NaN is different).
  `PolygonOnTriHashKey::TriRepId` is widened to `int64_t` before hashing; a null
  polygon marks its slot with `hash(int 1)`.

TopAbs:

- `TopAbs::Compose` reads its table as `[Or2][Or1]`: an INTERNAL or EXTERNAL `Or2`
  wins, so `Compose(EXTERNAL, INTERNAL)` is INTERNAL, unlike the table in the header
  comment.
- `TopAbs::Print(TopAbs_State)` indexes the names "ON", "IN", "OUT", "UNKNOWN" with
  the enumeration IN, OUT, ON, UNKNOWN, so IN prints "ON", OUT "IN" and ON "OUT".
- `ShapeTypeFromString`/`ShapeOrientationFromString` upper-case through
  `TCollection_AsciiString`: the name ends at the first NUL and only ASCII letters
  change case. Their `bool (..., T&)` overloads return an `Option` in Rust.

## Deliberate Rust deviations

- Getters that return `const&` natively return copies; `WeightsArray` lends the
  stored array to a closure. Overloads have distinct Rust names (`mirror_point`,
  `with_weights`, `set_trim_direction`...), output references become returned
  values, and parameter pairs are tuples.
- `Geom_Surface::ParametricTransformation` returns a `Result`, because the product of
  two `gp_GTrsf2d` in the extrusion can raise natively.
- Where native code dereferences a null handle (a null basis curve or surface given
  to a trimmed, offset or swept constructor, a null AdvApprox tolerance table or
  result), the port returns Standard_ConstructionError, Standard_NullObject or
  Standard_ProgramError.
- Where native code reads outside its buffers (`LocalD*` of a B-spline surface with
  wide knot ranges, the ranged multiplicity edits past the last knot), the port
  returns Standard_OutOfRange; the corpus avoids those inputs.
- `AdvApprox_ApproxAFunction::Degree()` returns 0 without a result (native value
  uninitialised); the evaluator is a Rust trait (closures implement it) borrowed for
  the computation.
- A failure of the surface inside the AdvApprox evaluator of the offset-surface isos
  stops the approximation and is returned, as the native exception would propagate.
- The native Bezier and B-spline hash comparisons are `noexcept` but read the second
  object with the first object's pole count; equal knots with a different pole
  count terminate the native process, and the port returns "different". The corpus
  never compares such pairs.
- `GeomEval_HyperboloidSurface::SheetMode` is a Rust enum with the two sheet modes;
  natively `SetMode` stores any integer cast to the enumeration.

## Remaining coverage

- `Geom_BSplineSurface_Test.cxx` (50) and `Geom_OffsetSurface_Test.cxx` (33) use
  `GeomAdaptor_Surface` and wait for the GeomAdaptor package;
  `Geom_CurveEval_Test.cxx` (11) needs `GeomAPI_PointsToBSpline`.
- The Adaptor3d, GeomAdaptor, GeomGridEval and GeomAPI suites wait for their
  packages.
- `InitFromJson` does not exist in Geom; `DumpJson` is compared on the corpus.
