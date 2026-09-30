# TKTopAlgo port status

**In progress; toolkit acceptance is not complete.** `crates/tktopalgo`
(`rustycad-tktopalgo`) is the port of the ModelingAlgorithms toolkit TKTopAlgo. It
depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase, TKBRep and TKGeomAlgo ports,
as the native toolkit does (its EXTERNLIB). Each of the 16 OCCT packages is one module
of the crate, declared up front in `src/lib.rs`; a module whose package is not ported
yet holds only its documentation. Ported so far: BRepClass, BRepTopAdaptor,
IntCurvesFace, Bisector, MAT, MAT2d, BRepBndLib (without AddOBB), BRepClass3d, BRepCheck,
BRepLib (without PointCloudShape), BRepGProp, BRepApprox and BRepIntCurveSurface. The numeric corpus has 19056 requests: BRepClass 4779,
BRepTopAdaptor 650, IntCurvesFace 424, Bisector 455, MAT and MAT2d 84, BRepBndLib 266,
BRepClass3d 996, BRepCheck with BRepLib_ValidateEdge 981, the BRep domains of IntStart
and IntPatch 752, BRepLib 2688, BRepGProp 4492, BRepApprox 2148, BRepIntCurveSurface 314, and the MakeVertex
stub pipeline 27.

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

The toolkit test inventory maps these suites (1 declaration):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| BRepLib_MakeWire_Test.cxx | brep_lib_make_wire | 1 | 1 of 1 | 1 of 1 |

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

BRepCheck_Face_Test.cxx (4 declarations) is not mapped: every case builds its planar
face and its wires with BRepBuilderAPI_MakeEdge, MakeWire and MakeFace, which the port
does not provide yet. The unchanged suite compiles and passes natively against the
shared build (4 of 4). Its four `ClassifyWires` cases (with the
`BRepCheck_Analyzer::IsValid` assertion of the first one) run as the `face_test_*`
behavior tests of
`crates/tktopalgo/tests/brep_check_wire_face.rs` on the same faces built with
BRep_Builder (40 circular holes, an offset-circle pcurve, an open and an unbounded wire
outside the face, on the face and its reverse); `tests/brep_check_package.rs`,
`brep_check_vertex_edge.rs`, `brep_check_shell.rs` and `brep_check_analyzer.rs` test
the other checkers, the analyzer, the package functions and BRepLib_ValidateEdge.
BRepBndLib_Test.cxx (6 declarations) is not mapped: it builds its shapes with
BRepBuilderAPI_MakeEdge and MakeFace and BRepPrimAPI_MakeCylinder, and meshes them with
BRepMesh_IncrementalMesh of TKMesh, which is not in the shared native build (the unchanged
suite does not link there). Its six cases run as the behavior tests of
`crates/tktopalgo/tests/brep_bnd_lib_behavior.rs` on the same shapes built with
BRep_Builder (the edge MakeEdge gives on a curve, the plane face MakeFace gives on an
infinite plane, and the 26-segment 3d polygon of deflection 0.1 that BRepMesh gives the
unit circle, which the reference bounds of the upstream cases pin), with the same values
and tolerances; they move to a 1:1 translation once BRepBuilderAPI and BRepPrimAPI are
ported and TKMesh is in the native build.

BRepClass3d_SolidClassifier_Test.cxx (5 declarations) is not mapped: its solids come from
BRepPrimAPI_MakeBox and BRepPrimAPI_MakeSphere of TKPrim, which the port does not provide.
The unchanged suite compiles and passes natively against the shared build (5 of 5). Its
five cases run as the behavior tests of `crates/tktopalgo/tests/brep_class3d_behavior.rs`
with the same points, tolerance and expected states, on the same box and sphere built
with BRep_Builder by the probe's shape reader (records in
`crates/tktopalgo/tests/solid_records`); they move to a 1:1 translation once BRepPrimAPI
is ported.

BRepGProp_Test.cxx (17 declarations) is not mapped: its shapes come from
BRepBuilderAPI_MakeEdge, MakeWire and MakeFace, BRepPrimAPI_MakeBox, MakeSphere and
MakeCylinder, and one case cuts a box out of a cylinder with BRepAlgoAPI_Cut of TKBO, which
is not in the shared native build (the unchanged suite does not link there). Sixteen cases
run as the behavior tests of `crates/tktopalgo/tests/brep_gprop_behavior.rs` on the same
shapes built with BRep_Builder (the solids by the probe's shape reader, records in
`tests/gprop_records`; the segment, arc, quarter disk and plane rectangle as MakeEdge and
MakeFace give them, in `tests/gprop_builders`), with the same values and tolerances;
`OCC49_CutShapeHasNoSymmetryAxis` needs the boolean cut and has no counterpart. They move
to a 1:1 translation once BRepBuilderAPI, BRepPrimAPI and BRepAlgoAPI are ported and TKBO
is in the native build. `tests/brep_gprop_units.rs` checks closed-form properties and the
native behaviours below.

BRepIntCurveSurface is not ported: BRepIntCurveSurface_Inter.cxx includes BRepBndLib
(level 4, not ported), which it calls to box each face.

BRepApprox and BRepIntCurveSurface have no upstream suite.
`tests/brep_approx_behavior.rs` and `tests/brep_int_curve_surface_behavior.rs` hold
their behavior tests on faces built by the probe's face reader (the surface tool, the
approximation line, a line of points approximated alone and on a plane and a cylinder;
the exploration of lines and a circle across the holed square and the cylinder, the
accessors without a current point and past the last face).

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
  `BRepTopAdaptorTopolTool` and `BRepTopAdaptor_HVertex` is `BRepTopAdaptorHVertex`
  (docs/topalgo-design.md sections 4 and 8). The vertex implements the TKG3d trait
  `Adaptor3dHVertex`, so its overrides answer through `Handle<dyn Adaptor3dHVertex>`;
  the tool keeps a default Adaptor3d_TopolTool as its parent field (the RTTI chain) and
  has the members its header declares plus the inherited `Identical`, `SamplePnts`,
  `BSplSamplePnts`, `UParameters`, `VParameters` and `IsUniformSampling`, and it
  implements the IntStart trait `TheTopolTool`, so IntStart and IntPatch take a BRep
  face as their domain; the overloads are `initialize_surface`,
  `initialize_curve`, `curve_orientation`, `vertex_orientation`, `curve_tol_3d` and
  `vertex_tol_3d`, as the Adaptor3d_TopolTool port names them. `Edge()` returns the
  edge (a `void*` to it natively). `BRepTopAdaptor_Tool` is `BRepTopAdaptorTool`, its
  `Init` overloads `init_face` and `init_surface`.
- IntCurvesFace (section 4): `IntCurvesFace_Intersector` is `IntCurvesFaceIntersector`
  (transient; `perform_line` and `perform_curve` take `&mut self`),
  `IntCurvesFace_ShapeIntersector` is `IntCurvesFaceShapeIntersector` (its intersectors
  in a `Vec`). The i-th point accessors return Standard_OutOfRange outside
  `[1, NbPnt]`, as the native sequences raise.
- BRepCheck (docs/topalgo-design.md section 6): `BRepCheck` is the unit struct
  `BRepCheck` (associated functions; `Print` writes to `std::io::Write`,
  `SelfIntersection` writes its edges through `&mut Edge`), `BRepCheck_Status` the enum
  `BRepCheckStatus` (native order, `from_index`, `name`) and its list `ListOfStatus`.
  `BRepCheck_Result` is the trait `BRepCheckResult` over the embedded `ResultBase`;
  `BRepCheck_Vertex`, `_Edge`, `_Wire`, `_Face`, `_Shell` and `_Solid` are
  `BRepCheckVertex`, `BRepCheckEdge`, `BRepCheckWire`, `BRepCheckFace`,
  `BRepCheckShell` and `BRepCheckSolid`, transient structs whose members take `&self`.
  `BRepCheck_Analyzer` is `BRepCheckAnalyzer` (`IsValid(S)` is `is_valid_shape`,
  `Result` returns the checker handle, null for a compound). The overloads are `status_on_shape` (the context
  iterator) and `status_on_shape_of(S)`, `geometric_controls` and
  `set_geometric_controls`; `SelfIntersect` writes `E1`/`E2` through `&mut Edge` (left
  unchanged when it does not set them, as the native references). Every member that can
  raise returns `Result`. `BRepLib_ValidateEdge` is `brep_lib::BRepLibValidateEdge`
  (its setters take `&mut self`; `UpdateTolerance` takes `&mut f64`).
- BRepLib (docs/topalgo-design.md section 7): `BRepLib_Command` is `BRepLibCommand`,
  `BRepLib_MakeShape` is `BRepLibMakeShape` (it dereferences to the command), and every
  maker `BRepLib_MakeX` is `BRepLibMakeX` dereferencing to `BRepLibMakeShape`;
  `BRepLibMakeSolid::face_status` shadows the base member as the native override does.
  The constructors and `Init` overloads take distinct names: `from_vertices`,
  `from_points`, `from_<gp curve>[_params|_points|_vertices]` (`lin`, `circ`, `elips`,
  `hypr`, `parab` and their `2d` forms), `from_curve[...]` and `from_pcurve[...]` (a
  Geom2d curve on a surface) with the tails `_params`, `_points`, `_vertices`,
  `_points_params` and `_vertices_params`, and `init_*` for the Init members;
  `BRepLibMakePolygon::from_{two,three,four}_{points,vertices}`, `add_point`,
  `add_vertex`; `BRepLibMakeWire::from_{edge,two_edges,three_edges,four_edges,wire,
  wire_edge}`, `add_edge`, `add_wire`, `add_list` (the list Add of BRepLib_MakeWire_1); `BRepLibMakeSolid::from_{comp_solid,shell,
  two_shells,three_shells,solid,solid_shell}`. The conversion operators map to the
  accessors (`edge`, `wire`, `vertex`, `solid`, `shape`), which return the typed shape by
  value. The status enumerations are `BRepLibEdgeError`, `BRepLibWireError`,
  `BRepLibFaceError`, `BRepLibShellError` and `BRepLibShapeModification`. The package
  functions of `BRepLib` are free functions of `brep_lib`; `Precision` and `Plane` are
  `precision`/`set_precision` and `plane`/`set_plane`, process-wide as natively. The
  re-shaper overload of `UpdateTolerances` is `update_tolerances_with_reshape`; the
  `SameParameter` overloads are `same_parameter_shape`, `same_parameter_shape_with_reshape`,
  `same_parameter_edge` and `same_parameter_edge_with_tolerance` (its `theNewTol` is
  `&mut f64`), and `ExtendFace` returns the extended face (`extend_face(F, ext, [UMin,
  UMax, VMin, VMax])`). `BRepLibMakeFace` has `from_{plane,cylinder,cone,sphere,torus}`
  with the tails `_bounds` and `_wire`, `from_surface[_bounds|_wire]`, `from_wire`,
  `from_face[_wire]`, `init_face`, `init_surface`, `init_surface_bounds`, `add`, `face`
  and `is_degenerated` (the flag and the actual tolerance); `BRepLibFindSurface` has
  `new` and `init` and the native accessors; `BRepLibFuseEdges` has `new`,
  `avoid_edges`, `set_concat_bspl`, `perform`, `nb_vertices`, `shape`, and `edges`,
  `result_edges` and `faces`, which return the maps the native members fill.

- BRepBndLib (docs/topalgo-design.md section 5): the static members of BRepBndLib.cxx
  are associated functions of the unit struct `BRepBndLib` (`add`, `add_close`,
  `add_optimal`); they extend the caller's `BoundingBox3` in place and return `Result`.
  `AddOBB` (BRepBndLib_1.cxx) needs BRepGProp and BRepBuilderAPI_Transform (stage B) and
  has no row yet: the audit lists it as the one missing member of the package.
  `Bnd_Box` implements the TKernel `BoundingVolume` trait (in TKMath), so
  `NCollection_UBTree<int, Bnd_Box>` is `UBTree<i32, BoundingBox3>`.

- BRepClass3d (docs/topalgo-design.md section 5): `BRepClass3d_SolidExplorer` is
  `BRepClass3dSolidExplorer` (its intersector map is `BRepClass3dMapOfInter`, the
  UB-tree `UBTree<i32, BoundingBox3>`), `BRepClass3d_SClassifier` is
  `BRepClass3dSClassifier`, `BRepClass3d_SolidClassifier` is
  `BRepClass3dSolidClassifier` (the base members through `Deref`),
  `BRepClass3d_Intersector3d` is `BRepClass3dIntersector3d`, the two selectors of
  BRepClass3d_BndBoxTree.hxx are `BRepClass3dBndBoxTreeSelectorPoint` and
  `BRepClass3dBndBoxTreeSelectorLine` (with `EdgeParam` and `VertParam`), and
  `BRepClass3d::OuterShell` is `BRepClass3d::outer_shell`. The in/out references of
  `FindAPointInTheFace`, `PointInTheFace`, `Segment` and `OtherSegment` stay `&mut`
  arguments (`FacePointSample` holds the point, its parameters and derivatives); the
  overloads are `find_a_point_in_the_face`, `_uv_param`, `_d1`, `_uv`, `_point`,
  `_parameters` and `point_in_the_face`, `_on_surface`, `_on_surface_d1`.
  `Intersector(F)` is `change_intersector` (`&mut`) with the shared view `intersector`;
  the protected `ClassifyUVPoint`, `ForceIn` and `ForceOut` are public. The audit reports
  0 missing and 0 stale rows (the destructors are excluded rows).

- BRepGProp (docs/topalgo-design.md section 9): `BRepGProp` is the unit struct
  `BRepGProp` (`linear_properties`, `surface_properties`, `surface_properties_eps`,
  `volume_properties`, `volume_properties_eps`, `volume_properties_gk` and
  `volume_properties_gk_plane`, whose four flags are `VolumeGKFlags`). The frameworks
  `BRepGProp_Cinert`, `_Sinert`, `_Vinert`, `_VinertGK`, `_MeshCinert` and `_MeshProps` are
  `BRepGPropCinert`, `BRepGPropSinert`, `BRepGPropVinert`, `BRepGPropVinertGK`,
  `BRepGPropMeshCinert` and `BRepGPropMeshProps` (with `MeshObjType`); they embed the
  TKGeomBase `GProps`, whose protected members are public fields, and reach it through
  `Deref`. The constructors are `with_face`, `with_domain`, `_eps`, `_origin` (`_point`
  for VinertGK) and `_plane` variants, the Performs `perform`, `perform_domain` and the same
  suffixes; the VinertGK tolerance and flags are `VinertGKOptions`. `BRepGProp_Face` is
  `BRepGPropFace` (`load_face`, `load_edge`, `load_iso`; the knot members fill an
  `Array1`, `GetUKnots`/`GetTKnots` return one), `BRepGProp_Domain` is `BRepGPropDomain`,
  `BRepGProp_EdgeTool` is `BRepGPropEdgeTool`, `BRepGProp_Gauss` is `BRepGPropGauss`
  (`compute_natural`, `compute_natural_surface`, `compute_domain`,
  `compute_domain_surface`, `compute_adaptive`, `compute_adaptive_surface` over a
  `GaussProps` output), and `BRepGProp_UFunction` and `_TFunction` are
  `BRepGPropUFunction` and `BRepGPropTFunction` (TKMath `math::Function`). The audit
  reports 0 missing and 0 stale rows (`GetAbsolutError`, declared but defined nowhere
  natively, is an excluded row).

- BRepApprox (docs/topalgo-design.md section 12): `BRepApprox_SurfaceTool` is the unit
  struct `BRepApproxSurfaceTool` (associated functions over `Handle<BRepAdaptorSurface>`,
  the alias `BRepApproxSurface`; `NbSamplesU(S, u1, u2)` is `nb_samples_u_range`), which
  implements the IntImp `ThePSurfaceTool` and the ApproxInt `ApproxIntSurfaceTool`
  traits; `BRepApprox_ApproxLine` is `BRepApproxApproxLine` (`from_curves`, `from_line`,
  `nb_pnts`, `point`), which implements `ApproxIntLine`. Every other class is an alias of
  the generic GeomInt instantiates too: `BRepApprox_Approx` is
  `ApproxIntApprox<BRepApproxApproxTypes>` (`perform`, `perform_on_surfaces`, and the
  private quadric overload `perform_quadric`), `BRepApprox_TheMultiLineOfApprox` is
  `ApproxIntMultiLine<BRepApproxApproxLine>` (its `LineTool` implementation is
  `BRepApprox_TheMultiLineToolOfApprox`), the SvSurfaces, Int2S, ZerParFunc and
  ZerImpFunc instances are the ApproxInt and IntImp generics over
  `BRepApproxSurfaceTool`, and the compute lines, gradients, least squares, constraint
  solvers and functions are the TKGeomBase generics. The api rows of the instance
  classes name the methods of those generics. `BRepApprox_Approx::Perform()` and the two
  `ResConstraint...::Error()` are excluded rows: declared, never defined natively. The
  audit reports 0 missing and 0 stale rows.
- BRepIntCurveSurface (section 12): `BRepIntCurveSurface_Inter` is
  `BRepIntCurveSurfaceInter`; `Init(S, C, Tol)` is `init_curve`, `Init(S, L, Tol)` is
  `init_line`, `Init(C)` is `init`; `load`, `next`, `point`, `pnt`, `u`, `v`, `w`, `state`,
  `transition` and `face` return `Result` (the native members can raise), `more` does
  not. The audit reports 0 missing and 0 stale rows.

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

BRepTopAdaptor domains of IntStart and IntPatch (docs/topalgo-design.md section 8): 752
requests (`tools/tktopalgo_corpus_breptopadaptor_domain.py`, probe
`tools/oracle/tktopalgo_probe_breptopadaptor_domain.cxx`) on faces built with
BRep_Builder: squares, the holed square, the trimmed plane, the disc, the cylinder,
sphere, cone and torus-patch faces, the Bezier and B-spline patches (with a C1 knot and
with a hole) and the toleranced square with 3d lines. `bta_sob` runs
`IntPatch_TheSOnBounds` of a quadric (planes across the faces and through their
vertices and edges, cylinders, a sphere and a cone) on the edges of the face with both
tolerance pairs, with and without the regularity recheck, and prints each path point
with its vertex through `Adaptor3d_HVertex` (value, orientation, parameter,
resolution) and the tool (3d tolerance and point); `bta_inside` runs
`IntPatch_TheSearchInside` of the same quadrics, sampled and classified by the tool;
`bta_intersection` runs `IntPatch_Intersection::Perform` on 21 pairs of face domains
(plane-plane through the hole and through the corners, coplanar, plane-cylinder,
plane-sphere, plane-cone, plane-torus, plane-spline, cylinder-trimmed plane,
disc-cylinder, cylinder-cylinder, sphere-cylinder and B-spline-Bezier), in both orders,
with three option sets, and prints every line with the TKGeomAlgo IntPatch printers.

IntCurvesFace: 424 requests (`tools/tktopalgo_corpus_intcurvesface.py`): `icf_line` (one
intersector on groups of lines through grid points, vertices and seeded points of the
analytic faces, vertical and tilted lines over the spline patches, tangent, axial,
in-plane and parallel lines, on infinite, finite, short and late ranges, with and
without the restriction and the boundary tolerances, and the unrestricted
boundary-tolerance lines of the toleranced squares), `icf_curve` (circles, segments and
a Bezier curve on every face) and `icf_shape` (compounds of three to four faces with
Perform, PerformNearest and Perform on a curve adaptor, after one or two Loads).

BRepBndLib: 266 requests (`tools/tktopalgo_corpus_brepbndlib.py`). Each request carries
a shape record (`tools/tktopalgo_corpus_shapes.py`, named shapes in
`tools/tktopalgo_corpus_solids.py`) that both probes build with BRep_Builder
(`tktopalgo_probe_shape.cxx` and `tp_probe/shape.rs`): vertices, edges with 3d curves
(lines, circles, Bezier and B-spline curves) and 3d polygons, faces on planes, cylinders,
spheres, cones, tori and Bezier surfaces with pcurves (seams get the pcurves of both
uses), triangulations and polygons on them, shells with located faces, and a solid or a
compound of shells, faces, edges and vertices, optionally located. `bbl_add` runs Add
(meshes used or not), AddClose and AddOptimal (the four flag pairs) on boxes (with
tolerances, 3d polygons, triangulations, translated and rotated), a box with a face
translated in its shell, hollow and open boxes, cylinders, spheres, a torus, a box with a
spherical cavity, a solid without shells, a meshed face, an edge with only a polygon on
a triangulation, an infinite plane, a face with only a triangulation, spherical patches
bounded by iso edges, Bezier patches (flagged natural or bounded by their boundary
pcurves), lone cylinder, sphere and torus faces, and a compound of free edges of every
kind and free vertices; the first boxes and two faces also go into boxes that already
hold points (with a gap).

BRepClass3d: 996 requests (`tools/tktopalgo_corpus_brepclass3d.py`), on the shape records
of the BRepBndLib corpus (the reader gains B-spline surfaces): boxes (tolerances 1e-7,
1e-3 and 0.3, translated, rotated, with a REVERSED face, 1e-10 thick, with a planar
B-spline top face), cylinders and spheres (tolerances 1e-7 and 0.2), a torus, hollow boxes
(cavity shell listed after or before the outer one, a box, sphere or cylinder cavity), two
cavities without an outer shell, an INTERNAL first shell, an open box, a square sheet, a
half-space bounded by a whole plane, a solid without shells, compounds of box shells and
a shell whose record holds the faces of another box. The families: `bc3_classify`
(SolidClassifier with Load, twice Load, the constructors, PerformInfinitePoint first, no
shape, then Destroy; on points inside, outside, on and within 1e-8 of faces, on edges and
vertices, on a face plane outside the face, seeded points; tolerances 1e-7, 1e-3, 0.5),
`bc3_infinite` (PerformInfinitePoint of both classifiers, twice on one explorer, and
OuterShell), `bc3_sclassifier` (the constructor point after point on one explorer),
`bc3_segment` (Segment and 12 OtherSegment calls: flag, line, length, face index),
`bc3_explorer` (map, shells, faces, intersector boxes, the members past the last shell
and face, a foreign face, Destroy, InitShape again, a default explorer), `bc3_findpoint`
(the six FindAPointInTheFace overloads, parameters 0 to 1), `bc3_pointinface` (the
three PointInTheFace overloads from indices 0 to 2000, on and off the surface),
`bc3_uvpoint` (ClassifyUVPoint), `bc3_tree` (both selectors on points and lines through
vertices, along and across edges, every Accept index including 0 and Extent + 1, result
indices 0 and past the end) and `bc3_inter3d` (Intersector3d on FORWARD and REVERSED
faces).
BRepLib: 2688 requests (1042 for the vertex and edge makers, 96 for the polygon, wire
and solid makers, 736 for the package functions, 678 for MakeFace, FindSurface,
SameParameter and ExtendFace, 136 for FuseEdges). `tools/tktopalgo_corpus_breplib.py`
appends one module per family, with the
record builders of `tools/tktopalgo_corpus_breplib_records.py`; both probes read the
records and print the shapes with `tktopalgo_probe_breplib_common.cxx` and
`tp_probe/brep_lib_common.rs`: vertices built with BRep_Builder, gp curves, Geom curves
(conics, Bezier, B-splines, periodic B-splines, trimmed and offset curves), gp 2d curves,
Geom2d curves and the surfaces of the face reader, and a canonical shape digest (the
sub-shapes in TopExp::MapShapes order with their type, orientation, Closed flag,
location, children, vertex points and tolerances, edge tolerances, flags, 3d curves,
curves on surfaces and vertex parameters, face tolerances and surfaces). The families:
`blm_vertex`, `blm_edge` (every MakeEdge constructor and Init on segments, the gp
curves, the Geom curves and curves on planes, cylinders, spheres, cones, tori, a Bezier
and a B-spline patch and a trimmed plane, each BRepLib_EdgeError value, infinite
parameters, later Init calls on a done or failed maker, the default and a changed
BRepLib::Precision), `blm_edge2d` (MakeEdge2d on the default and a tilted
BRepLib::Plane), `blm_polygon` (every constructor, Add, Close, closing on a
compared-equal vertex, refused segments), `blm_wire` (every MakeWire constructor and
Add: connected edges in every order and orientation, edges merged through vertex
tolerances, branches, disconnected, closed and degenerated edges, null and empty wires,
and the list Add on empty and started wires with coincident, chained and disconnected
vertices)
and `blm_solid` (every MakeSolid constructor, Add and FaceStatus, with a compsolid
sharing a face). The package functions (`tools/tktopalgo_corpus_breplib_package.py`,
`tktopalgo_probe_breplib_shapes.cxx` and `tp_probe/brep_lib_shapes.rs`) run on shape
records: faces of the face readers, BRepLib_MakeShell shells of every surface kind (a
B-spline surface of several C2 patches, segmented or not), compounds,
BRepLib_MakeEdge edges with enlarged vertices and BRep_Builder solids of the shells
(closed and open, as built and reversed, nested and empty); a grid mesher gives the faces
triangulations (with or without UV nodes) and their edges polygons (two for a seam).
The families: `blp_build3d`, `blp_samerange`, `blp_edgetol`, `blp_regularity`,
`blp_continuity`, `blp_inner`, `blp_sort`, `blp_bounding`, `blp_validrange` and
`blp_validrange2`, `blp_pcurveplane`, `blp_ccos` (BRepLib_CheckCurveOnSurface, one
checker per edge and one reused), `blp_shell` (every MakeShell constructor and Init),
`blp_normals`, `blp_deflection` and `blp_trinormals` (BRepLib_ToolTriangulatedShape),
`blp_tolerances` (both UpdateTolerances, with and without the face verification; the
re-shaper form prints the input and the re-shaper `Apply` of it) and `blp_orient`
(OrientClosedSolid on each solid). `tools/tktopalgo_corpus_breplib_face.py`,
`tktopalgo_probe_breplib_face.cxx` and `tp_probe/brep_lib_face.rs` add surface records
for offset surfaces, linear extrusions and revolutions, and shape records for polygon
wires, the wires and faces of bounded MakeFace faces, edges whose p-curve runs at
another speed than the 3d curve (Bezier, C0 B-spline, shifted ranges) and shapes whose
3d curves BuildCurves3d approximates with their flags cleared. The families:
`blm_face` (every MakeFace constructor and Init on every surface kind, with infinite,
swapped, periodic, out-of-range and degenerated bounds, polygon and face wires, planar
and non-planar wires with and without OnlyPlane), `blm_facedegen`, `blp_findsurface`
(tolerances, OnlyPlane and OnlyClosed on wires, faces, edges and shells),
`blp_sameparam` (the four overloads, forced or not) and `blp_extendface` (every
direction on analytic, periodic, spline, trimmed and offset faces).
`tools/tktopalgo_corpus_breplib_fuse.py` (`tktopalgo_probe_breplib_fuse.cxx`,
`tp_probe/brep_lib_fuse.rs`) runs `blp_fuseedges` on wires of pieces of one line,
circle, ellipse, open or closed B-spline, Bezier or trimmed curve, on polygons, on the
planar faces of those wires, on a face with a split side and on multi-patch shells,
with and without B-spline concatenation, with avoided edges, and in the three call
orders.

BRepApprox: 2148 requests (`tools/tktopalgo_corpus_brepapprox.py`, probes
`tktopalgo_probe_brepapprox.cxx`, `_parts.cxx` and `_tool.cxx` with the twins
`tp_probe/brep_approx.rs`, `brep_approx_parts.rs` and `brep_approx_tool.rs`). The
walking lines are those of `IntPatch_Intersection` on two faces read by `tp_face` with
their BRepTopAdaptor domains (17 of the face pairs of the domain corpus, in both
orders: plane-plane, plane-cylinder, plane-sphere, plane-cone, plane-torus,
plane-spline, disc-cylinder, cylinder-cylinder, sphere-cylinder and B-spline-Bezier,
with the parametric walk `geomint` 0 for the quadric pairs and both walks for the
spline pairs). `bra_approx` runs `BRepApprox_Approx` on them with six SetParameters
sets (the defaults, tolerances 1e-7 to 1e-3, degrees 1 to 8, iterations 0 to 5,
NbPntMax 4 to 30, with and without tangency, the three parametrizations), the six
choices of approximated spaces, on the faces (the ImpPrm and PrmPrm SvSurfaces) and on
the line alone, on all the points, on the default range and on sub-ranges of 3 to 6
points, on restricted and unrestricted adaptors and for a line index past the last;
`bra_line` reads approximation lines of 0 to 40 points and of the poles of three
B-spline curves (some null) at indices 0 to n+1 and approximates them alone; `bra_mline`
and `bra_sv` drive the multi-line (values, tangents, curvature, MakeMLBetween,
MakeMLOneMorePoint, Dump, `Parameters` of the three kinds) and the three SvSurfaces
kinds (Compute, Pnt, SeekPoint, the tangencies, SetUseSolver,
FillInitialVectorOfSolution); `bra_zerimp` and `bra_int2s` the ZerImpFunc and Int2S
instances on the faces; `bra_tool` every `BRepApprox_SurfaceTool` member on every face
of the adaptor corpus, restricted or not.

BRepIntCurveSurface: 314 requests (`tools/tktopalgo_corpus_brepintcurvesurface.py`,
probe `tktopalgo_probe_brepintcurvesurface.cxx`, twin
`tp_probe/brep_int_curve_surface.rs`) on the shape records of `tp_shape`: boxes
(tolerances 1e-7 and 1e-3, translated, rotated, with a flipped face, with a B-spline
top face), cylinders, spheres, a torus, hollow boxes, a box with a spherical cavity, an
open box, a square sheet, a half-space, two boxes and an empty solid. `bics_line` runs
`Init(shape, line, tol)` with the three tolerances 1e-7, 1e-3 and 0.5 for lines through
the centre, along the axes, through edges and vertices, tangent to the curved faces, in
the plane of a face, 1e-8 and 1e-4 outside a face and missing the shape; `bics_curve`
loads the shape once or twice and runs `Init(curve)` (or `Init(shape, curve, tol)`) for
finite, partial and infinite line segments, circles, cubic B-splines and parabolas;
`bics_state` reads a fresh object, `Init` before `Load`, `Load` followed by `Next` with
the previous curve, and `Next` past the last face. Every exploration prints each point
(face index, `Point`, `Pnt`, `U`, `V`, `W`, `State`, `Transition`) and the accessors past
the end.

## Preserved native behaviour

Each ported package adds its own subsection.

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
- `IntStart_SearchOnBoundaries` (`TreatLC`) takes the 3d curve of a straight edge that
  has solutions on a cylinder without a null check: a BRep domain whose line edges carry
  only pcurves (the squares, the holed square and the trimmed plane of the corpus)
  crashes natively against a cylinder that crosses them. Those searches and
  intersections are not in the corpus; the toleranced square and a vertical square with
  3d lines run that branch.
- `SamplePnts` on the tool calls its own `ComputeSamplePoints` (at least 10 samples in
  each direction) or the base `BSplSamplePnts`, and stores the counts in the base
  fields that `NbSamplesU` and `SamplePoint` read; its `SamplePoint` override keeps the
  regular grid of `ComputeSamplePoints` and ignores the `SamplePnts` parameters.

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

### BRepBndLib

- `FindExactUVBounds` calls `ElCLib::InPeriod` on the middle of each pcurve of a
  periodic surface but discards the result, so that point is never moved into the
  period before it is compared with the surface bounds; the port does the same (it skips
  the call, which has no other effect).
- `Add` bounds a plane face by the 3d curves of its geometric edges (the plane itself is
  not added, degenerated edges without curve are skipped) and then enlarges the box by
  the face tolerance; a face without surface uses its triangulation even when the
  meshes are not wanted; a triangulation whose box is void falls back to the geometry.
- An edge outside a face uses its 3d polygon, else the first polygon on a triangulation
  that it holds (whatever the face), else its curve; with `useTriangulation` false a
  geometric edge always uses its curve, but a polygon-only edge still uses its polygon.
- `AddClose` ignores the faces and every tolerance; it adds only the curves of the
  geometric edges and the vertices outside an edge.
- `AdjustFaceBox` moves each side of the surface box towards the edge box in the order
  x min, x max, y min, y max, z min, z max, and each test uses the sides moved before it
  (the rectangle of `IsModifySize` spans the current box).

### BRepClass3d

- `BRepClass3d_SolidExplorer::Reject` ignores the point: only a shape without faces
  rejects, and the classifiers then answer IN (the whole space). A classifier that was
  never loaded classifies against the empty explorer, so it answers IN as well.
- `InitShape` builds one intersector per face (restricted, tolerance
  Precision::Confusion()); a face met twice keeps the intersector of its last
  occurrence. Only the edges and vertices of the faces that are neither INTERNAL nor
  EXTERNAL (degenerated edges skipped) enter the tree.
- `OtherSegment` keeps the best steepness, the count of points found (`NbPointsOK`),
  the face point and its derivatives over the faces and the restarts: once 16 points
  were found, every later face tries a single grid index, and a face without a
  projection starts from the point of the face before. The edge parameter it passes to
  `PointInTheFace` is never changed, `encoreuneface` is always true (its branch is dead),
  and a restart (edge parameters 0.512345, 0.4, 0.6, 0.3, 0.7, 0.2, 0.8, 0.1, 0.9, then
  halved) replaces the intersector of a face whose infinite point is IN with an
  unrestricted one (the native map leaks the old one).
- `BRepClass3d_SClassifier::Perform` records the error state for a face whose
  intersector is not done and goes on, so a later crossing replaces it; the line
  selector's `IsCorrect` (an edge along the segment) is never read.
- `PerformInfinitePoint` starts in the ON state and keeps it when no probe (ten random
  points per face, a `math_BullardGenerator` seeded afresh at each call) meets a face
  IN it with an entering or leaving transition; its tolerance is unused.
- `PointInTheFace` writes the derivatives of the given (u, v) once it classifies IN,
  even when the point differs from the given one and the grid search goes on.
- `BRepClass3d_Intersector3d::Transition()` is uninitialized natively until a point is
  kept (the port starts it at Tangent; the corpus reads it only after `HasAPoint`); a
  Perform that keeps no point leaves the earlier results, it is done only when the
  intersection has points, and its `Prm` argument is unused.
- `BRepClass3d::OuterShell` returns the only shell of a solid without checking it, and
  otherwise the first shell (first face not INTERNAL) whose solid classifies the
  infinite point OUT.

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

### BRepCheck and BRepLib_ValidateEdge

- `BRepCheck::Add` removes the `NoError` entries only before a defect: a `NoError`
  added after a defect is appended.
- The statuses of each checker are kept in a DataMap with `TopTools_ShapeMapHasher`, so
  the context iterator visits the contexts in the order of the TShape addresses, which
  differs from run to run on both sides (the corpus sorts them). `InitContextIterator`
  skips the shape's own statuses wherever they are.
- `BRepCheck_Shell::NbConnectedSet` visits the edges of more than two faces in the same
  hash order. With several such edges the sets it finds, and their count, depend on it
  (a set whose first face is the last free one is not appended); `Closed` calls it, so
  the closure of such a shell (a cube with a face met twice) is not reproducible natively
  either. The corpus keeps shells with at most one such edge.
- `BRepCheck_Shell::Closed` never reports `RedundantFace` (its `!mapS.Add(aF)` tests the
  index an indexed map returns, never 0), and `Minimum` compares the face occurrences
  with the distinct faces reached, so a face met twice makes the shell `NotConnected`,
  which also hides the redundant face from `Orientation`. In a solid, `InContext`
  records any closure defect (its `NotClosed && S.Closed()` term is redundant).
  `Orientation` with `Update` records `BadOrientationOfSubshape` at the first bad edge,
  then checks whether reversing faces orients the shell (`UnorientableShape`); without
  `Update` it returns the bad orientation at once.
- `BRepCheck_Vertex` compares the point of the TVertex (without the vertex location)
  with the representations moved by the edge or face location divided by the vertex
  location. An end vertex occurring INTERNAL or EXTERNAL is not checked at the ends, one
  occurring FORWARD and REVERSED is checked at both. `Tolerance` keeps the previous
  control point for a point on a null curve and moves it by that representation's
  location again.
- `BRepCheck_Edge::Minimum` moves a curve on surface reference by its representation
  location only (not the edge location), and passes the range of a 3d reference through
  `TransformedParameter` of the already transformed curve. `InContext` in a face stops
  after the flags when SameParameter or SameRange is off, loads the second pcurve of a
  closed surface into the same adaptors, and projects the 3d curve when the face has no
  other pcurve on a plane; an edge whose only pcurve is its reference reports
  `NoCurveOnSurface` on a non-planar face. `Tolerance` moves the second pcurve of a
  closed surface by the representation location only; `CheckPolygonOnTriangulation`
  reads the second polygon of a polygon on a closed triangulation.
- `BRepCheck_Wire::Minimum` turns the geometric controls on. `SelfIntersect` maps the
  wire edges with the default hasher (an edge met with two orientations is intersected
  with itself as two edges), returns `NoCurveOnSurface` or `InvalidRange` for a later
  edge without recording them, and skips the inner points outside the 3d curve range of
  the edges (an edge without 3d curve has the range [0, 0]). `Closed2d` reads the current
  vertex of the finished wire explorer. `ChoixUV` reads the point and parameter of the
  last candidate even when it was skipped, and `IsClosed2dForPeriodicFace` reverses the
  seam at each matching vertex. `Orientation` fills a list of already passed edges that
  it never reads.
- `BRepCheck_Face::IntersectWires` never clears its wire map, so a computation after
  `GeometricControls(false)` then `(true)` finds every wire again (`RedundantWire`); a
  wire without a recorded box keeps the previous wire's box. `ClassifyWires` answers
  `NoError` when no wire holds another, and `OrientationOfWires` then reports
  `InvalidImbricationOfWires` for a finite face; a thin exterior wire (two edges between
  the same vertices) returns `NoError` without recording it. `IsInside` reads the start
  of the 3d curve at the first parameter of the pcurve.
- `BRepCheck_Solid` initializes its shell tools in order and stops at the second shell
  that is not a hole (`EnclosedRegion`); the pair scan that follows uses the tools after
  it without classifier, a null dereference natively when four or more closed shells
  get there (the corpus has three at most; the port raises Standard_NullObject).
- `BRepCheck_Analyzer::IsValid` reads only the first status of each checker, and
  `ValidSub` stops at the first sub-shape that has no statuses in the context (the rest
  are not read). The analyzer never checks an edge in a solid (only faces, wires and
  vertices in faces, vertices in edges and shells in solids), so free edges are found by
  the shell check. Its map keeps orientations apart: a face met reversed gets its own
  checkers.
- `BRepLib_ValidateEdge` counts fewer than one control point as one, compares the ends
  first when the parametrizations differ, corrects the exit tolerance by the precisions
  of `BRepCheck`, and uses the exact method only for a same-parameter edge.

### BRepGProp

- `BRepGProp_Domain` skips the INTERNAL and EXTERNAL edges in `Next` only: its
  constructor and `Init` return the first edge of the face whatever its orientation.
  `Value()` past the last edge casts the face itself (Standard_TypeMismatch).
- `BRepGProp_Face::VKnots` of a plane, cylinder, cone or any surface other than a
  sphere, torus or B-spline writes the *U* bounds of the face, and the natural adaptive
  integration searches them as V knots (a binary search on a possibly unsorted array).
  `GetFace()` returns the face FORWARD. `GetUKnots` loads the basis curve of an extrusion
  whatever the span flag; `GetTKnots` reads the V knots of a B-spline surface without
  using them and splits only a B-spline pcurve. `Load(F)` of a face without surface keeps
  the previous surface (`BRepAdaptor_Surface::Initialize`). `SIntOrder(0)` takes the
  logarithm of 0 and gives the largest order.
- `BRepGProp_Gauss` converts a by-plane volume with the by-point test: the mass and the
  centre of `BRepGProp_Vinert` by a plane are always 0 (its inertia is kept). The fixed
  natural Compute multiplies the mass by the half ranges after the centre is computed
  from the unscaled sums. The fixed domain Compute clamps the boundary points to the U
  and V bounds for a volume (OCC104), not for a surface.
- The adaptive Compute takes the U error estimate once, when the knot intervals are
  done; `EpsU` then stays 0, so the U intervals are never bisected, only the L
  intervals. A tolerance in `[0.001, 1)` runs one pass and returns it as the error.
- An edge without pcurve ends the fixed Compute and leaves its outputs unchanged, and
  the adaptive one returns Precision::Infinite() with its outputs unchanged: the Sinert
  or Vinert keeps the previous face's values, and the package functions (which reuse one
  framework) add them again. `Perform(S, Eps)` integrates an empty domain: a face with
  wires gives nothing.
- `BRepGProp_VinertGK` ignores the result of `Load(E)`, so an edge without pcurve
  integrates the previous boundary again; its centre and inertia keep their earlier
  values when not requested (the package reuses one framework for all faces), and below
  `Epsilon(AbsolutError)` the centre flag zeroes the mass and the centre.
  `SetNbKronrodPoints` stores a count nothing reads and `GetStateNumber` returns 0.
- `roughBaryCenter` averages the vertex points of the explorer (a shared vertex counts
  once per occurrence), else takes the first triangulation node.
- `volumeProperties` computes a shared solid once: an instance with the same location
  and orientation is skipped only with SkipShared, a mirrored or scaled relative location
  is integrated face by face, and the other orientation negates the mass and inertia (not
  the centre).
- `BRepGProp_MeshProps` keeps the static moments as the centre when the mass is below
  1e-20, copies the nodes for a mirroring or scaling location and moves the computation
  point for a rigid one. `BRepGProp_MeshCinert::Perform` reads the nodes from index 1
  (Standard_OutOfRange for another lower bound), skips the segments shorter than
  gp::Resolution() and takes the last Gauss point as the centre of a null length, as
  `BRepGProp_Cinert` takes its last evaluated point.
- `VolumePropertiesGK` adds the absolute errors of the faces and divides the sum by the
  volume only above `Epsilon(1)`; with OnlyClosed and no shell that BRepCheck_Shell finds
  closed it returns -1.

### BRepApprox and BRepIntCurveSurface

- `BRepApprox_SurfaceTool::NbSamplesU` and `NbSamplesV` (both overloads) answer 10 for
  every surface, where `Adaptor3d_HSurfaceTool` counts by surface kind; every other
  member calls the adaptor member of the same name.
- `BRepApprox_ApproxLine::NbPnts` reads the pole count of the first non-null curve (3d,
  then on the first surface, then on the second) before the points of the line, and
  `Point` reads the line only when it has points: a line built on an empty
  `IntSurf_LineOn2S` has no points and answers the origin with zero parameters at every
  index. The poles of a null curve are the default `gp_Pnt` and `gp_Pnt2d` (the origin);
  the flag `theTang` of the second constructor is unused.
- `BRepIntCurveSurface_Inter::More()` compares the face index with the face count, so a
  fresh object (index 0, no face) and an object after `Load` report a current face while
  `Point` and the other accessors raise StdFail_NotDone and `Face` raises
  Standard_OutOfRange. Each `Next` past the last face moves the face index one further.
- The points of a face come in the order of `IntCurveSurface_HInter` (not sorted by
  parameter); `FindPoint` keeps the state of the last classified point. A curve of line
  type is tested against the face boxes as the whole line even on a finite range, a
  finite curve against its own box, and an infinite curve of another type is never
  rejected. The face boxes (BRepBndLib::Add with the triangulation, enlarged by the
  tolerance) are built on first use and kept by every later `Init(curve)` until the
  next `Load`.

### BRepLib

- A failing `Init` of `BRepLib_MakeEdge` or `BRepLib_MakeEdge2d` only records its
  error: the edge, the vertices and the done flag of an earlier successful Init stay.
- `BRepLib_MakeEdge` on a 3d curve refuses a range shorter than gp::Resolution()
  (`BRepLib_LineThroughIdenticPoints`); on a curve on surface it has no such check, and
  swapped non-periodic parameters make the edge REVERSED. A closed 3d curve whose two
  given vertices are the same one is degenerated when its middle point is within
  `BRepLib::Precision()` of its start. The parameters are checked against the curve
  range with Precision::PConfusion(), so a line accepts `±Precision::Infinite()` and
  refuses anything beyond (`BRepLib_ParameterOutOfRange`).
- The vertex projection of `BRepLib_MakeEdge` takes a curve end first, when it is
  strictly the nearer one and within the vertex tolerance; on a curve on surface an
  Extrema point is accepted within Precision::Confusion() only, whatever the vertex
  tolerance.
- `BRepLib_MakeEdge2d` works at Precision::Confusion() (not `BRepLib::Precision()`),
  checks a vertex against its end only on a closed curve, and never sets its error on
  success: `Error()` is uninitialized natively until a failure (the probe prints it only
  for a maker that is not done). Its projection uses the closed form on lines and
  circles, which raises Standard_ConstructionError for a vertex at the centre of a
  circle; on other curves an Extrema_ExtPC2d that is done without extremum leaves the
  parameter uninitialized natively (the port reads 0.0; such a request, a vertex beyond
  the end of a B-spline, is not in the corpus).
- `BRepLib_MakePolygon::Add` marks the wire Closed as soon as the new vertex compares
  equal to the first one, before it builds the closing segment; the flag stays when
  BRepLib_MakeEdge refuses that segment. `Close()` does nothing on a wire flagged Closed.
- `BRepLib_MakeWire::Error()` is uninitialized natively until the first Add (a wire
  constructor on a null or empty wire adds nothing); the port starts at
  `BRepLib_EmptyWire`, the default constructor's value. `BRepLib_NonManifoldWire` is an
  internal state: every successful Add ends with `BRepLib_WireDone`, and a
  non-manifold wire forgets its free ends, so later edges keep their input orientation.
  A copied edge (connected through a vertex tolerance) replaces its vertex by the wire
  vertex, moved to the point weighted by the two tolerances and enlarged to cover both.
- The list `BRepLib_MakeWire::Add(L)` makes the maker done on an empty list (0 of 0
  edges added). The box of its vertex selector is never reset: it grows with every
  vertex searched, so a later search accepts any vertex whose tolerance sphere touches
  the current one inside the union of the earlier boxes (the sphere test still decides).
  The groups start from the first vertex of a hashed map, so the group order, and with it
  the order of new vertices, can differ from run to run natively; the merged points and
  tolerances do not depend on it. The first edge of an empty wire goes through the
  proximity check of `Add(E)`, the other edges connect without it.
- `BRepLib_MakeSolid` built from a compsolid gathers the faces in the iteration order of
  a hashed map (TShape addresses), so the face order of its shell differs from run to
  run natively too (the probe prints an order-free digest); a face met three times is
  deleted, then added again.
- `BRepLib_MakeShell` asks the adaptor whether the surface is periodic, not whether the
  range covers the period: on a cylinder, cone, sphere or torus restricted to part of
  its period the right edge of the last patch is still the left edge of the first one,
  so the face gets a seam between two distant isolines. On a V-periodic surface of one
  interval the seam range is set from the V parameters. Its final degenerated check
  adapts the 3d curve of each edge, so an edge whose 3d curve could not be built raises
  Standard_NullObject.
- `BRepLib::UpdateEdgeTol` looks for its reference curve on surface without advancing
  its iterator: it reads the first representation again and again (an edge without 3d
  curve whose first representation is not a curve on surface loops without end
  natively), and then skips the second representation instead of the reference, which
  is compared with itself. On a seam only the second p-curve is measured, and the edge
  tolerance is set, also downwards. On a SameParameter edge without 3d curve the
  parametric distance reads an unloaded 3d adaptor (a crash natively, Standard_NullObject
  in the port); such edges are not in the corpus.
- `BRepLib::BuildCurve3d` stores the image of a p-curve on a plane with the range of the
  edge looked up on the plane at the identity location (not the location of the curve
  on surface), and an edge without any curve reads an uninitialized first range
  natively (Standard_NullObject in both, from the 2d adaptor of the null curve).
- `BRepLib::SameRange` passes one output handle from one representation to the next, so
  a GeomLib::SameRange that assigns nothing reuses the previous curve.
- `BRepLib::EncodeRegularity` catches every exception of one shape (the edges before
  it keep their regularity) and of each `ContinuityOfFaces`.
- `BRepLib::EnsureNormalConsistency` reads the polygon of every edge shared by two
  faces on both triangulations; a seam (its face twice) without a seam polygon is a null
  dereference natively (Standard_NullObject in the port). The probe mesher gives seams
  their two polygons.
- `BRepLib::FindValidRange` may write `theFirst`, or both outputs, before it returns
  false; the port returns them in `ValidRange`. `BRepLib::BoundingVertex` leaves its
  outputs unchanged for fewer than two vertices (None in the port).
- `BRepLib_CheckCurveOnSurface::Init` keeps the curves on surface of an earlier Init
  when the new edge is degenerated or not geometric, and the second curve (of a seam)
  of an earlier Init when the new edge is not a seam.
- `BRepLib::ReverseSortFaces` classifies a rectangular trimmed surface through its own
  adaptor, where `SortFaces` takes its basis first.
- `BRepLib::UpdateTolerances` with the verification measures a face by the X extent of
  its box, but for Y and Z it overwrites the lower bound with the extent only when both
  sides are closed: a box open in Y or Z compares its raw lower bound instead. The box
  includes its gap, so edge tolerances enlarge the face tolerance. A vertex tolerance is
  set (not only enlarged) and gains twice the epsilon of the value; with the
  verification a vertex takes its computed tolerance even when smaller, once per vertex
  TShape. The updates are applied in hashed-map order; each shape is updated on its
  own, so only the order of the re-shaper records depends on it.
- `BRepLib::OrientClosedSolid` classifies the infinite point: a solid without shells is
  IN, so it is reversed and the call returns true.
- `BRepLib_MakeFace::Init(F)` records `BRepLib_FaceDone` without marking the maker done
  (so do the constructor from a face and `Init(S, false, TolDegen)`); `Add` marks it.
  The bounded Init checks swapped bounds against the natural bounds of the basis of a
  trimmed surface (a trimmed plane accepts any bounds), gives a closed U direction the
  seam lines UMax then UMin but a closed V direction VMin then VMax, and makes a face
  on an offset surface SameParameter (after trimming an offset extrusion or revolution
  in its infinite direction). The wire constructor sets the face tolerance to the
  larger of 1.2 times the reached and the searched tolerance.
- `BRepLib_FindSurface` keeps the surface and location of the last curve on surface
  searched when none is shared; `Is2DClosed` answers false on any failure (a missing
  p-curve included). The plane fit orients the plane of a closed wire so that the
  infinite point is outside.
- `BRepLib::SameParameter` on a C0 B-spline p-curve works on the curve handle of the
  adaptor: when the C1 conversion leaves it unchanged, the reparametrization and the
  origin change of a periodic curve modify the p-curve stored in the edge itself.
  `ComputeTol` returns the distance of a sample outside the surface domain when larger
  than the others, and 1.5 times the largest sample distance below 1 when fewer than
  10% of the samples are at least 1. On a planar face the edges without a curve on
  the plane take 1.05 times the deviation of their projection.
- `BRepLib::ExtendFace` tests the VMax bound (not VMin) before extending in VMin.
- `BRepLib_FuseEdges::Perform` does not build the chains: called first, it finds none,
  marks the shape done and leaves it unchanged, and later calls keep it. Without
  chains the result edges are never marked built. Two lines are on the same support
  when their directions are parallel (not when they are the same line); two
  B-splines when their poles are within the tolerance of the first edge and their
  knots do not exceed the other's by more than Precision::Confusion() (a signed
  difference). A closed fused B-spline is made periodic in place: when the first
  edge's curve has no location, the curve stored in that edge changes too. The range
  set on the fused edge from a curve on surface also moves its 3d range.

## Deviations

Each ported package lists where the port must differ from native behaviour.

### BRepClass3d
- The selectors' `Accept` cannot raise through the tree traversal: an exception of the
  extrema (or of the map for index 0) is kept by the selector, stops the traversal and
  is returned by `take_failure`, which the explorer and classifier check right after
  `Select` (natively the exception leaves `Select`).
- `FaceNormal` on a face without surface raises Standard_NullObject (a null dereference
  natively).

### BRepClass
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

### BRepTopAdaptor and IntCurvesFace
- The members natively dereferencing a null or wrong-typed handle raise
  Standard_NullObject (HVertex Parameter, Resolution and IsSame on other adaptors,
  InitVertexIterator without an arc, the samples without a surface); `Value()` and
  `Edge()` past the last arc raise Standard_NoSuchObject and `Next()` past it does
  nothing (null dereferences natively). SamplePoint with a zero U count raises
  Standard_ProgramError (a division by zero natively).
- `UParameters` and `VParameters` of the tool before `SamplePnts` raise
  Standard_NullObject (a null dereference natively). `Edge()` of the `TheTopolTool`
  implementation raises the Standard_NoSuchObject of `Edge()` past the last arc.
- The `TheTopolTool` members `InitVertexIterator`, `NextVertex`, `DomainIsInfinite`,
  `Edge` and `Orientation(C)` return `Result` for every domain, because the overrides of
  the tool can raise (docs/topalgo-design.md section 8).

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

BRepCheck and BRepLib_ValidateEdge: 981 requests (`tools/tktopalgo_corpus_brepcheck.py`
and its `_vertex`, `_edge`, `_wire`, `_face`, `_shell`, `_solid` and `_validate` modules). The shapes
are shape programs (`tools/tktopalgo_shape_program.py`): both probes build any shape
with BRep_Builder from the same numbers (`tktopalgo_probe_shapes.cxx` and
`tp_probe/shapes.rs`: surfaces, 3d curves and pcurves, locations, vertices, edges and
their representations, point representations of vertices, polygons on triangulations,
containers, orientations, locations and flags). `bck_check` builds one checker and runs
a script of its members and of the `BRepCheck_Result` members, then prints the statuses
of the shape and of every table shape; a context iteration prints its shapes sorted,
since its order follows the TShape addresses. The families: `bck_status` and `bck_add`
(the package list and printer, out-of-range statuses), `bck_prec` (lines, circles,
plain, trimmed and planar ellipses, cones and other surfaces), vertices at and off the
ends of their edges (closed edges, INTERNAL vertices, located edges, curves and
vertices, point representations on curves, pcurves and surfaces), edges (no, several
and degenerated 3d curves, flags, ranges empty or beyond Bezier curves and circles,
plain and trimmed; pcurves shifted within and beyond the tolerance, re-ranged, missing
on a plane (projected), located faces; edges with pcurves only; cylinder seams; cubes,
open cubes and solids; polygons on triangulations with and without parameters and on a
closed triangulation; Tolerance), wires (squares, a bow tie, a figure eight, a T,
overlapping and crossing edge pairs, holes, open, disconnected, empty, redundant and
misoriented wires, INTERNAL edges, cylinder seams with a 2d gap, a looping Bezier edge,
missing pcurves on a cylinder), faces (holes inside, crossing, outside, nested, touching
the outer wire at a vertex or along an edge, misoriented wires and faces, no surface, no
wire, an infinite face, a wire met twice, thin faces, the four BRepCheck_Face_Test
faces, half-infinite and clamped edges), shells (cubes whole, open and with reversed or
INTERNAL faces, repeated faces, disconnected faces, edges of three and four faces, a
band plain and twisted, an empty shell, solid contexts), solids of one to three shells
(holes, nested and disjoint shells, a shared face, a detached vertex, INTERNAL shells),
`bck_analyze` (the analyzer on every kind of shape, compounds and compsolids of valid
and invalid shapes, with the geometric controls off, in parallel and exact, IsValid of
every sub-shape and every checker's statuses in table order, the first face of
BRepCheck_Face_Test) and `ble_validate` (lines,
circles, ellipses on planes and cones, a curve on surface as reference, a B-spline
against a Bezier pcurve, with the same and another parametrization and every option).

BRepGProp: 4492 requests (`tools/tktopalgo_corpus_brepgprop.py` and
`tktopalgo_corpus_brepgprop_tools.py`, shapes of `tktopalgo_corpus_gprop_shapes.py` and the
earlier shape modules; the shape reader gains ellipse and parabola curves, extrusion,
revolution and offset surfaces, B-spline and ellipse pcurves, mirror and scale locations
and a top-level item repeating an earlier one with its own location and orientation, so
solids share their TShape). The families: `bgp_package` (every package function and flag
on boxes with tolerances, polygons and meshes, hollow, open, flat, located and rotated
boxes, cylinders, spheres, a torus, cone frustums, a ring whose end faces are holed,
cavities listed first or alone, an INTERNAL shell, compounds, a half-space, a solid
without shells, a meshed box with a REVERSED face, shared solids moved, rotated and
reversed, mirrored and scaled, then with a free face, the single faces below and free
edges), `bgp_cinert`, `bgp_mesh_cinert` and `bgp_mesh_nodes` (lines, circles, Bezier,
B-spline, ellipse and parabola edges, an edge with only a B-spline pcurve, 3d polygons,
polygons on triangulations, one naming a node past the mesh, node arrays indexed from 0,
1, 2 and -1), `bgp_mesh_props` and `bgp_mesh_calc` (both types and orientations with no,
identity, translated, rotated, scaled, mirrored and scaled-rotated locations, a mesh
naming a node past its end, a mesh without triangles; right, general, collinear, tiny and
repeated-vertex triangles with one-point, three-point and empty rules), `bgp_sinert`,
`bgp_vinert` and `bgp_vinert_gk` (every constructor and Perform by the location, a point
and a plane, alone and chained on one framework, eps 1e-6, 5e-4, 0.01 and -1e-3), and
`bgp_face`, `bgp_domain`, `bgp_gauss`, `bgp_knots`, `bgp_arrays` and `bgp_functions` on
single faces: a plate with a hole, a plane face bounded by a B-spline edge, ellipse discs
(flat and tilted), a spherical patch, Bezier and B-spline patches (natural and bounded by
their iso pcurves), a B-spline surface with 62 U intervals (the interval arrays grow), a
revolution, an extrusion (bounded, and whole with infinite V bounds), offsets of a Bezier
patch and of a plane, the lateral faces of a cylinder and a frustum, a torus, a sphere, a
box face, the faces of the ring, an infinite plane, a meshed face and a spherical patch
whose edge has no pcurve (the Compute overloads then return early). The Gauss-Kronrod
requests keep loose tolerances when they integrate the inertia, so that each request stays
fast in the debug build of the Rust twin (the slowest takes about 1 s there).

### BRepGProp
- A copy of `BRepGProp_Face` shares the surface adaptor (docs/topalgo-design.md section
  9): natively each copy owns one, so a later `Load(F)` of the original would not reach
  it. No BRepGProp path loads a face while a copy lives.
- The integrand functions keep an exception of the surface and answer false; the
  integrators return it once the integration ends (natively it leaves the integration).
- The face tool on a face without surface dereferences a null adaptor natively (a
  crash); the port raises Standard_NullObject. Those requests are not in the corpus.
- `CalculateProps` with fewer than `3 * NbGaussPoints` rule values raises
  Standard_OutOfRange where the native loop reads past the array (not in the corpus).
- The VinertGK tolerance and flags travel in `VinertGKOptions` and the package GK flags
  in `VolumeGKFlags` (their native defaults are `Default`).

### BRepCheck and BRepLib_ValidateEdge
- The native null dereferences raise Standard_NullObject: `BRepCheck_Edge::InContext`
  projecting a curve on surface reference on a plane (the native `GeomAdaptor_Curve`
  down cast is null), `Tolerance` with a null 3d curve or without an adaptable
  representation, `BRepLib_ValidateEdge` with a null adaptor, `PrecSurface` of a null
  handle. The corpus leaves the native crashes out.
- The analyzer's functor catches the failures of the checks as the native catch blocks
  do; a failure outside them (a missing checker) is returned by `Init` (it leaves the
  native loop). With the parallel flag the tasks run on the default pool of
  `OSD_Parallel`.
- The context iterator is a snapshot of the map taken by `InitContextIterator` (natively
  an iterator over the live map, invalid after a change of the map).
- The native mutex guards the status map only in parallel mode; the port always locks,
  which changes no result. `IsUnorientable` returns `Result` (it reads the statuses).

### BRepApprox and BRepIntCurveSurface
- The BRepApprox templates keep `void*` pointers to the caller's `BRepAdaptor_Surface`;
  the port passes and keeps `Handle<BRepAdaptorSurface>` (the shared object).
  `BRepApprox_ApproxLine::Point` is non-const natively and writes nothing; it takes
  `&self`. `NbPnts` on a line without curves and without points dereferences a null
  handle natively; the port raises Standard_NullObject.
- `BRepIntCurveSurface_Inter` leaves its tolerance uninitialized until `Load` (it is
  never read before); the port starts at 0. `Next` after `Load` of a shape with faces and
  before any `Init(curve)` dereferences the null curve natively; the port raises
  Standard_NullObject, and the corpus leaves that call out. `Init(curve)` keeps a shallow
  copy of the adaptor, where the native copy constructor also shares the evaluation
  cache of a B-spline curve (the values are the same).

### BRepLib
- A null curve, surface or vertex that the native makers dereference raises
  Standard_NullObject (a crash natively).
- The uninitialized native values (`BRepLib_MakeEdge2d::Error()` after a success,
  `BRepLib_MakeWire::Error()` before the first Add, the parameter of a MakeEdge2d
  projection without extremum) read the values given above.
- A null surface, p-curve or projection that `BRepLib::SameParameter`, `GetEdgeTol` or
  `BRepLib_FindSurface` dereferences raises Standard_NullObject (a crash natively);
  inside `Is2DClosed` it answers false, as the native signal handler does. A chain
  edge without 3d curve that `BRepLib_FuseEdges` dereferences raises it too.

