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
| TopTrans_SurfaceTransition_Test.cxx | toptrans_surface_transition | 2 | pass | pass |

## Verification

`python3 tools/verify_toolkit.py TKGeomAlgo --jobs 16 --source OCCT --native-build BUILD`
compiles the unchanged upstream suites against the native closure, runs them and the
Rust targets, checks exact runtime names, then runs `tools/verify_tkgeomalgo_numeric.py`.
While no suite is mapped it runs the numeric verifier alone and reports 0 mapped
declarations. Evidence is in `target/oracle-tkgeomalgo/`. CI runs it like the TKBRep
verifier.

## Deliberate Rust deviations

- TopTrans: the sector arrays of `TopTrans_SurfaceTransition` (NCollection_Array2 of
  trivial types) are uninitialized natively. The angles are reset before any read; the
  orientations and curvatures are read only after a write, except the curvature of a
  sector that only the plane `Compare` wrote, which the curved `Compare` then reads
  (indeterminate natively). The port starts them at 0 and FORWARD.
- TopTrans: `SurfaceTransition::reset`, `reset_line`, `compare` and `compare_plane`
  return `Result`, carrying the native Standard_ConstructionError.

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

TopTrans (981 requests): `tt_curve` and `tt_surface` run a command sequence (resets
and compares) on one fresh transition and compare the exception type and both states
after every command; `tt_states` compares the static state tables. Directions come
from discrete fans (multiples of 45 and 30 degrees, near-tolerance tilts), so equal
angles, tangent faces, the touch case, the incoherent sectors and the parallel-frame
exceptions all occur; the hand-made sequences cover the upstream scenarios and the
curved ties at a null cosine. A surface request uses only one Compare overload (see
the uninitialized sector curvature in the deviations above).

## Preserved native behaviour

- TopTrans: a `CurveTransition` that was never reset answers the states of its
  stored FORWARD transitions (OUT before, IN after), not UNKNOWN: natively `Init` is
  false after construction. `Reset(Tgt)` of a straight line keeps the previous
  normal. The default `gp_Dir` members are +X.
- TopTrans: `SurfaceTransition::Reset` stores the tangent and the normal before the
  cross product that raises Standard_ConstructionError for parallel ones; the
  transition then stays undefined. A tangent that follows no principal direction
  leaves it undefined (not implemented natively, "NYIxpu"). A non-positive `Tole`
  means `Precision::Angular`. The curved `FUN_refnearest` compares curvatures with a
  fixed `1.e-4`, not with the tolerance. The second pass of an INTERNAL curve takes
  the opposite sector of the indices that the first pass left behind, as natively.
- TopTrans: the header declares the private `UpdateReference` and `ComputeCos`, which
  have no definition in OCCT; they have no Rust counterpart.
- `Intf::PlaneEquation` of a degenerate triangle (normal shorter than
  `gp::Resolution`) returns the undivided normal and a polar distance of 0.
