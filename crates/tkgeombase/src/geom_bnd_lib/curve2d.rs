// Rust translation of GeomBndLib_Curve2d at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_Curve2d`: detect the kind of a 2D curve (adaptor or Geom handle) and
//! delegate to its evaluator.
use super::circle::Circle2d;
use super::ellipse::Ellipse2d;
use super::hyperbola::Hyperbola2d;
use super::line::Line2d;
use super::offset_curve2d::OffsetCurve2d;
use super::other_curve::OtherCurve2d;
use super::parabola::Parabola2d;
use super::spline_curve2d::{BSplineCurve2d, BezierCurve2d};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, exceptions};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::CurveType;

// The `std::variant` of evaluators; `Other` borrows the dispatcher's adaptor at call time.
enum Evaluator {
    Line(Line2d),
    Circle(Circle2d),
    Ellipse(Ellipse2d),
    Hyperbola(Hyperbola2d),
    Parabola(Parabola2d),
    Bezier(BezierCurve2d),
    BSpline(BSplineCurve2d),
    Offset(OffsetCurve2d),
    Other,
}

/// `GeomBndLib_Curve2d`: the box of any 2D curve through the evaluator of its kind.
/// An adaptor or a trimmed curve supplies the default range of `Box(Tol)`.
pub struct Curve2d<'a> {
    adaptor_ref: Option<&'a dyn Adaptor2dCurve2d>,
    adaptor_owned: Option<Geom2dAdaptorCurve>,
    evaluator: Evaluator,
    curve_type: CurveType,
}

// Run `$body` on the evaluator of `$curve` bound to `$eval` (an OtherCurve2d built on the
// adaptor for the `Other` kind).
macro_rules! dispatch {
    ($curve:expr, $eval:ident => $body:expr) => {
        match &$curve.evaluator {
            Evaluator::Line($eval) => $body,
            Evaluator::Circle($eval) => $body,
            Evaluator::Ellipse($eval) => $body,
            Evaluator::Hyperbola($eval) => $body,
            Evaluator::Parabola($eval) => $body,
            Evaluator::Bezier($eval) => $body,
            Evaluator::BSpline($eval) => $body,
            Evaluator::Offset($eval) => $body,
            Evaluator::Other => {
                let $eval = &OtherCurve2d::new($curve.required_adaptor()?);
                $body
            }
        }
    };
}

// `tryEmplaceEvaluator`: the `occ::down_cast` chain of the handle constructor.
fn detect(curve: &Handle<dyn Geom2dCurve>) -> Option<(CurveType, Evaluator)> {
    fn kind<T: RuntimeTyped>(curve: &Handle<dyn Geom2dCurve>) -> Option<Handle<T>> {
        let typed: Handle<T> = curve.cast::<T>();
        (!typed.is_null()).then_some(typed)
    }
    if let Some(h) = kind::<Geom2dLine>(curve) {
        return Some((CurveType::Line, Evaluator::Line(Line2d::new(h))));
    }
    if let Some(h) = kind::<Geom2dCircle>(curve) {
        return Some((CurveType::Circle, Evaluator::Circle(Circle2d::new(h))));
    }
    if let Some(h) = kind::<Geom2dEllipse>(curve) {
        return Some((CurveType::Ellipse, Evaluator::Ellipse(Ellipse2d::new(h))));
    }
    if let Some(h) = kind::<Geom2dHyperbola>(curve) {
        return Some((
            CurveType::Hyperbola,
            Evaluator::Hyperbola(Hyperbola2d::new(h)),
        ));
    }
    if let Some(h) = kind::<Geom2dParabola>(curve) {
        return Some((CurveType::Parabola, Evaluator::Parabola(Parabola2d::new(h))));
    }
    if let Some(h) = kind::<Geom2dBezierCurve>(curve) {
        return Some((
            CurveType::BezierCurve,
            Evaluator::Bezier(BezierCurve2d::new(h)),
        ));
    }
    if let Some(h) = kind::<Geom2dBSplineCurve>(curve) {
        return Some((
            CurveType::BSplineCurve,
            Evaluator::BSpline(BSplineCurve2d::new(h)),
        ));
    }
    let offset = kind::<Geom2dOffsetCurve>(curve)?;
    Some((
        CurveType::OffsetCurve,
        Evaluator::Offset(OffsetCurve2d::new(offset)),
    ))
}

// The evaluator of an adaptor kind: analytic kinds are rebuilt as Geom objects from the
// adaptor's gp value; an offset needs a Geom2dAdaptor_Curve on a Geom2d_OffsetCurve.
fn from_adaptor_kind(
    curve: &dyn Adaptor2dCurve2d,
    kind: CurveType,
) -> Result<Option<Evaluator>, Failure> {
    Ok(Some(match kind {
        CurveType::Line => Evaluator::Line(Line2d::new(Handle::new(Geom2dLine::from_lin2d(
            curve.line()?,
        )))),
        CurveType::Circle => Evaluator::Circle(Circle2d::new(Handle::new(
            Geom2dCircle::from_circ2d(curve.circle()?),
        ))),
        CurveType::Ellipse => Evaluator::Ellipse(Ellipse2d::new(Handle::new(
            Geom2dEllipse::from_elips2d(curve.ellipse()?),
        ))),
        CurveType::Hyperbola => Evaluator::Hyperbola(Hyperbola2d::new(Handle::new(
            Geom2dHyperbola::from_hypr2d(curve.hyperbola()?),
        ))),
        CurveType::Parabola => Evaluator::Parabola(Parabola2d::new(Handle::new(
            Geom2dParabola::from_parab2d(curve.parabola()?),
        ))),
        CurveType::BezierCurve => Evaluator::Bezier(BezierCurve2d::new(curve.bezier()?)),
        CurveType::BSplineCurve => Evaluator::BSpline(BSplineCurve2d::new(curve.bspline()?)),
        CurveType::OffsetCurve => return Ok(adaptor_offset(curve)),
        _ => return Ok(None),
    }))
}

// `dynamic_cast<const Geom2dAdaptor_Curve*>` then `occ::down_cast<Geom2d_OffsetCurve>`.
fn adaptor_offset(curve: &dyn Adaptor2dCurve2d) -> Option<Evaluator> {
    let geom_adaptor = Geom2dAdaptorCurve::project_from(curve)?;
    let offset: Handle<Geom2dOffsetCurve> = geom_adaptor.curve().cast();
    (!offset.is_null()).then(|| Evaluator::Offset(OffsetCurve2d::new(offset)))
}

impl<'a> Curve2d<'a> {
    /// `GeomBndLib_Curve2d(const Adaptor2d_Curve2d&)`: the adaptor's `GetType` selects the
    /// evaluator; an offset adaptor that is not a Geom2dAdaptor_Curve on a Geom2d_OffsetCurve,
    /// and every other kind, falls back to OtherCurve2d with the type OtherCurve;
    /// `Curve2d::from_adaptor(&adaptor)?`.
    pub fn from_adaptor(curve: &'a dyn Adaptor2dCurve2d) -> Result<Self, Failure> {
        let kind: CurveType = curve.get_type()?;
        let (curve_type, evaluator) = match from_adaptor_kind(curve, kind)? {
            Some(evaluator) => (kind, evaluator),
            None => (CurveType::OtherCurve, Evaluator::Other),
        };
        Ok(Self {
            adaptor_ref: Some(curve),
            adaptor_owned: None,
            evaluator,
            curve_type,
        })
    }
    /// `GetType()`: the detected kind; `curve.get_type()`.
    pub fn get_type(&self) -> CurveType {
        self.curve_type
    }
    fn adaptor(&self) -> Option<&dyn Adaptor2dCurve2d> {
        match (self.adaptor_ref, &self.adaptor_owned) {
            (Some(adaptor), _) => Some(adaptor),
            (None, Some(owned)) => Some(owned),
            (None, None) => None,
        }
    }
    fn required_adaptor(&self) -> Result<&dyn Adaptor2dCurve2d, Failure> {
        self.adaptor().ok_or_else(|| {
            exceptions::null_object("GeomBndLib_Curve2d: the OtherCurve2d evaluator has no adaptor")
        })
    }
    /// `Box(Tol)`: the adaptor's range when there is an adaptor, else the evaluator's
    /// whole curve; `curve.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        if let Some(adaptor) = self.adaptor() {
            let (u1, u2) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
            return self.bounding_box_range(u1, u2, tolerance);
        }
        dispatch!(self, eval => eval.bounding_box(tolerance))
    }
    /// `Box(U1, U2, Tol)`; `curve.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        dispatch!(self, eval => eval.bounding_box_range(u1, u2, tolerance))
    }
    /// `BoxOptimal(Tol)`: the adaptor's range when there is an adaptor, else the
    /// geometry's `[FirstParameter, LastParameter]`; `curve.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox2, Failure> {
        if let Some(adaptor) = self.adaptor() {
            let (u1, u2) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
            return self.bounding_box_optimal_range(u1, u2, tolerance);
        }
        let (u1, u2) = self.geometry_range()?;
        self.bounding_box_optimal_range(u1, u2, tolerance)
    }
    /// `BoxOptimal(U1, U2, Tol)`; `curve.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox2, Failure> {
        dispatch!(self, eval => eval.bounding_box_optimal_range(u1, u2, tolerance))
    }
    /// `Add(Tol, B)`: `B.Add(Box(Tol))`; `curve.add(0.0, &mut bounds)?`.
    pub fn add(&self, tolerance: f64, bounds: &mut BoundingBox2) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box(tolerance)?);
        Ok(())
    }
    /// `Add(U1, U2, Tol, B)`; `curve.add_range(0.0, 1.0, 0.0, &mut bounds)?`.
    pub fn add_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
        bounds: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box_range(u1, u2, tolerance)?);
        Ok(())
    }
    /// `AddOptimal(Tol, B)`; `curve.add_optimal(0.0, &mut bounds)?`.
    pub fn add_optimal(&self, tolerance: f64, bounds: &mut BoundingBox2) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box_optimal(tolerance)?);
        Ok(())
    }
    /// `AddOptimal(U1, U2, Tol, B)`; `curve.add_optimal_range(0.0, 1.0, 0.0, &mut bounds)?`.
    pub fn add_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
        bounds: &mut BoundingBox2,
    ) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box_optimal_range(u1, u2, tolerance)?);
        Ok(())
    }
    // `theEval.Geometry()->FirstParameter()/LastParameter()` of the evaluator.
    fn geometry_range(&self) -> Result<(f64, f64), Failure> {
        fn range<T: Geom2dCurve + RuntimeTyped>(h: &Handle<T>) -> Result<(f64, f64), Failure> {
            let curve = super::live(h, "GeomBndLib_Curve2d")?;
            Ok((curve.first_parameter(), curve.last_parameter()))
        }
        match &self.evaluator {
            Evaluator::Line(e) => range(e.geometry()),
            Evaluator::Circle(e) => range(e.geometry()),
            Evaluator::Ellipse(e) => range(e.geometry()),
            Evaluator::Hyperbola(e) => range(e.geometry()),
            Evaluator::Parabola(e) => range(e.geometry()),
            Evaluator::Bezier(e) => range(e.geometry()),
            Evaluator::BSpline(e) => range(e.geometry()),
            Evaluator::Offset(e) => range(e.geometry()),
            Evaluator::Other => {
                let adaptor = self.required_adaptor()?;
                Ok((adaptor.first_parameter()?, adaptor.last_parameter()?))
            }
        }
    }
}

impl Curve2d<'static> {
    /// `GeomBndLib_Curve2d(const occ::handle<Geom2d_Curve>&)`: the down_cast chain on the
    /// curve, or on the basis of a Geom2d_TrimmedCurve (whose adaptor then gives the range
    /// of `Box(Tol)`); other kinds use OtherCurve2d on a Geom2dAdaptor_Curve;
    /// `Curve2d::from_curve(&handle)?`.
    pub fn from_curve(curve: &Handle<dyn Geom2dCurve>) -> Result<Self, Failure> {
        let mut detection: Handle<dyn Geom2dCurve> = curve.clone();
        let mut adaptor_owned: Option<Geom2dAdaptorCurve> = None;
        let trimmed: Handle<Geom2dTrimmedCurve> = curve.cast();
        if let Some(trimmed) = trimmed.get() {
            adaptor_owned = Some(Geom2dAdaptorCurve::with_curve(curve)?);
            detection = trimmed.basis_curve();
        }
        let (curve_type, evaluator) = match detect(&detection) {
            Some(found) => found,
            None => {
                if adaptor_owned.is_none() {
                    adaptor_owned = Some(Geom2dAdaptorCurve::with_curve(curve)?);
                }
                (CurveType::OtherCurve, Evaluator::Other)
            }
        };
        Ok(Self {
            adaptor_ref: None,
            adaptor_owned,
            evaluator,
            curve_type,
        })
    }
}
