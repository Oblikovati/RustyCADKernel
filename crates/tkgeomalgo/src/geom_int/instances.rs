//! The GeomInt instances of the ApproxInt, IntImp, Approx and AppParCurves templates
//! (docs/geomalgo-design.md sections 6 and 20). Each native `_0.cxx` defines the macros
//! and includes the `.gxx`; each instance here is an alias of the Rust generic over the
//! same types: `Adaptor3d_HSurfaceTool` (`HSurfaceTool`), `IntSurf_Quadric`
//! (`IntSurfQuadricTool`) and `IntPatch_WLine`. The AppParCurves and Approx generics take
//! any `LineTool` per call, so their instances are the generic types themselves (the
//! functions are generic in the line type).
use crate::approx_int::{
    ApproxIntApprox, ApproxIntApproxTypes, ApproxIntImpPrmSvSurfaces, ApproxIntMultiLine,
    ApproxIntPrmPrmSvSurfaces,
};
use crate::int_imp::{HSurfaceTool, IntImpInt2S, IntImpZerImpFunc, IntImpZerParFunc};
use crate::int_patch::IntPatchWLine;
use crate::int_surf::IntSurfQuadricTool;
use rustycad_tkgeombase::app_par_curves::{
    BSpFunction, BSpGradient, Function, Gradient, GradientBfgs, LeastSquare, ResolConstraint,
};
use rustycad_tkgeombase::approx::{BSplComputeLine, ComputeLine};

/// The macros of `GeomInt_WLApprox_0.cxx`: `ThePSurfaceTool = Adaptor3d_HSurfaceTool`,
/// `TheWLine = IntPatch_WLine`.
#[derive(Clone, Copy, Debug, Default)]
pub struct GeomIntWLApproxTypes;

impl ApproxIntApproxTypes for GeomIntWLApproxTypes {
    type SurfaceTool = HSurfaceTool;
    type Line = IntPatchWLine;
}

/// `GeomInt_WLApprox`: `ApproxInt_Approx` over `Adaptor3d_Surface` handles and
/// `IntPatch_WLine`; `GeomIntWLApprox::new()`.
pub type GeomIntWLApprox = ApproxIntApprox<GeomIntWLApproxTypes>;

/// `GeomInt_TheMultiLineOfWLApprox`: `ApproxInt_MultiLine` over `IntPatch_WLine`; its
/// `LineTool` implementation is `GeomInt_TheMultiLineToolOfWLApprox`;
/// `GeomIntTheMultiLineOfWLApprox::with_line(wline, layout)`.
pub type GeomIntTheMultiLineOfWLApprox = ApproxIntMultiLine<IntPatchWLine>;

/// `GeomInt_ThePrmPrmSvSurfacesOfWLApprox`: `ApproxInt_PrmPrmSvSurfaces` over
/// `Adaptor3d_HSurfaceTool`; `GeomIntThePrmPrmSvSurfacesOfWLApprox::new(&s1, &s2)?`.
pub type GeomIntThePrmPrmSvSurfacesOfWLApprox = ApproxIntPrmPrmSvSurfaces<HSurfaceTool>;

/// `GeomInt_TheImpPrmSvSurfacesOfWLApprox`: `ApproxInt_ImpPrmSvSurfaces` over
/// `Adaptor3d_HSurfaceTool` and `IntSurf_Quadric`;
/// `GeomIntTheImpPrmSvSurfacesOfWLApprox::quadric_first(&quadric, &s2)`.
pub type GeomIntTheImpPrmSvSurfacesOfWLApprox = ApproxIntImpPrmSvSurfaces<HSurfaceTool>;

/// `GeomInt_TheInt2SOfThePrmPrmSvSurfacesOfWLApprox`: `IntImp_Int2S` over
/// `Adaptor3d_HSurfaceTool` (section 6);
/// `GeomIntTheInt2SOfThePrmPrmSvSurfacesOfWLApprox::new(&s1, &s2, 1e-10)?`.
pub type GeomIntTheInt2SOfThePrmPrmSvSurfacesOfWLApprox = IntImpInt2S<HSurfaceTool>;

/// `GeomInt_TheFunctionOfTheInt2SOfThePrmPrmSvSurfacesOfWLApprox`: `IntImp_ZerParFunc`
/// over `Adaptor3d_HSurfaceTool`;
/// `GeomIntTheFunctionOfTheInt2SOfThePrmPrmSvSurfacesOfWLApprox::new(&s1, &s2)?`.
pub type GeomIntTheFunctionOfTheInt2SOfThePrmPrmSvSurfacesOfWLApprox =
    IntImpZerParFunc<HSurfaceTool>;

/// `GeomInt_TheZerImpFuncOfTheImpPrmSvSurfacesOfWLApprox`: `IntImp_ZerImpFunc` over
/// `Adaptor3d_HSurfaceTool` and `IntSurf_QuadricTool`;
/// `GeomIntTheZerImpFuncOfTheImpPrmSvSurfacesOfWLApprox::with_surfaces(&s, &quadric)`.
pub type GeomIntTheZerImpFuncOfTheImpPrmSvSurfacesOfWLApprox =
    IntImpZerImpFunc<HSurfaceTool, IntSurfQuadricTool>;

/// `GeomInt_TheComputeLineOfWLApprox`: `Approx_BSplComputeLine` over the multi-line.
pub type GeomIntTheComputeLineOfWLApprox = BSplComputeLine;
/// `GeomInt_TheComputeLineBezierOfWLApprox`: `Approx_ComputeLine` over the multi-line.
pub type GeomIntTheComputeLineBezierOfWLApprox = ComputeLine;

/// `GeomInt_MyBSplGradientOfTheComputeLineOfWLApprox`: `AppParCurves_BSpGradient`, the
/// B-spline parameter optimizer of [`GeomIntTheComputeLineOfWLApprox`].
pub type GeomIntMyBSplGradientOfTheComputeLineOfWLApprox = BSpGradient;
/// `GeomInt_MyGradientbisOfTheComputeLineOfWLApprox`: `AppParCurves_Gradient`, the Bezier
/// parameter optimizer of [`GeomIntTheComputeLineOfWLApprox`].
pub type GeomIntMyGradientbisOfTheComputeLineOfWLApprox = Gradient;
/// `GeomInt_MyGradientOfTheComputeLineBezierOfWLApprox`: `AppParCurves_Gradient`, the
/// parameter optimizer of [`GeomIntTheComputeLineBezierOfWLApprox`].
pub type GeomIntMyGradientOfTheComputeLineBezierOfWLApprox = Gradient;

/// `GeomInt_BSpParLeastSquareOfMyBSplGradientOfTheComputeLineOfWLApprox`:
/// `AppParCurves_LeastSquare` of the B-spline gradient.
pub type GeomIntBSpParLeastSquareOfMyBSplGradientOfTheComputeLineOfWLApprox = LeastSquare;
/// `GeomInt_ParLeastSquareOfMyGradientbisOfTheComputeLineOfWLApprox`.
pub type GeomIntParLeastSquareOfMyGradientbisOfTheComputeLineOfWLApprox = LeastSquare;
/// `GeomInt_ParLeastSquareOfMyGradientOfTheComputeLineBezierOfWLApprox`.
pub type GeomIntParLeastSquareOfMyGradientOfTheComputeLineBezierOfWLApprox = LeastSquare;

/// `GeomInt_ResConstraintOfMyGradientbisOfTheComputeLineOfWLApprox`:
/// `AppParCurves_ResolConstraint` of the Bezier gradient of the B-spline line.
pub type GeomIntResConstraintOfMyGradientbisOfTheComputeLineOfWLApprox = ResolConstraint;
/// `GeomInt_ResConstraintOfMyGradientOfTheComputeLineBezierOfWLApprox`.
pub type GeomIntResConstraintOfMyGradientOfTheComputeLineBezierOfWLApprox = ResolConstraint;

/// `GeomInt_ParFunctionOfMyGradientbisOfTheComputeLineOfWLApprox`:
/// `AppParCurves_Function` over the multi-line.
pub type GeomIntParFunctionOfMyGradientbisOfTheComputeLineOfWLApprox =
    Function<GeomIntTheMultiLineOfWLApprox>;
/// `GeomInt_ParFunctionOfMyGradientOfTheComputeLineBezierOfWLApprox`.
pub type GeomIntParFunctionOfMyGradientOfTheComputeLineBezierOfWLApprox =
    Function<GeomIntTheMultiLineOfWLApprox>;
/// `GeomInt_BSpParFunctionOfMyBSplGradientOfTheComputeLineOfWLApprox`:
/// `AppParCurves_BSpFunction` over the multi-line.
pub type GeomIntBSpParFunctionOfMyBSplGradientOfTheComputeLineOfWLApprox =
    BSpFunction<GeomIntTheMultiLineOfWLApprox>;

/// `GeomInt_Gradient_BFGSOfMyGradientbisOfTheComputeLineOfWLApprox`:
/// `AppParCurves_Gradient_BFGS`, math_BFGS stopping on the fit tolerances too.
pub type GeomIntGradientBfgsOfMyGradientbisOfTheComputeLineOfWLApprox = GradientBfgs;
/// `GeomInt_Gradient_BFGSOfMyGradientOfTheComputeLineBezierOfWLApprox`.
pub type GeomIntGradientBfgsOfMyGradientOfTheComputeLineBezierOfWLApprox = GradientBfgs;
/// `GeomInt_BSpGradient_BFGSOfMyBSplGradientOfTheComputeLineOfWLApprox`:
/// `AppParCurves_BSpGradient_BFGS`.
pub type GeomIntBSpGradientBfgsOfMyBSplGradientOfTheComputeLineOfWLApprox = GradientBfgs;
