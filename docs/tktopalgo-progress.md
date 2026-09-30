# TKTopAlgo port status

**In progress; toolkit acceptance is not complete.** `crates/tktopalgo`
(`rustycad-tktopalgo`) is the port of the ModelingAlgorithms toolkit TKTopAlgo. It
depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep and TKGeomAlgo ports,
as the native toolkit does (its EXTERNLIB). Each of the 16 OCCT packages is one module
of the crate, declared up front in `src/lib.rs`; a module whose package is not ported
yet holds only its documentation. Ported so far: BRepClass, BRepTopAdaptor and
IntCurvesFace. The numeric corpus has 5880 requests: BRepClass 4779, BRepTopAdaptor 650,
IntCurvesFace 424 and the MakeVertex stub pipeline 27.

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

BRepTopAdaptor_FClass2d_Test.cxx (5 declarations) is not mapped: four cases build their
faces with BRepBuilderAPI_MakeEdge, MakeWire and MakeFace, which the port does not
provide yet. The unchanged suite compiles and passes natively against the shared build
(5 of 5). Its five cases run as the behavior tests of
`crates/tktopalgo/tests/brep_top_adaptor_behavior.rs` with the same inputs and checks, on
the same faces built with BRep_Builder by the probe's face reader (edges with 3d lines
and pcurves, as MakeEdge and MakeFace give on a plane; the cylinder with its seam); they
move to a 1:1 translation once BRepBuilderAPI is ported. IntCurvesFace has no upstream
suite; `tests/brep_top_adaptor_tools.rs` and `tests/int_curves_face_behavior.rs` hold its
behavior tests and those of the tool, vertex and Tool classes.

BRepIntCurveSurface is not ported: BRepIntCurveSurface_Inter.cxx includes BRepBndLib
(level 4, not ported), which it calls to box each face.

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
- BRepTopAdaptor (docs/topalgo-design.md section 4): `BRepTopAdaptor_FClass2d` is
  `BRepTopAdaptorFClass2d` (its lazy exact explorer behind a mutex, so `perform` and
  `test_on_restriction` take `&self`); `BRepTopAdaptor_TopolTool` is
  `BRepTopAdaptorTopolTool` and `BRepTopAdaptor_HVertex` is `BRepTopAdaptorHVertex`,
  each with a default Adaptor3d base as its parent field (the RTTI chain) and the
  members its header declares; the overloads are `initialize_surface`,
  `initialize_curve`, `curve_orientation`, `vertex_orientation`, `curve_tol_3d` and
  `vertex_tol_3d`, as the Adaptor3d_TopolTool port names them. `Edge()` returns the
  edge (a `void*` to it natively). `BRepTopAdaptor_Tool` is `BRepTopAdaptorTool`, its
  `Init` overloads `init_face` and `init_surface`. The tool does not implement the
  IntStart `TheTopolTool` trait (section 4): the generic consumers call the
  `Adaptor3d_HVertex` members of its vertices directly, and the concrete TKG3d struct
  cannot dispatch them to the overrides yet.
- IntCurvesFace (section 4): `IntCurvesFace_Intersector` is `IntCurvesFaceIntersector`
  (transient; `perform_line` and `perform_curve` take `&mut self`),
  `IntCurvesFace_ShapeIntersector` is `IntCurvesFaceShapeIntersector` (its intersectors
  in a `Vec`). The i-th point accessors return Standard_OutOfRange outside
  `[1, NbPnt]`, as the native sequences raise.

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

BRepTopAdaptor: 650 requests (`tools/tktopalgo_corpus_breptopadaptor.py`), on the
BRepClass faces and the faces of `tools/tktopalgo_corpus_adaptor_faces.py`: the lateral
face of a cone, a whole torus (two seams through one vertex) and a torus patch, a Bezier
patch, B-spline patches (with a C1 knot, and with a hole), squares whose edges carry 3d
lines of tolerance 0.1 (and the same without 3d curves), a thin half ring whose polygon
is refined, a wire the WireExplorer cannot walk, a one-segment wire and a sphere whose
pole edges are not flagged degenerated (the face reader gains the surface kinds 4 to 7
and the edge kind 6 for them). The families: `bta_fclass2d` (Perform and
TestOnRestriction, reframed or not, on grids, vertices, edge middles and points 1e-9 and
1e-5 beside them, periodic copies and seeded points, with three tolerance pairs),
`bta_destroy`, `bta_topol` (arcs, their vertices with the HVertex parameter and
resolution, samples and domain, restricted or not), `bta_classify`, `bta_tool` and
`bta_errors`.

IntCurvesFace: 424 requests (`tools/tktopalgo_corpus_intcurvesface.py`): `icf_line` (one
intersector on groups of lines through grid points, vertices and seeded points of the
analytic faces, vertical and tilted lines over the spline patches, tangent, axial,
in-plane and parallel lines, on infinite, finite, short and late ranges, with and
without the restriction and the boundary tolerances, and the unrestricted
boundary-tolerance lines of the toleranced squares), `icf_curve` (circles, segments and
a Bezier curve on every face) and `icf_shape` (compounds of three to four faces with
Perform, PerformNearest and Perform on a curve adaptor, after one or two Loads).

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

### BRepTopAdaptor

- `BRepTopAdaptor_FClass2d` starts its sampled bounds at `Umin = Vmin = Umax = Vmax = 0`
  (the `RealLast()` start is commented out), so the bounds always hold the UV origin;
  each wire's `CSLib_Class2d` receives the bounds accumulated so far (the earlier wires'
  points included, the later ones not), and `PerformInfinitePoint` classifies
  `(Umin - (Umax - Umin), Vmin - (Vmax - Vmin))`.
- A wire whose WireExplorer walk misses edges, whose polygon has at most three points or
  whose thickness (twice its area over its perimeter) is not representable is bad: the
  loop stops there (the later wires are not sampled), the first wire role becomes
  invalid and every classification is exact. An edge without pcurve ends the
  constructor at once: the earlier polygons stay, the first role is not invalidated and
  U1..V2 stay 0 (they are never read).
- The forgotten-degenerated check compares the *square* distance of ten 3d samples to
  Precision::Confusion(); a reversed range takes no step and counts as degenerated. The
  deflection of a triple is measured only when the square distance of its ends is not
  0.0 (a NaN counts).
- The exact classifier reuses one `BRepClass_FaceExplorer`, built with its recommended
  box switch on first use, so the probing state of one exact classification carries
  over to the next. An uncertain polygon answer is exact (tolerance `min(Toluv, 4)`) in
  Perform and ON in TestOnRestriction; on a bad wire TestOnRestriction classifies
  exactly with its own tolerance.
- Periodic reframing shifts a point by whole periods one at a time (a point 1e300 away
  loops without end natively; not in the corpus), then tries the copies up to the
  sampled upper bounds until one is IN or ON.
- `BRepTopAdaptor_TopolTool` builds its classifier with the tolerance of the first
  Classify or IsThePointOn and keeps it (later tolerances only change the ON tolerance
  of IsThePointOn). IsThePointOn does not check for a null face. Before the samples are
  computed, the sample grid origin and steps of the tool built on a surface are
  uninitialized natively (0 in the port; SamplePoint is not probed before them). A
  quadric counts 8 samples per unit of U and 7 per unit of V (at most 30 x 15), then at
  least 10 in each direction.
- `Vertex()` past the last vertex raises Standard_TypeMismatch: the explorer's current
  shape is then the edge. `BRepTopAdaptor_HVertex::Value()` is always
  `(RealFirst, RealFirst)`.
- The tool on a face with an edge without pcurve crashes natively on that arc; the port
  reads the empty pcurve adaptor. Those requests are not in the corpus.

### IntCurvesFace

- `IsParallel()` is set by a Perform whose done intersection has no point and never
  reset by a later Perform.
- On a restricted planar or quadric face the curve-surface intersection drops the
  points outside the face UV bounds before the classification, so the boundary
  tolerance (a point OUT with the smallest tolerance, ON with the largest, and within
  the largest 3d tolerance of an edge's 3d curve) only acts on an unrestricted surface
  or within the bounds. An edge without 3d curve makes that check raise
  Standard_NullObject (GeomAPI_ProjectPointOnCurve on a null curve), natively too.
- The topology tool keeps the classifier built by the first classification (tolerance
  0 without boundary tolerances, else the smallest UV tolerance); a second tool is built
  per Perform for the largest tolerance.
- A line that misses the polyhedron box leaves the intersector done with no point. A
  surface without extension along a direction leaves the intersector never done.
- `IntCurvesFace_ShapeIntersector::Load` appends its intersectors to those of an earlier
  Load while the face count restarts, so a second Load performs on the first faces
  loaded; the PerformNearest order counters are created once (above two faces) and
  never resized. PerformNearest skips a face whose range became empty, and SortResult
  still reads that face's earlier points; the points at the nearest parameter of later
  faces are kept.
- `(int)` casts of out-of-range doubles (sample counts of the polyhedron and of the
  quadric samples) give `INT_MIN`, as x86 `cvttsd2si` does.

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

## BRepTopAdaptor and IntCurvesFace deviations

- The members natively dereferencing a null or wrong-typed handle raise
  Standard_NullObject (HVertex Parameter, Resolution and IsSame on other adaptors,
  InitVertexIterator without an arc, the samples without a surface); `Value()` and
  `Edge()` past the last arc raise Standard_NoSuchObject and `Next()` past it does
  nothing (null dereferences natively). SamplePoint with a zero U count raises
  Standard_ProgramError (a division by zero natively).
- The base members that BRepTopAdaptor_TopolTool inherits without redeclaring them
  (`Identical`, `SamplePnts`, `BSplSamplePnts`, `UParameters`, `VParameters`,
  `IsUniformSampling`) are not members of the Rust struct (docs/topalgo-design.md
  section 4), and the struct does not implement IntStart's `TheTopolTool`.

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

MAT and MAT2d: the `tp_mat*` requests (84, `tools/tktopalgo_corpus_mat.py`) run the
bisecting-locus pipeline of `BRepMAT2d_BisectingLocus::Compute` (written again in the probe:
`MAT2d_CutCurve` on every curve, `MAT2d_Circuit`, `MAT2d_Tool2d`, `MAT2d_Mat2d`, `MAT_Graph`,
the fusion of the basic elements of several lines, `CompactArcs`/`CompactNodes`) and print
the circuit, the bisectors of the tool and the whole graph (arcs, nodes, basic elements and
their `MAT_Zone`). The figures are convex and concave polygons, rounded rectangles, a
D shape, B-spline and ellipse contours, contours with one and two holes, and open lines
(polylines, a segment and an arc, a B-spline, an open line inside a closed contour), on
both sides, with joins by arc and by intersection, and with the closed and the open
algorithms. Other requests exercise the classes one by one: edit scripts of
`MAT_ListOfEdge`, `MAT_Bisector::Dump`, a hand-made graph piece, `MAT2d_Connexion`
(`Reverse`, `IsAfter`, `Dump`), `MAT2d_CutCurve`, `MAT2d_MiniPath` and `MAT2d_Tool2d`.

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

### MAT and MAT2d

- `MAT_Graph::CompactArcs` and `CompactNodes` scan the indices from 1 until they found as
  many items as the count; when fewer items are bound the native scan never ends. The port
  stops at the largest bound key (the same result when the scan ends natively).
- `MAT2d_Mat2d` reads `intersectionpoint` before `IntersectBisector` writes it (it writes
  it only when it finds a point): natively uninitialised before the first point, 0 in the
  port.
- `MAT_Graph::FusionOfBasicElts` writes its six output references only on the paths that
  merge arcs; the others keep the caller's values (`MatFusion` fields keep theirs).
- `MAT2d_Circuit::Perform` on a figure of one line appends the length of the line once
  more to the line lengths, so `LineLength(2)` answers for a one-line figure.
- `MAT_Edge::Dump` prints nothing. `MAT_Bisector::Dump` dereferences its edges: a null edge
  is a native crash, Standard_NullObject in the port.
- `MAT_Zone` built on a basic element whose frontier reaches an infinite node turns from
  its start arc; a null start arc (an end element of an open line) is a native crash,
  Standard_NullObject in the port. The probe does not build these zones.
- The open algorithm (`CreateMatOpen`) with joins by arc on a figure that mixes a closed
  and an open line makes `MAT2d_Tool2d` read a point item of the circuit as a curve: a
  native crash, Standard_NullObject in the port. The corpus leaves that request out.
- `MAT_TList::Init(item)` runs past the end when the item is absent; on a looped list it
  loops forever, as natively.

Deviations: the native `void*` back-pointers (the linked arc of a node, the start and end
arcs of a basic element, the neighbour arcs and the nodes of an arc) are `WeakHandle`s
(docs/topalgo-design.md section 3); a destroyed target reads as a null handle where native
code reads a dangling pointer. The lists and `MAT2d_Mat2d` unlink their cyclic strong
handles on drop. Maps that the algorithms only look up (never iterate in an order that
reaches the output) are `HashMap`s; `MAT2d_BiInt::hash_code` is the native `hashBytes`.
Native null dereferences in these packages are Standard_NullObject errors.
