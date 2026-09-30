// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert.cxx
// (CurveToBSplineCurve, BSplineCurveBuilder) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dConvert::CurveToBSplineCurve`: the B-spline of a bounded Geom2d curve.
use super::approx_curve::ApproxCurve;
use super::comp_curve::CompCurveToBSplineCurve;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dBoundedCurve, Geom2dCircle, Geom2dConic,
    Geom2dCurve, Geom2dEllipse, Geom2dGeometry, Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve,
    Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkmath::convert::{ConicToBSplineCurve, ParameterisationType};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{
    self, Axis22d, Circle2, Ellipse2, Hyperbola2, Parabola2, Point2, Transform2,
};

/// `Geom2dConvert::CurveToBSplineCurve(C, Parameterisation = Convert_TgtThetaOver2)`:
/// the B-spline of a circle, an ellipse, a Bezier, B-spline or offset curve, or of a
/// trimmed line, conic, Bezier, B-spline or offset curve.
///
/// `parameterisation` applies to circles and ellipses only; offset curves are
/// approximated (tolerance 1e-4, C2, 16 segments, degree 14) and raise
/// Standard_ConstructionError when the approximation has no result. Any other
/// curve raises Standard_DomainError;
/// `geom2d_convert::curve_to_bspline_curve(&circle, ParameterisationType::TgtThetaOver2)?`.
pub fn curve_to_bspline_curve(
    curve: &Handle<dyn Geom2dCurve>,
    parameterisation: ParameterisationType,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    if curve.is_null() {
        return Err(failure(
            "Standard_NullObject",
            "Geom2dConvert::CurveToBSplineCurve: null curve; expected a Geom2d_Curve",
        ));
    }
    let trimmed: Handle<Geom2dTrimmedCurve> = curve.cast();
    match trimmed.get() {
        Some(ctrim) => trimmed_to_bspline(curve, ctrim, parameterisation),
        None => whole_to_bspline(curve, parameterisation),
    }
}

// The trimmed-curve branch: the bounds are clamped to a non-periodic basis, so that
// the basis Segment cannot raise.
fn trimmed_to_bspline(
    curve: &Handle<dyn Geom2dCurve>,
    ctrim: &Geom2dTrimmedCurve,
    parameterisation: ParameterisationType,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let basis: Handle<dyn Geom2dCurve> = ctrim.basis_curve();
    let (mut u1, mut u2) = (ctrim.first_parameter(), ctrim.last_parameter());
    if !basis.is_periodic() {
        if u1 < basis.first_parameter() {
            u1 = basis.first_parameter();
        }
        if u2 > basis.last_parameter() {
            u2 = basis.last_parameter();
        }
    }
    if basis.cast::<Geom2dLine>().is_some() {
        return trimmed_line(ctrim);
    }
    if let Some(circle) = basis.cast::<Geom2dCircle>().get() {
        let c2d = Circle2::from_x_axis(gp::ox2d(), circle.radius(), true)?;
        let convert =
            |a: f64, b: f64| ConicToBSplineCurve::from_circle_arc(&c2d, a, b, parameterisation);
        return elliptic_arc(circle, [u1, u2], parameterisation, &convert);
    }
    if let Some(ellipse) = basis.cast::<Geom2dEllipse>().get() {
        let e2d = Ellipse2::from_x_axis(
            gp::ox2d(),
            ellipse.major_radius(),
            ellipse.minor_radius(),
            true,
        )?;
        let convert =
            |a: f64, b: f64| ConicToBSplineCurve::from_ellipse_arc(&e2d, a, b, parameterisation);
        return elliptic_arc(ellipse, [u1, u2], parameterisation, &convert);
    }
    trimmed_other(curve, &basis, [u1, u2])
}

// The hyperbola, parabola, Bezier, B-spline and offset bases of a trimmed curve.
fn trimmed_other(
    curve: &Handle<dyn Geom2dCurve>,
    basis: &Handle<dyn Geom2dCurve>,
    [u1, u2]: [f64; 2],
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    if let Some(hyperbola) = basis.cast::<Geom2dHyperbola>().get() {
        let h2d = Hyperbola2::from_x_axis(
            gp::ox2d(),
            hyperbola.major_radius(),
            hyperbola.minor_radius(),
            true,
        )?;
        return bspline_curve_builder(
            hyperbola,
            &ConicToBSplineCurve::from_hyperbola(&h2d, u1, u2)?,
        );
    }
    if let Some(parabola) = basis.cast::<Geom2dParabola>().get() {
        let p2d = Parabola2::from_mirror_axis(gp::ox2d(), parabola.focal(), true)?;
        return bspline_curve_builder(parabola, &ConicToBSplineCurve::from_parabola(&p2d, u1, u2)?);
    }
    if basis.cast::<Geom2dBezierCurve>().is_some() {
        let bezier: Handle<Geom2dBezierCurve> = basis.copy()?.cast();
        bezier.segment(u1, u2)?;
        return bezier_to_bspline(&bezier);
    }
    if basis.cast::<Geom2dBSplineCurve>().is_some() {
        let bspline: Handle<Geom2dBSplineCurve> = basis.copy()?.cast();
        bspline.segment(u1, u2, precision::p_confusion())?;
        return Ok(bspline);
    }
    if basis.cast::<Geom2dOffsetCurve>().is_some() {
        // The native code approximates the trimmed curve itself, not its basis.
        return approximate_offset(curve);
    }
    Err(failure("Standard_DomainError", "No such curve"))
}

// A whole (untrimmed) ellipse, circle, Bezier, B-spline or offset curve.
fn whole_to_bspline(
    curve: &Handle<dyn Geom2dCurve>,
    parameterisation: ParameterisationType,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    if let Some(ellipse) = curve.cast::<Geom2dEllipse>().get() {
        let e2d = Ellipse2::from_x_axis(
            gp::ox2d(),
            ellipse.major_radius(),
            ellipse.minor_radius(),
            true,
        )?;
        let bspline = bspline_curve_builder(
            ellipse,
            &ConicToBSplineCurve::from_ellipse(&e2d, parameterisation)?,
        )?;
        bspline.set_periodic()?;
        return Ok(bspline);
    }
    if let Some(circle) = curve.cast::<Geom2dCircle>().get() {
        let c2d = Circle2::from_x_axis(gp::ox2d(), circle.radius(), true)?;
        let bspline = bspline_curve_builder(
            circle,
            &ConicToBSplineCurve::from_circle(&c2d, parameterisation)?,
        )?;
        bspline.set_periodic()?;
        return Ok(bspline);
    }
    if curve.cast::<Geom2dBezierCurve>().is_some() {
        return bezier_to_bspline(&curve.cast());
    }
    if curve.cast::<Geom2dBSplineCurve>().is_some() {
        return Ok(curve.copy()?.cast());
    }
    if curve.cast::<Geom2dOffsetCurve>().is_some() {
        return approximate_offset(curve);
    }
    Err(failure(
        "Standard_DomainError",
        "Geom2dConvert::CurveToBSplineCurve: the curve is not a bounded convertible curve",
    ))
}

// The trimmed line: its two end points, the trimmed curve's own bounds as knots
// (not the clamped ones) and degree 1.
fn trimmed_line(ctrim: &Geom2dTrimmedCurve) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let mut poles: Array1<'static, Point2> = one_based(2)?;
    poles[1] = ctrim.start_point()?;
    poles[2] = ctrim.end_point()?;
    let mut knots: Array1<'static, f64> = one_based(2)?;
    (knots[1], knots[2]) = (ctrim.first_parameter(), ctrim.last_parameter());
    let mut mults: Array1<'static, i32> = one_based(2)?;
    (mults[1], mults[2]) = (2, 2);
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &poles, &knots, &mults, 1, false,
    )?))
}

// A trimmed circle or ellipse. With RationalC1, an arc of 6 radians or more is
// converted as two halves joined by CompCurveToBSplineCurve, to avoid the numerical
// overflow of one conversion near 2 PI.
fn elliptic_arc(
    conic: &dyn Geom2dConic,
    [u1, u2]: [f64; 2],
    parameterisation: ParameterisationType,
    convert: &dyn Fn(f64, f64) -> Result<ConicToBSplineCurve, Failure>,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    if parameterisation != ParameterisationType::RationalC1 || u2 - u1 < 6.0 {
        return bspline_curve_builder(conic, &convert(u1, u2)?);
    }
    let umed: f64 = (u1 + u2) * 0.5;
    let first: Handle<Geom2dBSplineCurve> = bspline_curve_builder(conic, &convert(u1, umed)?)?;
    let second: Handle<Geom2dBSplineCurve> = bspline_curve_builder(conic, &convert(umed, u2)?)?;
    let mut joined = CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn Geom2dBoundedCurve>(),
        parameterisation,
    )?;
    joined.add(
        &second.cast::<dyn Geom2dBoundedCurve>(),
        precision::p_confusion(),
        true,
    )?;
    Ok(joined.bspline_curve())
}

/// `BSplineCurveBuilder`: the B-spline of a conversion made in the reference frame,
/// mirrored for an indirect conic frame, then moved to the conic's X axis.
fn bspline_curve_builder(
    conic: &dyn Geom2dConic,
    convert: &ConicToBSplineCurve,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let curve = Geom2dBSplineCurve::with_weights(
        convert.poles(),
        convert.weights(),
        convert.knots(),
        convert.multiplicities(),
        convert.degree(),
        convert.is_periodic(),
    )?;
    let axis: Axis22d = conic.position();
    if axis.x_direction().crossed(axis.y_direction()) < 0.0 {
        // The axis is left-handed: apply a symmetry to the curve.
        let mut sym = Transform2::default();
        sym.set_mirror_axis(gp::ox2d());
        curve.transform(sym)?;
    }
    let mut to_conic = Transform2::default();
    to_conic.set_transformation_between(conic.x_axis(), gp::ox2d());
    Ok(curve.transformed(to_conic)?.cast())
}

// A Bezier curve as a single-span B-spline on [0, 1].
pub(super) fn bezier_to_bspline(
    bezier: &Handle<Geom2dBezierCurve>,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let bezier: &Geom2dBezierCurve = bezier.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "Geom2dConvert: null Geom2d_BezierCurve; expected a curve",
        )
    })?;
    let degree: i32 = bezier.degree();
    let mut knots: Array1<'static, f64> = one_based(2)?;
    (knots[1], knots[2]) = (0.0, 1.0);
    let mut mults: Array1<'static, i32> = one_based(2)?;
    (mults[1], mults[2]) = (degree + 1, degree + 1);
    let poles: Array1<'static, Point2> = bezier.poles();
    let bspline = match bezier.weights() {
        Some(weights) if bezier.is_rational() => {
            Geom2dBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, false)?
        }
        _ => Geom2dBSplineCurve::new(&poles, &knots, &mults, degree, false)?,
    };
    Ok(Handle::new(bspline))
}

// The offset-curve approximation of CurveToBSplineCurve.
fn approximate_offset(
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let (tol2d, order, max_segments, max_degree): (f64, Shape, i32, i32) =
        (1.0e-4, Shape::C2, 16, 14);
    let approx = ApproxCurve::from_curve(curve, tol2d, order, max_segments, max_degree)?;
    if approx.has_result() {
        return Ok(approx.curve());
    }
    Err(failure(
        "Standard_ConstructionError",
        "Geom2dConvert::CurveToBSplineCurve: the offset curve approximation has no result",
    ))
}
