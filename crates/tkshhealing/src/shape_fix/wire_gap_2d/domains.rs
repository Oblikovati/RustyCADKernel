use super::PcurveGap;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dParabola,
};

pub(super) struct SearchDomains {
    pub(super) first: (f64, f64),
    pub(super) second: (f64, f64),
}

pub(super) fn for_pair(
    first: &PcurveGap,
    second: &PcurveGap,
    first_curve: &Handle<dyn Geom2dCurve>,
    second_curve: &Handle<dyn Geom2dCurve>,
) -> SearchDomains {
    SearchDomains {
        first: for_curve(first, first_curve, true),
        second: for_curve(second, second_curve, false),
    }
}

fn for_curve(entry: &PcurveGap, curve: &Handle<dyn Geom2dCurve>, first_edge: bool) -> (f64, f64) {
    if has_full_bspline_domain(curve) {
        return curve_domain(curve);
    }
    if has_full_conic_domain(curve) {
        return (0.0, std::f64::consts::TAU);
    }
    expanded_domain(entry, curve, first_edge)
}

fn has_full_bspline_domain(curve: &Handle<dyn Geom2dCurve>) -> bool {
    !curve.cast::<Geom2dBSplineCurve>().is_null() || !curve.cast::<Geom2dBezierCurve>().is_null()
}

fn has_full_conic_domain(curve: &Handle<dyn Geom2dCurve>) -> bool {
    !curve.cast::<Geom2dCircle>().is_null() || !curve.cast::<Geom2dEllipse>().is_null()
}

fn expanded_domain(
    entry: &PcurveGap,
    curve: &Handle<dyn Geom2dCurve>,
    first_edge: bool,
) -> (f64, f64) {
    let mut bounds = (entry.first, entry.last);
    if !is_extended_type(curve) {
        return bounds;
    }
    let span = bounds.1 - bounds.0;
    if first_edge == entry.reversed {
        bounds.0 -= 10.0 * span;
    } else {
        bounds.1 += 10.0 * span;
    }
    bounds
}

fn is_extended_type(curve: &Handle<dyn Geom2dCurve>) -> bool {
    !curve.cast::<Geom2dLine>().is_null()
        || !curve.cast::<Geom2dParabola>().is_null()
        || !curve.cast::<Geom2dHyperbola>().is_null()
}

fn curve_domain(curve: &Handle<dyn Geom2dCurve>) -> (f64, f64) {
    match curve.get() {
        Some(curve) => (curve.first_parameter(), curve.last_parameter()),
        None => (0.0, 0.0),
    }
}
