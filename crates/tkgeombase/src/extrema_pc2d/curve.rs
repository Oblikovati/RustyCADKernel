// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_Curve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC2d_Curve`: the evaluator of a curve's kind, chosen once at construction.
use super::adaptor_curve::CurveRef2d;
use super::{
    BSplineCurve, BezierCurve, Circle, Domain1D, Ellipse, ExtremaResult, Hyperbola, Line,
    OffsetCurve, OtherCurve, Parabola, SearchMode,
};
use crate::extrema_pc_core::PcCurveRef;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point2;

/// The specialized evaluator of [`Curve`] (`EvaluatorVariant`); `Uninitialized` is the
/// native `std::monostate` of a null curve.
pub enum EvaluatorVariant<'a> {
    /// No evaluator (`std::monostate`).
    Uninitialized,
    /// `ExtremaPC2d_Line`.
    Line(Line),
    /// `ExtremaPC2d_Circle`.
    Circle(Circle),
    /// `ExtremaPC2d_Ellipse`.
    Ellipse(Ellipse),
    /// `ExtremaPC2d_Hyperbola`.
    Hyperbola(Hyperbola),
    /// `ExtremaPC2d_Parabola`.
    Parabola(Parabola),
    /// `ExtremaPC2d_BezierCurve`.
    Bezier(BezierCurve),
    /// `ExtremaPC2d_BSplineCurve`.
    BSpline(BSplineCurve),
    /// `ExtremaPC2d_OffsetCurve`.
    Offset(OffsetCurve<'a>),
    /// `ExtremaPC2d_OtherCurve`.
    Other(OtherCurve<'a>),
}

/// Point-curve extrema on any 2D curve (`ExtremaPC2d_Curve`).
///
/// The domain is fixed at construction. An adaptor given as such must outlive the
/// evaluator when it is dispatched as an offset or general curve; a Geom2d curve gets
/// its own adaptor for those kinds. An uninitialized evaluator answers NotDone;
/// `Curve::from_geom(&curve)?.perform(point, 1e-9, SearchMode::MinMax)?`.
pub struct Curve<'a> {
    evaluator: EvaluatorVariant<'a>,
    not_done: ExtremaResult,
}

impl<'a> Curve<'a> {
    fn with_evaluator(evaluator: EvaluatorVariant<'a>) -> Self {
        Self {
            evaluator,
            not_done: ExtremaResult::not_done(),
        }
    }
    /// Evaluator over the adaptor bounds (`ExtremaPC2d_Curve(const Adaptor2d_Curve2d&)`);
    /// `Curve::from_adaptor(&adaptor)?`.
    pub fn from_adaptor(curve: &'a dyn Adaptor2dCurve2d) -> Result<Self, Failure> {
        let domain = Domain1D::new(curve.first_parameter()?, curve.last_parameter()?);
        Ok(Self::with_evaluator(from_adaptor(curve, domain)?))
    }
    /// Evaluator over `[u_min, u_max]` (`ExtremaPC2d_Curve(theCurve, theUMin, theUMax)`);
    /// `Curve::from_adaptor_range(&adaptor, 0.0, 1.0)?`.
    pub fn from_adaptor_range(
        curve: &'a dyn Adaptor2dCurve2d,
        u_min: f64,
        u_max: f64,
    ) -> Result<Self, Failure> {
        Ok(Self::with_evaluator(from_adaptor(
            curve,
            Domain1D::new(u_min, u_max),
        )?))
    }
    /// Evaluator of a Geom2d curve (`ExtremaPC2d_Curve(const occ::handle<Geom2d_Curve>&)`):
    /// the natural domain, the trimmed bounds of a trimmed curve; a null curve stays
    /// uninitialized; `Curve::from_geom(&curve)?`.
    pub fn from_geom(curve: &Handle<dyn Geom2dCurve>) -> Result<Self, Failure> {
        if curve.is_null() {
            return Ok(Self::with_evaluator(EvaluatorVariant::Uninitialized));
        }
        let trimmed: Handle<Geom2dTrimmedCurve> = curve.cast();
        if let Some(trimmed) = trimmed.get() {
            let domain = Domain1D::new(trimmed.first_parameter(), trimmed.last_parameter());
            return Ok(Self::with_evaluator(from_geom(
                &trimmed.basis_curve(),
                Some(domain),
            )?));
        }
        Ok(Self::with_evaluator(from_geom(curve, None)?))
    }
    /// Evaluator over `[u_min, u_max]`, intersected with the bounds of a trimmed curve
    /// (`ExtremaPC2d_Curve(theCurve, theUMin, theUMax)`);
    /// `Curve::from_geom_range(&curve, 0.0, 1.0)?`.
    pub fn from_geom_range(
        curve: &Handle<dyn Geom2dCurve>,
        u_min: f64,
        u_max: f64,
    ) -> Result<Self, Failure> {
        if curve.is_null() {
            return Ok(Self::with_evaluator(EvaluatorVariant::Uninitialized));
        }
        let (mut base, mut first, mut last) = (curve.clone(), u_min, u_max);
        let trimmed: Handle<Geom2dTrimmedCurve> = curve.cast();
        if let Some(trimmed) = trimmed.get() {
            base = trimmed.basis_curve();
            // Intersect bounds: take max of mins, min of maxs
            first = rustycad_tkg2d::geom2d::native::std_max(u_min, trimmed.first_parameter());
            last = rustycad_tkg2d::geom2d::native::std_min(u_max, trimmed.last_parameter());
        }
        Ok(Self::with_evaluator(from_geom(
            &base,
            Some(Domain1D::new(first, last)),
        )?))
    }
    /// True unless the evaluator is uninitialized (`IsInitialized`); `curve.is_initialized()`.
    pub fn is_initialized(&self) -> bool {
        !matches!(self.evaluator, EvaluatorVariant::Uninitialized)
    }
    /// The specialized evaluator; `curve.evaluator()`.
    pub fn evaluator(&self) -> &EvaluatorVariant<'a> {
        &self.evaluator
    }
    /// The interior extrema from the specialized evaluator (`Perform`);
    /// `curve.perform(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        use EvaluatorVariant as E;
        Ok(match &mut self.evaluator {
            E::Uninitialized => &self.not_done,
            E::Line(e) => e.perform(point, tol, mode),
            E::Circle(e) => e.perform(point, tol, mode),
            E::Ellipse(e) => e.perform(point, tol, mode),
            E::Hyperbola(e) => e.perform(point, tol, mode),
            E::Parabola(e) => e.perform(point, tol, mode),
            E::Bezier(e) => e.perform(point, tol, mode)?,
            E::BSpline(e) => e.perform(point, tol, mode)?,
            E::Offset(e) => e.perform(point, tol, mode)?,
            E::Other(e) => e.perform(point, tol, mode)?,
        })
    }
    /// The interior extrema and the extremal ends (`PerformWithEndpoints`);
    /// `curve.perform_with_endpoints(point, 1e-9, SearchMode::MinMax)?`.
    pub fn perform_with_endpoints(
        &mut self,
        point: Point2,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        use EvaluatorVariant as E;
        Ok(match &mut self.evaluator {
            E::Uninitialized => &self.not_done,
            E::Line(e) => e.perform_with_endpoints(point, tol, mode),
            E::Circle(e) => e.perform_with_endpoints(point, tol, mode),
            E::Ellipse(e) => e.perform_with_endpoints(point, tol, mode),
            E::Hyperbola(e) => e.perform_with_endpoints(point, tol, mode),
            E::Parabola(e) => e.perform_with_endpoints(point, tol, mode),
            E::Bezier(e) => e.perform_with_endpoints(point, tol, mode)?,
            E::BSpline(e) => e.perform_with_endpoints(point, tol, mode)?,
            E::Offset(e) => e.perform_with_endpoints(point, tol, mode)?,
            E::Other(e) => e.perform_with_endpoints(point, tol, mode)?,
        })
    }
}

// `initFromAdaptor`: the switch on the adaptor's GetType().
fn from_adaptor<'a>(
    curve: &'a dyn Adaptor2dCurve2d,
    domain: Domain1D,
) -> Result<EvaluatorVariant<'a>, Failure> {
    use EvaluatorVariant as E;
    Ok(match curve.get_type()? {
        CurveType::Line => E::Line(Line::with_domain(curve.line()?, domain)),
        CurveType::Circle => E::Circle(Circle::with_domain(curve.circle()?, domain)),
        CurveType::Ellipse => E::Ellipse(Ellipse::with_domain(curve.ellipse()?, domain)),
        CurveType::Hyperbola => E::Hyperbola(Hyperbola::with_domain(curve.hyperbola()?, domain)),
        CurveType::Parabola => E::Parabola(Parabola::with_domain(curve.parabola()?, domain)),
        CurveType::BezierCurve => E::Bezier(BezierCurve::with_domain(curve.bezier()?, domain)),
        CurveType::BSplineCurve => E::BSpline(BSplineCurve::with_domain(curve.bspline()?, domain)),
        CurveType::OffsetCurve => E::Offset(OffsetCurve::with_domain(curve, domain)),
        _ => E::Other(OtherCurve::with_domain(curve, domain)),
    })
}

// `occ::down_cast<T>(theCurve)` when it succeeds.
fn kind<T: RuntimeTyped>(curve: &Handle<dyn Geom2dCurve>) -> Option<Handle<T>> {
    let typed: Handle<T> = curve.cast::<T>();
    (!typed.is_null()).then_some(typed)
}
// The analytical classes are tested with `curve.cast::<T>().get()`.

// `initFromGeomCurve`: the analytical kinds and the splines by class, the rest through
// an owned adaptor.
fn from_geom<'a>(
    curve: &Handle<dyn Geom2dCurve>,
    domain: Option<Domain1D>,
) -> Result<EvaluatorVariant<'a>, Failure> {
    use EvaluatorVariant as E;
    if let Some(line) = curve.cast::<Geom2dLine>().get() {
        let line = line.lin2d();
        return Ok(E::Line(with_optional(
            line,
            domain,
            Line::new,
            Line::with_domain,
        )));
    }
    if let Some(circle) = curve.cast::<Geom2dCircle>().get() {
        let circle = circle.circ2d()?;
        return Ok(E::Circle(with_optional(
            circle,
            domain,
            Circle::new,
            Circle::with_domain,
        )));
    }
    if let Some(ellipse) = curve.cast::<Geom2dEllipse>().get() {
        let ellipse = ellipse.elips2d()?;
        return Ok(E::Ellipse(with_optional(
            ellipse,
            domain,
            Ellipse::new,
            Ellipse::with_domain,
        )));
    }
    if let Some(hyperbola) = curve.cast::<Geom2dHyperbola>().get() {
        let hyperbola = hyperbola.hypr2d()?;
        let evaluator = with_optional(hyperbola, domain, Hyperbola::new, Hyperbola::with_domain);
        return Ok(E::Hyperbola(evaluator));
    }
    if let Some(parabola) = curve.cast::<Geom2dParabola>().get() {
        let parabola = parabola.parab2d()?;
        let evaluator = with_optional(parabola, domain, Parabola::new, Parabola::with_domain);
        return Ok(E::Parabola(evaluator));
    }
    from_geom_spline(curve, domain)
}

// The Bezier and BSpline classes, then the owned adaptor of the offset and other curves.
fn from_geom_spline<'a>(
    curve: &Handle<dyn Geom2dCurve>,
    domain: Option<Domain1D>,
) -> Result<EvaluatorVariant<'a>, Failure> {
    use EvaluatorVariant as E;
    if let Some(bezier) = kind::<Geom2dBezierCurve>(curve) {
        let evaluator = with_optional(bezier, domain, BezierCurve::new, BezierCurve::with_domain);
        return Ok(E::Bezier(evaluator));
    }
    if let Some(bspline) = kind::<Geom2dBSplineCurve>(curve) {
        let evaluator = with_optional(
            bspline,
            domain,
            BSplineCurve::new,
            BSplineCurve::with_domain,
        );
        return Ok(E::BSpline(evaluator));
    }
    // For offset and other curves, store adaptor and use it
    let adaptor: Geom2dAdaptorCurve = match domain {
        Some(domain) => Geom2dAdaptorCurve::with_range(curve, domain.min, domain.max)?,
        None => Geom2dAdaptorCurve::with_curve(curve)?,
    };
    let is_offset: bool = adaptor.get_type()? == CurveType::OffsetCurve;
    let owned: CurveRef2d<'a> = PcCurveRef::Shared(Handle::new(adaptor).cast());
    if is_offset {
        return Ok(E::Offset(OffsetCurve::from_curve_ref(owned, domain)?));
    }
    Ok(E::Other(OtherCurve::from_curve_ref(owned, domain)?))
}

// The two constructor overloads: with and without a domain.
fn with_optional<G, E>(
    geometry: G,
    domain: Option<Domain1D>,
    whole: impl FnOnce(G) -> E,
    bounded: impl FnOnce(G, Domain1D) -> E,
) -> E {
    match domain {
        Some(domain) => bounded(geometry, domain),
        None => whole(geometry),
    }
}
