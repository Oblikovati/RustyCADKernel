# TKGeomAlgo design notes

This note fixes how the TKGeomAlgo port (`crates/tkgeomalgo`, see
[tkgeomalgo-progress.md](tkgeomalgo-progress.md)) maps C++ constructs that the earlier
design notes ([geom-design.md](geom-design.md), [topology-design.md](topology-design.md))
do not cover. Each package adds its section before it codes the construct.

## 1. Generic templates (`.gxx` and `.pxx`)

OCCT writes much of TKGeomAlgo as generic source files: a `.gxx` (or a `.pxx`/`.lxx`)
body is included by one or more `_0.cxx` files after macros name the concrete types
(`IntCurve_IntConicConic`, `IntCurveSurface_TheInterferenceOfHInter`,
`TopClass_FaceClassifier`...). The rule for the whole toolkit:

- The template body is ported **once**, in the module of the package that owns the
  `.gxx`, as a Rust generic (a struct or function generic over a trait).
- The trait holds exactly the operations the template calls on its type parameters
  (the `ThePolygon`, `ThePolyhedron`, `TheTool` macros), under the native names; it is
  owned by the package of the template.
- Each native instantiation is a type alias (or a thin wrapper when the instance adds
  members), declared in the package that instantiates it natively, with the concrete
  types implementing the trait there.
- Floating-point operation order and control flow follow the template body; a
  monomorphized Rust generic executes the same operations as each C++ instance.

The sections below record each template as it is ported.

## 2. Intf_InterferencePolygonPolyhedron.gxx

The template computes the interference between a 3d polygon (or straight lines) and a
triangulated polyhedron. Its `.gxx` names four macro types: `Polygon3d` and
`Polyhedron` (the data) and `ToolPolygon3d` and `ToolPolyh` (classes of static members
over the data). OCCT instantiates it twice, as
`IntCurveSurface_TheInterferenceOfHInter` (TKGeomAlgo) and
`HLRBRep_TheInterferenceOfInterCSurf` (TKHLR).

- The body is `intf::InterferencePolygonPolyhedron<PT, HT>` in
  `crates/tkgeomalgo/src/intf/interference_polygon_polyhedron`, generic over two
  traits of static functions that mirror the tool classes: `intf::Polygon3dTool`
  (`Bounding`, `DeflectionOverEstimation`, `Closed`, `NbSegments`, `BeginOfSeg`,
  `EndOfSeg`) and `intf::PolyhedronTool` (`Bounding`, `ComponentsBounding`,
  `DeflectionOverEstimation`, `Triangle`, `Point`, `TriConnex`, `IsOnBound`,
  `GetBorderDeflection`). Each trait has an associated type for its data
  (`Polygon`, `Polyhedron`), so the data types stay free of the traits, as the
  native tools are separate classes.
- An instantiating package declares its tool classes as unit structs that implement
  the traits (for example `IntCurveSurface_ThePolygonToolOfHInter`) and the
  interference as a type alias:
  `type TheInterferenceOfHInter = InterferencePolygonPolyhedron<ThePolygonToolOfHInter, ThePolyhedronToolOfHInter>;`.
  Its api rows map the members of the instance header to the generic's methods.
- Tool members that index arrays can raise natively (Standard_OutOfRange), so the
  trait members that take an index return `Result`, and so do the constructors and
  `Perform` overloads of the generic.
- `Intf_Interference` is the protected base of every interference; the generic and
  `InterferencePolygon2d` hold one and expose it through `Deref`.
- The two private `Intersect` overloads have the same body after the plane equation;
  the five-argument one computes the plane equation and the distances and calls the
  nine-argument one, so the body exists once.
## 3. IntImpParGen_Intersector.gxx

`IntImpParGen_Intersector` intersects an implicit curve with a bounded parametric one
through the roots of the signed distance (`IntImpParGen_MyImpParTool`, expanded natively
in each `_0.cxx`). It is ported once in `int_imp_par_gen` as
`Intersector<T: IntersectorTypes>` and `MyImpParTool<'a, T>`:

- `IntersectorTypes` carries the four macros as associated types: `Imp: ImpTool` (the
  `ImpTool`), `Par: ParTool` (the `ParTool`, whose associated `Curve` is the `ParCurve`)
  and `Project: ProjectOnPCurveTool` over the same curve. The traits hold the members the
  template calls, under their native names; `D1`/`D2` write through `&mut` outputs because
  a native tool may leave an output unchanged (IntCurve_PConicTool::D2 of a line does not
  write the second derivative) and the template reads the stale value.
- `MyImpParTool` implements the tkmath `Function`/`FunctionWithDerivative` traits; a curve
  exception is kept and reported as `false` (the traits carry no error), and Perform
  returns it once `math_FunctionAllRoots` stops, where the native exception would have
  left the constructor.
- The variables that the native Perform keeps across its steps (the D2 outputs, the
  transitions, the end flags) live in one `PerformState`, so every step reads what the
  native code reads.
- Instantiations: `int_curve::ConicTypes` (IConicTool, PConicTool over PConic,
  ProjectOnPConicTool) gives `IntCurve_IntImpConicParConic` and
  `IntCurve_MyImpParToolOfIntImpConicParConic` as aliases; Geom2dInt declares the
  Adaptor2d_Curve2d instantiation (`Geom2dInt_TheIntersectorOfTheIntConicCurveOfGInter`).

## 4. The IntCurve curve templates and their Geom2dInt instances

IntCurve owns the templates that Geom2dInt instantiates on `Adaptor2d_Curve2d`; each is a
generic in `int_curve`, and Geom2dInt declares the native classes as aliases:

- `IntCurve_IntConicCurveGen.gxx` is `IntConicCurveGen<T: IntersectorTypes<Imp =
  IConicTool>>` (the `TheImpTool` is fixed to IConicTool natively, the `ThePCurve`,
  `ThePCurveTool`, `TheProjPCur` and `IntCurve_TheIntersector` macros come from the
  IntImpParGen instantiation `T`): `Geom2dInt_TheIntConicCurveOfGInter`.
- `IntCurve_UserIntConicCurveGen.gxx` is `UserIntConicCurveGen<T>`, which also needs the
  curve tool members of the trait `PCurveTool` (GetType, the conics, the intervals):
  `Geom2dInt_IntConicCurveOfGInter`. Its five InternalPerform overloads differ only by the
  pair order of IntCurve_IntConicConic; the port dispatches on the conic ranks
  (`IntConicConic::perform_ordered`), reversing the parameters when the curve's kind comes
  first, as each native case does.
- `IntCurve_DistBetweenPCurvesGen.gxx` is `DistBetweenPCurvesGen<'a, T: PCurveTool>`, a
  tkmath `FunctionSetWithDerivatives` (a curve exception is kept and reported as `false`):
  `Geom2dInt_TheDistBetweenPCurvesOfTheIntPCurvePCurveOfGInter`.
- `Geom2dInt_Geom2dCurveTool` implements `ParTool` and `PCurveTool`; `Geom2dTypes` is the
  IntImpParGen instantiation (`Geom2dInt_TheIntersectorOfTheIntConicCurveOfGInter` and its
  `MyImpParTool`).
- The three Extrema aliases of Geom2dInt (`TheCurveLocator`, `TheLocateExtPC`, `PCLocF`)
  instantiate TKGeomBase templates with Geom2dInt_Geom2dCurveTool. Those templates call
  only Value, D0-D2, DN, GetType and the parameter bounds, which forward to the adaptor
  in Geom2dInt_Geom2dCurveTool and Extrema_Curve2dTool alike, so the aliases reuse the
  Extrema_Curve2dTool instantiations of the Extrema port instead of implementing the whole
  `ExtremaCurveTool` trait for a second tool.

## 5. The IntCurve curve-curve templates and Geom2dInt_GInter

The templates that need the Intf polygon interference are generics in `int_curve`, and
Geom2dInt declares the native classes as aliases:

- `IntCurve_Polygon2dGen.gxx` is `Polygon2dGen<T: ParTool>`
  (`Geom2dInt_ThePolygon2dOfTheIntPCurvePCurveOfGInter`); it implements the Intf
  `Polygon2d` trait, which stands for the `Intf_Polygon2d` base class, and keeps the
  protected `myBox` itself.
- `IntCurve_ExactIntersectionPoint.gxx` is `ExactIntersectionPoint<'a, T: PCurveTool>`
  (`Geom2dInt_ExactIntersectionPointOfTheIntPCurvePCurveOfGInter`): it borrows the curves
  through its `DistBetweenPCurvesGen`, and the four in-out arguments of its polygon Perform
  are two `(segment, parameter)` seats passed by `&mut`.
- `IntCurve_IntPolyPolyGen.gxx` is `IntPolyPolyGen<T: IntersectorTypes>` with
  `T::Par: PCurveTool` (`Geom2dInt_TheIntPCurvePCurveOfGInter`): the macros `TheCurve`,
  `TheCurveTool` and `TheProjPCur` come from the IntImpParGen instantiation, and the
  polygon and exact-point macros are the two generics above over `T::Par`. The body is
  split by responsibility: the public Performs and HeadOrEndPoint
  (`int_poly_poly_gen.rs`), findIntersect and GetIntersection (`int_poly_poly_find.rs`),
  the self intersection (`int_poly_poly_self.rs`) and the steps both repeat
  (`int_poly_poly_zone.rs`: positions, transitions and tangent-zone ends; the self
  intersection passes its curve twice, which gives its native code exactly). The protected
  Perform overloads are the public `perform_iter` and `perform_self_iter`.
- `IntCurve_IntCurveCurveGen.gxx` is `IntCurveCurveGen<T: IntersectorTypes<Imp =
  IConicTool>>` (`Geom2dInt_GInter`). Its 36 native cases of InternalPerform are one
  classification of both curves as `GpConic` or not: two conics go to
  `IntConicConic::perform_ordered` in pair order, reversed when the second kind ranks
  first; a conic and another curve go to `IntConicCurveGen::perform_any_conic` with the
  conic first, reversed when it is the second curve; two other curves go to
  `IntPolyPolyGen`. The results merge as in UserIntConicCurveGen (`merge_result`).
- The template reads the curve tool's D2 outputs only after D2 writes them for every curve
  kind of the one instantiation (Geom2dInt_Geom2dCurveTool), so the normals start at zero
  for each transition instead of persisting across calls as the native locals do.

## 6. IntImp templates

IntImp (`crates/tkgeomalgo/src/int_imp`) holds five generic bodies. Each is one Rust
generic over the tool traits of `int_imp/tools.rs`, which carry the native macro names
and exactly the calls the bodies make:

| Template | Rust generic | Tool traits | TKGeomAlgo instances |
| --- | --- | --- | --- |
| `IntImp_ZerParFunc.gxx/.lxx` | `IntImpZerParFunc<T>` | `ThePSurfaceTool` | `IntWalk_TheFunctionOfTheInt2S`, `GeomInt_TheFunctionOfTheInt2SOfThePrmPrmSvSurfacesOfWLApprox` |
| `IntImp_Int2S.gxx/.lxx` | `IntImpInt2S<T>` | `ThePSurfaceTool` | `IntWalk_TheInt2S`, `GeomInt_TheInt2SOfThePrmPrmSvSurfacesOfWLApprox` |
| `IntImp_ZerImpFunc.gxx/.lxx` | `IntImpZerImpFunc<P, I>` | `ThePSurfaceTool`, `TheISurfaceTool` | `IntPatch_TheSurfFunction`, `GeomInt_TheZerImpFuncOfTheImpPrmSvSurfacesOfWLApprox` |
| `IntImp_ZerCSParFunc.gxx` | `IntImpZerCSParFunc<P, C>` | `ThePSurfaceTool`, `TheCurveTool` | `IntCurveSurface_TheCSFunctionOfHInter` |
| `IntImp_IntCS.gxx` | `IntImpIntCS<F>` | `TheFunction` (with its `ThePSurfaceTool` and `TheCurveTool`) | `IntCurveSurface_TheExactHInter` (on `IntCurveSurface_TheCSFunctionOfHInter`), `IntPatch_CurvIntSurf` (on `IntPatch_CSFunction`) |

- Every TKGeomAlgo instance uses `Adaptor3d_HSurfaceTool` as `ThePSurfaceTool` and
  `IntSurf_QuadricTool` as `TheISurfaceTool`. Both implementations live once in
  `int_imp/tools.rs` (`HSurfaceTool`, and the trait implemented for
  `IntSurfQuadricTool`), since one type can implement a trait only once; the
  instantiating packages declare their instances as aliases, e.g.
  `type IntWalkTheInt2S = IntImpInt2S<HSurfaceTool>`. `TheCurveTool` is implemented by
  the package that owns the curve tool (`IntCurveSurface_TheHCurveTool`,
  `IntPatch_HCurve2dTool`), and `TheFunction` by the package of a hand-written
  function (`IntPatch_CSFunction`); `IntImpZerCSParFunc` implements it here.
- The functions hold their surfaces and curves by value (a cloned handle) where the
  native classes keep `void*` pointers to the caller's objects.
- The tool calls can raise natively, and the exception unwinds through
  `math_FunctionSetRoot`. The functions keep the first failure of their
  `math::FunctionSetWithDerivatives` callbacks
  (`rustycad_tkgeombase::failure_capture::FailureCapture`), answer false, and
  `IntImpInt2S`/`IntImpIntCS` raise it as soon as the solver returns. Their inherent
  `value`, `derivatives` and `values` return the failure directly.
- The probe (`tools/oracle/tkgeomalgo_probe_intimp.cxx`) drives the native instances
  above and the Rust generics with the same tools, so the numeric corpus checks the
  template bodies.

## 7. AppBlend template

`AppBlend_AppSurf.gxx/.lxx` is instantiated as `GeomFill_AppSurf`, `GeomFill_AppSweep`
(TKGeomAlgo) and `BRepBlend_AppSurf` (TKFillet) on a `TheLine` and a
`TheSectionGenerator`. The class stores neither: they are only arguments of its
`Perform` methods. The port is therefore one type, `app_blend::AppBlendAppSurf`,
whose `perform`, `perform_smoothing` and `perform_by_pieces` are generic over the
traits `TheLine` (`NbPoints`, `Point`) and `TheSectionGenerator` (`GetShape`,
`Knots`, `Mults`, both `Section` overloads, `Parameter`) of `app_blend/tools.rs`; the
sections are exchanged through `AppBlendSection`. Each native instance is an alias of
`AppBlendAppSurf`, and the packages that own a line and a generator implement the
traits. `AppBlend_Approx` is the trait `AppBlendApprox`.

The probe (`tools/oracle/tkgeomalgo_probe_appblend.cxx`) instantiates the `.gxx`
itself on a probe line and generator, as the `_0.cxx` files do, so the numeric corpus
checks the template without the GeomFill package.

## 8. TopClass_Classifier2d.pxx and TopClass_FaceClassifier.pxx

TopClass holds two header-only templates: namespaces of inline functions that take every
field of the instantiating class by reference. Geom2dHatch instantiates both
(`Geom2dHatch_FClass2dOfClassifier` and `Geom2dHatch_Classifier`); BRepClass and
IntTools will instantiate them again. The port keeps the fields in the generic and the
namespace functions as its methods, so each instance class is an alias:

- `TopClass_Classifier2d` is `topclass::Classifier2d<I: Classifier2dIntersector>`. The
  struct holds the fields the native instance class passes to `Reset` and `Compare`
  (`myIsSet`, `myFirstCompare`, `myFirstTrans`, `myLin`, `myParam`, `myTolerance`,
  `myTrans`, `myIntersector`, `myClosest`, `myState`, `myIsHeadOrEnd`, with their
  native initial values); `reset` and `compare` are the template bodies. The trait
  `Classifier2dIntersector` holds the `TheIntersector` operations: the segment Perform,
  the IntRes2d results and LocalGeometry, with the edge type as an associated type.
- `TopClass_FaceClassifier` is `topclass::FaceClassifier<X: FaceExplorer>`. It holds the
  fields of the native instance class (the `Classifier2d<X::Intersector>`, the closest edge,
  its parameter and position, the rejected and no-wires flags); `perform`, `state`, `edge`
  and `edge_parameter` are the template bodies. The trait `FaceExplorer` holds the
  `TheFaceExplorer` operations (CheckPoint, Reject, Segment, OtherSegment, the wire and
  edge iteration, RejectWire, RejectEdge, CurrentEdge); the native `FClass2d` parameter is
  always the Classifier2d instance over the explorer's intersector, as in every OCCT
  instantiation.
- The C++ output references (`Segment(P, L, Par)`, `CurrentEdge(E, Or)`,
  `LocalGeometry(E, U, T, N, C)`) become returned tuples; members that evaluate a curve
  return `Result`, and the templates propagate the error where the native exception would
  leave them, after the same field writes.
- The closest intersection point is kept as a copy (the native code keeps a pointer into
  the intersector's result list, which the next Perform replaces).
- Instantiations: Geom2dHatch declares `FClass2dOfClassifier = Classifier2d<Intersector>`
  and `Classifier = FaceClassifier<Elements>`, with `Geom2dHatch_Intersector` and
  `Geom2dHatch_Elements` implementing the traits.

## 9. Geom2dGcc: the expanded GccIter and GccGeo classes

At the pinned commit Geom2dGcc no longer includes `.gxx` templates: the former
GccIter and GccGeo generics are expanded in the package as concrete classes over
`Geom2dAdaptor_Curve` (`Geom2dGcc_Circ2d3TanIter`, `Geom2dGcc_Circ2d2TanOnGeo`,
`Geom2dGcc_Lin2dTanOblIter`...), and `Geom2dGcc_CurveTool` is a plain class of static
members. There is nothing to instantiate, so the port has no generic and no trait of
its own:

- Each class is a concrete struct in `geom2d_gcc` over `Geom2dAdaptorCurve`, and
  `CurveTool` is a unit struct of associated functions.
- `Geom2dGcc_QCurve` and `Geom2dGcc_QualifiedCurve` have identical members and bodies;
  the port has one struct, `QualifiedCurve`, and `QCurve` is its alias.
- The function classes implement the tkmath traits their native base classes stand for
  (`math_FunctionWithDerivative`, `math_FunctionSetWithDerivatives`) and borrow their
  curves, where the native classes hold copies of the adaptors: nothing mutates an
  adaptor during a solve, so a borrow gives the same values. A curve evaluation failure
  is kept and reported as `false` (the traits carry no error); the solver that ran the
  function returns it once the tkmath algorithm stops, where the native exception
  would have left it.
- The `Geom2dGcc_Type1`/`Type2`/`Type3` enums are public enums; a curve slot that the
  selected type does not fill is `None`, and evaluating it gives the
  Standard_NullObject that the native null handle stands for.
- The fixed-size solution arrays reuse `gcc_ana::solutions` (`Slots`, `Tangencies`):
  the same bounds check and the same zero start for slots that native leaves
  uninitialized.

## 10. IntWalk templates

IntWalk (`crates/tkgeomalgo/src/int_walk`) holds one `.gxx` template, the implicit
walking `IntWalk_IWalking.gxx/.lxx`, beside the concrete parametric walking
`IntWalk_PWalking` and the IntImp instances `IntWalk_TheInt2S` and
`IntWalk_TheFunctionOfTheInt2S`.

- The IntImp instances are aliases of the IntImp generics on `HSurfaceTool`
  (section 6): `type IntWalkTheInt2S = IntImpInt2S<HSurfaceTool>` and
  `type IntWalkTheFunctionOfTheInt2S = IntImpZerParFunc<HSurfaceTool>`. Their api rows
  map the instance headers to the generic members. `IntWalk_PWalking` holds an
  `IntWalkTheInt2S`, as the native class holds its instance.
- `IntWalk_IWalking.gxx` is `IntWalkIWalking<T: IWalkingTypes>`. OCCT instantiates it
  as `IntPatch_TheIWalking` (TKGeomAlgo) and `Contap_TheIWalking` (TKHLR). The trait
  `IWalkingTypes` of `int_walk/iwalking/tools.rs` names the macro types that vary:
  `ThePSurfaceTool` (the IntImp trait), `TheIWFunction` (the trait `TheIWFunction`,
  with the members the body calls: `Set`, `Value`, `Values`, `Derivatives`, `Root`,
  `Tolerance`, `IsTangent`, `Point`, `Direction3d`, `Direction2d`, `PSurface()->Value`
  and the copy) and `IntWalk_TheIWLine` (the trait `TheIWLine`, the members of the
  line class that the body calls).
- `ThePointOfPath`, `ThePointOfLoop` and their tools are `IntSurf_PathPoint`,
  `IntSurf_InteriorPoint`, `IntSurf_PathPointTool` and `IntSurf_InteriorPointTool` in
  both instances, and the body names `IntSurf_InteriorPoint` and `IntSurf_PntOn2S`
  itself, so the generic uses the IntSurf types directly. `ThePOPIterator` and
  `ThePOLIterator` are sequences of these points: slices, read 1-based with the
  native Standard_OutOfRange.
- The lines are transients reached through handles: the body keeps a line in its
  sequence and goes on editing it through the handle. `TheIWLine` members therefore
  take `&self`, as the members of other transients do (a `SharedState` inside the
  implementation).
- `TheIWFunction` is implemented here for `IntImpZerImpFunc<P, I>` (the function of
  `IntPatch_TheSurfFunction`). The line class of the instances is
  `IntPatch_TheIWLineOfTheIWalking` (a hand-written class of IntPatch); IntPatch
  implements `TheIWLine` for it when it is ported.
- The body is split by responsibility: the constructor, `Perform` and the
  consultation (`iwalking/mod.rs`), the stop tests (`stop_tests.rs`), the holes and
  the line tests (`holes.rs`), the open and the closed lines (`open_line.rs`,
  `close_line.rs`), `TestDeflection` (`deflection.rs`) and the point helpers
  (`points.rs`). The locals that the native `ComputeOpenLine` and `ComputeCloseLine`
  keep across the iterations of their loops (`N`, `SaveN`, `Psol`, `aStatus`, the
  flags) live in one state struct per loop, so every step reads what the native code
  reads.
- The probe (`tools/oracle/tkgeomalgo_probe_intwalk*.cxx`) includes the `.gxx` itself
  on a probe class, as the `_0.cxx` files do, with `Adaptor3d_HSurfaceTool`,
  `IntPatch_TheSurfFunction` and `IntPatch_TheIWLineOfTheIWalking`; its Rust twin
  instantiates the generic with `HSurfaceTool`, `IntImpZerImpFunc` and a probe port of
  that line class.
