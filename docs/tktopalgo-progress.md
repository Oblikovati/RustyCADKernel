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

BRepClass_FaceExplorer_Test.cxx (8 declarations) is not mapped: every case builds its
faces with BRepBuilderAPI_MakeEdge, MakeWire and MakeFace (and one with
BRepPrimAPI_MakeSphere of TKPrim), which the port does not provide yet. The unchanged
suite compiles and passes natively against the shared build (8 of 8). Its eight cases
run as the behavior tests of `crates/tktopalgo/tests/brep_class_behavior.rs`, on the
same faces built with BRep_Builder by the probe's face reader (a holed square, the
lateral face of a cylinder with its seam, a square with a reversed wire, a disc, a
sphere face with its degenerated poles), checking the same properties; they move to a
1:1 translation once BRepBuilderAPI is ported.

## API mapping

- BRepClass (docs/topalgo-design.md section 2): `BRepClass_Edge` is `BRepClassEdge` (with
  `BndBoxState`), `BRepClass_FaceExplorer` is `BRepClassFaceExplorer`,
  `BRepClass_Intersector` is `BRepClassIntersector` (its IntConicCurveOfGInter base and
  IntRes2d results through `Deref`), `BRepClass_FClass2dOfFClassifier` and
  `BRepClass_FacePassiveClassifier` are aliases of `topclass::Classifier2d` over the
  intersector, `BRepClass_FClassifier` is an alias of `topclass::FaceClassifier` over the
  explorer, and `BRepClass_FaceClassifier` is `BRepClassFaceClassifier`. The output
  references become returned tuples (`Segment`, `OtherSegment`, `CurrentEdge`,
  `LocalGeometry`); the constructors of the face classifier are
  `with_explorer`, `with_face_point2d` and `with_face_point3d`, and its 3d Perform is
  `perform_point3d`. The protected `ComputeFaceBounds` is public. Every member that can
  raise returns `Result`.

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

BRepClass: 4779 requests (`tools/tktopalgo_corpus_brepclass.py`). Each request carries
a face record (`tools/tktopalgo_corpus_faces.py`) that both probes build with
BRep_Builder (`tktopalgo_probe_face.cxx` and `tp_probe/face.rs`): rectangles with and
without holes (a reversed wire or reversed edges), a disc, the lateral face of a
cylinder with its seam (and one with a pcurve running towards -u), a sphere face with
its degenerated poles, a polygon of twelve Bezier edges (boxes recommended), a square
with a gap at a vertex of tolerance 0.5, an infinite and a half-infinite strip, a
trimmed plane (finite surface bounds), a face with an INTERNAL edge, a triangle with a
Bezier edge whose tangent is undefined at both ends, a rectangle with a nearly flat
Bezier edge, a square with an edge without pcurve and a face without wires. The
families: `brc_edge` (BRepClass_Edge scripts), `brc_explorer` (edges, next edges,
pcurves and boxes), `brc_segment` and `brc_checkpoint` (segments and the point check
from grid, vertex, edge, extension, far and infinite points), `brc_intersect` and
`brc_intersect2` (every explorer edge with and without boxes, in every box state, on
finite and RealLast segments and with small and large tolerances, and one intersector
reused on two edges), `brc_classify` (FClassifier and both FaceClassifier constructors
on grids and seeded points), `brc_classify3d` (points near and away from the surfaces),
`brc_passive` (Reset and Compare, lines through vertices) and `brc_defaults`.

## Preserved native behaviour

### BRepClass

- `BRepClass_Intersector::Perform` appends its ON point (the origin of the segment
  within the tolerance of the pcurve) to the results of the previous Perform, and a
  Perform on an edge without pcurve, or whose box misses the segment, only marks the
  results not done and keeps them. The classifiers reuse one intersector for all the
  edges, so their scans read the earlier points too (`brc_intersect2` compares it).
- Its gap check (`CheckSkip`) reads the last parameter of the segment domain. A
  RealLast segment has none: Standard_DomainError leaves Perform when the ends of an
  edge and of its next edge both project beyond the segment origin. The classifiers
  never meet it, since their segments are finite.
- `BRepClass_Intersector::LocalGeometry` of an edge without pcurve dereferences a null
  curve natively (a crash); those requests are not in the corpus. An undefined tangent
  is the chord to a nearby point, and a chord within PConfusion leaves the tangent the
  caller passed, a default (1, 0) direction.
- `BRepClass_FaceExplorer::CheckPoint` moves a point whose distance to the centre of
  the face bounds is infinite to `(UMin - dU, VMin - dV)`, and a point so far that the
  bounds are below its `Epsilon` to twice that epsilon from the centre, in its
  direction; both return false, so the classifier checks the point again.
- The explorer probes an edge at the fractions 0.123, 0.3341 and 0.5452 of its range
  (from its last parameter), skips a fraction whose segment is tangent to the edge
  except the last one, whose segment then aims at the point of the edge nearest to the
  classified point. It never rejects a point, a wire or an edge.
- `BRepClass_FClassifier::Position()` is uninitialized natively until a classification
  finds an edge; the TopClass generic starts it at `IntRes2d_Head`. The corpus prints it
  only once an edge is set.
- `BRepClass_FacePassiveClassifier` and `BRepClass_FClass2dOfFClassifier` are the same
  class (their headers and bodies are identical).

## BRepClass deviations

- The TopClass trait's `CheckPoint` cannot fail, so a failure of the face bounds (no
  surface, BRepTools::UVBounds, the direction to a point at the exact centre of
  degenerate bounds) is kept by the explorer and returned by the next `Segment` or
  `OtherSegment` of the classifier. The classifier then records `Rejected()` false,
  where the native exception leaves the flag unchanged; natively a face without a
  surface is a null dereference (a crash), the port returns Standard_NullObject.
- `InitEdges` past the last wire gives an empty edge range and `CurrentEdge` past the
  last edge raises Standard_OutOfRange; both are unchecked vector accesses natively.
- `BRepClass_FaceClassifier` keeps the `rejected` flag of its 3d Perform itself
  (docs/topalgo-design.md section 2); every inherited member reads it first, with the
  native results.
