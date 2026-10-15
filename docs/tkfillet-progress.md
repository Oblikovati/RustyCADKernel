# TKFillet progress

TKFillet (`crates/tkfillet`) ports the OCCT fillet and chamfer toolkit at
3d097a0328e71b826377d4814ab05ec3c3d23871, package by package in dependency order:
ChFiDS, Blend, BlendFunc, BRepBlend, ChFiKPart, ChFi2d, ChFi3d, BRepFilletAPI,
FilletSurf.

## Mapping rules

- Value classes (`ChFiDS_CommonPoint`, `ChFiDS_FaceInterference`, `ChFiDS_Regul`, ...)
  are plain structs. Their `Change*` accessors are `&mut` accessors.
- The transient classes that the builders share and change through several handles
  (`ChFiDS_SurfData`, `ChFiDS_Stripe`, `ChFiDS_Spine` and its subclasses) are
  `chfi_ds::Shared<T>` handles (`Arc<Mutex<T>>`). A null native handle is `None`.
  `chfi_ds::lock` gives the mutable view.
- `ChFiDS_ElSpine` is a curve adaptor: a `Handle<ElSpine>` whose state sits behind a
  lock, as every adaptor of the port.
- The inheritance `ChFiDS_Spine` -> `ChFiDS_FilSpine` / `ChFiDS_ChamfSpine` is the
  `SpineObject` trait (the virtual members) plus `Deref` to the `Spine` part.
- Output references become returned values, overloads take distinct names, and the
  native exceptions become `Failure` values with the native exception type.
- The BlendFunc classes implement the Blend traits (`BlendAppFunction`, `BlendFunction`,
  `BlendFuncInv`) and `math::FunctionSetWithDerivatives`. Classes that share their text
  share one implementation: `two_surfaces` (ConstRad, EvolRad), `fillet_inverse`
  (ConstRadInv, EvolRadInv), `gen_chamfer` and `chamfer_macro` (the BlendFunc_GenChamfer
  subclasses and ChAsym), `const_throat` and `const_throat_inv` (the throat chamfers with
  and without penetration). The abstract `BlendFunc_GenChamfer` and
  `BlendFunc_GenChamfInv` are the `GenChamferFunction` and `GenChamfInvFunction` traits.

## Native quirks kept

- `ChFiDS_Stripe::Reset` nullifies the first pcurve twice and keeps the last one.
- `ChFiDS_SurfData::Copy` does not copy the curve indices and their flags.
- `ChFiDS_FilSpine::SetRadius(Law_Function, IinC)` puts the law into a composite that is
  dropped: only the split flag and the radii change.
- `ChFiDS_FilSpine::IsConstant(IE)` is true and `Radius(IE)` is 0 when no radius is set.
- `ChFiDS_SurfData::Vertex(First, OnS)` and `Get2dPoints(First, OnS)` select the last
  point on the second face for an `OnS` other than 1 and 2.
- BlendFunc keeps the `static thread_local` scratch of `ComputeValues` (one per class),
  `BlendFunc_ConstRadInv::Derivatives` derives the plane normal with `theD` where
  `Values` uses `-nplan.d2cur`, and the order-2 positioning of `BlendFunc_EvolRad` keeps
  the normal derivatives of the previous call.
- `BlendFunc_Chamfer::Set(Dist1, Dist2, Choix)` keeps the distances unsigned while
  `BlendFunc_ChamfInv` signs them by the choice.
- `BlendFunc_ConstThroatInv::Derivatives` (and the penetration variant) takes the
  differences to the points of the previous evaluation before it evaluates the surfaces.
- The first-order `BlendFunc_ChAsym::Section` writes the poles of the previous evaluation;
  `BlendFunc_ChAsymInv::ComputeValues` uses the reversed `2 nps1` in the guide derivative
  when the curve lies on the second surface.
- The `myTConv` of the restriction functions is uninitialized natively until `GetShape`
  meets a non-linear shape (the port starts at `TgtThetaOver2`), and
  `BRepBlend_SurfRstEvolRad` leaves the restriction parameter uninitialized until a
  solution; the probes set the first and read the second only after a solution.
- `BRepBlend_AppSurface` implements `AppBlend_Approx`, whose `TolCurveOnSurf` has no
  failure in the Rust trait: a native exception there reads NaN through the trait, and
  `tol_curve_on_surf_checked` returns it.
- `BRepBlend_Walking` (and the line builders) call the 4-argument
  `BRepBlend_Extremity::SetValue(P, U, V, Tol)` and constructor, which are the curve
  overloads `(P, W, Param, Tol)`: `V` lands in the guide parameter and the old `V` stays.
- `BRepBlend_Walking`: `ToCorrectOnRst1/2` are set by the first section on the
  restrictions and never reset, and `CorrectedParam` is written by
  `CorrectExtremityOnOneRst` even when it fails; `Transition` passes the derivatives of
  the surface where `CSLib::Normal` expects those of the normal; the control of a
  reframing on the second restriction uses `min(tolerance(1), tolerance(1))`;
  `Continu(OnS1)` removes the old last point with the new ones; a stop on the guide
  deflection does not set the line ends; `CorrectExtremityOnOneRst` reads the current arc
  of the domain iterator.
- `BRepBlend_SurfRstLineBuilder`: the first extremity on the restriction gets the
  parameters of the point on the surface; the reframing vertices, the arc and the
  unhooking flag persist between the steps (a stale vertex can end a line on the bound);
  the section on both restrictions takes the third value of the surface reframing in
  `InternalPerform` (the second in `PerformFirstSection`), and its end on the arc takes
  the guide parameter.
- `BRepBlend_RstRstLineBuilder`: when both reframings on the first restriction succeed and
  its end point wins, the vertex goes to the second restriction; the end on the second
  restriction takes the tangent on the first one; the end point reframings read the
  restriction parameter of the last vertex examined even when none matched (as do those
  of the surface-restriction builder).
- `ChFi3d::ConcaveSide` moves the point on the second face by the inward direction of
  the first face (`ChFi3d_Coefficient(dint1, DU2, DV2)`) when the faces are tangent
  without turning back.
- `ChFi2d_AnaFilletAlgo`: both neighbours of the fillet take the orientation of the
  first edge; the cut of two crossing edges is found by `GeomAPI_ExtremaCurveCurve`.
- `ChFi2d_FilletAlgo`: the shifted point is filled up to `delta * 999` (not shifted);
  the projections compare `LowerDistanceParameter` with the confusion as a distance.
- `ChFi2d_Builder`: when the chamfer line does not cut the second edge, `ComputePoint`
  returns the surface point at the origin and an uninitialized parameter (the port uses
  0.0); `ModifyChamfer(Chamfer, E1, E2, D1, D2)` does not use `E1`.

## Verification

`python3 tools/verify_toolkit.py TKFillet` runs the numeric oracle
`tools/verify_tkfillet_numeric.py`: `tools/oracle/tkfillet_probe.cxx` and its Rust twin
`crates/tkfillet/examples/numeric_probe.rs` on `tools/tkfillet_corpus.py`.

| Package | Probe operations |
| --- | --- |
| ChFiDS | `fl_spine`, `fl_arc_spine`, `fl_fil_law` |
| BlendFunc | `fl_const_rad`, `fl_const_rad_inv`, `fl_evol_rad`, `fl_evol_rad_inv`, `fl_chamfer`, `fl_chamfer_inv` |
| BRepBlend | `fl_rad_inv`, `fl_blend_tool`, `fl_surf_rst`, `fl_rst_rst`, `fl_app_func`, `fl_walking`, `fl_surf_rst_walk`, `fl_rst_rst_walk` |
| ChFi2d | `fl_chfi2d_fillet`, `fl_chfi2d_chamfer`, `fl_chfi2d_builder` |
| ChFi3d | `fl_hatching_domains` (`ChFi3d_ComputeHatchingDomains`), `fl_concave_side` (`ChFi3d::ConcaveSide` on every edge of box, prism, cylinder, cone, sphere, torus and revolved solids), `fl_search_sing` (`ChFi3d_SearchSing` and its `math_FunctionRoot`) |
| ChFi3d_Builder_0 | `fl_b0_scalars`, `fl_bound_srf`, `fl_bound_fac`, `fl_enlarge_box`, `fl_b0_faces`, `fl_b0_spines`, `fl_inter_plane_edge`, `fl_b0_pcurves`, `fl_reparam_pcurv`, `fl_project_pcurv`, `fl_b0_bounds`, `fl_compute_arete`, `fl_fil_common_point`, `fl_b0_stripes`, `fl_b0_tols`, `fl_int_cs`, `fl_computes_int_pc`, `fl_b0_inter2d`, `fl_b0_topology`, `fl_b0_smooth`, `fl_compute_curves`, `fl_int_traces`, `fl_el_spine` |
| ChFiKPart | `fl_kpart_fcts`, `fl_corner_spine`, `fl_kpart_make` (every `ChFiKPart_MakeFillet`, `ChFiKPart_MakeChamfer` and `ChFiKPart_MakeChAsym` overload with every orientation), `fl_kpart_corner` (the three `ChFiKPart_ComputeData::ComputeCorner`) |
| BRepFilletAPI | `fl_chfi2d_builder` (`BRepFilletAPI_MakeFillet2d` on the edited face) |

The GTests of `ChFi3d_Hatching_Test.cxx` are translated in
`crates/tkfillet/tests/chfi3d_hatching.rs`.

## Data structure

ChFiKPart (`ChFiKPart_ComputeData` and every `ChFiKPart_Make*` function) and the free
functions of `ChFi3d_Builder_0` that take the data structure (`ChFi3d_FilDS`,
`ChFi3d_StripeEdgeInter`, `ChFi3d_IsInFront`, `ChFi3d_SearchFD`, `ChFi3d_BoundSurf`,
`ChFi3d_SolidIndex`, `ChFi3d_IndexPointInDS`, `ChFi3d_FilPointInDS`,
`ChFi3d_FilVertexInDS`, `ChFi3d_FilCurveInDS`, `ChFi3d_ExtrSpineCarac`,
`ChFi3d_SetPointTolerance` and the data-structure overload of `ChFi3d_EnlargeBox`) write
into the ported `TopOpeBRepDS_DataStructure`, borrowed by the functions. The overloads
of ChFiKPart are named after the surfaces and the spine (`make_fillet_pln_pln`,
`make_chamfer_pln_cyl_line`, `make_ch_asym_pln_con`, ...).
`ChFiKPart_Sphere` keeps the native NaN path: when the circle through the three points
is wider than the sphere the centre is NaN and the sphere is still written (the
comparisons are negated as the native ones). `ChFi3d_FilDS` is checked through the
fillet builders.

## Remaining

The ChFi3d builders (`ChFi3d_Builder`, `ChFi3d_FilBuilder`, `ChFi3d_ChBuilder` and the
`ChFi3d_Builder_*` member files), `BRepFilletAPI_MakeFillet`,
`BRepFilletAPI_MakeChamfer`, `FilletSurf_Builder` and `FilletSurf_InternalBuilder`
rebuild the solid with `TopOpeBRepBuild_HBuilder` and are not ported yet. Their GTests
(`BRepFilletAPI_MakeFillet_Test.cxx`, `BRepFilletAPI_MakeChamfer_Test.cxx`) and the
`BFuseSimpleTest.BlendBoxWithCylinder*` GTests of TKBO are `pending_dependency` rows in
`porting/parity.json`.

Everything of these packages that does not touch the data structure is ported:
`BRepFilletAPI_LocalOperation` (the `BRepFilletAPILocalOperation` trait),
`BRepFilletAPI_MakeFillet2d`, `ChFi3d::ConcaveSide`, `ChFi3d_SearchSing`,
`ChFi3d_FilletShape`, the geometric and topological helpers of `ChFi3d_Builder_0`
(including `ChFi3d_ComputeCurves`, `ChFi3d_IntTraces` and `ChFi3d_PerformElSpine`),
`ChFi3d_ComputeHatchingDomains`, the ChFiKPart helpers (`ChFiKPart_InPeriod`,
`ChFiKPart_PCurve`, `ChFiKPart_ProjPC`, `ChFiKPart_CornerSpine`) and the FilletSurf
statuses. `ChFi3d_EnlargeFace` is declared but never defined natively.
