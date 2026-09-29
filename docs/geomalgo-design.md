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
