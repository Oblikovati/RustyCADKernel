// Rust translation of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomGridEval_Curve`: the grid evaluator that picks a specialized one per curve kind.
use super::{
    GeomGridEvalBSplineCurve, GeomGridEvalBezierCurve, GeomGridEvalCircle, GeomGridEvalCurveD1,
    GeomGridEvalCurveD2, GeomGridEvalCurveD3, GeomGridEvalEllipse, GeomGridEvalHyperbola,
    GeomGridEvalLine, GeomGridEvalOffsetCurve, GeomGridEvalOtherCurve, GeomGridEvalParabola,
};
use crate::adaptor3d::Adaptor3dCurve;
use crate::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
};
use crate::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d_grid_eval::grid_loops::empty;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The specialized evaluator held by `GeomGridEval_Curve` (`EvaluatorVariant`);
/// `Uninitialized` is the native `std::monostate`, whose grids are empty.
pub enum GeomGridEvalCurveEvaluator<'a> {
    /// No evaluator: a null curve or a curve class without a specialized evaluator.
    Uninitialized,
    /// `GeomGridEval_Line`.
    Line(GeomGridEvalLine),
    /// `GeomGridEval_Circle`.
    Circle(GeomGridEvalCircle),
    /// `GeomGridEval_Ellipse`.
    Ellipse(GeomGridEvalEllipse),
    /// `GeomGridEval_Hyperbola`.
    Hyperbola(GeomGridEvalHyperbola),
    /// `GeomGridEval_Parabola`.
    Parabola(GeomGridEvalParabola),
    /// `GeomGridEval_BezierCurve`.
    Bezier(GeomGridEvalBezierCurve),
    /// `GeomGridEval_BSplineCurve`.
    BSpline(GeomGridEvalBSplineCurve),
    /// `GeomGridEval_OffsetCurve`.
    Offset(GeomGridEvalOffsetCurve),
    /// `GeomGridEval_OtherCurve`, the adaptor fallback.
    Other(GeomGridEvalOtherCurve<'a>),
}

// Calls the same member on whichever evaluator is held; the Line evaluator cannot fail.
macro_rules! dispatch {
    ($self:ident, $eval:ident => $call:expr) => {
        match &$self.evaluator {
            GeomGridEvalCurveEvaluator::Uninitialized => Ok(empty()),
            GeomGridEvalCurveEvaluator::Line($eval) => Ok($call),
            GeomGridEvalCurveEvaluator::Circle($eval) => $call,
            GeomGridEvalCurveEvaluator::Ellipse($eval) => $call,
            GeomGridEvalCurveEvaluator::Hyperbola($eval) => $call,
            GeomGridEvalCurveEvaluator::Parabola($eval) => $call,
            GeomGridEvalCurveEvaluator::Bezier($eval) => $call,
            GeomGridEvalCurveEvaluator::BSpline($eval) => $call,
            GeomGridEvalCurveEvaluator::Offset($eval) => $call,
            GeomGridEvalCurveEvaluator::Other($eval) => $call,
        }
    };
}

/// Grid evaluator of any 3D curve (`GeomGridEval_Curve`).
///
/// From a curve handle, trimmed wrappers are stripped and the basis class selects the
/// evaluator in the order line, circle, ellipse, hyperbola, parabola, Bezier,
/// B-spline, offset; any other class (the GeomEval curves) and a null handle leave it
/// uninitialized, so every grid is empty and the type is `OtherCurve`. Trimming bounds
/// are not applied. From an adaptor, a `GeomAdaptor_Curve` goes through its curve
/// (ignoring the adaptor range), and any other adaptor through the
/// `GeomGridEval_OtherCurve` fallback with the adaptor's `GetType`;
/// `GeomGridEvalCurve::from_curve(&curve).evaluate_grid(&params)?`.
pub struct GeomGridEvalCurve<'a> {
    evaluator: GeomGridEvalCurveEvaluator<'a>,
    curve_type: CurveType,
}

impl<'a> GeomGridEvalCurve<'a> {
    /// Evaluator of an adaptor (`GeomGridEval_Curve(const Adaptor3d_Curve&)`); the
    /// fallback borrows `curve`. Fails when the adaptor's `GetType` raises;
    /// `GeomGridEvalCurve::from_adaptor(&adaptor)?`.
    pub fn from_adaptor(curve: &'a dyn Adaptor3dCurve) -> Result<Self, Failure> {
        let mut result = Self::uninitialized();
        result.initialization_adaptor(curve)?;
        Ok(result)
    }
    /// Evaluator of a curve handle (`GeomGridEval_Curve(const occ::handle<Geom_Curve>&)`);
    /// `GeomGridEvalCurve::from_curve(&curve)`.
    pub fn from_curve(curve: &Handle<dyn GeomCurve>) -> Self {
        let mut result = Self::uninitialized();
        result.initialization_curve(curve);
        result
    }
    fn uninitialized() -> Self {
        Self {
            evaluator: GeomGridEvalCurveEvaluator::Uninitialized,
            curve_type: CurveType::OtherCurve,
        }
    }
    // `initialization(const Adaptor3d_Curve&)`.
    fn initialization_adaptor(&mut self, curve: &'a dyn Adaptor3dCurve) -> Result<(), Failure> {
        if let Some(geom_adaptor) = GeomAdaptorCurve::project_from(curve) {
            self.initialization_curve(&geom_adaptor.curve());
            return Ok(());
        }
        self.curve_type = curve.get_type()?;
        self.evaluator = GeomGridEvalCurveEvaluator::Other(GeomGridEvalOtherCurve::new(curve));
        Ok(())
    }
    // `initialization(const occ::handle<Geom_Curve>&)`.
    fn initialization_curve(&mut self, curve: &Handle<dyn GeomCurve>) {
        let (curve_type, evaluator) = classify(&extract_basis_curve(curve));
        self.curve_type = curve_type;
        self.evaluator = evaluator;
    }
    /// The held evaluator (the private `myEvaluator`, read-only); `evaluator.evaluator()`.
    pub fn evaluator(&self) -> &GeomGridEvalCurveEvaluator<'a> {
        &self.evaluator
    }
    /// The detected kind (`GetType`); `evaluator.get_type() == CurveType::Line`.
    pub fn get_type(&self) -> CurveType {
        self.curve_type
    }
    /// Points at every parameter (`EvaluateGrid`); `evaluator.evaluate_grid(&params)?`.
    pub fn evaluate_grid(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Point3>, Failure> {
        dispatch!(self, eval => eval.evaluate_grid(params))
    }
    /// Points and D1 (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD1>, Failure> {
        dispatch!(self, eval => eval.evaluate_grid_d1(params))
    }
    /// Points, D1 and D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD2>, Failure> {
        dispatch!(self, eval => eval.evaluate_grid_d2(params))
    }
    /// Points, D1, D2 and D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, GeomGridEvalCurveD3>, Failure> {
        dispatch!(self, eval => eval.evaluate_grid_d3(params))
    }
    /// Derivative of order `n` at every parameter (`EvaluateGridDN`);
    /// `evaluator.evaluate_grid_dn(&params, 2)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector3>, Failure> {
        dispatch!(self, eval => eval.evaluate_grid_dn(params, n))
    }
}

/// `ExtractBasisCurve`: the basis under any number of trimmed-curve wrappers.
fn extract_basis_curve(curve: &Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    let mut result: Handle<dyn GeomCurve> = curve.clone();
    loop {
        let trimmed: Handle<GeomTrimmedCurve> = result.cast::<GeomTrimmedCurve>();
        let Some(wrapper) = trimmed.get() else {
            return result;
        };
        result = wrapper.basis_curve();
    }
}

// The `occ::down_cast` chain of `initialization`: the first class that matches wins;
// a null handle matches none and leaves the evaluator uninitialized.
fn classify<'a>(basis: &Handle<dyn GeomCurve>) -> (CurveType, GeomGridEvalCurveEvaluator<'a>) {
    use GeomGridEvalCurveEvaluator as E;
    if let Some(line) = kind::<GeomLine>(basis) {
        return (CurveType::Line, E::Line(GeomGridEvalLine::new(line)));
    }
    if let Some(circle) = kind::<GeomCircle>(basis) {
        return (
            CurveType::Circle,
            E::Circle(GeomGridEvalCircle::new(circle)),
        );
    }
    if let Some(ellipse) = kind::<GeomEllipse>(basis) {
        return (
            CurveType::Ellipse,
            E::Ellipse(GeomGridEvalEllipse::new(ellipse)),
        );
    }
    if let Some(hyperbola) = kind::<GeomHyperbola>(basis) {
        let evaluator = GeomGridEvalHyperbola::new(hyperbola);
        return (CurveType::Hyperbola, E::Hyperbola(evaluator));
    }
    if let Some(parabola) = kind::<GeomParabola>(basis) {
        return (
            CurveType::Parabola,
            E::Parabola(GeomGridEvalParabola::new(parabola)),
        );
    }
    classify_bounded(basis)
}

// The B-spline family and the offset curve, then the uninitialized state.
fn classify_bounded<'a>(
    basis: &Handle<dyn GeomCurve>,
) -> (CurveType, GeomGridEvalCurveEvaluator<'a>) {
    use GeomGridEvalCurveEvaluator as E;
    if let Some(bezier) = kind::<GeomBezierCurve>(basis) {
        return (
            CurveType::BezierCurve,
            E::Bezier(GeomGridEvalBezierCurve::new(bezier)),
        );
    }
    if let Some(bspline) = kind::<GeomBSplineCurve>(basis) {
        let evaluator = GeomGridEvalBSplineCurve::new(bspline);
        return (CurveType::BSplineCurve, E::BSpline(evaluator));
    }
    if let Some(offset) = kind::<GeomOffsetCurve>(basis) {
        return (
            CurveType::OffsetCurve,
            E::Offset(GeomGridEvalOffsetCurve::new(offset)),
        );
    }
    (CurveType::OtherCurve, E::Uninitialized)
}

// `occ::down_cast<T>(basis)` when it succeeds.
fn kind<T: RuntimeTyped>(basis: &Handle<dyn GeomCurve>) -> Option<Handle<T>> {
    let typed: Handle<T> = basis.cast::<T>();
    (!typed.is_null()).then_some(typed)
}
