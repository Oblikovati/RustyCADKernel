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
- The probe (`tools/oracle/tkgeomalgo_probe_iwalk.cxx`) runs the template through its
  IntPatch instance `IntPatch_TheIWalking`; its Rust twin runs the alias
  `IntPatchTheIWalking` (section 16).

## 11. IntCurveSurface templates

IntCurveSurface writes its algorithm as header templates (`.pxx`) that two classes
instantiate: `IntCurveSurface_HInter` (TKGeomAlgo, on `Adaptor3d_Curve` and
`Adaptor3d_Surface` handles) and `HLRBRep_InterCSurf` (TKHLR, on `gp_Lin` and
`HLRBRep_Surface*`). `IntPatch_Polyhedron` and the HLRBRep polygon and polyhedron also
call the polygon and polyhedron utilities. The port (`crates/tkgeomalgo/src/int_curve_surface`):

- **Tool traits.** `InterCurveTool` (the `CurveTool` of the templates) extends the IntImp
  `TheCurveTool` with the calls the templates add (GetType, the conics, the intervals,
  NbSamples, SamplePars, IsPeriodic, Period, D0); `InterSurfaceTool` (the `SurfaceTool`)
  extends `ThePSurfaceTool` with GetType, the quadrics and the torus, the sample counts,
  the intervals, UTrim, D0, closure and periodicity, and the swept and offset members
  (BasisSurface, OffsetValue, Direction, BasisCurve, AxeOfRevolution). The trimmed
  surface, basis surface and basis curve are Adaptor3d handles for every instance, as
  natively. `IntCurveSurface_TheHCurveTool` implements both curve traits; the IntImp
  `HSurfaceTool` (Adaptor3d_HSurfaceTool) implements `InterSurfaceTool` here.
- **Utilities.** `IntCurveSurface_InterUtils.pxx`, `_PolygonUtils.pxx`,
  `_PolyhedronUtils.pxx` and `_QuadricCurveExactInterUtils.pxx` are modules of generic
  functions over those traits (`inter_utils`, `polygon_utils`, `polyhedron_utils`,
  `quad_curv_exact`), under the native names in snake case. The polyhedron utilities
  work on slices of points, parameters and bound flags, which the native code reaches
  through `void*` arrays.
- **Section points.** `SectionPointToParameters` and `CollectInterferencePoints` read
  a polyhedron and a polygon only through `Triangle`, `Point`, `Parameters` and
  `ApproxParamOnCurve`; the traits `SectionPolyhedron` and `SectionPolygon` hold those
  calls, so a test can pass its own polyhedron, as the upstream suite does.
- **The polygon.** `IntCurveSurface_ThePolygonOfHInter` and its tool are the generics
  `InterPolygon<C>` and `InterPolygonTool<C>` over an `InterCurveTool`, aliased as
  `ThePolygonOfHInter` and `ThePolygonToolOfHInter` on `TheHCurveTool`; HLRBRep's
  polygon, which runs the same utilities on `gp_Lin`, will be an alias too. The tool
  implements the Intf `Polygon3dTool`.
- **The polyhedron.** `IntCurveSurface_ThePolyhedronOfHInter` is a concrete struct on an
  Adaptor3d surface handle (the grid evaluator and the deflection read the adaptor);
  its tool implements the Intf `PolyhedronTool`. `TheInterferenceOfHInter` is the alias
  `InterferencePolygonPolyhedron<ThePolygonToolOfHInter, ThePolyhedronToolOfHInter>`.
- **The algorithm.** `IntCurveSurface_Inter.pxx` is `InterGen<T: InterTypes>`, a struct
  that holds the `IntCurveSurface_Intersection` base (reached through `Deref`) and
  implements the public and protected members of the instance class. `InterTypes`
  names the curve and surface tools, the polygon and polyhedron with their Intf tools
  (the polygon and polyhedron constructors are the traits `SamplePolygon` and
  `SamplePolyhedron`). The CS function, the exact intersector and the quadric-curve
  intersector are fixed by the tools, as in both native instances:
  `IntImpZerCSParFunc<S, C>`, `IntImpIntCS<...>` and `QuadCurvExact<S, C>` (with
  `QuadCurvFunc<C>`). `IntCurveSurface_HInter` is `InterGen<HInterTypes>`.
- **Callbacks.** The native instance passes lambdas that call back into its own
  members (Perform with bounds, InternalPerform, Append); the generic calls the
  members of `InterGen` directly, which runs the same code in the same order.
- **Exceptions.** Tool calls, the IntAna and ProjLib constructors, and indexed reads
  can raise natively; every member that reaches one returns `Result`, and the
  exception leaves the algorithm where the native one would unwind.

## 12. IntStart templates and IntPolyh

IntStart has no header of its own: its two templates are instantiated by IntPatch
(`IntPatch_TheSOnBounds`, `IntPatch_TheSearchInside`) and by Contap in TKHLR
(`Contap_TheSearch`, `Contap_TheSearchInside`). Both instances use the same concrete
arc, vertex and domain types, so the port fixes them and makes a trait only of what
varies. The port (`crates/tkgeomalgo/src/int_start`):

- **Fixed types.** `TheArc` is `Handle<dyn Adaptor2dCurve2d>` and `TheVertex` is
  `Handle<Adaptor3dHVertex>` in every instance. `ThePSurfaceTool` is the IntImp trait
  `ThePSurfaceTool` (Adaptor3d_HSurfaceTool in both instances). `TheArcTool` and
  `TheSITool` are named by the macros but not called by the bodies, so the traits
  leave them out.
- **The domain.** `TheTopolTool` is `Adaptor3d_TopolTool` in both instances, but the
  native class is virtual (BRepTopAdaptor_TopolTool overrides `Edge()` and the
  iteration), so the calls the bodies make are the trait `TheTopolTool` (sampling,
  classification, the arc and vertex iteration, `Identical`, `Edge`). `Edge()` returns
  `Option<TopoDsEdge>` where the native one returns a `void*` to a `TopoDS_Edge`;
  `Adaptor3dTopolTool` implements it with `None`, as its native `Edge()` returns null.
- **The function and the tool.** `TheFunction` of `SearchOnBoundaries` is the trait
  `TheArcFunction` (a tkmath `FunctionWithDerivative` with `Set(arc)`, `NbSamples`,
  `GetStateNumber`, `Valpoint`, `Quadric`, `Surface`, `LastComputedPoint` and the direct
  `Value`/`Values` calls, fallible); `TheSOBTool` is the trait `TheSOBTool` of static
  members; `IntStart_ThePathPoint` and `IntStart_TheSegment` are the traits
  `ThePathPoint` and `TheSegment`. `TheFunction` of `SearchInside` is the trait
  `TheInsideFunction` (a `FunctionSetWithDerivatives` with the members the body calls),
  implemented here for `IntImpZerImpFunc` (IntPatch_TheSurfFunction).
- **The generics.** `IntStart_SearchOnBoundaries.gxx/.lxx` is
  `SearchOnBoundaries<T: SearchOnBoundariesTypes>` and `IntStart_SearchInside.gxx/.lxx`
  is `SearchInside<T: SearchInsideTypes>`; the type bundles name the traits above. IntPatch
  and Contap declare their instances as aliases when they are ported.
- **Static helpers.** The file-static functions of the `.gxx` (`FindVertex`,
  `BoundedArc`, `PointProcess`, `TreatLC`, `IsRegularity`, `ComputeBoundsfromInfinite`,
  the `MinFunction` wrapper and `SolInfo`) are private functions of the module. The
  `std::sort` of `SolInfo` is the libstdc++ port `rustycad_tkernel::stl::sort::sort_by`,
  so equal parameters keep the native order.
- **Exceptions.** Tool, function and adaptor calls that can raise return `Result`; a
  failure caught by a tkmath solver callback is kept by the function and raised when the
  solver returns (section 6). `math_FunctionAllRoots` that is not done raises
  `Standard_Failure`, as natively.
- **The probe.** `tools/oracle/tkgeomalgo_probe_intstart.cxx` runs both templates through
  their IntPatch instances (`IntPatch_TheSOnBounds`, `IntPatch_TheSearchInside`), with a
  probe domain whose `Edge()` returns a real edge, and includes the boundary `.gxx` on a
  probe class with a probe tool whose arcs "have been seen", so that the branches
  unreachable with IntPatch_HInterTool and Adaptor3d_TopolTool run too. The Rust twin
  (`crates/tkgeomalgo/examples/ga_probe/int_start*.rs`) uses the IntPatch classes
  (section 16) and keeps only those two fixtures.

IntPolyh has no template. Its classes are concrete structs in
`crates/tkgeomalgo/src/int_polyh`:

- `IntPolyh_Array<Type>` is the generic `IntPolyhArray<T>`, and the six array typedefs
  are aliases of it. It keeps the native storage model: `Init(N)` sets element `N` of the
  underlying `NCollection_DynamicArray` (so the storage holds `N + 1` default items),
  `IncrementNbItems` grows it by the increment, and `operator=` (`Copy`) keeps the
  target's increment where the copy constructor (`Clone`) copies it. Native `Value` and
  `operator[]` do not check the index (reading past the storage is undefined behaviour):
  the Rust `value`/`change_value` return Standard_OutOfRange and `Index` panics there.
- `IntPolyh_Triangle::MiddleRefinement` and `MultipleMiddleRefinement` are methods on a
  triangle that is usually an element of the array they grow. The port takes the
  triangle out of the array (a copy), runs the body and writes it back; the body never
  writes the refined triangle through the array before its last two assignments, so the
  result is the native one.
- `IntPolyh_MaillageAffinage` keeps its couples in an `NCollection_List`
  (`rustycad_tkernel::ncollection::List`); `StartPointsChain` walks them by position in a
  vector (the native iterators kept by `CheckCoupleAndGetAngle2` are positions) and puts
  them back in order. The interfering triangles come from the tkmath BVH port
  (`BvhBoxSet` with `BvhLinearBuilder(10)` and a pair traversal), as natively.
- The `printf` dumps write the native text to a `fmt::Write`; the two diagnostics that
  the native algorithm prints to standard output (`CheckSameSP` with undefined edges and
  a null side in `TriangleEdgeContact`) print to standard output too.

## 13. GeomFill filling part

GeomFill has no `.gxx` template of its own except the `GeomFill_AppSurf` instance of
`AppBlend_AppSurf.gxx`; the filling part (`crates/tkgeomalgo/src/geom_fill`, one file per
class, beside the sweep part of the same module) maps the class hierarchies:

- **AppSurf.** `GeomFill_AppSurf` is the alias `GeomFillAppSurf = AppBlendAppSurf`
  (section 7): `GeomFill_Line` implements `TheLine` (its points are the section
  indices) and `GeomFill_SectionGenerator` implements `TheSectionGenerator`. The
  generator reads its sections and parameters through arrays and sequences that raise
  natively, so the trait members `GetShape` and `Parameter` return `Result` too; the
  template reads the parameters only where the native code does (the knot
  reparametrization runs after the pole tables are allocated).
- **Profiler.** `GeomFill_Profiler` is a struct holding the section B-splines as
  `Handle<GeomBSplineCurve>` (every added curve is converted to one); its two
  derived classes embed it and reach it through `Deref`: `GeomFill_SectionGenerator`
  (with its shared parameter array, `Handle<HArray1<f64>>`, which `SetParam` edits in
  place as natively) and `GeomFill_Generator`, whose `perform` replaces the virtual
  `Perform` (nothing calls it through the base class).
- **Filling schemes.** `GeomFill_Filling` is a struct with the protected pole and
  weight arrays (`None` stands for a null handle); `GeomFill_Coons`,
  `GeomFill_Curved` and `GeomFill_Stretch` embed it and reach it through `Deref`, and
  their `Init` overloads take distinct names. `GeomFill_BezierCurves` and
  `GeomFill_BSplineCurves` assign the scheme's `GeomFill_Filling` part, as the native
  slicing assignment `Caro = GeomFill_Stretch(...)` does.
- **Boundaries.** The abstract transient `GeomFill_Boundary` is the trait
  `GeomFillBoundary`, reached through `Handle<dyn GeomFillBoundary>`. Its private
  tolerances live in each concrete struct as a `GeomFillBoundaryTolerances` (a lock,
  since `Tol3d(Tol)` edits a shared object) that the trait reaches through one
  required accessor; `HasNormals`, `Norm`, `D1Norm` and `Points` are provided
  methods with the native bodies. `Reparametrize` edits a shared boundary, so
  `GeomFill_SimpleBound`, `GeomFill_BoundWithSurf` and `GeomFill_DegeneratedBound` keep
  their parameter law or range behind a lock. `GeomFill_BoundWithSurf` holds a copy
  of the `Adaptor3d_CurveOnSurface` (the native member is a copy).
- **Tangent fields.** `GeomFill_TgtField` is the trait `GeomFillTgtField`
  (`IsScalable` and `Scale` provided, false and empty as natively); the two `D1`
  overloads are `d1` and `d1_with_value`. `GeomFill_TgtOnCoons` holds its
  `Handle<GeomFillCoonsAlgPatch>` and bound index.
- **Coons patch.** `GeomFill_CoonsAlgPatch` is a handle-shared struct whose blending
  laws sit behind a lock (`SetFunc` edits the patch that the tangent fields share);
  the fixed arrays `bound[4]`, `c[4]` and `a[2]` are read without a bounds check
  natively, so `Bound`, `Corner` and `Func` raise Standard_OutOfRange outside them.
- **Constrained filling.** `GeomFill_ConstrainedFilling` is a plain struct with the
  native fields; its file static helpers (`sortbounds`, `coonscnd`, `killcorners`,
  `inqadd`, `mklin`) are private functions, and the evaluator class
  `GeomFill_ConstrainedFilling_Eval` is a closure over the struct passed to
  `AdvApprox_ApproxAFunction`. A boundary or field exception inside the evaluator is
  kept (the native exception unwinds through the approximation), the evaluator
  answers an error code, and the failure is raised when the approximation returns.
  The debug members `CheckTgteField`, `CheckApprox` and `CheckResult` write to a
  `fmt::Write` where the native ones write to `std::cout`.

## 15. GeomAPI: the interpolation body and the Extrema members

GeomAPI (`crates/tkgeomalgo/src/geom_api`) has no `.gxx` template, but two constructs
need a rule.

- **One interpolation body.** `GeomAPI_Interpolate.cxx` is `Geom2dAPI_Interpolate.cxx`
  with `gp_Pnt`, `gp_Vec` and `Geom_BSplineCurve` for `gp_Pnt2d`, `gp_Vec2d` and
  `Geom2d_BSplineCurve`, and with 3 in place of 2 in the coordinate loops and the
  `PLib::EvalLagrange` dimension. The body is ported once, as the generic
  `geom2d_api::InterpolateGen<S: InterpolateSpace>` (the files `interpolate`,
  `interpolate_build` and `interpolate_perform`). The trait
  `geom2d_api::interpolate_space::InterpolateSpace` names the point (a tkmath
  `bsplclib::CurvePoint`, which gives the coordinates and the vector type), the curve
  and the point, vector and curve members the body calls. `geom2d_api::Interpolate`
  is the alias on `Interpolate2dSpace` and `geom_api::Interpolate` the alias on
  `Interpolate3dSpace`, declared in the package of each class. The loops keep the
  native order (`value += |Coord(jj)|` from 0, `ratio * Coord(jj)`), so each
  instance runs the operations of its native class.
- **Extrema members.** Each GeomAPI class holds an Extrema object and the adaptors
  that the Extrema object points to natively; in Rust an Extrema object borrows its
  adaptors, so it cannot live beside them in one struct. Each class takes the rule that
  reproduces the native state:
  - `GeomAPI_ProjectPointOnSurf` keeps its `Extrema_ExtPS` (the sampling grid, the tree
    and the swept-surface algorithms persist between Performs, and `SetExtremaAlgo` and
    `SetExtremaFlag` act between an Init and a Perform). `Extrema_ExtPS` takes its
    surface through a shared handle (`ExtPS::initialize_shared`, as
    `Extrema_GenExtPS` already does with `SurfaceRef`), so the object holds
    `ExtPS<'static>` and the adaptor handle.
  - `GeomAPI_ExtremaCurveCurve` keeps one `Extrema_ExtCC` from Init to Init, and
    `Extrema_ExtCC` keeps its `Extrema_ECC` solutions and its end points across
    Performs. The object records the arguments of each Init and replays them, in
    order, on one `ExtCC` over its own adaptors (which each Init reloads, as natively);
    the replay also rebuilds `Extrema()`. The cost is one Extrema computation per
    earlier Init of the same object.
  - `GeomAPI_ProjectPointOnCurve`, `GeomAPI_ExtremaCurveSurface` and
    `GeomAPI_ExtremaSurfaceSurface` keep the results of their Extrema object: its
    state does not carry from one computation to the next (`Extrema_ExtPC` resets
    its samples and results at each Perform; the two others are initialized and
    performed on local adaptors at each Init, and `Extrema_ExtSS` is even copied
    from a new object). `Extrema()` rebuilds the same object on stored adaptors.
- **GeomFill Gordon.** `GeomFill_GordonUtilities.pxx` is an internal header of free
  functions and small structs; it is ported as private modules of `geom_fill`, one per
  step (parameters, contacts, normalization, ordering, reordering, reparametrization,
  rebuild, network preparation, fallback), which `GeomFill_Gordon` and
  `GeomFill_NetworkSurface` call as natively. The rational denominator product of
  `GeomFill_NetworkSurface` goes through the `bsplclib::EvaluatorFunction` callback,
  which cannot return `Result`; the evaluator stores the first failure in a `RefCell`
  and the caller raises it after the call, as the native exception would leave it.
  `OSD_Parallel::For` over the contact searches is a sequential loop: each iteration
  writes only its own slot, so the order does not change the result.

## 16. IntPatch core

IntPatch (`crates/tkgeomalgo/src/int_patch`) is the surface-surface intersection. Its
core (the records, the lines, the tools and the template instances) lands first; the
intersection algorithms build on it.

- **Lines.** The abstract transients `IntPatch_Line` and `IntPatch_PointLine` are the
  traits `IntPatchLine` and `IntPatchPointLine: IntPatchLine`, reached through
  `Handle<dyn IntPatchLine>` and `Handle<dyn IntPatchPointLine>` (docs/geom-design.md
  sections 1 and 2). The fields of `IntPatch_Line` (the type, the transitions, the
  situations and the iso flags) live in an embedded `IntPatchLineBase` behind a lock;
  its protected constructors are `IntPatchLineBase::new(tang, IntPatchLineTransitions,
  typ)`, where `IntPatchLineTransitions` names the three of them (IN/OUT types, TOUCH
  situations, undecided). The concrete lines `IntPatchWLine`, `IntPatchRLine`,
  `IntPatchGLine` and `IntPatchALine` keep their own fields in one `SharedState` each,
  since the algorithms edit the lines through their handles. `IntPatch_PointLine` has
  one static member, `CurvatureRadiusOfIntersLine`, which is the free function
  `curvature_radius_of_inters_line`.
- **Vertices.** The four lines declare the same vertex members (`svtx`, `fipt`, `lapt`,
  `indf`, `indl` and their inline accessors); they share one `VertexList`. `Vertex(i)`
  and `FirstPoint()` return copies; `ChangeVertex(i)` runs a closure on the vertex in
  place. The `ComputeVertexParameters` bodies are split by phase (the WLine one into
  the seam duplication, the restriction filters, the merge with the walking points and
  the end vertices), and the locals the native loops keep across their phases (`Tol`,
  `nbvtx`, `nbponline`) live in one state struct, so each phase reads the stale counts
  the native code reads.
- **Tools.** `IntPatch_HInterTool` is a struct with the sampled range and associated
  functions; it implements the IntStart trait `TheSOBTool`. `IntPatch_HCurve2dTool`
  implements the IntImp trait `TheCurveTool`: the IntImp curve tool is split into
  `TheCurveTool` (the bounds and the resolution, all that `IntImp_IntCS` calls) and
  `TheCurve3dTool` (the 3d `Value` and `D1` of `IntImp_ZerCSParFunc`), since the 2d
  arcs of `IntPatch_CurvIntSurf` have no 3d value. `IntPatch_ArcFunction` implements
  the IntStart `TheArcFunction` and `IntPatch_CSFunction` the IntImp `TheFunction`
  (section 6); both keep a failure of their solver callbacks and raise it when the
  solver returns.
- **Instances.** The instances are aliases over type bundles, as the `_0.cxx` files
  name their macros: `IntPatchTheSurfFunction = IntImpZerImpFunc<HSurfaceTool,
  IntSurfQuadricTool>`, `IntPatchCurvIntSurf = IntImpIntCS<IntPatchCSFunction>`,
  `IntPatchTheSOnBounds = SearchOnBoundaries<IntPatchTheSOnBoundsTypes>`,
  `IntPatchTheSearchInside = SearchInside<IntPatchTheSearchInsideTypes>` and
  `IntPatchTheIWalking = IntWalkIWalking<IntPatchTheIWalkingTypes>` (sections 10 and 12).
  The domain of the IntStart instances is `dyn TheTopolTool`, since the native
  `Adaptor3d_TopolTool` is virtual. The path point, the segment and the walking line
  (`IntPatchThePathPointOfTheSOnBounds`, `IntPatchTheSegmentOfTheSOnBounds`,
  `IntPatchTheIWLineOfTheIWalking`) are the classes the IntStart and IntWalk probes and
  tests use; the probe-only copies of them are gone.
- **Dumps.** The `Dump` members write to a `fmt::Write` the text the native ones print
  with `printf` and `std::cout` (`%+10.20f`, the default `operator<<(double)`, and `%10p`
  addresses, which are the Rust object addresses). The probe captures the native process
  output to compare the dumps of points and lines without arcs or vertices.
