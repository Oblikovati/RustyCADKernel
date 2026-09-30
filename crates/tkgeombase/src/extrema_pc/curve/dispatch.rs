// Rust translation of initFromAdaptor, initFromGeomCurve and the std::visit of Perform in
// ModelingData/TKGeomBase/ExtremaPC/ExtremaPC_Curve.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluator selection of ExtremaPC_Curve and the call of the selected evaluator.
use super::super::adaptor_curve::CurveRef3d;
use super::super::{
    BSplineCurve, BezierCurve, Circle, Domain1D, Ellipse, ExtremaResult, Hyperbola, Line,
    OffsetCurve, OtherCurve, Parabola, SearchMode,
};
use super::{EvaluatorVariant, kind, shared};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomParabola,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;

/// `initFromAdaptor`: the switch on the adaptor's GetType(); the offset and other kinds
/// keep the adaptor reference.
pub(super) fn from_adaptor(
    curve: CurveRef3d<'_>,
    domain: Domain1D,
) -> Result<EvaluatorVariant<'_>, Failure> {
    use EvaluatorVariant as E;
    let curve_kind: CurveType = kind(&curve)?;
    let Some(adaptor) = curve.get() else {
        return Ok(E::Uninitialized);
    };
    Ok(match curve_kind {
        CurveType::Line => E::Line(Line::with_domain(adaptor.line()?, domain)),
        CurveType::Circle => E::Circle(Circle::with_domain(adaptor.circle()?, domain)),
        CurveType::Ellipse => E::Ellipse(Ellipse::with_domain(adaptor.ellipse()?, domain)),
        CurveType::Hyperbola => E::Hyperbola(Hyperbola::with_domain(adaptor.hyperbola()?, domain)),
        CurveType::Parabola => E::Parabola(Parabola::with_domain(adaptor.parabola()?, domain)),
        CurveType::BezierCurve => E::Bezier(BezierCurve::with_domain(adaptor.bezier()?, domain)?),
        CurveType::BSplineCurve => {
            E::BSpline(BSplineCurve::with_domain(adaptor.bspline()?, domain)?)
        }
        CurveType::OffsetCurve => E::Offset(OffsetCurve::from_curve_ref(curve, Some(domain))?),
        _ => E::Other(OtherCurve::from_curve_ref(curve, Some(domain))?),
    })
}

/// `initFromGeomCurve`: the analytical kinds and the splines by class, the rest through
/// an owned adaptor.
pub(super) fn from_geom<'a>(
    curve: &Handle<dyn GeomCurve>,
    domain: Option<Domain1D>,
) -> Result<EvaluatorVariant<'a>, Failure> {
    use EvaluatorVariant as E;
    if let Some(line) = curve.cast::<GeomLine>().get() {
        return Ok(E::Line(with_optional(
            line.lin(),
            domain,
            Line::new,
            Line::with_domain,
        )));
    }
    if let Some(circle) = curve.cast::<GeomCircle>().get() {
        return Ok(E::Circle(with_optional(
            circle.circ()?,
            domain,
            Circle::new,
            Circle::with_domain,
        )));
    }
    if let Some(ellipse) = curve.cast::<GeomEllipse>().get() {
        let evaluator = with_optional(ellipse.elips()?, domain, Ellipse::new, Ellipse::with_domain);
        return Ok(E::Ellipse(evaluator));
    }
    if let Some(hyperbola) = curve.cast::<GeomHyperbola>().get() {
        let evaluator = with_optional(
            hyperbola.hypr()?,
            domain,
            Hyperbola::new,
            Hyperbola::with_domain,
        );
        return Ok(E::Hyperbola(evaluator));
    }
    if let Some(parabola) = curve.cast::<GeomParabola>().get() {
        let evaluator = with_optional(
            parabola.parab()?,
            domain,
            Parabola::new,
            Parabola::with_domain,
        );
        return Ok(E::Parabola(evaluator));
    }
    from_geom_spline(curve, domain)
}

// The Bezier and BSpline classes, then the owned adaptor of the offset and other curves.
fn from_geom_spline<'a>(
    curve: &Handle<dyn GeomCurve>,
    domain: Option<Domain1D>,
) -> Result<EvaluatorVariant<'a>, Failure> {
    use EvaluatorVariant as E;
    let bezier: Handle<GeomBezierCurve> = curve.cast();
    if !bezier.is_null() {
        let evaluator = match domain {
            Some(domain) => BezierCurve::with_domain(bezier, domain)?,
            None => BezierCurve::new(bezier)?,
        };
        return Ok(E::Bezier(evaluator));
    }
    let bspline: Handle<GeomBSplineCurve> = curve.cast();
    if !bspline.is_null() {
        let evaluator = match domain {
            Some(domain) => BSplineCurve::with_domain(bspline, domain)?,
            None => BSplineCurve::new(bspline)?,
        };
        return Ok(E::BSpline(evaluator));
    }
    // For offset and other curves, store adaptor and use it
    let adaptor: GeomAdaptorCurve = match domain {
        Some(domain) => GeomAdaptorCurve::with_range(curve, domain.min, domain.max)?,
        None => GeomAdaptorCurve::with_curve(curve)?,
    };
    let owned = shared(Handle::new(adaptor));
    if kind(&owned)? == CurveType::OffsetCurve {
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

/// The std::visit of `Perform` / `PerformWithEndpoints`: `query` is the point, the
/// tolerance, the mode and whether the endpoints are included.
pub(super) fn run<'r>(
    evaluator: &'r mut EvaluatorVariant<'_>,
    not_done: &'r ExtremaResult,
    (point, tol, mode, ends): (Point3, f64, SearchMode, bool),
) -> Result<&'r ExtremaResult, Failure> {
    use EvaluatorVariant as E;
    macro_rules! call {
        ($e:expr) => {
            if ends {
                $e.perform_with_endpoints(point, tol, mode)
            } else {
                $e.perform(point, tol, mode)
            }
        };
    }
    match evaluator {
        E::Uninitialized => Ok(not_done),
        E::Line(e) => call!(e),
        E::Circle(e) => call!(e),
        E::Ellipse(e) => call!(e),
        E::Hyperbola(e) => call!(e),
        E::Parabola(e) => call!(e),
        E::Bezier(e) => call!(e),
        E::BSpline(e) => call!(e),
        E::Offset(e) => call!(e),
        E::Other(e) => call!(e),
    }
}
