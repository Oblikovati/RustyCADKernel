// Rust translation of ModelingData/TKG2d/Geom2dGridEval/Geom2dGridEval_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGridEval_Curve`: the grid evaluator that picks a specialized one per curve kind.
use super::grid::empty;
use super::{
    Geom2dGridEvalBSplineCurve, Geom2dGridEvalBezierCurve, Geom2dGridEvalCircle,
    Geom2dGridEvalCurveD1, Geom2dGridEvalCurveD2, Geom2dGridEvalCurveD3, Geom2dGridEvalEllipse,
    Geom2dGridEvalHyperbola, Geom2dGridEvalLine, Geom2dGridEvalOffsetCurve,
    Geom2dGridEvalOtherCurve, Geom2dGridEvalParabola,
};
use crate::adaptor2d::Adaptor2dCurve2d;
use crate::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use crate::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point2, Vector2};

/// The specialized evaluator held by `Geom2dGridEval_Curve` (`EvaluatorVariant`);
/// `Uninitialized` is the native `std::monostate`, whose grids are empty.
pub enum Geom2dGridEvalEvaluator<'a> {
    /// No evaluator: a null curve or a curve class without a specialized evaluator.
    Uninitialized,
    /// `Geom2dGridEval_Line`.
    Line(Geom2dGridEvalLine),
    /// `Geom2dGridEval_Circle`.
    Circle(Geom2dGridEvalCircle),
    /// `Geom2dGridEval_Ellipse`.
    Ellipse(Geom2dGridEvalEllipse),
    /// `Geom2dGridEval_Hyperbola`.
    Hyperbola(Geom2dGridEvalHyperbola),
    /// `Geom2dGridEval_Parabola`.
    Parabola(Geom2dGridEvalParabola),
    /// `Geom2dGridEval_BezierCurve`.
    Bezier(Geom2dGridEvalBezierCurve),
    /// `Geom2dGridEval_BSplineCurve`.
    BSpline(Geom2dGridEvalBSplineCurve),
    /// `Geom2dGridEval_OffsetCurve`.
    Offset(Geom2dGridEvalOffsetCurve),
    /// `Geom2dGridEval_OtherCurve`, the adaptor fallback.
    Other(Geom2dGridEvalOtherCurve<'a>),
}

// Calls the same member on whichever evaluator is held; the Line evaluator cannot fail.
macro_rules! dispatch {
    ($self:ident, $eval:ident => $call:expr) => {
        match &$self.evaluator {
            Geom2dGridEvalEvaluator::Uninitialized => Ok(empty()),
            Geom2dGridEvalEvaluator::Line($eval) => Ok($call),
            Geom2dGridEvalEvaluator::Circle($eval) => $call,
            Geom2dGridEvalEvaluator::Ellipse($eval) => $call,
            Geom2dGridEvalEvaluator::Hyperbola($eval) => $call,
            Geom2dGridEvalEvaluator::Parabola($eval) => $call,
            Geom2dGridEvalEvaluator::Bezier($eval) => $call,
            Geom2dGridEvalEvaluator::BSpline($eval) => $call,
            Geom2dGridEvalEvaluator::Offset($eval) => $call,
            Geom2dGridEvalEvaluator::Other($eval) => $call,
        }
    };
}

/// Grid evaluator of any 2D curve (`Geom2dGridEval_Curve`).
///
/// From a curve handle, trimmed wrappers are stripped and the basis class selects the
/// evaluator in the order line, circle, ellipse, hyperbola, parabola, Bezier,
/// B-spline, offset; any other class (the Geom2dEval curves) and a null handle leave
/// it uninitialized, so every grid is empty and the type is `OtherCurve`. Trimming
/// bounds are not applied. From an adaptor, a `Geom2dAdaptor_Curve` goes through its
/// curve (ignoring the adaptor range), and any other adaptor through the
/// `Geom2dGridEval_OtherCurve` fallback with the adaptor's `GetType`;
/// `Geom2dGridEvalCurve::from_curve(&curve).evaluate_grid(&params)?`.
pub struct Geom2dGridEvalCurve<'a> {
    evaluator: Geom2dGridEvalEvaluator<'a>,
    curve_type: CurveType,
}

impl<'a> Geom2dGridEvalCurve<'a> {
    /// Evaluator of an adaptor (`Geom2dGridEval_Curve(const Adaptor2d_Curve2d&)`); the
    /// fallback borrows `curve`. Fails when the adaptor's `GetType` raises;
    /// `Geom2dGridEvalCurve::from_adaptor(&adaptor)?`.
    pub fn from_adaptor(curve: &'a dyn Adaptor2dCurve2d) -> Result<Self, Failure> {
        let mut result = Self::uninitialized();
        result.initialization_adaptor(curve)?;
        Ok(result)
    }
    /// Evaluator of a curve handle (`Geom2dGridEval_Curve(const occ::handle<Geom2d_Curve>&)`);
    /// `Geom2dGridEvalCurve::from_curve(&curve)`.
    pub fn from_curve(curve: &Handle<dyn Geom2dCurve>) -> Self {
        let mut result = Self::uninitialized();
        result.initialization_curve(curve);
        result
    }
    fn uninitialized() -> Self {
        Self {
            evaluator: Geom2dGridEvalEvaluator::Uninitialized,
            curve_type: CurveType::OtherCurve,
        }
    }
    // `initialization(const Adaptor2d_Curve2d&)`.
    fn initialization_adaptor(&mut self, curve: &'a dyn Adaptor2dCurve2d) -> Result<(), Failure> {
        if let Some(geom_adaptor) = Geom2dAdaptorCurve::project_from(curve) {
            self.initialization_curve(&geom_adaptor.curve());
            return Ok(());
        }
        self.curve_type = curve.get_type()?;
        self.evaluator = Geom2dGridEvalEvaluator::Other(Geom2dGridEvalOtherCurve::new(curve));
        Ok(())
    }
    // `initialization(const occ::handle<Geom2d_Curve>&)`.
    fn initialization_curve(&mut self, curve: &Handle<dyn Geom2dCurve>) {
        let (curve_type, evaluator) = classify(&extract_basis_curve(curve));
        self.curve_type = curve_type;
        self.evaluator = evaluator;
    }
    /// The held evaluator (the private `myEvaluator`, read-only); `evaluator.evaluator()`.
    pub fn evaluator(&self) -> &Geom2dGridEvalEvaluator<'a> {
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
    ) -> Result<Array1<'static, Point2>, Failure> {
        dispatch!(self, eval => eval.evaluate_grid(params))
    }
    /// Points and D1 (`EvaluateGridD1`); `evaluator.evaluate_grid_d1(&params)?`.
    pub fn evaluate_grid_d1(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD1>, Failure> {
        dispatch!(self, eval => eval.evaluate_grid_d1(params))
    }
    /// Points, D1 and D2 (`EvaluateGridD2`); `evaluator.evaluate_grid_d2(&params)?`.
    pub fn evaluate_grid_d2(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD2>, Failure> {
        dispatch!(self, eval => eval.evaluate_grid_d2(params))
    }
    /// Points, D1, D2 and D3 (`EvaluateGridD3`); `evaluator.evaluate_grid_d3(&params)?`.
    pub fn evaluate_grid_d3(
        &self,
        params: &Array1<'_, f64>,
    ) -> Result<Array1<'static, Geom2dGridEvalCurveD3>, Failure> {
        dispatch!(self, eval => eval.evaluate_grid_d3(params))
    }
    /// Derivative of order `n` at every parameter (`EvaluateGridDN`);
    /// `evaluator.evaluate_grid_dn(&params, 2)?`.
    pub fn evaluate_grid_dn(
        &self,
        params: &Array1<'_, f64>,
        n: i32,
    ) -> Result<Array1<'static, Vector2>, Failure> {
        dispatch!(self, eval => eval.evaluate_grid_dn(params, n))
    }
}

/// `ExtractBasisCurve`: the basis under any number of trimmed-curve wrappers.
fn extract_basis_curve(curve: &Handle<dyn Geom2dCurve>) -> Handle<dyn Geom2dCurve> {
    let mut result: Handle<dyn Geom2dCurve> = curve.clone();
    loop {
        let trimmed: Handle<Geom2dTrimmedCurve> = result.cast::<Geom2dTrimmedCurve>();
        let Some(wrapper) = trimmed.get() else {
            return result;
        };
        result = wrapper.basis_curve();
    }
}

// The `occ::down_cast` chain of `initialization`: the first class that matches wins.
fn classify<'a>(basis: &Handle<dyn Geom2dCurve>) -> (CurveType, Geom2dGridEvalEvaluator<'a>) {
    use Geom2dGridEvalEvaluator as E;
    if let Some(line) = kind::<Geom2dLine>(basis) {
        return (CurveType::Line, E::Line(Geom2dGridEvalLine::new(line)));
    }
    if let Some(circle) = kind::<Geom2dCircle>(basis) {
        return (
            CurveType::Circle,
            E::Circle(Geom2dGridEvalCircle::new(circle)),
        );
    }
    if let Some(ellipse) = kind::<Geom2dEllipse>(basis) {
        return (
            CurveType::Ellipse,
            E::Ellipse(Geom2dGridEvalEllipse::new(ellipse)),
        );
    }
    if let Some(hyperbola) = kind::<Geom2dHyperbola>(basis) {
        let evaluator = Geom2dGridEvalHyperbola::new(hyperbola);
        return (CurveType::Hyperbola, E::Hyperbola(evaluator));
    }
    if let Some(parabola) = kind::<Geom2dParabola>(basis) {
        return (
            CurveType::Parabola,
            E::Parabola(Geom2dGridEvalParabola::new(parabola)),
        );
    }
    classify_bounded(basis)
}

// The B-spline family and the offset curve, then the uninitialized state.
fn classify_bounded<'a>(
    basis: &Handle<dyn Geom2dCurve>,
) -> (CurveType, Geom2dGridEvalEvaluator<'a>) {
    use Geom2dGridEvalEvaluator as E;
    if let Some(bezier) = kind::<Geom2dBezierCurve>(basis) {
        return (
            CurveType::BezierCurve,
            E::Bezier(Geom2dGridEvalBezierCurve::new(bezier)),
        );
    }
    if let Some(bspline) = kind::<Geom2dBSplineCurve>(basis) {
        let evaluator = Geom2dGridEvalBSplineCurve::new(bspline);
        return (CurveType::BSplineCurve, E::BSpline(evaluator));
    }
    if let Some(offset) = kind::<Geom2dOffsetCurve>(basis) {
        return (
            CurveType::OffsetCurve,
            E::Offset(Geom2dGridEvalOffsetCurve::new(offset)),
        );
    }
    (CurveType::OtherCurve, E::Uninitialized)
}

// `occ::down_cast<T>(basis)` when it succeeds.
fn kind<T: RuntimeTyped>(basis: &Handle<dyn Geom2dCurve>) -> Option<Handle<T>> {
    let typed: Handle<T> = basis.cast::<T>();
    (!typed.is_null()).then_some(typed)
}
