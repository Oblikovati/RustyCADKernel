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

## Verification

`python3 tools/verify_toolkit.py TKFillet` runs the numeric oracle
`tools/verify_tkfillet_numeric.py`: `tools/oracle/tkfillet_probe.cxx` and its Rust twin
`crates/tkfillet/examples/numeric_probe.rs` on `tools/tkfillet_corpus.py`.

| Package | Probe operations |
| --- | --- |
| ChFiDS | `fl_spine`, `fl_arc_spine`, `fl_fil_law` |
| BlendFunc | `fl_const_rad`, `fl_const_rad_inv`, `fl_evol_rad`, `fl_evol_rad_inv`, `fl_chamfer`, `fl_chamfer_inv` |
| BRepBlend | `fl_rad_inv`, `fl_blend_tool`, `fl_surf_rst`, `fl_rst_rst`, `fl_app_func`, `fl_walking` |
