// Rust translation of the curve comparison of BRepAlgo::ConcatenateWireC0
// (ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Whether an edge curve lies on the curve of the previous edge (same line, circle,
//! ellipse, hyperbola or parabola within the linear and angular tolerances), and its
//! parameters on it.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;

/// The tolerances of the comparison (Precision::Confusion and Precision::Angular).
pub(super) struct SameCurveTolerances {
    pub lin_tol: f64,
    pub ang_tol: f64,
}

/// The parameters on the previous curve of the points at `(fpar, lpar)` of the curve
/// when both are the same conic or line of type `a_type`; `None` otherwise.
pub(super) fn same_conic(
    a_type: CurveType,
    a_ga_curve: &GeomAdaptorCurve,
    ga_prev_curve: &GeomAdaptorCurve,
    (fpar, lpar): (f64, f64),
    t: &SameCurveTolerances,
) -> Result<Option<(f64, f64)>, Failure> {
    match a_type {
        CurveType::Line => same_line(a_ga_curve, ga_prev_curve, (fpar, lpar), t),
        CurveType::Circle => same_circle(a_ga_curve, ga_prev_curve, (fpar, lpar), t),
        CurveType::Ellipse => same_ellipse(a_ga_curve, ga_prev_curve, (fpar, lpar), t),
        CurveType::Hyperbola => same_hyperbola(a_ga_curve, ga_prev_curve, (fpar, lpar), t),
        CurveType::Parabola => same_parabola(a_ga_curve, ga_prev_curve, (fpar, lpar), t),
        _ => Ok(None),
    }
}

/// The same line: the previous origin on the line and parallel directions.
fn same_line(
    c: &GeomAdaptorCurve,
    prev: &GeomAdaptorCurve,
    (fpar, lpar): (f64, f64),
    t: &SameCurveTolerances,
) -> Result<Option<(f64, f64)>, Failure> {
    let (a_line, prev_line) = (c.line()?, prev.line()?);
    if !(a_line.contains(prev_line.location(), t.lin_tol)
        && a_line
            .direction()
            .is_parallel(prev_line.direction(), t.ang_tol))
    {
        return Ok(None);
    }
    let p1: Point3 = elclib::line_value(fpar, a_line.position());
    let p2: Point3 = elclib::line_value(lpar, a_line.position());
    Ok(Some((
        elclib::line_parameter(prev_line.position(), p1),
        elclib::line_parameter(prev_line.position(), p2),
    )))
}

/// The same circle: the centres and radii within the tolerance and parallel axes.
fn same_circle(
    c: &GeomAdaptorCurve,
    prev: &GeomAdaptorCurve,
    (fpar, lpar): (f64, f64),
    t: &SameCurveTolerances,
) -> Result<Option<(f64, f64)>, Failure> {
    let (a_circle, prev_circle) = (c.circle()?, prev.circle()?);
    let same = a_circle.location().distance(&prev_circle.location()) <= t.lin_tol
        && (a_circle.radius() - prev_circle.radius()).abs() <= t.lin_tol
        && a_circle.axis().is_parallel(prev_circle.axis(), t.ang_tol);
    if !same {
        return Ok(None);
    }
    let p1 = elclib::circle_value(fpar, a_circle.position(), a_circle.radius());
    let p2 = elclib::circle_value(lpar, a_circle.position(), a_circle.radius());
    Ok(Some((
        elclib::circle_parameter(prev_circle.position(), p1),
        elclib::circle_parameter(prev_circle.position(), p2),
    )))
}

/// The same ellipse: the foci and radii within the tolerance and parallel axes.
fn same_ellipse(
    c: &GeomAdaptorCurve,
    prev: &GeomAdaptorCurve,
    (fpar, lpar): (f64, f64),
    t: &SameCurveTolerances,
) -> Result<Option<(f64, f64)>, Failure> {
    let (e, p) = (c.ellipse()?, prev.ellipse()?);
    let same = e.focus1().distance(&p.focus1()) <= t.lin_tol
        && e.focus2().distance(&p.focus2()) <= t.lin_tol
        && (e.major_radius() - p.major_radius()).abs() <= t.lin_tol
        && (e.minor_radius() - p.minor_radius()).abs() <= t.lin_tol
        && e.axis().is_parallel(p.axis(), t.ang_tol);
    if !same {
        return Ok(None);
    }
    let p1 = elclib::ellipse_value(fpar, e.position(), e.major_radius(), e.minor_radius());
    let p2 = elclib::ellipse_value(lpar, e.position(), e.major_radius(), e.minor_radius());
    let u1 = elclib::ellipse_parameter(p.position(), p.major_radius(), p.minor_radius(), p1)?;
    let u2 = elclib::ellipse_parameter(p.position(), p.major_radius(), p.minor_radius(), p2)?;
    Ok(Some((u1, u2)))
}

/// The same hyperbola: the foci and radii within the tolerance and parallel axes.
fn same_hyperbola(
    c: &GeomAdaptorCurve,
    prev: &GeomAdaptorCurve,
    (fpar, lpar): (f64, f64),
    t: &SameCurveTolerances,
) -> Result<Option<(f64, f64)>, Failure> {
    let (h, p) = (c.hyperbola()?, prev.hyperbola()?);
    let same = h.focus1().distance(&p.focus1()) <= t.lin_tol
        && h.focus2().distance(&p.focus2()) <= t.lin_tol
        && (h.major_radius() - p.major_radius()).abs() <= t.lin_tol
        && (h.minor_radius() - p.minor_radius()).abs() <= t.lin_tol
        && h.axis().is_parallel(p.axis(), t.ang_tol);
    if !same {
        return Ok(None);
    }
    let p1 = elclib::hyperbola_value(fpar, h.position(), h.major_radius(), h.minor_radius());
    let p2 = elclib::hyperbola_value(lpar, h.position(), h.major_radius(), h.minor_radius());
    let u1 = elclib::hyperbola_parameter(p.position(), p.major_radius(), p.minor_radius(), p1);
    let u2 = elclib::hyperbola_parameter(p.position(), p.major_radius(), p.minor_radius(), p2);
    Ok(Some((u1, u2)))
}

/// The same parabola: the apices, foci and focal lengths within the tolerance and
/// parallel axes.
fn same_parabola(
    c: &GeomAdaptorCurve,
    prev: &GeomAdaptorCurve,
    (fpar, lpar): (f64, f64),
    t: &SameCurveTolerances,
) -> Result<Option<(f64, f64)>, Failure> {
    let (a, p) = (c.parabola()?, prev.parabola()?);
    let same = a.location().distance(&p.location()) <= t.lin_tol
        && a.focus().distance(&p.focus()) <= t.lin_tol
        && (a.focal() - p.focal()).abs() <= t.lin_tol
        && a.axis().is_parallel(p.axis(), t.ang_tol);
    if !same {
        return Ok(None);
    }
    let p1 = elclib::parabola_value(fpar, a.position(), a.focal());
    let p2 = elclib::parabola_value(lpar, a.position(), a.focal());
    Ok(Some((
        elclib::parabola_parameter(p.position(), p1),
        elclib::parabola_parameter(p.position(), p2),
    )))
}
