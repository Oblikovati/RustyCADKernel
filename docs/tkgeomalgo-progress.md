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

## API mapping

- GccEnt: the static members of the `GccEnt` class are free functions of
  `gcc_ent`; overloads take distinct names (`unqualified_lin`/`unqualified_circ`,
  `enclosed_lin`/`enclosed_circ`, `outside_lin`/`outside_circ`), the `bool` overload of
  `PositionFromString` returns `Option`, and `GccEnt_BadQualifier` is a `Failure` of
  that type (`gcc_ent::bad_qualifier`).
- GccInt: the abstract `GccInt_Bisec` is the trait `Bisec`, reached through
  `Handle<dyn Bisec>` (docs/geom-design.md); the five bisectors are structs with plain
  fields, since none has a mutator.
- Hatch: overloads take distinct names (`add_line_at_distance`, `trim_range`,
  `trim_segment`, `nb_intervals_on`, `set_tolerance`), output references become returned
  tuples, and the friend-only data of `Hatch_Parameter` and `Hatch_Line` stays private
  to the `hatch` module.

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

GccEnt: `gccent_position` feeds the five positions to `GccEnt::PositionToString` and
`GccEnt::Print` with eight names for both `PositionFromString` overloads (case variants,
the empty string, an unknown name, a trailing blank); `gccent_qualified` builds qualified
circles and lines with every position and the seven GccEnt makers (55 requests).

GccInt: `gccint_bisec` builds the five bisector kinds on three frames and calls all six
accessors, the RTTI name and `IsKind` (15 requests).

Hatch: `hatch_hatcher` adds lines of the four kinds, trims them with whole lines, parameter
ranges and oriented segments, and queries every line and interval including the
out-of-range indices around them: a closed square in both modes and orientations, open
and infinite intervals, polygon vertices on hatching lines, two intersections closer than
the tolerance, parallel and identical trimming lines, degenerate segments, segments near
and beyond RealLast, a negative tolerance, and 24 seeded random polygons (38 requests).

## Preserved native behaviour

- `Intf::PlaneEquation` of a degenerate triangle (normal shorter than
  `gp::Resolution`) returns the undivided normal and a polar distance of 0.
- `GccEnt_QualifiedCirc::Qualifier` and `GccEnt_QualifiedLin::Qualifier` return the stored
  `GccEnt_unqualified` for an unqualified argument, although the header documents
  `GccEnt_noqualifier`.
- `GccInt_Bisec` accessors raise `Standard_NotImplemented`, not the `Standard_DomainError`
  that the header documents.
- `Hatch_Line::AddIntersection` removes an existing intersection closer than the tolerance
  and drops the new one (akm OCC109), so two close crossings cancel each other.
- `Hatch_Hatcher::StartIndex` and `EndIndex` accept J = 1 on a line without intersections
  in unoriented mode (they return 0, 0), where `Start` and `End` raise
  `Standard_OutOfRange`. Every line and intersection index outside its sequence raises
  `Standard_OutOfRange`, as the native `NCollection_Sequence` bounds check does.
- `Hatch_Hatcher::Trim(P1, P2)` scales a segment vector whose X (else Y) exceeds
  0.9 RealLast by the inverse of that component; a vector that overflowed to infinity
  becomes NaN and the segment is ignored.
