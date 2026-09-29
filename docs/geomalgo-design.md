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

## 2. IntImpParGen_Intersector.gxx

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
