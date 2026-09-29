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

## Verification

`python3 tools/verify_toolkit.py TKGeomAlgo --jobs 16 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, runs them and the
Rust targets, checks exact runtime names, then runs `tools/verify_tkgeomalgo_numeric.py`.
While no suite is mapped it runs the numeric verifier alone and reports 0 mapped
declarations. Evidence is in `target/oracle-tkgeomalgo/`. CI runs it like the TKBRep
verifier.

## IntRes2d, IntImpParGen and IntCurve

IntRes2d (`int_res2d`), IntImpParGen (`int_imp_par_gen`) and IntCurve (`int_curve`, the
conic classes; the curve-curve templates follow with Geom2dInt) are ported in full:
`tools/audit_api_rows.py` reports 0 missing and 0 stale rows for each. The root class
`IntRes2d_Intersection` is a struct that the algorithms embed and reach through `Deref`;
its protected members (`Insert`, `Append`, `SetValues`, `ResetFields`, the `done` field)
are public so that algorithms of later toolkits can derive from it. Every constructor
overload is a named constructor and every `Perform` overload of IntCurve_IntConicConic a
`perform_<pair>` method; accessors that raise return `Result` with the native exception
type. IntImpParGen_Intersector.gxx is a generic (docs/geomalgo-design.md section 2).

Preserved native behaviour:

- `Clone` of `IntRes2d_Intersection` (and of the algorithms embedding it) is the native
  copy constructor: the copy has the results but is neither done nor reversed;
  `Intersection::assign` is the assignment operator, which copies both flags.
- `IntRes2d_Intersection::Append` merges composite results with references into the
  segment it rewrites, so a later branch of the same iteration reads the rewritten
  segment; a joined segment keeps the positions of the appended part, an appended one
  takes the relabelled positions.
- `IntRes2d_Domain::EquivalentParameters` does not raise on an open domain (0 and 0).
- `IntCurve_PConicTool::D2` of a line does not write the second derivative; the port
  writes through `&mut` outputs so that the intersector reads the stale value as natively.
- `IntCurve_IntConicConic::Perform(gp_Lin2d, gp_Parab2d)` offsets the line along
  `(dy, dx)` of its direction (not its normal); its bounded branch searches with the
  exact tolerance 1e-7 first, then again with the given ones when no point is found.
- `Perform(gp_Parab2d, gp_Hypr2d)` keeps the 50 samples of a hyperbola PConic (the other
  overloads set 20).
- `SET_BOUNDED_DOMAIN` compares the upper bound with the domain's FIRST parameter, which
  raises Standard_DomainError on a domain with only a last point.
- The line-ellipse overload evaluates the ellipse at the first bound of an arc for its
  second point and, when that point is at the end of the ellipse domain, takes the LINE
  domain's last parameter.
- The line-circle and line-line overloads ignore `TolConf` where the native code does,
  and the tube of `LineCircleGeometricIntersection` uses `R` for both radii (`Patch !!!`).
- The circle-circle, line-circle and line-ellipse overloads store their arcs in native
  arrays of 4 that numerical cases could overflow; the port keeps every arc in a vector.
- A default `IntCurve_IConicTool` (no curve) writes "Erreur sur le type de la courbe" to
  std::cout from its members; the port does not print (the probe discards std::cout).
- `IntCurve_IConicTool(gp_Lin2d)` and `IntCurve_PConic(gp_Lin2d)` return `Result` because
  the port's `gp_Ax22d` constructor does; a line direction cannot make it fail.

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

IntRes2d, IntImpParGen, IntCurve: `tkgeomalgo_corpus_intcurve.py` adds 2044 requests:
`intcurve_conics` intersects every ordered conic pair of IntCurve_IntConicConic
(crossing, tangent, coincident, disjoint and nearly tangent pairs; direct and indirect
frames; closed, open, wrapped, semi-infinite and unbounded domains; both tolerances and
the reversed flag; 150 seeded random pairs); `intcurve_imppar` runs
IntCurve_IntImpConicParConic on every implicit/parametric kind with its sample count and
EpsX; `intcurve_itool`, `intcurve_ptool` and `intcurve_function` evaluate the tools,
including the centre of a circle or ellipse and the default tool; `intcurve_interval`
covers Interval and PeriodicInterval; `imppar_transition` and `imppar_domain` the
transition, position and normalisation helpers; `intres2d_*` the domain accessors,
Insert and the composite Append.

## Preserved native behaviour

- `Intf::PlaneEquation` of a degenerate triangle (normal shorter than
  `gp::Resolution`) returns the undivided normal and a polar distance of 0.
