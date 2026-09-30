//! The BRepApprox instances of the ApproxInt, IntImp, Approx and AppParCurves templates
//! (docs/topalgo-design.md section 12, docs/geomalgo-design.md sections 6 and 20). Each
//! native `_0.cxx` defines the macros and includes the `.gxx`; each instance here is an
//! alias of the Rust generic over the same types: `BRepApprox_SurfaceTool`
//! ([`BRepApproxSurfaceTool`]), `IntSurf_Quadric` (`IntSurfQuadricTool`) and
//! `BRepApprox_ApproxLine` ([`BRepApproxApproxLine`]). The AppParCurves and Approx
//! generics take any `LineTool` per call, so their instances are the generic types
//! themselves.
use super::approx_line::BRepApproxApproxLine;
use super::surface_tool::BRepApproxSurfaceTool;
use rustycad_tkgeomalgo::approx_int::{
    ApproxIntApprox, ApproxIntApproxTypes, ApproxIntImpPrmSvSurfaces, ApproxIntMultiLine,
    ApproxIntPrmPrmSvSurfaces,
};
use rustycad_tkgeomalgo::int_imp::{IntImpInt2S, IntImpZerImpFunc, IntImpZerParFunc};
use rustycad_tkgeomalgo::int_surf::IntSurfQuadricTool;
use rustycad_tkgeombase::app_par_curves::{
    BSpFunction, BSpGradient, Function, Gradient, GradientBfgs, LeastSquare, ResolConstraint,
};
use rustycad_tkgeombase::approx::{BSplComputeLine, ComputeLine};

/// The macros of `BRepApprox_Approx_0.cxx`: `ThePSurfaceTool = BRepApprox_SurfaceTool`,
/// `TheWLine = BRepApprox_ApproxLine`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepApproxApproxTypes;

impl ApproxIntApproxTypes for BRepApproxApproxTypes {
    type SurfaceTool = BRepApproxSurfaceTool;
    type Line = BRepApproxApproxLine;
}

/// `BRepApprox_Approx`: `ApproxInt_Approx` over `BRepAdaptor_Surface` and
/// `BRepApprox_ApproxLine`; `BRepApproxApprox::new()`.
pub type BRepApproxApprox = ApproxIntApprox<BRepApproxApproxTypes>;

/// `BRepApprox_TheMultiLineOfApprox`: `ApproxInt_MultiLine` over `BRepApprox_ApproxLine`;
/// its `LineTool` implementation is `BRepApprox_TheMultiLineToolOfApprox`;
/// `BRepApproxTheMultiLineOfApprox::with_line(line, layout)`.
pub type BRepApproxTheMultiLineOfApprox = ApproxIntMultiLine<BRepApproxApproxLine>;

/// `BRepApprox_ThePrmPrmSvSurfacesOfApprox`: `ApproxInt_PrmPrmSvSurfaces` over
/// `BRepApprox_SurfaceTool`; `BRepApproxThePrmPrmSvSurfacesOfApprox::new(&s1, &s2)?`.
pub type BRepApproxThePrmPrmSvSurfacesOfApprox = ApproxIntPrmPrmSvSurfaces<BRepApproxSurfaceTool>;

/// `BRepApprox_TheImpPrmSvSurfacesOfApprox`: `ApproxInt_ImpPrmSvSurfaces` over
/// `BRepApprox_SurfaceTool` and `IntSurf_Quadric`;
/// `BRepApproxTheImpPrmSvSurfacesOfApprox::quadric_first(&quadric, &s2)`.
pub type BRepApproxTheImpPrmSvSurfacesOfApprox = ApproxIntImpPrmSvSurfaces<BRepApproxSurfaceTool>;

/// `BRepApprox_TheInt2SOfThePrmPrmSvSurfacesOfApprox`: `IntImp_Int2S` over
/// `BRepApprox_SurfaceTool`;
/// `BRepApproxTheInt2SOfThePrmPrmSvSurfacesOfApprox::new(&s1, &s2, 1e-10)?`.
pub type BRepApproxTheInt2SOfThePrmPrmSvSurfacesOfApprox = IntImpInt2S<BRepApproxSurfaceTool>;

/// `BRepApprox_TheFunctionOfTheInt2SOfThePrmPrmSvSurfacesOfApprox`: `IntImp_ZerParFunc`
/// over `BRepApprox_SurfaceTool`;
/// `BRepApproxTheFunctionOfTheInt2SOfThePrmPrmSvSurfacesOfApprox::new(&s1, &s2)?`.
pub type BRepApproxTheFunctionOfTheInt2SOfThePrmPrmSvSurfacesOfApprox =
    IntImpZerParFunc<BRepApproxSurfaceTool>;

/// `BRepApprox_TheZerImpFuncOfTheImpPrmSvSurfacesOfApprox`: `IntImp_ZerImpFunc` over
/// `BRepApprox_SurfaceTool` and `IntSurf_QuadricTool`;
/// `BRepApproxTheZerImpFuncOfTheImpPrmSvSurfacesOfApprox::with_surfaces(&s, &quadric)`.
pub type BRepApproxTheZerImpFuncOfTheImpPrmSvSurfacesOfApprox =
    IntImpZerImpFunc<BRepApproxSurfaceTool, IntSurfQuadricTool>;

/// `BRepApprox_TheComputeLineOfApprox`: `Approx_BSplComputeLine` over the multi-line.
pub type BRepApproxTheComputeLineOfApprox = BSplComputeLine;
/// `BRepApprox_TheComputeLineBezierOfApprox`: `Approx_ComputeLine` over the multi-line.
pub type BRepApproxTheComputeLineBezierOfApprox = ComputeLine;

/// `BRepApprox_MyBSplGradientOfTheComputeLineOfApprox`: `AppParCurves_BSpGradient`, the
/// B-spline parameter optimizer of [`BRepApproxTheComputeLineOfApprox`].
pub type BRepApproxMyBSplGradientOfTheComputeLineOfApprox = BSpGradient;
/// `BRepApprox_MyGradientbisOfTheComputeLineOfApprox`: `AppParCurves_Gradient`, the
/// Bezier parameter optimizer of [`BRepApproxTheComputeLineOfApprox`].
pub type BRepApproxMyGradientbisOfTheComputeLineOfApprox = Gradient;
/// `BRepApprox_MyGradientOfTheComputeLineBezierOfApprox`: `AppParCurves_Gradient`, the
/// parameter optimizer of [`BRepApproxTheComputeLineBezierOfApprox`].
pub type BRepApproxMyGradientOfTheComputeLineBezierOfApprox = Gradient;

/// `BRepApprox_BSpParLeastSquareOfMyBSplGradientOfTheComputeLineOfApprox`:
/// `AppParCurves_LeastSquare` of the B-spline gradient.
pub type BRepApproxBSpParLeastSquareOfMyBSplGradientOfTheComputeLineOfApprox = LeastSquare;
/// `BRepApprox_ParLeastSquareOfMyGradientbisOfTheComputeLineOfApprox`.
pub type BRepApproxParLeastSquareOfMyGradientbisOfTheComputeLineOfApprox = LeastSquare;
/// `BRepApprox_ParLeastSquareOfMyGradientOfTheComputeLineBezierOfApprox`.
pub type BRepApproxParLeastSquareOfMyGradientOfTheComputeLineBezierOfApprox = LeastSquare;

/// `BRepApprox_ResConstraintOfMyGradientbisOfTheComputeLineOfApprox`:
/// `AppParCurves_ResolConstraint` of the Bezier gradient of the B-spline line.
pub type BRepApproxResConstraintOfMyGradientbisOfTheComputeLineOfApprox = ResolConstraint;
/// `BRepApprox_ResConstraintOfMyGradientOfTheComputeLineBezierOfApprox`.
pub type BRepApproxResConstraintOfMyGradientOfTheComputeLineBezierOfApprox = ResolConstraint;

/// `BRepApprox_ParFunctionOfMyGradientbisOfTheComputeLineOfApprox`:
/// `AppParCurves_Function` over the multi-line.
pub type BRepApproxParFunctionOfMyGradientbisOfTheComputeLineOfApprox =
    Function<BRepApproxTheMultiLineOfApprox>;
/// `BRepApprox_ParFunctionOfMyGradientOfTheComputeLineBezierOfApprox`.
pub type BRepApproxParFunctionOfMyGradientOfTheComputeLineBezierOfApprox =
    Function<BRepApproxTheMultiLineOfApprox>;
/// `BRepApprox_BSpParFunctionOfMyBSplGradientOfTheComputeLineOfApprox`:
/// `AppParCurves_BSpFunction` over the multi-line.
pub type BRepApproxBSpParFunctionOfMyBSplGradientOfTheComputeLineOfApprox =
    BSpFunction<BRepApproxTheMultiLineOfApprox>;

/// `BRepApprox_Gradient_BFGSOfMyGradientbisOfTheComputeLineOfApprox`:
/// `AppParCurves_Gradient_BFGS`, math_BFGS stopping on the fit tolerances too.
pub type BRepApproxGradientBfgsOfMyGradientbisOfTheComputeLineOfApprox = GradientBfgs;
/// `BRepApprox_Gradient_BFGSOfMyGradientOfTheComputeLineBezierOfApprox`.
pub type BRepApproxGradientBfgsOfMyGradientOfTheComputeLineBezierOfApprox = GradientBfgs;
/// `BRepApprox_BSpGradient_BFGSOfMyBSplGradientOfTheComputeLineOfApprox`:
/// `AppParCurves_BSpGradient_BFGS`.
pub type BRepApproxBSpGradientBfgsOfMyBSplGradientOfTheComputeLineOfApprox = GradientBfgs;
