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

## 2. IntImp templates

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

## 3. AppBlend template

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
