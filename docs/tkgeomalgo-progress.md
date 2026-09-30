# TKGeomAlgo port status

**Every package is ported; three upstream suites still wait for other toolkits.**
`crates/tkgeomalgo`
(`rustycad-tkgeomalgo`) is the port of the ModelingAlgorithms toolkit TKGeomAlgo. It
depends on the TKernel, TKMath, TKG2d, TKG3d, TKGeomBase and TKBRep ports, as the
native toolkit does (its EXTERNLIB). Each of the 33 OCCT packages is one module of the
crate, declared up front in `src/lib.rs`; a module whose package is not ported yet
holds only its documentation. All 33 packages are ported (NLPlate has no sources at the
pinned commit), and each audits with 0 missing and 0 stale api rows. Three suites are not
mapped yet because they build shapes with toolkits that are not ported:
GeomFill_BSplineCurves_Test and GeomFill_CorrectedFrenet_Test (TKTopAlgo, TKOffset,
TKShHealing) and GeomPlate_BuildPlateSurface_Test (BRepBuilderAPI); their cases run as
behaviour tests.

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

The toolkit test inventory maps these suites (198 declarations):

| Source | Rust target | Declarations | Native | Rust |
| --- | --- | ---: | --- | --- |
| TopTrans_SurfaceTransition_Test.cxx | toptrans_surface_transition | 2 | pass | pass |
| Intf_Tool_Test.cxx | intf_tool | 5 | pass | pass |
| GccAna_Circ2d3Tan_Test.cxx | gcc_ana_circ2d_3tan | 18 | pass | pass |
| Geom2dAPI_ExtremaCurveCurve_Test.cxx | geom2dapi_extrema_curve_curve | 1 | pass | pass |
| Geom2dAPI_InterCurveCurve_Test.cxx | geom2dapi_inter_curve_curve | 6 | pass | pass |
| Geom2dAPI_Interpolate_Test.cxx | geom2dapi_interpolate | 1 | pass | pass |
| Geom2dAPI_PointsToBSpline_Test.cxx | geom2dapi_points_to_bspline | 2 | pass | pass |
| Geom2dConvert_BSplineCurveToBezierCurve_Test.cxx | geom2dconvert_bspline_curve_to_bezier_curve | 1 | pass | pass |
| IntSurf_LineOn2S_Test.cxx | int_surf_line_on2s | 3 | pass | pass |
| IntSurf_Quadric_Test.cxx | int_surf_quadric | 1 | pass | pass |
| Plate_Plate_Test.cxx | plate_plate | 2 | pass | pass |
| Geom2dHatch_Elements_Test.cxx | geom2dhatch_elements | 3 | pass | pass |
| Geom2dHatch_Intersector_Test.cxx | geom2dhatch_intersector | 3 | pass | pass |
| Geom2dGcc_Lin2d2Tan_Test.cxx | geom2dgcc_lin2d_2tan | 2 | pass | pass |
| Geom2dGcc_Circ2d2TanRad_Test.cxx | geom2dgcc_circ2d_2tan_rad | 7 | pass | pass |
| Geom2dGcc_Circ2d3Tan_Test.cxx | geom2dgcc_circ2d_3tan | 12 | pass | pass |
| IntCurveSurface_IntersectionPoint_Test.cxx | int_curve_surface_intersection_point | 3 | pass | pass |
| IntCurveSurface_InterUtils_Test.cxx | int_curve_surface_inter_utils | 1 | pass | pass |
| IntCurveSurface_ThePolygonOfHInter_Test.cxx | int_curve_surface_the_polygon_of_h_inter | 1 | pass | pass |
| IntCurveSurface_ThePolyhedronOfHInter_Test.cxx | int_curve_surface_the_polyhedron_of_h_inter | 6 | pass | pass |
| IntPolyh_Intersection_Test.cxx | int_polyh_intersection | 3 | pass | pass |
| IntPolyh_Point_Test.cxx | int_polyh_point | 10 | pass | pass |
| GeomAPI_ProjectPointOnSurf_Test.cxx | geomapi_project_point_on_surf | 2 | pass | pass |
| GeomAPI_PointsToBSpline_Test.cxx | geomapi_points_to_bspline | 1 | pass | pass |
| GeomAPI_IntCS_Test.cxx | geomapi_int_cs | 5 | pass | pass |
| GeomAPI_ExtremaCurveSurface_Test.cxx | geomapi_extrema_curve_surface | 6 | pass | pass |
| GeomAPI_ExtremaSurfaceSurface_Test.cxx | geomapi_extrema_surface_surface | 2 | pass | pass |
| GeomAPI_PointsToBSplineSurface_Test.cxx | geomapi_points_to_bspline_surface | 1 | pass | pass |
| GeomAPI_IntSS_Test.cxx | geomapi_int_ss | 6 | pass | pass |
| GeomFill_Gordon_Test.cxx | geomfill_gordon | 64 | pass | pass |
| GeomFill_NSections_Test.cxx | geom_fill_nsections | 1 | pass | pass |
| GeomFill_GuideTrihedronAC_Test.cxx | geom_fill_guide_trihedron_ac | 4 | pass | pass |
| IntPatch_Polyhedron_Test.cxx | int_patch_polyhedron | 5 | pass | pass |
| IntPatch_PolyhedronBVH_Test.cxx | int_patch_polyhedron_bvh | 8 | pass | pass |

GeomFill_BSplineCurves_Test.cxx is not mapped yet: three of its five declarations
build faces and offsets (BRepBuilderAPI, BRepCheck, BRepGProp, BRepOffset and ShapeFix
of TKTopAlgo, TKOffset and TKShHealing, and GeomAPI::To3d), which neither the port nor
the shared native build (no TKOffset or TKShHealing library) provides. Its two GeomFill
cases (OCC13904 and OCC27704) run as behavior tests (`geom_fill_skin_behavior` and
`geom_fill_filling_behavior`) and in the numeric corpus.

GeomFill_CorrectedFrenet_Test.cxx is not mapped yet: its `ActualReproducerCase` builds a
wire with GC_MakeSegment, BRepBuilderAPI_MakeEdge and ShapeExtend_WireData and adapts
it with BRepAdaptor_CompCurve, which the port does not provide and which needs
TKShHealing, absent from the shared native build. Its three other cases run as the
behavior tests of `geom_fill_corrected_frenet_behavior`.

GeomPlate_BuildPlateSurface_Test.cxx is not mapped: its A1 and A2 cases (and the shared
helper of the file) build their boundaries as `BRepAdaptor_Curve` of edges made by
`BRepBuilderAPI_MakeEdge` (TKTopAlgo), which the port does not provide (the native suite
itself compiles and passes against the shared build, which links TKTopAlgo). Its six
cases run as behavior tests (`geom_plate_behavior`): the same Bezier curves go through
`GeomAdaptor_Curve` on their natural range, which the edge of `MakeEdge` evaluates
identically, and the progress observer checks the same scope names
(`Calculating the surface filled`, `Plate_Plate::SolveTI1()`, `math_Gauss
LU_Decompose`). Its GeomPlate content is also in the numeric corpus.

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
- GccAna: each native constructor overload is a named constructor returning `Result`,
  named after its argument kinds and locus (`Circ2d3Tan::circ_lin_pnt`,
  `Circ2d2TanOn::lin_pnt_on_circ`, `Lin2d2Tan::tangent_to_circles`...); each
  `GccAna_<Class>_<n>.cxx` file is one submodule. The fixed-size `NCollection_Array1`
  fields are `solutions::Slots`, 1-based with the native bounds check, so a solver that
  overruns its arrays raises Standard_OutOfRange as natively; `Tangencies` holds the
  `TheSame`/point/parameter arrays of one argument. The consultation members keep the
  checks each native class writes (some skip the `WellDone` test, `Circ2d2TanOn`'s
  `ThisSolution` tests nothing), and output references become returned tuples.
- IntWalk: `IntWalk_PWalking` is the struct `IntWalkPWalking`; the performing
  constructor is `with_start`, the Perform with a parameter box `perform_bounded`,
  `PerformFirstPoint` returns an `Option`, in-out arguments become `&mut` or returned
  values, and every member that can raise returns `Result`. Its protected searches
  (`ComputePasInit`, the two distance minimizations, `SeekPointOnBoundary`,
  `HandleSingleSingularPoint`, `ExtendLineInCommonZone`) are public so that the probe
  drives them. `IntWalk_TheInt2S` and `IntWalk_TheFunctionOfTheInt2S` are aliases of the
  IntImp generics; the template `IntWalk_IWalking.gxx` is the generic `IntWalkIWalking`
  over the traits `TheIWFunction` and `TheIWLine` ([geomalgo-design.md](geomalgo-design.md)
  section 10), with the open-line Perform named `perform_open`. The IWalking members
  belong to the header of each instance (`IntPatch_TheIWalking.hxx`), so their api rows
  come with IntPatch. The package has no GoogleTest suite at this commit.

- HatchGen, TopClass, Geom2dHatch: see the section below; overloads take distinct names
  (`Domain::bounded`/`semi_infinite`, `Hatcher::trim`/`trim_curve`/`trim_hatching`,
  `compute_domains`/`compute_domains_of`, `add_element`/`add_element_curve`), and the protected
  `Element(IndE)` and `Hatching(IndH)` of the hatcher are public as `element_mut` and
  `hatching_mut`.
- Geom2dGcc: see the section below; each constructor overload is a named constructor
  returning `Result`, named after its argument kinds and locus
  (`Circ2d3TanIter::circ_lin_curve`, `Circ2d2TanOnIter::curve_pnt_on_circle`,
  `Lin2d2Tan::through_point_with_parameter`...), the `Results` overloads are `results_ana`
  and `results_geo`, and `Geom2dGcc_QCurve` is an alias of `QualifiedCurve`.

- IntCurveSurface: the `.pxx` templates are generics (docs/geomalgo-design.md section
  11): `InterGen<T: InterTypes>` is the algorithm (`HInter` its alias), `InterPolygon<C>`
  and `InterPolygonTool<C>` the polygon, the utilities are the modules `inter_utils`,
  `polygon_utils` and `polyhedron_utils`; the instance classes are aliases
  (`TheCSFunctionOfHInter`, `TheExactHInter` on the IntImp generics,
  `TheInterferenceOfHInter` on the Intf generic, `TheQuadCurvExactHInter`,
  `TheQuadCurvFuncOfTheQuadCurvExactHInter`). The protected members of `HInter` and of
  `IntCurveSurface_Intersection` are public (`perform_bounds`, `internal_perform_*`,
  `perform_conic_surf_<conic>`, `append_int_ana`, `append_curve_point`,
  `append_curve_segment`; `append`, `append_point`, `append_segment`, `set_values`,
  `reset_fields` and the `done`/`my_is_parallel` fields). Overloads take distinct
  names, output references become returned tuples, and `Dump` members write to a
  `fmt::Write` where the native ones write to `std::cout`.

- GeomFill (filling part, [geomalgo-design.md](geomalgo-design.md) section 13):
  `GeomFill_AppSurf` is the alias `GeomFillAppSurf` of `AppBlendAppSurf`, with
  `GeomFillLine` and `GeomFillSectionGenerator` implementing its line and generator
  traits; `GeomFill_Profiler` is a struct that `GeomFillSectionGenerator` embeds
  (`Deref`). Output arrays are assigned as natively (`NCollection_Array1::operator=`
  takes the source bounds and length), `GetShape` returns the four counts, and the two
  `Section` overloads are `section_d1` and `section`. `GeomFill_Generator` embeds the
  profiler too. `GeomFill_Coons`, `GeomFill_Curved` and `GeomFill_Stretch` embed their
  `GeomFill_Filling` result (`Deref`); their four-row `Init` overloads take the rows as
  an array (`init`, `init_weighted`, `with_poles`, `with_weights`), the two-row ones of
  `GeomFill_Curved` are `init_two_rows` and `init_two_weighted_rows`. The `Init`
  overloads of `GeomFill_BezierCurves` and `GeomFill_BSplineCurves` are `init_four`,
  `init_three` and `init_two` (constructors `with_four`, `with_three`, `with_two`).
  `GeomFill_Boundary` and `GeomFill_TgtField` are the traits `GeomFillBoundary` and
  `GeomFillTgtField` (the tolerance setters are `set_tol3d` and `set_tolang`, the
  field's `D1(W, V, DV)` is `d1_with_value`); `GeomFill_CoonsAlgPatch::Func(f1, f2)` is
  `func_pair`; `GeomFill_CornerState`'s overloaded setters are `set_gap`, `set_tgt_ang`
  and `set_nor_ang`; `GeomFill_ConstrainedFilling`'s `Init` overloads are `init_three`
  and `init_four`, `ReBuild` is `re_build`, `Eval` writes into a slice, and the
  `Check` members write to a `fmt::Write` where the native ones write to `std::cout`.

- IntStart: the package has no header; its two templates are the generics
  `SearchOnBoundaries<T: SearchOnBoundariesTypes>` and `SearchInside<T: SearchInsideTypes>`
  over the tool traits of `int_start::tools` (docs/geomalgo-design.md section 12), which
  IntPatch and Contap will instantiate as aliases. The api rows name the template
  members (`IntStart_SearchOnBoundaries::Perform`...) with the `.gxx`/`.lxx` source. The
  performing constructor of `SearchInside` is `with_domain`, the Perform from a start
  point `perform_from`; the output references of the tool members become returned
  values (`Option` for `HasFirstPoint`/`HasLastPoint`).

- IntPolyh: concrete structs (docs/geomalgo-design.md section 12). `IntPolyh_Array<T>` is
  `IntPolyhArray<T>` and the six array typedefs are aliases; `operator[]` is `Index`/`IndexMut`
  and `Value`/`ChangeValue` return Standard_OutOfRange past the storage. The constructors are
  named (`IntPolyhIntersection::with_samples`/`with_parameters`,
  `IntPolyhMaillageAffinage::with_samples`, `IntPolyhSectionLine::with_size`), the
  `FillArrayOfPnt` overloads are `fill_array_of_pnt`, `_shift`, `_params`, `_shift_params`
  and `_normals` (an optional deflection for `const double*`), `CommonBox(B1, B2, ...)` is
  `common_box_of`, the `Dump(int)` overloads are `dump_index`, `TriangleEdgeContact` takes its
  fourteen geometric arguments in `IntPolyhTriangleEdge`, and output references become
  returned arrays or tuples (`get_line_point` returns `[x, y, z, u1, v1, u2, v2, incidence]`).
  The members that can raise (surface evaluations, `Bnd_Box::Get`, the coplanar `IsParallel`)
  return `Result`.
- GeomAPI (see the section below): one named constructor or `init_*` method per native
  overload (`ProjectPointOnSurf::with_bounds_tolerance`, `init_surface`...), the
  conversion operators as `as_int`, `as_real`, `as_point`, `as_curve` and `as_surface`,
  output references as returned tuples, and `GeomAPI::To2d`/`To3d` as the free functions
  `to_2d` and `to_3d`. `GeomAPI_IntCS::Parameters(Index, U1, V1, U2, V2)` is
  `segment_parameters`. The PointsToBSplineSurface settings travel as the tuple
  `SurfaceFitSettings` and the height grid as `HeightGrid`.
- GeomFill Gordon: `GeomFill_Gordon` is `GeomFillGordon`, its nested enums and report are
  `GordonStatus`, `GordonApproximationMode`, `GordonBuildStage` and `GordonBuildReport`;
  `GeomFill_NetworkSurface` is `GeomFillNetworkSurface` with `NetworkSurfaceStatus`, and
  its Init takes the curve families, the parameters, the grids and the closed seams as
  tuples. `GeomFill_GordonUtilities.pxx` is internal: its functions are private modules
  (`gordon_params`, `gordon_contacts`, `gordon_normalize`, `gordon_order`,
  `gordon_reorder`, `gordon_reparam`, `gordon_rebuild`, `gordon_network`,
  `gordon_fallback`).
- GeomPlate ([geomalgo-design.md](geomalgo-design.md) section 19): the classes are
  `GeomPlateBuildPlateSurface`, `GeomPlateSurface` (a `GeomSurface`),
  `GeomPlateCurveConstraint` and `GeomPlatePointConstraint` (handle-shared, their fields
  behind a lock), `GeomPlateMakeApprox`, `GeomPlatePlateG0Criterion` and
  `GeomPlatePlateG1Criterion` (`AdvApp2VarCriterion`), `GeomPlateBuildAveragePlane`
  and `GeomPlateAij`. Overloads take distinct names: the builder constructors
  `with_curves` (the old arrays), `with_surface` and `new`, `add_curve`/`add_point`,
  `G0Error(Index)` as `g0_error_of` (and G1, G2); `GeomPlateCurveConstraint::empty`
  and `new`; `GeomPlatePointConstraint::with_point` and `on_surface`;
  `GeomPlateMakeApprox::with_criterion` and `with_order`;
  `GeomPlateBuildAveragePlane::new` and `with_normals`. `LPropSurf` returns the write
  guard of the member `GeomLProp_SLProps`, `D1`/`D2` of the constraints return
  `GeomSurfaceD1`/`GeomSurfaceD2` (the point constraint's `D1` a tuple), and the
  private `ProjectedCurve` of the builder, which nothing calls natively, is kept as a
  crate method with a unit test.

- IntPatch (the core; docs/geomalgo-design.md section 16): `IntPatch_Line` and
  `IntPatch_PointLine` are the traits `IntPatchLine` and `IntPatchPointLine` over an
  embedded `IntPatchLineBase`; the lines are handle-shared transients whose members take
  `&self`. The constructors by transition kind are `new`/`new_touch`/`new_undecided`
  (WLine, RLine) or take an `IntPatchLineTransitions` (GLine `from_line`...`from_hyperbola`,
  ALine `new`); `Vertex(i)`, `Point(i)` and `FirstPoint()` return copies, `ChangeVertex`
  takes a closure, `FirstPoint(int&)` is `first_point_index`. `IntPatch_Point`'s
  `SetValue` overloads are `set_value_tangent`, `set_value` and `set_pnt_on_2s`;
  `IntPatch_ArcFunction`'s `Set` overloads are `set_arc`/`set_surface` and its
  evaluations `value_at`/`derivative_at`/`values_at`; the path point constructors are
  `on_vertex`/`on_arc`; `CurvatureRadiusOfIntersLine` is the free function
  `curvature_radius_of_inters_line`. The template instances are aliases
  (`IntPatchTheSurfFunction`, `IntPatchCurvIntSurf`, `IntPatchTheSOnBounds`,
  `IntPatchTheSearchInside`, `IntPatchTheIWalking`) whose rows name the generic members.
  `IntPatch_RLine::SetParamOnS1/S2` are declared but defined nowhere (excluded rows).
- IntPatch polyhedra and polygons: `IntPatch_Polyhedron` reuses the IntCurveSurface
  polyhedron helpers; its constructors are `new` (automatic subdivision) and
  `with_subdivisions`, `Point(P, i, j, U, V)` is `set_point`, `Point(i, U, V)` is
  `point_with_parameters`, `DeflectionOverEstimation(flec)` is
  `set_deflection_over_estimation`. `Perform`, the `*Singularity(bool)` setters and the
  `Has*Singularity` queries are declared but defined nowhere (excluded rows).
  `IntPatch_PolyhedronBVH` implements the TKMath `BvhSet`/`BvhObject` traits (`Box` is
  `element_box`, the polyhedron constructor `with_polyhedron`); `IntPatch_BVHTraversal`
  implements `BvhPairTraverse` privately and `Perform(aSet, aSet, true)` is `perform_self`.
  `IntPatch_InterferencePolyhedron` derefs to `Interference` (constructors `new`, `between`,
  `of_self`; `Perform(one)` is `perform_self`). `IntPatch_Polygo` is the trait
  `IntPatchPolygo` over `Polygon2d`; `IntPatch_PolyLine`'s setters are `set_w_line` and
  `set_r_line`, `IntPatch_PolyArc::Closed` is `is_closed`.
- IntPatch analytic intersections (docs/geomalgo-design.md section 17):
  `IntPatch_ImpImpIntersection`, `IntPatch_ImpPrmIntersection`, `IntPatch_ALineToWLine`,
  `IntPatch_RstInt` and `IntPatch_SpecialPoints` map member by member (45 api rows, all
  implemented). The constructors that run `Perform` are `with_surfaces`; the second
  `MakeWLine` is `make_wline_range`; `IntStatus` is the enum `IntPatchImpImpStatus`; the
  static classes are unit structs with associated functions. The protected members the
  probe reaches (`StepComputing`, `CheckDeflection`, `GetSectionRadius`,
  `CorrectEndPoint`, `ProcessSphere`, `ProcessCone`, `GetTangentToIntLineForCone`) are
  public and have rows. The five classes have no GoogleTest suite at this commit.

- GeomFill (sweep part, [geomalgo-design.md](geomalgo-design.md) section 14): the
  trihedron, location and section laws are the traits `GeomFillTrihedronLaw`,
  `GeomFillTrihedronWithGuide`, `GeomFillLocationLaw` and `GeomFillSectionLaw`; the
  `gp_Vec&` and `gp_Mat&` outputs are the frame structs `TrihedronD0..D2` and
  `LocationD0..D2`, the `D0` overload with 2d poles is `d0_poles2d`, and `Error`
  arguments are `&mut f64`. `GeomFill_CorrectedFrenet(ForEvaluation)` is
  `with_evaluation`. The `GeomFill_SectionPlacement::Perform` overloads are `perform`,
  `perform_on_path` and `perform_at`. `GeomFill_NSections` constructors are `new`,
  `with_params`, `with_section_domain`, `with_domain` and `with_reference`;
  `GeomFill_SweepSectionGenerator` constructors are `new`, `with_radius`,
  `with_section`, `with_sections` and `with_rails` (with the matching `init_*`), and
  its `Section` overloads are `section_d1` and `section`. `GeomFill_Pipe` constructors
  are `new`, `with_radius`, `with_section`, `on_surface`, `with_direction`,
  `with_two_sections`, `with_sections`, `with_rails`, `with_adaptor_rails` and
  `with_guide` (with the matching `init_*`); its `Perform` overloads are `perform` and
  `perform_with_parameters`, and `GenerateParticularCase(B)` is
  `set_generate_particular_case`. `GeomFill_AppSweep` is the alias `GeomFillAppSweep`
  of `AppBlendAppSurf`. The static `GeomFill` members are functions of `geom_fill`
  (`surface`, `get_circle`, `get_circle_d1`, `get_circle_d2`, `get_shape`, `knots`,
  `mults`, `get_minimal_weights`, `get_tolerance`). The `GeomFill_Tensor` call
  operators are `value` and `change_value`.
- IntPatch parametric intersection and front end (docs/geomalgo-design.md section 18):
  `IntPatch_WLineTool` is the unit struct `IntPatchWLineTool` (`compute_purged_wline`,
  `join_wlines`, `extend_two_wlines`, the constant `MY_MAX_CONCAT_ANGLE`).
  `IntPatch_LineConstructor` is `IntPatchLineConstructor` (`new`, `perform`, `nb_lines`,
  `line`). `IntPatch_PrmPrmIntersection` is `IntPatchPrmPrmIntersection`; its `Perform`
  overloads are `perform` (the main one, with `ClearFlag`), `perform_polyhedra`,
  `perform_polyhedron1`, `perform_polyhedron2`, `perform_self`, `perform_self_polyhedron`,
  `perform_with_points` and `perform_from_start`, the four marching tolerances of the
  polyhedral and start-point overloads are one `IntPatchPrmPrmTolerances`, and the grid
  members take arrays for the cell coordinates (`remplit_lin([x1, y1, z1], [x2, y2, z2],
  map)`). `IntPatch_Intersection` is `IntPatchIntersection` (`new`, `with_surfaces`,
  `with_surface`, `set_tolerances`, `perform` and `perform_with_points` with the three
  flags in one `IntPatchIntersectionOptions`, `perform_from_start`, `perform_self`, the
  result accessors, `dump`, and the static `check_singular_points`, `define_uv_max_step`
  and `prepare_surfaces`). The domains are `&(dyn TheTopolTool + 'static)` (the virtual
  `Adaptor3d_TopolTool`) and the line sequences `Vec<Handle<dyn IntPatchLine>>`.

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
- Intf: undefined native behaviour gets a defined result: an empty tangent zone's
  `InfoFirst`/`InfoSecond` cast `RealLast` to int as x86-64 does (INT_MIN, checked
  against the native probe); a seventh `HyprBox` segment raises Standard_OutOfRange
  where native code writes past its arrays. Members that index records return
  `Result` with the native Standard_OutOfRange or Standard_NoSuchObject. `Dump`
  writes to a `fmt::Write` where native code writes to `std::cout`.
- Intf: `Intf_Polygon2d` is the trait `Polygon2d`; `Intf_Interference` is held by the
  interferences and reached through `Deref`/`DerefMut`, its protected members through
  `section_points_mut`, `section_lines_mut`, `tangent_zones_mut` and `set_tolerance`.
  The generic `.gxx` is `InterferencePolygonPolyhedron<PT, HT>` over the
  `Polygon3dTool` and `PolyhedronTool` traits ([geomalgo-design.md](geomalgo-design.md)
  section 2); IntCurveSurface and HLRBRep instantiate it when they are ported.
- IntWalk: the walking data of `IntWalk_IWalking.gxx` are vectors whose `operator[]` does
  not check its index natively; the reads that can pass the end (the start points of the
  open-line Perform) raise Standard_OutOfRange. The point sequences are slices read
  1-based with the native Standard_OutOfRange; the line class is reached through the
  trait `TheIWLine`, whose members take `&self` (the lines are shared handles edited
  after they are stored).

## IntRes2d, IntImpParGen and IntCurve

IntRes2d (`int_res2d`), IntImpParGen (`int_imp_par_gen`) and IntCurve (`int_curve`) are
ported in full:
`tools/audit_api_rows.py` reports 0 missing and 0 stale rows for each. The root class
`IntRes2d_Intersection` is a struct that the algorithms embed and reach through `Deref`;
its protected members (`Insert`, `Append`, `SetValues`, `ResetFields`, the `done` field)
are public so that algorithms of later toolkits can derive from it. Every constructor
overload is a named constructor and every `Perform` overload of IntCurve_IntConicConic a
`perform_<pair>` method; accessors that raise return `Result` with the native exception
type. IntImpParGen_Intersector.gxx is a generic (docs/geomalgo-design.md section 3), and so
are the IntCurve curve templates (sections 4 and 5).

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

## Geom2dInt and Geom2dAPI

Geom2dInt (`geom2d_int`) holds the curve tool `Geom2dCurveTool`, the projection
`TheProjPCurOfGInter` and the IntImpParGen instantiation `Geom2dTypes`; its other classes
are aliases of the IntCurve and Extrema generics (docs/geomalgo-design.md sections 4 and
5), among them the curve-curve intersector `GInter`. Geom2dAPI (`geom2d_api`) ports
`ExtremaCurveCurve`, `InterCurveCurve`, `ProjectPointOnCurve`, `Interpolate` and
`PointsToBSpline`; the conversion operators are `as_int`, `as_real`, `as_point` and
`as_curve`. Both packages are ported in full: `tools/audit_api_rows.py` reports 0
missing and 0 stale rows for each.

Preserved native behaviour and deliberate differences:

- `Geom2dInt_Geom2dCurveTool::IsComposite` and `Geom2dAPI_Interpolate::ClearTangents` are
  declared but never defined natively; they have excluded rows.
- Extrema_ExtCC2d and Extrema_ExtPC2d borrow their curves, which the Geom2dAPI objects own:
  the objects keep the results of their construction (with the exception each Extrema
  accessor raised) and `extrema()` rebuilds the deterministic Extrema object on the stored
  adaptors.
- `Geom2dAPI_Interpolate::Load(Tangents, TangentFlags)` shares the caller's flag array,
  which Perform completes (the end tangents it builds are flagged there, visible to the
  caller); BuildParameters reads the previous parameter at the point index, so a point
  array not starting at 1 raises Standard_OutOfRange; `Load(InitialTangent, FinalTangent)`
  writes the flags and tangents at 1 and NbPoints whatever the bounds. PLib::EvalLagrange
  leaves its result unset natively when two parameters are too close; the port reads 0.
- `Geom2dAPI_PointsToBSpline::Init(Points, Params, ...)` reads `Params(i)` for i in
  `2..Nbp-1` whatever the lower bound of `Params`; the variational Init swallows the
  exception of AppDef_Variational::Approximate, as natively.
- IntCurve_IntPolyPolyGen recurses on a tangent zone with `Tol` as the confusion
  tolerance and `TolConf` as the tolerance (the arguments are swapped natively), and
  tests the full polygon representation as `NbSegments() == sample count`, which the
  constructor never gives (a polygon of n samples has n - 1 segments), so the second
  search on full polygons runs whenever the first one misses a root.
- The error flag of IntCurve_ExactIntersectionPoint is never reset: once one solve fails,
  IntPolyPolyGen skips every later section point of the same interference.
- The self intersection keeps only the last of the section points on the same pair of
  chords and gives nothing for a tangent zone whose ends are within EpsX (the pair
  intersection inserts a point there). Its end scan (PosSegment) is computed and
  discarded, as natively.
- HeadOrEndPoint takes the transitions of an end point at the original parameters: the
  second-order test replaces the snapped end point by the curve point there.
- IntCurve_IntCurveCurveGen splits composite curves by the native tail recursion over the
  intervals of the second curve (ported as a loop); a clipped interval of at most 1e-10
  (or NaN) length is skipped. Its self intersection of a curve with both bounds infinite
  returns not done without resetting the previous results.
- `ApproxParamOnCurve` with an index past the last point prints "OutOfRange
  Polygon2d::ApproxParamOnCurve" on std::cout natively; the port does not print (the probe
  discards std::cout). The polygon index slots past the sample count are uninitialized
  natively and 0 in the port, where reading one raises Standard_OutOfRange. Only a chord
  index past the last point reads them (for example a seat given to
  IntCurve_ExactIntersectionPoint by the caller); the intersectors pass only the chords of
  the interference, so the corpus leaves that undefined case out.
- `GetIntersection` leaves its sample indices at -1 when every distance is NaN and then
  reads out of bounds if the current distance exceeds sqrt(RealLast); the port reads the
  first sample.

## HatchGen, TopClass and Geom2dHatch

HatchGen (`hatch_gen`), TopClass (`topclass`) and Geom2dHatch (`geom2d_hatch`) are ported in
full: `tools/audit_api_rows.py` reports 0 missing and 0 stale rows for each (TopClass has no
header; its six template functions have rows on the `.pxx` files). The abstract
`HatchGen_IntersectionPoint` is embedded in `PointOnElement` and `PointOnHatching` and reached
through `Deref`; its pure virtual `Dump` is the trait `IntersectionPointDump`. The TopClass
templates are the generics `Classifier2d` and `FaceClassifier` (docs/geomalgo-design.md
section 6); `Geom2dHatch_FClass2dOfClassifier` and `Geom2dHatch_Classifier` are their aliases.
Curves are `Geom2dAdaptor_Curve` values that the elements and hatchings own; the Rust
constructors take them by value, and `Elements::current_edge` returns a shallow copy (the
native `CurrentEdge` assigns a copy). Setters that share the getter's name take a `set_`
prefix, output references become returned tuples, and the `Dump` members write the native
`std::cout` text to a `fmt::Write`. The GTest suites Geom2dHatch_Elements_Test and
Geom2dHatch_Intersector_Test are translated 1:1.

Preserved native behaviour:

- `HatchGen_PointOnElement(IntRes2d_IntersectionPoint)` reads `IsOpposite` of the element's
  transition when the hatching's transition is a TOUCH, which raises Standard_DomainError when
  the element's transition is not a TOUCH; the constructor returns `Result`.
- `HatchGen_Domain::SetPoints()`, `SetFirstPoint()` and `SetSecondPoint()` without argument clear
  only the flags; the stored points stay (they are not observable through the accessors).
- `HatchGen_PointOnHatching::AddPoint` drops a point on element that is identical to a stored
  one (same fields, parameters within the confusion).
- `Geom2dHatch_Hatcher::Trim(IndH, IndE)` handles an intersection segment only when it has both
  ends: the treatments of the first and of the last end are inside the test of both ends, so
  a half-bounded segment adds no point. It writes " Intersector -> Done = False " on std::cout
  when the intersection is not done; the port does not print (the probe discards std::cout).
- `RemElement` scans copies of the hatching points: a point is removed when all its points on
  element belong to the removed element (or when it has none), but a point kept keeps its
  points on that element; the domains are cleared when such a point was seen.
- `GlobalTransition` computes the element parameter from the point position, then overwrites
  it with the point's parameter (upstream finding #25, kept because removing it caused blend
  regressions).
- `ComputeDomains`: with two points, OUT-OUT becomes OUT-IN at the first point and IN-OUT at
  the second (lbr, 1997); an IN-IN point without an opened segment and without a saved point
  sets IncoherentParity (or marks the hatching done when a domain exists) and goes on to the
  next point, skipping the domain and last-point steps (its `return` is commented out); a
  hatching without intersection point is one infinite domain when its classification point
  (at the last parameter, else the first, else 0) is IN.
- `Geom2dHatch_Hatching::TrimFailed(false)` and `ClrPoints` keep the status; `TrimFailed(true)`
  sets TrimFailure.
- The copy constructor of `Geom2dHatch_Elements` copies nothing: the copy is empty, with the
  probing at element 0 and fraction 0 (`Elements::copy_of`). Its `OtherSegment` probes each
  FORWARD or REVERSED element at fractions 0.123 + k 0.2111 below 0.8 and rejects a probe whose
  segment is nearly tangent to the element unless it is the element's last probe.
- `Geom2dHatch_Hatcher::IsDone()` (without index) is declared but never defined natively; it
  has an excluded row, as the inline destructor of `Geom2dHatch_Elements` (it only clears).
- `Geom2dHatch_Intersector::LocalGeometry` evaluates the adaptor's Geom2d curve (the basis of
  a trimmed curve) whatever the adaptor's bounds; an undefined tangent gives (1, 0) and a zero
  curvature. `Perform(L, P, Tol, E)` intersects on a fresh Geom2dInt_GInter with the
  parametric tolerances and copies its results (SetValues); `Intersect` uses the stored
  tolerances. The copy constructor keeps the results without the done and reversed flags, the
  assignment (`Hatcher::Intersector(I)`) copies both.

Deliberate Rust deviations:

- The native default constructors of `Geom2dHatch_Element` (orientation) and
  `Geom2dHatch_Classifier` (edge position) leave a member uninitialized; the port starts them at
  FORWARD and `IntRes2d_Head`. The classifier probe primes the classifier on a square first, so
  that the position it reads is defined natively.
- `Geom2dHatch_Elements` keeps its edge iterator as a rank in the map's bucket order, since a
  Rust iterator cannot borrow the map it lives beside; a map changed during an edge iteration
  invalidates the native iterator.
- The implicit copy and assignment of Geom2dInt_GInter (which the hatcher uses) copy the inner
  intersectors with their copy constructor, so their done and reversed flags are not copied;
  every Perform sets those flags before reading them.

## Geom2dGcc

Geom2dGcc (`geom2d_gcc`) is ported in full: `tools/audit_api_rows.py` reports 0 missing and 0
stale rows (238 rows). OCCT 8 expands the old GccIter and GccGeo templates into concrete
classes, so the `...Iter` and `...Geo` classes are concrete structs (docs/geomalgo-design.md
section 9). The qualified curve holds a copy of its `Geom2dAdaptor_Curve`; the function classes
borrow their curves (`&'a Geom2dAdaptorCurve`) and implement the tkmath `Function`,
`FunctionWithDerivative`, `FunctionSet` and `FunctionSetWithDerivatives` traits. The fixed-size
solution arrays are the GccAna `solutions::Slots` and `Tangencies`, with the native bounds
checks. `Geom2dGcc_IsParallel` is a `Failure` of that type (`geom2d_gcc::is_parallel`), and
`Geom2dGcc_Type1`, `Type2` and `Type3` are enums. The GTest suites Geom2dGcc_Lin2d2Tan_Test,
Geom2dGcc_Circ2d2TanRad_Test and Geom2dGcc_Circ2d3Tan_Test are translated 1:1; the Lin2d2Tan
suite calls `GeomAPI::To2d`, whose toolkit is not ported, so the test file builds the same
projected curve with `ProjLib::ProjectedCurve` and `make_curve`.

Preserved native behaviour:

- `Lin2dTanObl(Qualified1, TheLin, Param1, TolAng, Angle)` hands its arguments to the solver as
  (TolAng, Param1) and never counts the solution it stores, so NbSolutions stays 0. The
  sampling of both constructors raises Geom2dGcc_IsParallel (from `Lin2dTanOblIter`'s
  `Intersection2`) when a solution is parallel to TheLin. `IsParallel2` of `Lin2dTanObl` is
  declared but never defined natively; its row is excluded.
- `Lin2d2TanIter(curve, curve)` raises gp_VectorWithNullMagnitude for two straight curves
  (their second derivative is null).
- `FunctionTanCuCuCu::Values` and `Derivatives` return true and write nothing, so the
  `Circ2d3TanIter` search stops at its start point; three `FunctionTanCuCuCu` constructors with
  points are declared but never defined natively (excluded rows).
- `FunctionTanCuCuOnCu(circle, curve, OnCirc, Rad)` builds the curve-curve-on-circle type: its
  evaluation reads the null first curve (a crash natively; the port raises Standard_NullObject).
- `Circ2d3TanIter` slips: the curve-line-point and curve-circle-point forms evaluate the curve
  at the wrong parameter; the curve-curve-point and circle-circle-curve forms test Qualified1
  for the second argument; `IsTheSame2` reads the third flag and `IsTheSame3` is always true, so
  `Circ2d3Tan::Tangency3` of an iterated solution raises. `Circ2d3Tan` sets NbSolutions to 1 for
  an iteration even when it fails, and its (line, circle, circle) case permutes the qualifiers
  wrongly.
- `Circ2d2TanRadGeo(curve, curve)` never writes the second qualifier and stores the start
  parameters, not the refined ones.
- `Circ2d2TanOnGeo(circle, circle)` fills `Rbid(1)` twice and leaves `Rbid(2)` unset, and
  reads `Radius(i)` at the bisector index (a third radius raises Standard_OutOfRange).
- `Circ2d2TanOnIter`: the point forms search 4 unknowns with vectors of 3, so math_FunctionSetRoot
  raises Standard_DimensionError on the start point (math_FunctionSetRoot.cxx reads the fourth
  tolerance past the end without a check); the consultations return 0 for every parameter; the
  circle-curve-on-line form takes its centre from `X(1)`; `Func.Value` runs at the start point
  before the done test. `Circ2d2TanOn(curve, point)` therefore always raises
  Standard_DimensionError for a non-analytic curve.
- `Circ2d2TanOn` goes on to the curve iteration after its line and circle loci (the iteration
  is not in an `else`), and leaves the centre parameter unset for iterated solutions.
- A GccAna solution `TheSame` to its argument raises StdFail_NotDone out of the
  `Circ2dTanCen`, `Circ2d2TanRad` and `Circ2d2TanOn` constructors, which read every tangency.
  The `Results` members of `Circ2d2TanRad`, `Circ2d2TanOn` and `Circ2d3Tan` are public.
- `FunctionTanCirCu` on a zero-length range loops forever natively (its sampling step is 0);
  no request builds one.
- The three-circle `GccAna_Circ2d3Tan` reuses the quartic coefficients A, B, C for the X
  quadratic of a touching configuration; the port now does the same (it found 4 circles for
  the BUC60622 case where native finds 3).

Deliberate Rust deviations:

- A tangency function keeps the first failure of a curve evaluation (`raised()`), and the
  solvers return it after the tkmath search stops, where native code throws from inside the
  search.
- A null point handle raises Standard_NullObject where native code dereferences it.
- Slots that native leaves uninitialized start at 0; the corpus avoids reading them.

## GeomAPI and GeomFill Gordon

GeomAPI (`geom_api`) ports every class:
`to_2d`/`to_3d`, `ProjectPointOnCurve`, `ProjectPointOnSurf`, `ExtremaCurveCurve`,
`ExtremaCurveSurface`, `ExtremaSurfaceSurface`, `IntCS`, `IntSS` (over `GeomInt_IntSS`,
see ApproxInt and GeomInt below), `Interpolate` (the 3D instance of the interpolation body
shared with Geom2dAPI), `PointsToBSpline` and `PointsToBSplineSurface`.
`tools/audit_api_rows.py` reports 0 missing and 0 stale rows for the package. How the classes hold their
Extrema objects is in [geomalgo-design.md](geomalgo-design.md) section 15.

GeomFill ports `GeomFill_GordonUtilities.pxx`, `GeomFill_Gordon` and
`GeomFill_NetworkSurface` (the other GeomFill classes belong to the filling and sweep
parts); every public member of the two classes has an api row. The three TKG3d suites
that need GeomAPI (`Geom_CurveEval_Test.cxx`, 11; `GeomAPI_ExtremaCurveCurve_Test.cxx`,
47; `GeomAPI_Interpolate_Test.cxx`, 1) are translated in this crate as
`tkg3d_geom_curve_eval`, `tkg3d_geomapi_extrema_curve_curve` and
`tkg3d_geomapi_interpolate`; their ledger rows keep the toolkit TKG3d, so the TKG3d
verifier runs them.

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

The numeric corpus has 29035 requests: TopTrans 981; Intf 2240; GccEnt 55; GccInt 15;
Hatch 38; GccAna 2682; IntRes2d with IntImpParGen and IntCurve 2044; Geom2dInt and
Geom2dAPI 962; the curve-curve classes 1077; IntSurf 334; Law 266; Plate 106;
LocalAnalysis 406; FairCurve 178; IntImp 146; AppBlend 115; HatchGen 432; Geom2dHatch
with TopClass 1116; Geom2dGcc 3968; IntWalk 717; IntCurveSurface 3077; the GeomFill
filling classes 568; IntStart 674; IntPolyh 522; GeomAPI 1980; GeomFill Gordon and
NetworkSurface 145; the IntPatch core 924; the GeomFill sweep classes 1075; GeomPlate
163; the IntPatch analytic intersections 582; the IntPatch parametric intersection
and front end 333; and ApproxInt, GeomInt and GeomAPI_IntSS 1114.

Intf (2240 requests): `intf_triangle` compares `Intf::PlaneEquation` and `Intf::Contain`
on 54 triangles (both orientations, coincident and collinear vertices, a normal that
underflows below `gp::Resolution`, huge coordinates, points on edges and vertices,
outside and off the plane). `intf_spoint`, `intf_sline`, `intf_tzone` and
`intf_interference` run seeded scripts over every member of the records and of
`Intf_Interference` (the protected members through a probe subclass), with section
points drawn from a few logical identities so that equal points, shared line ends
and shared zone edges are frequent, and with out-of-range indices and empty records
for the exceptions; the dumps are captured text. `intf_tool2d` and `intf_tool3d` run
one to three box computations on one `Intf_Tool` for lines (with exactly zero
components), hyperbolas and parabolas over void, whole, finite and open domains,
tangent faces included. `intf_ipoly2d` runs the performs and constructors of
`Intf_InterferencePolygon2d` on crossing, overlapping, near-parallel, touching,
degenerate and self-crossing polylines. `intf_ipp` instantiates the generic
`Intf_InterferencePolygonPolyhedron.gxx` in the probe over a polygon and a
triangulated height field (with its outer edges as boundary edges, and a degenerate
triangulation), and runs every Perform, Interference and constructor overload, with
and without the grid.

TopTrans (981 requests): `tt_curve` and `tt_surface` run a command sequence (resets
and compares) on one fresh transition and compare the exception type and both states
after every command; `tt_states` compares the static state tables. Directions come
from discrete fans (multiples of 45 and 30 degrees, near-tolerance tilts), so equal
angles, tangent faces, the touch case, the incoherent sectors and the parallel-frame
exceptions all occur; the hand-made sequences cover the upstream scenarios and the
curved ties at a null cosine. A surface request uses only one Compare overload (see
the uninitialized sector curvature in the deviations above).

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

GccAna: `gccana_circ3tan`, `gccana_2tanrad`, `gccana_2tanon` and `gccana_tanonrad` run
every constructor overload of Circ2d3Tan, Circ2d2TanRad, Circ2d2TanOn and Circ2dTanOnRad
on hand-built configurations (tangent, nested, concentric, secant, parallel, coincident
and far arguments, each qualifier including the rejected ones, the upstream suite's
cases) and on seeded random arguments; `gccana_tancen`, `gccana_lin2tan`,
`gccana_lintanobl`, `gccana_linbisec` and `gccana_bisec` do the same for Circ2dTanCen,
Lin2d2Tan, Lin2dTanObl, Lin2dBisec and the five bisector classes. Each request prints
the solution count and order and, from index 0 to NbSolutions + 1, every solution and
the consultations its detail mask selects; `touching_circles` adds six three-circle
configurations that touch (BUC60622) at two tolerances (2682 requests).

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

Geom2dInt curve-curve, Geom2dAPI_InterCurveCurve: `tkgeomalgo_corpus_ginter.py` adds 1077
requests: `g2int_polygon` builds polygons of every parametric curve with 2 to 40 samples,
refines them in overlapping, far and touching boxes and closes them; `g2int_exact` runs
IntCurve_ExactIntersectionPoint from inner, first, last and out-of-range polygon seats and
from explicit bounds; `g2int_polypoly` intersects crossing, tangent, overlapping,
disjoint and self-crossing curves (a Bezier loop, a periodic figure eight) through the
polygon intersector, its self intersection and its protected iterations (up to past the
last one); `g2int_ginter` runs Geom2dInt_GInter on every pair of 22 curves (conics,
Bezier, B-spline with C0 knots, periodic, rational, trimmed and offset curves) through
every constructor and Perform, with ComputeDomain, inner, semi-infinite and infinite
domains; `g2api_intercc` runs Geom2dAPI_InterCurveCurve with its points, trimmed
segments and out-of-range indices, including the upstream OCC23706 and OCC29289 curves.

Geom2dInt, Geom2dAPI: `tkgeomalgo_corpus_geom2d.py` adds 962 requests: `g2int_tool` and
`g2int_proj` run the curve tool and the projection on lines, conics, rational and
polynomial Bezier and B-spline curves (several C1 intervals, periodic), trimmed and offset
curves; `g2int_conic_curve` intersects every conic with every such curve through both
conic-curve classes (whole, inner and unbounded domains, the reversed flag);
`g2int_functions` evaluates the distance and conic functions; `g2api_extrema`,
`g2api_project`, `g2api_interpolate` and `g2api_points_to_bspline` call every member of the
four Geom2dAPI classes, with out-of-range indices, reversed ranges, periodic and
non-periodic interpolations with parameters and tangents, and every PointsToBSpline Init.

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

IntImp: 146 requests. `ii_choix` covers `ChoixRef` inside and outside 0..3;
`ii_tangence` drives `IntImp_ComputeTangence` through its early exits (null
derivatives and normals), tangent patches and the sorted isoparametrics, with seeded
random derivatives. The templates run through their native TKGeomAlgo instances and
the Rust generics: `ii_zerpar` (`IntWalk_TheFunctionOfTheInt2S`), `ii_int2s`
(`IntWalk_TheInt2S` from a start point and on each isoparametric, inside and beyond
the patch bounds, on tangent patches), `ii_zerimp` (`IntPatch_TheSurfFunction` on
patches against quadrics) and `ii_intcs` (`IntCurveSurface_TheExactHInter` on lines and
circles against planes, cylinders, spheres and tori, with and without margin).

IntWalk: 717 requests. `pw_walk` marches IntWalk_PWalking between 28 pairs of planes,
cylinders, spheres, tori, cones, Bezier and B-spline surfaces (closed circles, open lines
ending on the boundaries, a line through a corner, Viviani's curve through its tangent
double point, a B-spline tangent to a plane on a flat zone, near-tangent and tangent
contacts, identical surfaces, starts without solution) with seven deflection, confusion
and step sets, both constructors, the Perform with a parameter box, PutToBoundary and
SeekAdditionalPoints, and reports every consultation member with the out-of-range
indices; `pw_first` runs PerformFirstPoint; `pw_edit` TestDeflection, TestArret,
RepartirOuDiviser, RemoveAPoint (below, inside and past the line) and AddAPoint after a
Perform; `pw_min` the protected searches through a probe subclass (the gradient and
Newton minimizations with and without steps, the singular point on the borders,
ComputePasInit, SeekPointOnBoundary at both ends, ExtendLineInCommonZone on each
isoparametric). `iw_walk` instantiates `IntWalk_IWalking.gxx` on a probe class with
IntPatch_TheSurfFunction: spheres, planes, cylinders and cones (one of radius 0, whose
lines cross at its apex) walked on plane, cylinder, sphere and torus patches from path
points on the boundaries (with a passing point and multiplicities on a seam), from
interior points (closed lines, lines that open on the border or reach the degenerated
border at a pole), reversed, with the hole filling, with a function tolerance under the
roots (the steps collapse into lonely ends), the open-line Perform after the full one
and an object never performed; each line reports its points, ends, passing points and
tangency.

AppBlend: 115 requests. `ab_appsurf` instantiates `AppBlend_AppSurf.gxx` on a probe
line and section generator (rational quadratic sections with 0 to 2 extra 2d curves,
with or without derivatives) and compares every query after `Perform`,
`Perform(SpApprox)`, `PerformSmoothing` and `Perform(NbMaxP)`, each run twice, with
known parameters, every parametrization and continuity, the no-tangent switch,
invalid criterion weights and a null line; `ab_context` covers the two switches of
`AppBlend_Debug.cxx`.

HatchGen, TopClass, Geom2dHatch: `tkgeomalgo_corpus_hatchgen.py` and
`tkgeomalgo_corpus_geom2dhatch.py` add 1548 requests (432 and 1116). `hgen_element` builds the
point on element of every pair of hatching and element transitions (including the raising
TOUCH-against-crossing pair), changes one field of a copy and compares both ways within zero,
small and huge confusions; `hgen_hatching` and `hgen_domain` cover the point merges, the
order tests, every domain setter and the dumps. `g2hatch_local`, `g2hatch_perform` and
`g2hatch_intersect` run LocalGeometry (a degenerate B-spline included), the segment Perform
(bounded and to RealLast) and Intersect with the copy and assignment of the intersector;
`g2hatch_elements`, `g2hatch_classify` and `g2hatch_fclass` run the explorer, its probing
segments, the classifier and FClass2d on polygons of both orientations, a disc, an annulus, a
half disc, regions closed by Bezier and B-spline arcs, an open boundary, INTERNAL and EXTERNAL
edges, an infinite line and collinear split edges, from points inside, outside, on edges, on
vertices and near them; `g2hatch_hatcher` hatches every such region with lines crossing it,
through its vertices, along its edges and tangent to it, with every keep-flag pair and
operation sequences (removals, tolerance and flag changes, clears, the protected members);
`g2hatch_hatching` runs every Hatching member.

Geom2dGcc: `tkgeomalgo_corpus_geom2dgcc.py` and `tkgeomalgo_corpus_geom2dgcc_circles.py` add
3968 requests. `g2gcc_qualified`, `g2gcc_tool` and `g2gcc_func1` to `g2gcc_func4` run the four
qualifier makers, both qualified-curve classes with every position, every curve-tool member
and every tangency function constructor with Value, Derivative(s), Values and InitDerivative on
lines, circles (whole and trimmed), an ellipse, a parabola, Bezier, B-spline (C0 knots,
periodic, rational), offset and trimmed curves, bounded or not. The solver operations
(`g2gcc_lin2tan`, `g2gcc_lintanobl`, `g2gcc_tancen`, `g2gcc_2tanrad`, `g2gcc_3tan`,
`g2gcc_2tanon`, their `_geo` and `_iter` classes) run every constructor overload with each
qualifier (the rejected GccEnt_noqualifier included), tangent, secant, nested and far
arguments, start parameters on and off the solutions, parallel and degenerate cases and the
`Results` members, and print every consultation from index 0 to NbSolutions + 1. The detail
masks and the skipped configurations keep off the slots native leaves uninitialized (listed in
the corpus module), and the circle-curve-on-circle function and iteration (a native crash) are
left out. The native builder zeroes the stack before each solver: the `Circ2d3TanIter` search
reads math_FunctionSetRoot's residual buffer, which `FunctionTanCuCuCu` never writes, so its
native result depends on what earlier requests left on the stack. GccAna gains 12 touching
three-circle requests (`touching_circles`, the BUC60622 configurations).

IntCurveSurface: 3077 requests (`tkgeomalgo_corpus_intcurvesurface*.py`) over a
geometry catalog of 21 curves (lines, conics, polynomial and rational Bezier and
B-spline curves with C1 knots, periodic and degree 1, offset and trimmed curves) and
29 surfaces (planes, cylinders, cones, spheres, tori, Bezier and multi-patch B-spline
surfaces, revolutions and extrusions of lines, parabolas, hyperbolas and Bezier
curves, offsets of each, trimmed surfaces; natural infinite and given domains).
`ics_hinter` runs `IntCurveSurface_HInter::Perform` on every curve against every
surface, on tangent, grazing, four-root, axis, in-plane, parallel, missing and
null-offset cases, and through its polygon and polyhedron overloads;
`ics_hinter_protected` drives each protected member through a probe subclass;
`ics_records` and `ics_intersection` the records (merged points, flags, copies,
out-of-range indices, dumps); `ics_hcurve` every curve tool member (C0 to CN
intervals, sample counts and B-spline sample parameters over partial spans and
deflections); `ics_polygon` and `ics_polyhedron` every polygon, polyhedron and tool
member on uniform and explicit samplings (degenerate triangles at poles and apexes,
too-short parameter arrays, every TriConnex case); `ics_utils` the polygon and
polyhedron utilities on degenerate data; `ics_interference`, `ics_exact` and
`ics_quadcurv` the instance classes; `ics_estlim`, `ics_project` and
`ics_interutils` the InterUtils functions.

GeomFill filling part (gfill, 568 requests): `gfill_line` reads line points;
`gfill_profiler` adds B-splines (several degrees, rational, periodic, mixed periodic and
not), Bezier curves, circles and ellipses (trimmed or not, through the C1
approximation branch) and trimmed lines to a profiler and reads it before and after
`Perform` at three knot tolerances, with wrong array sizes, indices out of range, and
knot vectors that the insertion cannot unify (the mean-knot branch, and one that
raises); `gfill_secgen` runs every generator query and `SetParam` on arrays of lower
bound 1, 0, 2 and -3; `gfill_appsurf` skins those sections through `GeomFill_AppSurf`
in the four Perform modes, with imposed parameters, and on lines shorter and longer
than the section list. `gfill_generator` rules those sections; `gfill_scheme` runs the
Stretch, Coons and Curved schemes (four and two rows) on rows of 1 to 8 poles, with and
without weights, mismatched rows, rows of lower bound 0 and 2, and a second
non-rational `Init`; `gfill_bezier` and `gfill_bspline` fill two, three and four
boundaries in the three styles (chains in every order and orientation, rational
boundaries, different knot ranges and distributions, closed boundaries, unjoined
curves, too few poles for the Coons style, the OCC27704 curves, and the four ways two
curves touch in the curved style). `gfill_bound` evaluates the three boundaries
(lines, arcs and B-splines; 2d segments on planes, cylinders and spheres) before and
after `Reparametrize` (plain, reversed, with end derivatives, on other ranges);
`gfill_corner` drives the corner state; `gfill_patch` evaluates the Coons patch, the
tangent fields of its four sides and the patch with swapped laws; `gfill_cfill` fills
three and four boundaries (constrained, free and mixed, in any order and orientation,
with a degenerated side), with and without the corner checks, with `SetDomain` and
`ReBuild`, the `Check` members' text, and a second `Init` on the same framework.

IntStart: 674 requests (`tkgeomalgo_corpus_intstart.py`) over the surface catalog of
IntCurveSurface and nine quadrics (planes, cylinders, spheres, cones including a nearly
flat one, a torus). `is_sob` runs the boundary search with the IntPatch tools on every
surface against every quadric (finite, periodic and infinite restrictions, the exact
conic-quadric intersection and the sampled one), with the regularity recheck, twice in a
row, never performed, on arcs on the quadric or tangent to it (spheres and tori of radius
1 +- 1e-9 on the unit square), with larger tolerances (Brent refinement, merged tangent
zeros), with an edge behind the domain (straight and tangent to a cylinder, off it,
skew, degenerated, regular), and with a tool whose arcs have been seen (known points,
vertices and segments, indices past the known points). `is_inside` runs the inside
search on every surface against every quadric, from start points inside, outside and on
the bounds, tangent and coincident, and without a Perform. Every consultation is printed
from index 0 to Nb + 1. The branches not reached are the state of a math_FunctionAllRoots
point at an end of the arc (`GetPointState`) and the merge with a new point already at
the same parameter of the arc.

IntPolyh: 522 requests (`tkgeomalgo_corpus_intpolyh.py`) over the IntCurveSurface surface
catalog. `ph_inter` intersects every pair of 15 surfaces (planes, cylinders, a cone, spheres,
a torus, Bezier and B-spline surfaces, a revolution, offsets and trimmed surfaces) with the
default sampling, then five pairs with given sample counts (2 to 30) and given parameters
(short arrays included), and tangent, grazing, coincident, parallel, disjoint and very
small-in-large pairs; it prints every section line and tangent zone with the reads past them
that stay in the allocated storage. `ph_maillage` drives IntPolyh_MaillageAffinage step by
step (standard, shifted and parameter fills, the enlarged zone, each refinement, the
comparison and the chaining, and the error paths of void boxes and short parameters) and
prints its whole state. `ph_tricontact` and `ph_edgecontact` test triangle pairs (crossing,
apart, coplanar, touching, null, random) and triangle-edge pairs on both surfaces (every
vertex and side crossing, coplanar overlaps and crossings, null edges, sliver triangles for
the last coordinate combinations). `ph_triangle` refines grid triangles of five surfaces
once and repeatedly; the record operations cover every member of the points, start points,
section lines, arrays, couples, edges and tools with their dumps. Not reached: the overlap
cases that need the side to project backwards (see below), a coplanar next couple
on the first chaining side, and a next point equal to the initial one.

IntPatch: 542 requests for the core records, lines and tools (`tkgeomalgo_corpus_intpatch.py`,
`_lines.py`, `_tools.py`). `ip_point` edits points by command scripts (values, parameters,
vertices and arcs of both domains with the eight transition kinds, reversal, multiplicity,
fixture ranks out of range) and compares the dumps of points without arcs, captured from the
native process output. `ip_iwline` and `ip_pathpoint` edit the walking line of
IntPatch_TheIWalking and the path points and segments of IntPatch_TheSOnBounds. `ip_wline`,
`ip_rline`, `ip_gline` and `ip_aline` build each line kind with every transition kind, run
every vertex edit on valid and out-of-range indices and run `ComputeVertexParameters` on
vertices before, on, between and after the points, on the arcs of either surface, on seam
pairs, with confused points, periods and the ImpImp creation way; the lines without arcs
are dumped in every mode. `ip_hinter`, `ip_hc2d`, `ip_arcf`, `ip_csf` and `ip_curvature` run
the tools on the IntCurveSurface surface catalog, the Geom2dInt curve catalog and the IntStart
quadrics (the unset quadric included). The IntStart (`is_*`) and IntWalk (`iw_walk`)
requests now run through the IntPatch instances on both sides. Left out: `SetPoint` on a
restriction line without points and `Bezier()`/`BSpline()` of other curves, which dereference
null handles natively.

GeomFill sweep part (gfs, 1075 requests): `gfs_shape`, `gfs_circle`, `gfs_convert` and
`gfs_tensor` cover the `GeomFill` class functions, the two arc convertors and
`GeomFill_Tensor` (104, 85, 42 and 4 requests: every parameterisation over angles
beyond a full turn, reflex and zero arcs, short and shifted arrays, uninitialized
convertors, indices outside the tensor). `gfs_trihedron` (146) evaluates every
trihedron law on lines, conics, Bezier and B-spline curves, inflections (the Frenet
singular points), a cusp, a straight B-spline, periodic and ranged curves and curves
on surfaces, with the queries, `Copy`, `SetInterval` and the guide members.
`gfs_section` (51) runs the uniform, evolved and N-section laws (a single section, equal
and coaxial circles, segments, several B-splines, periodic sections, section and path
domains, a reference surface with transformations) on arrays of the right size, of one
pole less and of other bounds. `gfs_location` (36) runs the curve-and-trihedron, draft
and guided location laws with `LocFunction::DN`, `SetTrsf`, the draft stop surface and
the guided section and rotation. `gfs_placement` (334) places points and curve sections
(conics, planar and skew B-splines and Bezier curves, lines, periodic and trimmed
curves) on five paths under five laws with the three `Perform` overloads and the four
transformations. `gfs_sweepfunc` (16), `gfs_sweep` (92) and `gfs_blendfunc` (13) cover
the sweep functions and every elementary sweep (plane, cylinder, cone, sphere, torus,
extrusion, revolution) and approximation, with the second build, the restrictions and
the traces. `gfs_sweepsec` (33) covers every kind of section generator with the
AppSweep approximation, `gfs_pipe` (105) every pipe constructor with both `Perform`
overloads, and `gfs_surface` (14) `GeomFill::Surface`. Left out: parameters below the
first sample of a corrected Frenet law (its `GetAngleAT` loops forever natively),
`LocFunction::DN` of another order, the empty pipe's `Perform`, `Resolution(1)` of a
draft law without stop surface, the evaluation of a copied guided law, `Section()` of a
guided law without section, `SetStopSurf` before `SetCurve`, `Section` and
`ModifiedSection` of a point section, and the evaluators of a single-section law (arrays
of -1 poles): each reads a null or uninitialized value, or never returns, natively.

IntPatch polyhedra and polygons: 382 more requests (`tkgeomalgo_corpus_intpatch_poly.py`,
924 IntPatch requests in all). `ip_polyh` builds the polyhedron of every IntCurveSurface
catalog surface with the automatic and given subdivisions (0 clamped to 1, thin grids, poles
with null edges) and compares its points, triangles, boxes, plane equations, containment and
`TriConnex` around the first triangles. `ip_bvh` builds the BVH sets and the traversal pairs
of overlapping, far, identical and empty sets; `ip_interf` computes the interference of
crossing, tangent, coplanar and nearly coplanar polyhedra (tangent zones) and the self
interference. `ip_polyline` and `ip_polyarc` build the 2d polygons of walking and
restriction lines and of arcs (parabola deflection, resampling near the other box, offsets,
the construction errors); `ip_t3bits` edits the bit sets.

IntPatch parametric intersection: 333 requests (`tkgeomalgo_corpus_intpatch_wltool.py`,
`tkgeomalgo_corpus_intpatch_prmprm.py`, `tkgeomalgo_corpus_intpatch_extend.py` and
`tkgeomalgo_corpus_intpatch_inter.py`). `ip_purge` purges walking lines (two equal points,
equal points among a few, long straight and curved lines for the tube criteria, lines leaving
the domains at either end, periodic and C0 surfaces, uneven steps, the bad distributions,
vertices out of the line). `ip_join` joins the walking lines of two cylinders in the four end
orders, across the seam, with several candidates, a line of another kind, points at the ends,
the same cylinder twice and a plane. `ip_lc` splits every line kind at its vertices against
domains made of the fixture arcs, with and without a 3d tolerance (a probe subclass of
`Adaptor3d_TopolTool` overriding the arc iteration and `Tol3d`), and codes walking lines on
the arcs as restriction lines. `ip_ppgrid` and `ip_ppdepart` run the grid members and
`PointDepart` on crossing, tangent, disjoint and far apart surface pairs. `ip_extend` (24)
extends walking lines on two quadrics: gaps in the four end orders, shared ends, arcs of a
sphere and a plane across the seam and at a pole, a cylinder and a tilted plane, a cone near
its apex, lines whose tangents turn too much, critical points and three pieces in a row.
`ip_isingular` (23) runs the static members of `IntPatch_Intersection` on poles and flat
cones; `ip_inter` (107) runs the front end on every pair of surface kinds of the
IntCurveSurface catalog (bounded and infinite), on nearly degenerated cones and tori, with
the three flags, with start points, from a start point and on self-intersections;
`ip_prmprm` (74) runs every `Perform` overload of the parametric intersection on Bezier,
B-spline, offset and revolution surfaces with the deflection options, and `NewLine` on the
lines found. Left out: a
restriction line on neither surface, whose part between two vertices dereferences a null arc
natively.
IntPatch analytic intersections: 582 requests (`tkgeomalgo_corpus_intpatch_impimp.py`
348, `tkgeomalgo_corpus_intpatch_impprm.py` 234). `ip_impimp` (153) intersects every pair
of plane, cylinder, cone, sphere and torus patches in coaxial, secant, tangent,
coincident, disjoint and near-degenerate positions (axes 1e-9 rad from parallel, gaps and
radius differences of 1e-9 to 1e-11, a cone of semi-angle 1e-9), on bounded and natural
domains, with two tolerance sets and the flag that keeps the restriction lines; a pair
without a quadric raises Standard_ConstructionError. `ip_aline2wline` (40) converts the
analytic lines of ten pairs with four settings (point counts, tolerances, a range);
`ip_special` (89) and `ip_special_pole` (66) run every `IntPatch_SpecialPoints` member on
poles, apexes, seams and degenerate tangents. `ip_impprm` (210) intersects plane,
cylinder, cone and sphere patches with Bezier, B-spline (C0 included), offset,
revolution, extrusion and trimmed patches in both orders, with patches through the poles,
the apex and the seams, patches whose edges lie on the quadric (restriction lines),
two tolerance sets and a fine step, the start point, a second `Perform`, a far patch and
the pairs without a quadric; `ip_rstint` (24) puts the restriction vertices of a
cylinder and of plane patches on walking and restriction lines, with and without end
vertices, run once and twice. Left out: `IntPatch_RstInt` on an analytic or conic line
and `DecomposeResult` on a restriction line without a point set (both dereference a null
handle natively).

## ApproxInt and GeomInt

ApproxInt (`approx_int`) and GeomInt (`geom_int`) follow
[geomalgo-design.md](geomalgo-design.md) section 20. The five ApproxInt templates are Rust
generics: `ApproxIntMultiLine<L>` (its `LineTool` implementation is
`ApproxInt_MultiLineTool`), `ApproxIntPrmPrmSvSurfaces<T>`, `ApproxIntImpPrmSvSurfaces<T>`
and `ApproxIntApprox<B>`, with the trait `ApproxIntSvSurfaces` for `ApproxInt_SvSurfaces`
and the unit struct `ApproxIntKnotTools`. GeomInt declares every native instance as an
alias (`GeomIntWLApprox`, `GeomIntTheMultiLineOfWLApprox`, the SvSurfaces, the IntImp
functions, and the Approx and AppParCurves instances over the tkgeombase generics, as AppDef
does), and ports `GeomInt::AdjustPeriodic` (`adjust_periodic`), `GeomInt_IntSS`
(`GeomIntIntSS`), `GeomInt_LineConstructor`, `GeomInt_LineTool` and
`GeomInt_ParameterAndOrientation`. Overloads take distinct names: `GeomIntIntSS::perform`,
`perform_adaptors`, `perform_from_start`, `perform_adaptors_from_start`, with the three
approximation flags in one `GeomIntIntSSApprox`; `build_pcurves_bounds` and
`build_pcurves`; `GeomIntWLApprox::perform_on_surfaces` and `perform` (on the line alone),
with the flags and the range in one `ApproxIntRange`; the multi-line constructors
`with_sv_surfaces` and `with_line` over an `ApproxIntMultiLineLayout`. `TreatRLine` takes
its in-out curves in one `GeomIntRLineCurves`. `tools/audit_api_rows.py` reports 0 missing
and 0 stale rows for ApproxInt, GeomInt and GeomAPI.

ApproxInt and GeomInt: `tkgeomalgo_corpus_geomint.py` adds 1114 requests. `gi_intss` (312)
runs `GeomInt_IntSS` on 40 pairs of planes, cylinders, cones, spheres, tori, Bezier, B-spline,
offset, revolution and extrusion surfaces and trimmed ones, with five flag sets (with and
without approximation and 2d curves), the constructor, the swapped order, start points
inside and outside the domains, adaptors with trimmed domains, two Performs in a row and one
surface with itself, then every accessor with the indices 0 and N+1; `gapi_intss` (14) runs
`GeomAPI_IntSS`. `gi_wlapprox` (312) approximates the walking lines of 13 intersections with
six `SetParameters` sets (tolerances 1e-7 to 1e-3, degrees 1 to 8, 0 to 5 iterations, 4 to 30
points per interval, with and without tangency, the three parametrizations) over the 3d and
2d spaces, on and without the surfaces, over whole lines, sub-ranges and the default range.
`gi_mline` (105) and `gi_sv` (24) drive the multi-line (values and tangents, with the
indices outside, `MakeMLBetween`, `MakeMLOneMorePoint`, `Dump`) and the PrmPrm and ImpPrm
SvSurfaces (every member, the two stored results, the solver switch,
`FillInitialVectorOfSolution` across the periods), including a degenerated Bezier edge and a
cone apex. `gi_knots` (56) and `gi_partype` (15) run `ApproxInt_KnotTools` and
`GeomInt_WLApprox::Parameters`; `gi_lconst` (85), `gi_ltool` (41) and `gi_lines` (23) run
the line constructor and the line tool on every line kind and on the lines of 23
intersections (with the decomposition at the seams); `gi_pcurves` (47), `gi_trim` (36),
`gi_mkbsp` (7), `gi_rline` (33), `gi_adjust` (1) and `gi_porient` (3) run the static
members. Left out: `BuildPCurves` of a curve whose two projections fail (the null 2d curve
is dereferenced natively), the ImpPrm SvSurfaces on a surface that is not a quadric (the
parameters of `SeekPoint` stay uninitialized natively), and `SetTolFixTangents` and
`TolFixTangents`, declared without a definition (a call does not link).

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
- Intf records: `Intf_SectionPoint::IsOnSameEdge` compares the second argument's first
  addresses in the vertex-vertex case of the first argument. `Intf_SectionLine::IsEnd`
  returns the number of points (not 2) for the last point; `IsClosed` compares the
  first and last points (the `closed` flag is never read, and the copy constructor
  drops it); `Append`/`Prepend` of a line empty the argument. The `Dump` of section
  points and tangent zones prints nothing (their bodies are compiled out), and
  `Intf_TangentZone::Insert` returns false (its body is commented out);
  `PolygonInsert` only appends, or prepends a point inside the first range.
- `Intf_Interference::Insert(TangentZone)` inserts the shared point a second time when
  its walk comes back to it, then re-inserts the grown zone recursively.
- `Intf_InterferencePolygon2d`: the self Perform keeps the segment count of the last
  two-polygon computation; `Clean` never resets its one-segment flag between zones;
  the suppression of close candidate points shifts the points but not their
  parameter arrays.
- `Intf_Tool`: the segment arrays keep their values between calls; `Hypr2dBox`,
  `Parab2dBox` and `ParabBox` sort the parameters and border codes but not the
  coordinates; `Inters3d` merges equal parameters inside its exchange sort, so one
  pair can be removed several times; `HyprBox` bounds its segments to [-10, 10] and a
  whole domain to [-100, 100].
- The generic polygon-polyhedron interference classifies the plane point on the
  first vertex, then the first edge, before the second vertex, so a line through the
  second corner of a triangle is an EDGE point of the first edge; `TriConnex` is
  called on an edge hit and its result is unused.
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
- The GccAna solvers keep their native slips: Circ2d3Tan(circle, circle, line) tests
  Qualified2 instead of Qualified3 for its outside and unqualified lines; the
  three-circle solver measures the qualifier distances of circles 2 and 3 from the
  centre of circle 1; Circ2d3Tan(line, line, line) compacts its solutions without
  moving their qualifiers; Lin2d2Tan(circle, circle) stores some qualifiers in slot 1
  again and copies Qualified1 into qualifier2; Lin2dTanObl(circle) intersects the first
  solution for the second; Circ2d2TanOn(circle, circle, line) qualifies its general
  solutions with the radius at the bisector index (a third bisector raises
  Standard_OutOfRange) and computes the first argument parameter at the second tangency
  point; Circ2d2TanOn(line, line, circle) builds its circles at intersection point `i`;
  Circ2dTanOnRad(line, circle) truncates its side test to an integer; the `CenterOn3`
  members return the tangency point, not the centre; Circ2dTanOnRad(circle, circle)
  raises Standard_OutOfRange in its single-centre case (it writes qualifier slot 0).
- Some native paths leave array slots uninitialized and later read them (the Rbid(2)
  radius of an unqualified first circle in Circ2d2TanOn, TheSame1 of the nbsol 3 case of
  Circ2d2TanRad(circle, line), TheSame2 of an enclosing and an unqualified circle in
  Circ2d2TanRad, the side cote3(2) of two unqualified-line triples in
  Circ2d3Tan(line, line, line), the slot-2 data of Circ2dTanOnRad(point, circle), the
  argument parameters of Circ2d2TanOn(circle, point, line), the qualifier-2 slots of
  Lin2d2Tan(circle, circle) and the qualifier of Circ2d2TanRad(line, point) beyond the
  diameter). The port starts every slot at zero (default positions and circles), which
  is deterministic; the numeric corpus avoids reading those slots, since their native
  value is whatever the allocator returned.

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

### NLPlate

- NLPlate has no sources at this commit: `PACKAGES.cmake` lists it, but its
  `FILES.cmake` is empty since OCCT commit caad597813 ("Clean up dead headers #1442")
  deleted every NLPlate file. Only two deprecated aliases under
  `src/Deprecated/NCollectionAliases` refer to it: `NLPlate_StackOfPlate.hxx` (a
  typedef of `NCollection_List<Plate_Plate>`) and
  `NLPlate_SequenceOfHGPPConstraint.hxx`, which includes the deleted
  `NLPlate_HGPPConstraint.hxx`. The audit finds no public member;
  `crates/tkgeomalgo/src/nl_plate` stays an empty module.

### IntImp

- The five `.gxx` templates are generics (docs/geomalgo-design.md, "IntImp
  templates"); their tool calls can raise, and the failure reaches the caller of
  `IntImpInt2S`/`IntImpIntCS` as the native exception would.
- `IntImp_IntCS::Perform` is declared with the bounds `u0, v0, u1, v1` but defined
  (and called) with `u0, u1, v0, v1`; the port follows the definition.
- The bound check of `IntImp_Int2S::Perform` re-reads the moved start parameters, not
  the new solution, when it checks the reciprocal patch, and compares the second
  parameter there without the resolution. It then restarts on `ChoixRef` of the
  1-based index of the moved parameter (4 wraps to 0): the isoparametric of the next
  parameter, where the first check fixes the moved one.
- `IntImp_ZerImpFunc::IsTangent` uses the gradient of the last `Derivatives`/`Values`
  (zero before any): after `Value` alone it reuses a stale one. `Direction2d` of a
  vanishing 2d tangent raises `Standard_ConstructionError` from `IsTangent`.
- `IntImp_ZerParFunc` reads its isoparametric before `ComputeParameters` sets it
  (uninitialized natively; U on the first patch here), and the second
  `IntImp_Int2S` constructor leaves `tangent` uninitialized (false here).

### IntWalk

- A tangent passing path point (state 1) that an open line reaches makes
  `ComputeOpenLine` loop forever natively: the point is added again at each step (it is
  never marked processed and the previous point does not move), so the line and the
  memory grow without bound. The port keeps the loop; the corpus leaves the case out.
- The full `Perform` loads a passing path point that is not tangent (state 2) as state 11,
  a stop point, so only the tangent passing points pass. It checks the interior points
  for tangency (`IsTangentExtCheck`) before it sets the surface on the function.
- `Clear` keeps the single points, the lonely ends, the hole points and the estimated
  section ranges, so a second Perform accumulates them (the ranges grow with each
  Perform). The open-line `Perform` clears nothing and adds no dummy walking data: on a
  fresh object it reads past the data (undefined natively; the port raises
  Standard_OutOfRange), after a full Perform it reads the earlier entries and appends to
  the earlier lines.
- `TestArretAjout` sets its index to each added end in turn, so after a miss `N` holds
  the last one; on an arrival on an end added by a reopened line (a negative index),
  `ComputeOpenLine` reads `lines.Value(N)` and raises Standard_OutOfRange.
- A line that stops on the border adds `Psol` whenever it stopped on an added point or on
  the border, although `Psol` is only recomputed when `TestArretCadre` found no start
  point: a line cut on a start point ends with the `Psol` of an earlier step.
- `TestArretCadre` computes the section tangent (which can raise) and its reversal
  against the last chord and discards them; the tangent stored at the cut is
  `previousd3d`. `RemoveTwoEndPoints` of a hole point whose two ends were already taken
  raises Standard_NoSuchObject (`First` of an empty list), and `IsValidEndPoint` of a
  point that is not a hole point raises it too (the map's `operator()`).
- `IntWalk_PWalking`: the first constructor leaves the line a null handle (done is
  true); the members that read it crash natively and raise Standard_NullObject here. The
  second constructor leaves the tangency flags, the boundary index and `previoustg`
  uninitialized when its Perform returns early; they start at false and 0 here.
- `DistanceMinimizeByGradient` returns true at once unless both surfaces are Bezier or
  B-spline surfaces, keeps the gradient of its start while the steps succeed, and always
  runs until 61 failed steps. `DistanceMinimizeByExtrema` uses `D2U · D1V` as the mixed
  second derivative.
- The marching loop grows a step after five accepted points by at most a tenth of the
  current step (`t = 0.1 * pasuv`) when a quarter of the gap exceeds a tenth of the
  initial step. `RepartirOuDiviser` sets `tglast` on a step collapse only when the
  previous point is not tangent ("IS IT ENOUGH ????"). `PutToBoundary` returns the
  result of its last `SeekPointOnBoundary` only, and `IsParallel` calls a line of fewer
  than three points parallel to both boundaries.
- With a small confusion tolerance, the osculating-circle test of `TestDeflection` finds
  nearly every curved step too great, so the accepted points come from the "step too
  small" branch of the chord deflection test; the corpus also runs large tolerances.
- `IntImp_ZerImpFunc` gained `Clone`, the native copy constructor that the walking uses
  (`aFuncForDuplicate`).

### AppBlend

- Two sections narrow the stored degrees (`dmin = 1`, and `dmax = 1` without
  tangents): later computations of the same object keep them.
- `Perform` without `SpApprox` computes the reached tolerances but does not store
  them (`TolReached` keeps its earlier values), and with known parameters it raises
  `StdFail_NotDone` from `AppDef_Compute::SplineValue`.
- The continuity chain of the B-spline approximation turns C0 into continuity 3
  (`if C0 ...; if C1 ... else if C2 ... else 3`).
- `Perform(NbMaxP)` measures the pole distances of each Bezier piece on the curve of
  the piece's own index, and `NbMaxP = 0` divides by zero (a crash natively; a
  `Standard_DivideByZero` error here, not in the corpus). The native `scal` switch of
  its box scaling is a constant true.
- `Curves2dShape` returns the U pole count of the surface as the 2d pole count.
- The `AppBlend_Debug.cxx` switches are process-wide, as the native file statics.

### IntCurveSurface

- `Perform` marks the result done before it computes: an exception leaves the points
  found so far, done. `PerformBounds` clamps the patch to `1e50` (bug26525).
- `IntCurveSurface_Intersection::Append(Inter, First, Last)` ignores the range, and
  `Append(Point)` drops a point whose (u, v, w) lie within 1e-8 of a kept one with the
  same transition. `HInter::AppendSegment` is empty: no result has segments.
- `ProcessIntAna` and `PerformCurveQuadric` read the quadric parameters of each point
  into locals shared across the loop; `AppendIntAna` on a surface that is not a
  plane, cylinder, cone or sphere reads them uninitialized natively (the port reads
  0; the corpus calls it on quadrics only).
- `SortStartPoints` snaps the w (then u) of a point within `10 * PConfusion` of the
  previous one to it before the next pass, and `ProcessSortedPoints` compares each
  point with the previous unsnapped one, starting from `u - 1`.
- `EstLimForInfOffs` sets NoIntersection only for a line parallel to (or on) the
  equivalent cylinder or cone; a line meeting it nowhere widens the V bounds from
  RealLast to [-10, 10]. `EstLimForInfRevl` takes an arbitrary point `(u, u + 1,
  u + 2)` when the basis curve lies on the axis.
- `TriConnex` compares the triangle column with `nbdeltaV`, though triangle columns run
  to `2 nbdeltaV`; `ComputeBorderDeflection` starts at RealFirst; the singularity flags
  and `Point(thePnt, lig, col, U, V)` store nothing the algorithm reads (the setter is
  empty).
- The polygon tool `Dump` numbers its output by a process-wide call counter (a static
  shared by every instance of the generic in the port); the probe masks the number.
- `NbSamples` truncates a double count, `(int)` of NaN being undefined (0 here).

GeomAPI (1980 requests) and GeomFill Gordon (145 requests): `gapi_ppc` and `gapi_pps`
project points (on the geometry, at centres and axes, far away) on every curve and
surface kind through every constructor and Init, with the out-of-range indices 0 and
N+1, repeated Performs and switched Extrema algorithms and flags; `gapi_ecc`, `gapi_ecs`
and `gapi_ess` run the extrema of curve pairs (parallel, concentric, tangent, repeated
Inits on one object, the Total members), curve-surface and surface-surface pairs on
whole and bounded domains; `gapi_to2d`/`gapi_to3d` the plane conversions of every curve
kind; `gapi_intcs` curve-surface intersections with points and segments;
`gapi_interpolate` periodic and open interpolations with parameters, end and full
tangents, and too-close points; `gapi_points_to_bspline` and `gapi_p2bss` every
approximation overload (least squares, smoothing, interpolation; every
parametrization; periodic; heights; grids whose bounds do not start at 1).
`gfill_gordon` runs 38 networks (trimmed-line grids, bumps, saddles, rational and
degree-9 rational networks, periodic and closed curves, meanders with several contacts,
arcs, Bezier and cubic iso-curve networks, disjoint and too small networks) in exact
mode, in approximate-fallback mode with the parallel flag and two Performs, and at a
second tolerance, printing every status and report field and the surface;
`gfill_network` runs prepared networks: closed seams (with gaps within and beyond the
tolerance, non-unit parameters), compatible and incompatible rational families, the
degree overflow, zero and negative weights, too few curves, a wrong grid size and two
Performs. The hyperbola-circle pair of `gapi_ecc` is left out: natively it does not
return.

GeomPlate (163 requests, `tkgeomalgo_corpus_geomplate.py`): `gplate_point` and
`gplate_curve` read every member of the point and curve constraints (points, 3d lines,
circles and Bezier curves, 2d lines and Bezier curves on planes, cylinders, spheres and
Bezier surfaces; criterion laws; projected curves and their approximation; the setters;
the order and law exceptions), `gplate_curve_empty` the empty curve constraint;
`gplate_surface` evaluates GeomPlate_Surface (D0 to D2, the D3/DN/IsCNu exceptions,
bounds, a copy and a plate surface on a plate surface) on five initial surfaces;
`gplate_average`, `gplate_normals` and `gplate_halfspace` run the average plane of point
clouds (every option, lines, degenerate clouds), of normal sets (one, two, opposite,
many) and successive HalfSpace calls; `gplate_build` performs 47 builders (the three
constructors; point constraints of order G0 to G2; closed and open Bezier contours with
and without initial surface; G1 and G2 contours of curves on planes, cylinders and
spheres, reordered and reversed; mixed G0/G1 and G1/G1 corners; criterion laws; degrees 2
to 6, one to four iterations, tolerances and the anisotropy flag; the non-linear first
plate with the free and incremental loops; construction errors) and reads every result,
the out-of-range constraint indices and both contours; `gplate_approx` runs MakeApprox
with CritOrder -1 to 2, C0 to C2 (and the refused G1), and explicit G0 and G1 criteria.
Values that native OCCT leaves uninitialized are not read (see the quirks below), and a
glibc `MALLOC_PERTURB_` run of the native probe gives the same output for every request.

### GeomFill (filling part)

- `GeomFill_SectionGenerator::SetParam` shares the given array and copies it into itself
  from index 1 (`myParams(i) = Params(Lower + i - 1)` on the same array): a lower bound
  of 0 spreads the first value over the array, then raises Standard_OutOfRange at the
  last index; any lower bound above 1 raises at once. The native constructor sets the
  parameters only when the empty profile holds sections, so `Parameter` without
  `SetParam` dereferences a null handle (a crash natively, Standard_NullObject here;
  not in the corpus), and `GeomFill_AppSurf` with known parameters needs `SetParam`.
- `GeomFill_SectionGenerator::Section` never computes derivatives and leaves the 2d
  poles unchanged; the poles and weights are assigned, so the arrays take the section's
  length.
- `GeomFill_Profiler::Perform` segments periodic curves of a non-periodic profile,
  and when the knot insertion leaves different knot counts it sets the mean inner knots
  on the original curves, which raises when their knot counts differ. The consultation
  members all name `Degree` in their StdFail_NotDone message.
- `GeomFill_Coons` puts `P2` and `P4` on the first and last rows of the grid, while
  `GeomFill_Stretch`, `GeomFill_Curved` and the Coons weights put `W4`/`P4` first and
  `W2`/`P2` last; `GeomFill_BezierCurves` and `GeomFill_BSplineCurves` pass the Coons
  scheme `(P1, P4, P3, P2)`. With fewer than four poles in a direction the Coons
  blending reads the cubic Hermite poles directly. A non-rational `Init` of a scheme
  keeps the weights of an earlier rational one (`Weights` still returns them). The
  weighted two-row `GeomFill_Curved` sizes its weight grid by the weight rows.
- `Init(C1, C2)` of `GeomFill_BezierCurves` and `GeomFill_BSplineCurves` in the curved
  style reverses the caller's `C1` (not its copy) when the end of `C1` meets the start
  of `C2`, and raises Standard_OutOfRange when the curves do not touch. The B-spline
  ruled surface is periodic in U only when a curve is rational (the non-rational
  constructor call omits the flag). `SetSameDistribution` always inserts knots
  through the weights, also for non-rational curves.
- `GeomFill_CoonsAlgPatch::Value(U, V)` evaluates all four boundaries at `V`; `D1U`
  and `D1V` evaluate them at `U` or `V` by side. `GeomFill_TgtOnCoons::D1(W)` takes its
  second-order term from the corrected tangent, `D1(W, T, DT)` from the uncorrected
  one, so the two derivatives differ. `GeomFill_BoundWithSurf::D1Norm` at a point of
  null Gram determinant multiplies the normal difference by `1e-12` instead of
  dividing by it.
- `GeomFill_ConstrainedFilling::MatchKnots` compares the domains with the knot
  multiplicities (not the knots) when it looks for the index of a domain knot, so a
  `Law::MixTgt` pivot moves only when a multiplicity lies within the tolerance of a
  domain. A second `Init` keeps the tangent
  fields, tangent poles, corner states (gaps included) and corner twists of the first
  where it sets none; `SetDomain` matches the sorted boundaries by identity and
  ignores any other boundary. `GeomFill_SimpleBound` and `GeomFill_BoundWithSurf`
  store their new `Law_BSpFunc` before `SetCurve`.
- `GeomFill_Coons` with fewer than two poles in a direction allocates an array of
  negative size natively (Standard_RangeError here; not in the corpus). A default
  scheme (no `Init`) dereferences a null grid in `NbUPoles`, `Poles` and `Weights`
  (Standard_NullObject here; not in the corpus).

### IntStart

- `SearchOnBoundaries`: `FindVertex` gives the vertex solutions the boundary tolerance
  as their tolerance. A new point of `PointProcess` has 1000 times the tolerance, at
  most 0.005 (#24643), where `TreatLC` caps its point at 0.001; the new point is the
  last point the loops copied, so it carries that point's vertex (unread, since
  `Vertex()` raises on a new point).
- A rejected arc (the six first-order samples keep one sign) is not solution. The
  regularity recheck runs only when isolated zeros exist (`Nbp > 0`), and without the
  recheck the null intervals are dropped whenever an isolated zero exists.
- `TreatLC` and `IsRegularity` read the edge behind the domain: with
  `Adaptor3d_TopolTool`, whose `Edge()` is null, the tangent line on a cylinder is
  never special and no edge is regular. The branch of arcs that "have been seen" is
  unreachable with IntPatch_HInterTool (its `HasBeenSeen` is false); the probe reaches
  it through its own tool.
- `ComputeBoundsfromInfinite` takes two Newton steps from far points and widens the
  result tenfold; its `NbEchant` output is not read. `(1e10, -1e10)` is returned when
  the distance is flat at 0.
- The tangent zeros of an exact line intersection are merged only when the three
  distances have one sign; the node count of the minimization is `ceil((b - a) / tol)`
  with the parameters clamped to 1e9 ("floating point exception 569").
- `SearchInside`: the sampling steps are the half-cells `2 (umax - umin) / NbSamplesU`,
  the solver box is clipped with `std::max`/`std::min`, and the twelve extra starts are
  the four corners at 2% of the steps twice, then at 0.5%. The classification tolerance
  is `1000` times the larger resolution, at most 0.1% of the steps.

### IntPolyh

- `IntPolyh_Array::Init(N)` sets the item `N` of its dynamic array: the storage holds `N + 1`
  items, and a second `Init` resets only that item, so the points, edges and triangles of a
  second fill (the enlarged-zone retry of `PerformMaillage`) keep the degenerated flags, the
  intersection flags, deflections and cached boxes of the first one where they are not
  rewritten. `operator=` (`Copy`) keeps the target's increment; the one-`int` constructor
  cannot be called natively (it is ambiguous with the `(N, increment)` one). `Dump` titles
  any array "ArrayOfSectionLines" and compiles only for points, start points and lines.
- `IntPolyh_SectionLine` holds a spare point: `NbStartPoints()` is its length minus one
  (-1 for a line never initialized), `Init` ignores its argument, `Destroy` is empty.
- `IntPolyh_Point::Divide` returns the default point for a divisor not above
  Precision::Computational (bug #18); the dumps print `%+8.3eg`, a stray `g` after each
  number. `IntPolyh_StartPoint::CheckSameSP` compares `lambda1` with the other `lambda1`
  (its comment says `lambda2`) and prints both points (numbered 128 and 129, the octal
  `00200` and `00201`) when both edges are unknown.
- `FillArrayOfPnt(SurfID, isShiftFwd, Upars, Vpars, theDeflTol)` always fills the first
  surface (it passes 1 to the overload it calls). `IsDegenerated` samples the isoline at
  `i * step`, not from the first bound. `CommonBox(B1, B2, ...)` ignores its box arguments.
  `GetFinTE` and `GetFinTT` are declared but defined nowhere (excluded rows).
- `TriContact` leaves the angle unchanged when a normal is null, so `TriangleCompare` stores
  the angle of the previous couple in contact. `MiddleRefinement` numbers the new triangles
  and edges differently for each longest side and leaves the split edge pointing at the two
  killed triangles.
- `CalculPtsInterTriEdgeCoplanaires` divides the edge and side vectors by their squared
  magnitudes before `IsParallel`; on the crossing, the end `alpha == 0` is overwritten by
  the inner case (the second test has no `else`) and the second point of surface 2 reads
  the side orientation from the first triangle; on the overlap, surface 2 mixes the U of
  one point with the V of the other, both points take `alpha1` and `lambda1`, one case
  starts `PEP2` from `PE2`, and the two cases where the side's end projects before its start
  need rounding to happen (the side runs from `PT1` to `PT2`, so only a side far shorter
  than its coordinates can project backwards). `TriangleEdgeContact` compares
  `V2` with `V1` after the first coplanar pair and gives surface 2 `alpha` instead of `beta`
  on the side `PT2PT3`.
- `StartPointsChain` stores several points of the next couple in the tangent zones on the
  first side only: on the second side the test sits in the `else` of the single-point case.
  The chaining shares one next point between both sides of a couple, so a step that finds
  several points stores the previous point with the new couple.
- `IntPolyh_Intersection::NbPointsInLine` past the lines answers -1 (the allocated empty
  lines) and `GetTangentZonePoint` past the zones the default point; `AnalyzeIntersection`
  never fails, so `IsDone` is always true once the constructor returns.

### IntPatch

- `IntPatch_Point::SetValue(Pt, Tol, Tangent)` clears the domain, vertex and multiplicity
  flags but keeps the arcs, vertices and transitions; `ReverseTransition` replaces each
  transition by a fresh one, so a TOUCH or undecided transition becomes the default
  undecided one. `IsTangencyPoint` never raises, despite its documentation.
- `IntPatch_WLine`: the first and last indices are uninitialized until set.
  `ComputeVertexParameters`: in the seam phase a vertex that already exists on the second
  restriction resets `encoreunefois` even after the first restriction created one; a vertex
  before the line inserted as its first point shifts the parameters of the vertices `j != 1`
  (not `j != i`); a first vertex prepended at the end does not shift `indl`; the ImpImp way
  keeps the 2d parameters of the line point and takes the 3d point of the vertex. The
  confused points are removed while more than 2 points are left, then after a
  substitution while more than 1 is. `InsertVertexBefore` clamps a copy of its index to 1
  but inserts at the original one (Standard_OutOfRange below 1 on a non-empty line).
- `IntPatch_RLine::ComputeVertexParameters` ignores its tolerance and always makes the
  vertices 1 and `nbvtx` the first and last points; its duplicate filter also compares the
  parameters on the arcs, and skips the pair when the arcs differ.
- `IntPatch_GLine::Replace` appends the point and ignores the index (the assignment is
  commented out). `AddVertex` brings a circle or ellipse parameter into `[first, last]` by
  turns of 2 Pi and drops a point still 1e-6 before the first one; on the other conics it
  drops the points outside `[first, last]`. `ComputeVertexParameters` shifts the indices
  for `indl > j` (the other lines use `>=`), its S2 filter tests `IsVertexOnS1`, the seam
  moves stop after the next sort once a pair leaves the U range of the first pair
  (BUC61033), and at the end it resets `indf`/`indl` to 1 and `nbvtx` without setting the
  `fipt`/`lapt` flags.
- `IntPatch_ALine::ComputeVertexParameters` decreases `indl` for each end vertex removed
  (and never `indf`); the branch that would prepend a copy of the last vertex at the first
  parameter of a periodic line tests the first vertex against the first parameter inside the
  branch where they are equal, so it never runs.
- `IntPatch_HInterTool`: `NbSamplesU/V` ignore their range, `SamplePoint` perturbs the
  third cell of four by `dv *= 1.001; dv *= 0.999`, the `SingularOn*` members answer false,
  and the known-point members raise Standard_OutOfRange (no arc has been seen).
  `IntPatch_HCurve2dTool::NbSamples` answers 25 for the non-spline curves and caps at 50.
- `IntPatch_ArcFunction::Values` with a quadric that is not set leaves `F` unchanged and
  gives a null derivative (the quadric writes nothing).
- `IntPatch_Polyhedron::PlaneEquation` leaves the polar distance unchanged on a null edge;
  `Dump` writes nothing and `Point(P, i, j, U, V)` does nothing. The bounding tests keep the
  native positive comparisons, so a NaN coordinate gives the native boxes.
- `IntPatch_InterferencePolyhedron`: `Perform(Obje)` (one polyhedron) only clears the
  result and sets the tolerance, so it finds nothing, while the constructor of one
  polyhedron computes the self interference. `Intf_TangentZone::Insert` never accepts a
  point, so the edge crossings are offered twice and dropped. A segment already found on
  either triangle (both are checked, no short circuit) still inserts its two points.
- `IntPatch_PolyLine`: the line type is uninitialized until a line is set. `IntPatch_Polygo::Dump`
  numbers its dumps with a static counter shared by every polygon.
- `IntPatch_PrmPrmIntersection_T3Bits` allocates `(size^3) >> 5` words: the bits of a last
  incomplete word are not stored, and for `size^3 < 32` the native constructor writes before
  its empty array (undefined behaviour).

### IntPatch parametric intersection

- `IntPatch_WLineTool::ComputePurgedWLine` compares the point count of the last pass of
  its equal-points step with 30 to skip the other steps; a vertex whose parameter is out
  of the line raises Standard_OutOfRange in the later steps, and a line without vertices
  that leaves a domain raises it when its end vertex is moved. `JoinWLines` reads the
  radii of both surfaces as cylinders (Standard_NoSuchObject otherwise), even for lines
  that will not be joined.
- `IntPatch_LineConstructor`: the mode is unused. On a conic line the tolerance of the
  first domain is taken when the second domain has an arc (`myDom1->Init(); if
  (myDom2->More())`), the same test is made on a restriction on the second surface, and
  the tolerance persists from one part to the next; with `Adaptor3d_TopolTool` itself,
  `Tol3d` raises Standard_DomainError on a bounded domain. A restriction part gets the
  last vertex index 2 even for a single vertex (`AppendSameVertexR` is compiled out).
  UNKNOWN states count as OUT on conic lines only.
- `IntPatch_PrmPrmIntersection::NewLine` adds the point `Indice`, not `Indice + 1`, when it
  skips the points before the sampled abscissa, and compares `1 / (a + b)` with the step.
  `PointDepart` clears the neighbours of a cell only in the first map. `RemplitLin` sets
  only the first cell of each halved segment, so between cells two apart the middle cell
  stays unset.
- `IntPatch_WLineTool::ExtendTwoWLines` keeps references to the end points of the lines
  while it inserts points, so a reference taken before an insertion at the start of a line
  reads the new point after it; the tests that disable an extension read the bits set by
  the previous lines of the sequence.
- `IntPatch_PrmPrmIntersection::Perform`: the images of the start points on the opposite
  bound of a closed direction share one point, whose 3d point is the origin; the flag of
  the periodic adjustment of a new point is not reset between the parameters; the
  processing of several walking lines keeps the vertex count of the first line when the
  line is replaced; in the polyhedral overloads the first point of a section line is read
  into the parameter minima of the previous line.
- `IntPatch_Intersection::Dump` does nothing (its body needs `DUMPOFIntPatch_Intersection`).
  `SetTolerances` clamps the values with two tests that keep a NaN. The infinite surfaces
  are trimmed on the V bounds of both surfaces, then on the U bounds.

### IntPatch analytic intersections

- `IntPatch_ImpImpIntersection`: the plane-plane and cone-torus pairs do not update the
  empty flag, and the plane-cone pair answers a failure for an empty result. `IntCyCo`
  gives its tangent point the parameters of the first quadric twice. `PointOnSecondDom`
  reads the vertex count again after each vertex (the added vertices are visited too) and
  leaves the current point at the last one its vertex search read. `PutPointsOnLine`
  keeps the vertex and the underlying vertex of the previous point for a new point. The
  restriction line check keeps the side, arc and range of the previous line when a line
  has no arc or no end point. `FindLine` replaces the parameter list with those of an
  accepted analytic line but appends those of the arc search and of the chosen conic.
  `IntCySp` does not test the open ends; `IntCyCo`, `IntCoCo` and `IntCoSp` evaluate the
  curve at the last parameter tried when no tangent is found, which can raise.
- `IntPatch_RstInt::PutVertexOnLine`: the counter of the periodic shifts is not reset
  between the V shifts (a torus runs the three U shifts with the first V shift only); the
  V2 loop of `Recadre` on a torus compares with V1 of the point first; the result of
  `Project` is not tested (the parameter is kept when the projection fails); the end
  points of an arc at its vertices are computed and unused.
- `IntPatch_ImpPrmIntersection`: `DecomposeResult` adjusts the point set of each line in
  place (U in `[0, 2 Pi]`, snapped to the bounds, seam points inserted) even when the line
  is not decomposed, and a line of at most two points or a restriction line that is not an
  isoline is dropped from the result when another line is decomposed. `ToSmooth` of the
  last end runs its sampling loop from `NbPoints - max(NbPoints/5, 5) - 2` to
  `max(NbPoints/5, 5)`, which iterates only on lines of 11 and 12 points: elsewhere the
  mean step is 0, a smoothed last point takes the U of its neighbour exactly, and
  `AddVertices` never adds the last vertex. The torus case
  of the decomposition is unreachable (a torus is no quadric of this intersection), and
  `SplitOnSegments` is called with `Tang` false. The trimmed arc of a restriction part is
  discarded (the call only raises).
- `IntPatch_ALineToWLine::MakeWLine` keeps the stale reads of its loops (the parameters of
  the reference point reused for the special points) and the fall-throughs of
  `IsPoleOrSeam`; `IntPatch_SpecialPoints::ProcessCone` keeps its input U when no
  generatrix is chosen, and `FuncPreciseSeam` answers false on any Standard_Failure.

### ApproxInt and GeomInt

- `ApproxInt_ImpPrmSvSurfaces::Compute` with the quadric second stores the parameters on S2
  as `(tu2, tu2)`, so the stored result matches a later call only when `u2 == v2`; the loops
  meant to bring the quadric U within Pi of its input compare it with itself and do nothing,
  and the quadric derivatives are taken at the input parameters. `SeekPoint` sets the stored
  point and the computed flag but not the stored parameters. A start point that cannot be
  brought into a non-periodic range clears the tangent flag of both stored results.
  The native function keeps pointers to the quadric and to the surface handle given to the
  constructor, which must outlive it (`GeomInt_WLApprox` keeps them on its stack).
- `ApproxInt_PrmPrmSvSurfaces::Compute`: a solver that is not done keeps the stored tangent
  flag; an empty result clears it.
- `ApproxInt_MultiLine::MakeMLBetween` keeps the SvSurfaces on the new line only for more
  than 10 source intervals, answers an empty line of indices 1..1 when too few points were
  made or the line turns too sharply in a parameter plane, and restores the solver switch
  only when it returns (an exception leaves it on). `Dump` prints with `printf`.
- `ApproxInt_KnotTools::BuildKnots` appends to the output without clearing it (only the early
  exit clears it); `BuildCurvature` keeps its Lagrange buffers from one point to the next.
- `ApproxInt_Approx`: `SetParameters` without tangency sets the PassPoint constraints, which a
  later call with tangency does not reset; `NbMultiCurves` is always 1 and `Value` ignores its
  index; a knot interval without a Bezier piece ends the approximation before the pieces are
  joined; `Perform()` is declared without a definition.
- `GeomInt_IntSS::Perform` clears the 3d curves but not the 2d curve sequences, so after a
  second Perform `LineOnS1(i)` and `LineOnS2(i)` read the curves of the first one;
  `LineOnS1` does not skip the boundaries as `Line` does. With a start point outside a
  domain the intersector is not run and its previous result is read again. The in-out
  tolerance of the 2d curves grows from part to part; the circle parts keep their untrimmed
  2d curves, and a single whole turn ends the loop. `SetTolFixTangents` and
  `TolFixTangents` are declared without a definition.
- `GeomInt_LineConstructor`: the walking parts of a plane with an extrusion or a revolution
  are merged by the integer part of their parameters; a restriction line without vertex on a
  domain is classified at its first free vertex.

## ApproxInt and GeomInt deviations

- The `void*` to the SvSurfaces is a shared `Rc<RefCell<dyn ApproxIntSvSurfaces>>`, and the
  ImpPrm and IntImp functions hold their quadric and surface by value where the native ones
  keep pointers to the constructor arguments.
- A null dereference of the native code raises Standard_NullObject: `BuildPCurves` when both
  projections fail, the line constructor before `Load`, a restriction line without arc in
  `TreatRLine`, a missing line in the multi-line.
- `NCollection_LocalArray` is a slice and `NCollection_DynamicArray` a `Vec`; the Lagrange
  result buffer of `BuildCurvature` starts at 0 where the native one is uninitialized until
  the first successful evaluation.
- `MakeMLBetween` with `High < Low` raises Standard_OutOfRange (`High == Low - 1`, the native
  write into an empty array) or Standard_OutOfMemory (the native allocation of a negative
  size).

## IntStart deviations

- The native code ignores the result of `Func.Value` in `FindVertex`, the recheck
  samples and `ComputeBoundsfromInfinite`, and reads the output variable, which is
  uninitialized (or the previous value) when the function answers false. The port keeps
  the previous value, 0 at first; the IntPatch and Contap functions never answer false.
- `TheTopolTool` is a trait, as the native domain is virtual; `Edge()` returns the edge
  itself (`Option<Edge>`) where the native one returns a `void*`.

## IntPolyh deviations

- The native `Value` and `operator[]` of the arrays do not check the index: past the storage
  the read is undefined. The port returns Standard_OutOfRange from `value`/`change_value` and
  panics in `Index`/`IndexMut`; the corpus reads only inside the storage.
- `MiddleRefinement` and `MultipleMiddleRefinement` run on an item of the array they grow;
  the port refines a copy and writes it back, which gives the native result (the body never
  writes the refined triangle through the array before its last two assignments).
- `StartPointsChain` and `GetNextChainStartPoint` walk the couples by position in a vector
  taken from the list and put back in order; the positions stand for the native iterators.
- The diagnostics of `CheckSameSP` and of a null side in `TriangleEdgeContact` print to
  standard output, as natively; the dumps write to a `fmt::Write`.

### GeomAPI

- `GeomAPI_ExtremaCurveCurve::Init(C1, C2, U1min, U1max, U2min, U2max)` loads the whole
  curves into its adaptors: the Total members (`TotalPerform`) work on the full curves,
  not on the ranges. `TotalPerform` compares each pair of end points with the squared
  distance from before the end-point checks (it never updates it), so the last pair
  nearer than that distance wins, not the nearest one; the NaN comparisons keep their
  native result.
- `GeomAPI_ExtremaCurveSurface` is done when the Extrema answers parallel, even with 0
  extrema; its Extrema object is built on local adaptors at each Init.
- `GeomAPI_Interpolate::ClearTangents` is declared but never defined natively; its row is
  excluded. The interpolation keeps the Geom2dAPI quirks (the shared flag array, the
  previous-parameter read of BuildParameters).
- `GeomAPI_PointsToBSpline::Init(Points, Params, ...)` reads `Params(i)` from the lower
  bound of `Points`; `GeomAPI_PointsToBSplineSurface` sets its multipoints at the grid
  indices, so a grid that does not start at (1, 1) raises Standard_OutOfRange.
  `operator occ::handle<Geom_BSplineSurface>()` calls `Surface()` and raises
  StdFail_NotDone when not done.
- Extrema_ExtPC on a periodic B-spline far outside its period (bounds near 1e100)
  computes a period shift that overflows `int`; natively the overflow is undefined, the
  port wraps as x86-64 does (`wrapping_sub`), which the corpus checks.

### GeomFill Gordon

- `SetParallelMode(true)` runs the contact search sequentially: each search is
  independent and writes its own slot, so the contacts are the same; `IsParallelMode`
  still answers true.
- `GeomFill_NetworkSurface::Perform` makes the curve families compatible in place (the
  stored copies are raised in degree and knot-refined), so a second Perform starts from
  the prepared curves; its catch-all turns every exception into ConstructionFailed.
- The approximate fallback measures its profile and guide deviations by mapping the fixed
  parameter through the surface parameter mapping twice, as natively.

### GeomFill (sweep part)

- `GeomFill_TrihedronLaw::D1` and `D2` both raise Standard_NotImplemented with the
  message ` GeomFill_TrihedronLaw::D2`. Frenet-based laws raise Standard_OutOfRange for
  the G1 and G2 continuities in `NbIntervals`/`Intervals`.
- `GeomFill_CorrectedFrenet::GetAngleAT` loops forever for a parameter below the first
  sampled one (not in the corpus). `GeomFill_DiscreteTrihedron::Init` appends to the
  samples of an earlier curve. `GeomFill_DraftTrihedron::GetAverageLaw` returns the
  plain mean of 21 frames, not normalized.
- `GeomFill_GuideTrihedronPlan` declares its own `myCurve` and `myTrimmed`, hiding the
  base members, so the base `GetInterval` dereferences a null curve (not in the
  corpus); its `D2` always answers false and `Origine` does nothing.
- `GeomFill_PlanFunc::D2` is empty and `D2E` writes only `DFDT`.
  `GeomFill_FunctionGuide::DSDT` prints `Not implemented` on the standard output for a
  varying section.
- `GeomFill_QuasiAngularConvertor` uses `-2/21` and the `b + 11` offset in its second
  derivative, and its first and second derivatives compute the weight derivative with
  different formulas; both are kept.
- `GeomFill_LocationDraft::D2` answers true when the trace search fails.
  `GeomFill_LocationGuide::GetAverageLaw` averages 22 path points spaced by a tenth of
  the interval (the last beyond its end). A guided law transforms its output matrices
  before writing them, so the transformation shows only when a rotating law answers
  false.
- `GeomFill_SweepFunction::SetInterval` maps the path interval to the section from
  `myf`, not from `myfOnS`; the function keeps the placement of its last evaluation and
  hands it to the next one.
- `GeomFill_Sweep::Build` with `GeomFill_Location` reads the section's B-spline surface
  and drops it; a second `Build` keeps the 2d curves of the first approximation.
  `GeomFill_Sweep` stores `myExchUV`, `isUReversed` and `isVReversed` only in `Build`.
- Along a curve, `GeomFill_SweepSectionGenerator::Perform` composes a transformation
  that is set only when the tangent turns back, so every section stays in place; a
  second `Perform` appends to the transformations. The rail kind reports seven poles
  whatever the parameterisation (the polynomial arcs have eight).
- `GeomFill_NSections::ComputeSurface` reparameterises a B-spline section of the caller
  in place; `BarycentreOfSurf` does not restart U at each row; with fewer than two
  sections there is no surface, the evaluators answer false and `SectionShape`,
  `Knots` and `Mults` leave their outputs unchanged.
- `GeomFill_Pipe(Path, FirstSect, Option)` builds a Frenet law for `IsDarboux` and
  raises Standard_ConstructionError for the guided and discrete options.
  `CheckSense` reverses the placed sections in place. A two-section pipe whose law
  refuses the path is `GeomFill_ImpossibleContact` and its `Perform` does nothing, while
  `Perform(WithParameters, Polynomial)` then falls to `ApproxSurf`, which raises
  Standard_ConstructionError. `KPartT4` keeps the untrimmed cylinder when its trim
  raises. The empty pipe's `Perform` dereferences null adaptors (not in the corpus).
- `GeomFill::Surface` leaves two trimmed coaxial circles to `GeomFill_Generator`.

### GeomPlate

- `Perform` without constraint returns with the plate reset, so `IsDone()` is true and
  `Surface()` null, as the native test notes.
- `G0Error(Index)`, `G1Error(Index)` and `G2Error(Index)` gather the gaps into arrays of
  `NbPtsOnCur` values while `EcartContraintesMil` writes one value per interval of the
  curve: a curve with fewer intervals (or of order -1) leaves values uninitialized
  natively (zeros here; never read by the corpus), one with more raises
  Standard_OutOfRange. The global errors are uninitialized until a check ran (points
  only, a failed plate).
- `EcartContraintesMil` and `VerifSurface` read at least three intervals: a curve with
  fewer than three parameters reads past them (Standard_OutOfRange).
- The G2 constraints pass the second derivatives `(D2U, D2V, D2UV)` to
  `Plate_D2(duu, duv, dvv)`, so D2V becomes `duv` and D2UV `dvv`, for the target and the
  initial surface alike.
- `VerifPoints` overwrites its distance, angle and curvature with each point, so only the
  last point constraint of each order counts; the results are discarded by `Perform`.
- `Disc2dContour` and `Disc3dContour` start their samples at the second interval (the
  quarter points of the first interval are skipped), ignore their count argument, and
  initialize the projector (with the current tolerances in 2d, the resolutions of Tol3d
  in 3d).
- `ComputeSurfInit` sums the curve lengths for a point count it never uses, and its
  non-linear branch leaves the intermediate plate surface in `Surface()` when the later
  plate fails.
- When a curve has no continuous projection, `Perform` approximates the current plate
  surface, which it has just nullified unless the non-linear branch set it: natively a
  null dereference (Standard_NullObject here, not in the corpus).
- `GeomPlate_Surface::Copy` keeps the initial surface and the plate but not the bounds;
  `UPeriod`/`VPeriod` return 0 (a `false` converted to a real) instead of raising;
  `Continuity` is C0, `IsCNu`/`IsCNv` raise Standard_Failure, `EvalD3`/`EvalDN` raise
  Geom_UndefinedDerivative, and the reversing and transforming members do nothing.
- `GeomPlate_MakeApprox` with a CritOrder outside -1..=1 computes nothing and leaves both
  errors uninitialized (zero here).
- `GeomPlate_BuildAveragePlane` leaves its box uninitialized when no plane is found, and
  a flux normal (option 2) with no boundary points gives a null frame whose line raises
  Standard_ConstructionError; the normals constructor raises Standard_OutOfRange without
  normals (`OptVec(0)`).
- `GeomPlate_PointConstraint(Pt, Order)` accepts orders -1 to 1 although its message
  says "0 or -1".

## GeomPlate deviations

- Null handles that the native code dereferences (a missing surface or law, the
  `LPropSurf` of a point without surface, the criteria of the empty curve constraint, a
  null initial surface in `Bounds` of GeomPlate_Surface) raise Standard_NullObject or
  give the stored bounds.
- The projector of the builder holds its adaptor through a shared handle
  (`ExtPS::initialize_shared`); natively it points to a local adaptor, which dies when
  the member that initialized it returns.
- A surface failure inside the MakeApprox evaluator is kept, reported by the error code
  and raised when `AdvApp2Var_ApproxAFunc2Var` returns
  ([geomalgo-design.md](geomalgo-design.md) section 19).

## IntCurveSurface deviations

- The polyhedron arrays keep the native 1-based layout: index 0 is the origin with
  null parameters (uninitialized natively), and an index past the grid, an
  out-of-bounds read natively, raises Standard_OutOfRange; after `destroy` every point
  access raises it. `IsOnBound` reads false outside the flags.
- Curve exceptions inside the quadric distance function are kept and raised when the
  root search returns, as for the IntImp functions (docs/geomalgo-design.md section 6).

## GeomFill filling-part deviations

- `GeomFill_CoonsAlgPatch::Bound`, `Corner` and `Func` read fixed arrays without a
  bounds check natively; an index outside them raises Standard_OutOfRange here (and so
  do `GeomFill_ConstrainedFilling::Boundary` and a `GeomFill_TgtOnCoons` of a side
  outside 0..3).
- `GeomFill_ConstrainedFilling::Eval` writes past a too short result natively; the
  slice raises Standard_OutOfRange here. A boundary or field exception inside the
  approximation evaluator is kept and raised when `AdvApprox_ApproxAFunction` returns
  (docs/geomalgo-design.md section 13). A second `Init` whose kept tangent poles no
  longer fit the new grid reads past the native temporary arrays (undefined
  behaviour); `PerformS1` raises Standard_OutOfRange here (not in the corpus).
- Null handles that the native code dereferences (a missing curve, surface, patch,
  approximation result or parameter array) raise Standard_NullObject.
- `GeomFill_BoundWithSurf` keeps its own `Adaptor3d_CurveOnSurface` through the new
  `Adaptor3dCurveOnSurface::copy_of` of TKG3d (the implicit native copy constructor).

## IntPatch deviations

- The lines return copies of their vertices and points; `ChangeVertex` runs a closure on
  the vertex in place (a reference cannot outlive the line's lock).
- A walking or restriction line without a point set dereferences a null handle natively
  in `SetPoint` and the box queries; the port raises Standard_NullObject (walking line) or
  Standard_DomainError (restriction line, as its `NbPnts` natively). A null arc or surface
  in the tools and functions raises Standard_NullObject.
- The `Dump` members write to a `fmt::Write`; the addresses they print are those of the
  Rust objects.
- `IntPatch_CSFunction` keeps copies of the handles where the native one keeps pointers to
  the caller's handles. `IntPatch_ArcFunction`'s IntStart side answers 0 for `F` where a
  quadric that is not set leaves the caller's variable uninitialized.
- `IntPatch_Polyhedron` keeps the IntCurveSurface layout: index 0 is the origin with null
  parameters (uninitialized natively) and an index past the grid, an out-of-bounds read
  natively, raises Standard_OutOfRange; `Triangle` past the grid answers the native
  arithmetic. `IntPatch_PolyhedronBVH` borrows its polyhedron (the native one keeps a
  pointer).
- `IntPatch_PolyLine` raises Standard_DomainError before a line is set (the native read of
  an uninitialized type) and Standard_NullObject for a null line.
  `IntPatch_PrmPrmIntersection_T3Bits` panics on a bit outside its words and is empty for
  `size^3 < 32`, where the native code writes out of bounds.

## IntPatch parametric intersection deviations

- The IntStart trait `TheTopolTool` has the domain members the IntPatch algorithms call
  (`is_the_point_on`, `domain_is_infinite`, `sample_pnts`, `u_parameters`, `v_parameters`,
  `is_uniform_sampling`; `Tol3d(C)` is the `tol_3d_arc` of the analytic classes).
- `FUN_PL_Intersection` and the trimming of infinite surfaces in `IntPatch_Intersection`
  cast the surface adaptor to `GeomAdaptor_Surface` without a check; another adaptor
  raises Standard_NullObject.
- A restriction line on neither surface raises Standard_NullObject in
  `IntPatch_LineConstructor::Perform` where the native code dereferences a null arc.
- The locals the native `Perform` bodies read before any assignment on some branches (the
  parameters of a section point of the `Intf_EXTERNAL` type) keep their previous value, 0 at
  first.

## IntPatch analytic-intersection deviations

- `ProcessBounds` of `IntPatch_ImpImpIntersection` loops forever natively on a line list
  that holds a line other than an analytic one (the loop does not advance); no caller
  builds such a list, and the port raises Standard_ProgramError.
- A native `occ::down_cast` that the code dereferences (`FindLine` on another line kind,
  `IntPatch_RstInt` on an analytic or conic line, `DecomposeResult` on a restriction line
  without a point set, `MakeWLine` on a null line) raises Standard_NullObject or
  Standard_DomainError instead of crashing.
- `IntPatch_RstInt` omits the `nbTreated` bookkeeping of the native merge: the counter
  never reaches the value that would change the flow.
- The quadric parameters of a point on an unset quadric are 0 (natively uninitialized);
  the intersections always set their quadrics, so no result depends on them.
- `IntPatch_RLine` gains the crate copy `duplicate` for `DecomposeResult`, and the domain
  trait `TheTopolTool` gains defaulted 3d tolerance and orientation queries that
  `IntPatch_RstInt` calls.

## GeomFill sweep-part deviations

- `GeomFill_SnglrFunc` declares no RTTI natively (its dynamic type is
  `Adaptor3d_Curve`); the port registers it as `GeomFill_SnglrFunc`.
- `GeomFill_SectionPlacement` rebuilds its section projector at each projection, where
  the native member is initialized once; each native `Perform` of it starts from its
  initialized state, so the results are the same. The private `SectionAxis` and
  `GeomFill_Sweep::BuildProduct` (with `GeomFill_Sweep_Eval`) are called nowhere
  natively and are not ported. `GeomFill_CircularBlendFunc` drops the member `distmin`,
  written and never read.
- `GeomFill_Darboux` on a curve that is not an `Adaptor3d_CurveOnSurface` raises
  Standard_TypeMismatch where the native down cast gives a null handle that is then
  dereferenced.
- `GeomFill_SweepFunction`, `GeomFill_NSections` and `GeomFill_SweepSectionGenerator`
  read and write their arrays without bounds checks natively; an index outside them
  raises Standard_OutOfRange here.
- Null handles that the native code dereferences (a missing curve, law, section,
  surface or adaptor) raise Standard_NullObject. `GeomFill_LocFunction::DN` of an order
  outside 0..2 reads a null pointer natively and raises Standard_NullObject here.
- Uninitialized native members start at 0 or false: `GeomFill_Tensor`, the convertors
  before `Init`, `GeomFill_Sweep`'s `myExchUV`, `isUReversed` and `isVReversed` before
  `Build`, and `myNbSections` and `myPolynomial` of a
  `GeomFill_SweepSectionGenerator` built with arguments before `Perform`.
  `GeomFill_NSections::SectionShape` without a surface answers `(0, 0, 0)`, where the
  native one leaves its outputs unchanged.
