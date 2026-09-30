// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_CurveToAnaCurve.cxx
// (GetCircle, ComputeCircle, IsArrayPntPlanar, ConicDefinition, ComputeEllipse)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The circle and ellipse recognizers of `GeomConvert_CurveToAnaCurve`.
use super::curve_to_ana_curve::AnaCurveFit;
use crate::array_access::matrix;
use crate::gce::{MakeCirc, Root};
use crate::geom_lib::live;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomEllipse};
use rustycad_tkmath::elclib::{self, ElementaryCurve};
use rustycad_tkmath::gp::{
    self, Axis2, Axis3, Circle3, Direction3, Ellipse3, Point3, Transform3, Vector3, Xyz,
};
use rustycad_tkmath::math::{Gauss, Matrix, Vector};
use std::f64::consts::PI;

/// `GetCircle(Circ, P0, P1, P2)`: see `CurveToAnaCurve::get_circle`.
pub(super) fn get_circle(
    circ: &mut Circle3,
    p0: Point3,
    p1: Point3,
    p2: Point3,
) -> Result<bool, Failure> {
    // Control if points are not aligned (should be done by MakeCirc).
    let max_coord: f64 = precision::infinite().sqrt();
    if [p0, p1, p2].iter().any(|point| beyond(*point, max_coord)) {
        return Ok(false);
    }
    // Building the circle.
    let mkc = MakeCirc::through_points(p0, p1, p2)?;
    if !mkc.is_done() {
        return Ok(false);
    }
    *circ = mkc.value()?;
    if circ.radius() < gp::resolution() {
        return Ok(false);
    }
    // Align to P0.
    let mut axe: Axis2 = circ.position();
    axe.set_x_direction(Direction3::from_vector(Vector3::between(
        circ.location(),
        p0,
    ))?)?;
    circ.set_position(axe);
    Ok(true)
}

// A coordinate of `point` beyond `max_coord` in absolute value.
fn beyond(point: Point3, max_coord: f64) -> bool {
    point.x().abs() > max_coord || point.y().abs() > max_coord || point.z().abs() > max_coord
}

/// `ComputeCircle`: see `CurveToAnaCurve::compute_circle`.
pub(super) fn compute_circle(
    c3d: &Handle<dyn GeomCurve>,
    tol: f64,
    (c1, c2): (f64, f64),
    fit: &mut AnaCurveFit,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let curve: &dyn GeomCurve = live(c3d, "GeomConvert_CurveToAnaCurve::ComputeCircle")?;
    if c3d.cast::<GeomCircle>().is_some() {
        *fit = AnaCurveFit {
            first: c1,
            last: c2,
            deviation: 0.0,
        };
        return Ok(c3d.clone());
    }
    let ca: f64 = (c1 + c1 + c2) / 3.0;
    let cb: f64 = (c1 + c2 + c2) / 3.0;
    let mut crc = Circle3::default();
    if !get_circle(
        &mut crc,
        curve.value(c1)?,
        curve.value(ca)?,
        curve.value(cb)?,
    )? {
        return Ok(Handle::default());
    }
    // It remains to check that it is a circle: take 20 points.
    let du: f64 = (c2 - c1) / 20.0;
    let mut a_max: f64 = 0.0;
    for i in 0..=20 {
        let dist: f64 = crc.distance(curve.value(c1 + du * f64::from(i))?);
        if dist > tol {
            return Ok(Handle::default()); // not done
        }
        if dist > a_max {
            a_max = dist;
        }
    }
    fit.deviation = a_max;
    (fit.first, fit.last) = periodic_range(&crc, curve, (c1, c2))?;
    Ok(Handle::new(GeomCircle::from_circ(crc)).cast())
}

// The parameters on a closed conic of the ends of `[c1, c2]`: the first in
// [0, 2Pi) (snapped to 0 within PConfusion of 0 or 2Pi), the middle and the last
// each in the period following the previous one.
fn periodic_range<C: ElementaryCurve<Point = Point3>>(
    conic: &C,
    curve: &dyn GeomCurve,
    (c1, c2): (f64, f64),
) -> Result<(f64, f64), Failure> {
    let pi2: f64 = 2.0 * PI;
    let mut cf: f64 = elclib::parameter(conic, curve.value(c1)?)?;
    cf = elclib::in_period(cf, 0.0, pi2);
    // The first parameter should be close to zero.
    if cf.abs() < precision::p_confusion() || (pi2 - cf).abs() < precision::p_confusion() {
        cf = 0.0;
    }
    let mut cm: f64 = elclib::parameter(conic, curve.value((c1 + c2) / 2.0)?)?;
    cm = elclib::in_period(cm, cf, cf + pi2);
    let cl: f64 = elclib::parameter(conic, curve.value(c2)?)?;
    Ok((cf, elclib::in_period(cl, cm, cm + pi2)))
}

// `IsArrayPntPlanar(HAP, Norm, prec)`: the normal (P2-P1)^(P3-P1) when every
// point lies in its plane through P1, within the angle `prec`.
fn is_array_pnt_planar(points: &[Point3], prec: f64) -> Result<Option<Direction3>, Failure> {
    if points.len() < 3 {
        return Ok(None);
    }
    let (p1, p2, p3): (Point3, Point3, Point3) = (points[0], points[1], points[2]);
    if p1.distance(&p2) < prec || p1.distance(&p3) < prec {
        return Ok(None);
    }
    let (v1, v2): (Vector3, Vector3) = (Vector3::between(p1, p2), Vector3::between(p1, p3));
    if v1.is_parallel(v2, prec)? {
        return Ok(None);
    }
    let nv: Vector3 = v1.crossed(v2);
    if [nv.x(), nv.y(), nv.z()]
        .iter()
        .any(|coord| precision::is_infinite(*coord))
    {
        return Ok(None);
    }
    if nv.magnitude() < gp::resolution() {
        return Ok(None);
    }
    for pn in &points[3..] {
        let dist1: f64 = p1.distance(pn);
        if dist1 < prec || precision::is_infinite(dist1) {
            return Ok(None);
        }
        if !nv.is_normal(Vector3::between(p1, *pn), prec)? {
            return Ok(None);
        }
    }
    Ok(Some(Direction3::from_vector(nv)?))
}

/// The center, main axis and radii (Rmin, Rmax) of a central conic.
struct ConicAxes {
    center: Point3,
    main_axis: Direction3,
    rmin: f64,
    rmax: f64,
}

// `ConicDefinition(a, b1, c, d1, e1, f, IsParab, IsEllip, ...)` of the conic
// a*x2 + b1*x*y + c*y2 + d1*x + e1*y + f = 0. The only caller passes IsParab
// false and IsEllip true, so only the central ellipse branch is reachable; the
// empty parabola branch and the hyperbola branches are not ported.
fn conic_definition(
    a: f64,
    [b1, c, d1, e1]: [f64; 4],
    f: f64,
) -> Result<Option<ConicAxes>, Failure> {
    // Change of variable: a*x2 + 2*b*x*y + c*y2 + 2*d*x + 2*e*y + f = 0.
    let (b, d, e): (f64, f64, f64) = (b1 / 2.0, d1 / 2.0, e1 / 2.0);
    let eps: f64 = 1.0e-08; // tol:native ComputedForm threshold
    // The determinants of | a b d ; b c e ; d e f | and | a b ; b c |.
    let gdet: f64 = a * c * f + 2.0 * b * d * e - c * d * d - a * e * e - b * b * f;
    let pdet: f64 = a * c - b * b;
    let xcen: f64 = (b * e - c * d) / pdet;
    let ycen: f64 = (b * d - a * e) / pdet;
    let Some((cos2t, auxil)) = double_angle(a - c, 2.0 * b, eps) else {
        return Ok(None);
    };
    let cost: f64 = ((1.0 + cos2t) / 2.0).sqrt();
    let sint: f64 = ((1.0 - cos2t) / 2.0).sqrt();
    let aprim: f64 = (a + c + auxil) / 2.0;
    let cprim: f64 = (a + c - auxil) / 2.0;
    if aprim.abs() < gp::resolution() || cprim.abs() < gp::resolution() {
        return Ok(None);
    }
    let term1: f64 = -gdet / (aprim * pdet);
    let term2: f64 = -gdet / (cprim * pdet);
    let Some((rmin, rmax)) = ellipse_radii(term1, term2, eps) else {
        return Ok(None);
    };
    Ok(Some(ConicAxes {
        center: Point3::new(xcen, ycen, 0.0),
        main_axis: Direction3::new(cost, sint, 0.0)?,
        rmin,
        rmax,
    }))
}

// cos(2t) and sqrt(term1^2 + term2^2) of the principal axes; None when only
// `term1` is below `eps`.
fn double_angle(term1: f64, term2: f64, eps: f64) -> Option<(f64, f64)> {
    if term2.abs() <= eps && term1.abs() <= eps {
        return Some((1.0, 0.0));
    }
    if term1.abs() < eps {
        return None;
    }
    let t2d: f64 = term2 / term1; // skl 21.11.2001
    Some((
        1.0 / (1.0 + t2d * t2d).sqrt(),
        (term1 * term1 + term2 * term2).sqrt(),
    ))
}

// The radii (Rmin, Rmax) of an ellipse, None when one is not above `eps`.
fn ellipse_radii(term1: f64, term2: f64, eps: f64) -> Option<(f64, f64)> {
    if term1 <= eps || term2 <= eps {
        return None;
    }
    let (rmin, rmax): (f64, f64) = (term1.sqrt(), term2.sqrt());
    if rmax < rmin {
        return Some((term2.sqrt(), term1.sqrt()));
    }
    Some((rmin, rmax))
}

/// `ComputeEllipse`: see `CurveToAnaCurve::compute_ellipse`.
pub(super) fn compute_ellipse(
    c3d: &Handle<dyn GeomCurve>,
    tol: f64,
    (c1, c2): (f64, f64),
    fit: &mut AnaCurveFit,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let curve: &dyn GeomCurve = live(c3d, "GeomConvert_CurveToAnaCurve::ComputeEllipse")?;
    if c3d.cast::<GeomEllipse>().is_some() {
        *fit = AnaCurveFit {
            first: c1,
            last: c2,
            deviation: 0.0,
        };
        return Ok(c3d.clone());
    }
    let prec: f64 = precision::p_confusion();
    let (samples, a_trans): ([Point3; 5], Vector3) = centered_samples(curve, (c1, c2), prec)?;
    let Some(ndir) = is_array_pnt_planar(&samples, prec)? else {
        return Ok(Handle::default());
    };
    if ndir.x().abs() < gp::resolution()
        && ndir.y().abs() < gp::resolution()
        && ndir.z().abs() < gp::resolution()
    {
        return Ok(Handle::default());
    }
    let tr: Transform3 =
        Transform3::transformation_to(Axis3::from_normal(Point3::default(), ndir)?);
    let Some((coefficients, zn)) = conic_coefficients(&samples, tr)? else {
        return Ok(Handle::default());
    };
    let Some(ellipse) = fitted_ellipse(coefficients, zn, (tr, ndir), a_trans)? else {
        return Ok(Handle::default());
    };
    finish_ellipse(ellipse, curve, (c1, c2), tol, fit)
}

// The five samples of `[c1, c2]` (of its first 4/5 when closed) moved by the
// opposite of their barycenter, and that translation.
fn centered_samples(
    curve: &dyn GeomCurve,
    (c1, c2): (f64, f64),
    prec: f64,
) -> Result<([Point3; 5], Vector3), Failure> {
    let p_start: Point3 = curve.value(c1)?;
    let p_end: Point3 = curve.value(c2)?;
    let is_clos: bool = p_start.distance(&p_end) < prec;
    let c2n: f64 = if is_clos { c2 - (c2 - c1) / 5.0 } else { c2 };
    let mut samples: [Point3; 5] = [p_start; 5];
    let mut a_bc: Xyz = Xyz::default();
    a_bc += p_start.xyz();
    let dc: f64 = (c2n - c1) / 4.0;
    for i in 1..5 {
        let point: Point3 = curve.value(c1 + dc * f64::from(i))?;
        samples[i as usize] = point;
        a_bc += point.xyz();
    }
    a_bc /= 5.0;
    a_bc *= -1.0;
    let a_trans = Vector3::from(a_bc);
    for sample in &mut samples {
        sample.translate(a_trans);
    }
    Ok((samples, a_trans))
}

// The coefficients (A, B, C, D, E) of the conic A*x2 + B*x*y + C*y2 + D*x + E*y
// = -1 through the samples in the frame of `tr` (math_Gauss), and the Z of the
// last sample there; None when the system is singular.
fn conic_coefficients(
    samples: &[Point3; 5],
    tr: Transform3,
) -> Result<Option<([f64; 5], f64)>, Failure> {
    let mut dt: Matrix<'static> = matrix(1, 5, 1, 5, 0.0)?;
    let f: Vector<'static> = Vector::filled(1, 5, -1.0)?;
    let mut zn: f64 = 0.0;
    for (row, sample) in (1..=5).zip(samples) {
        let pt: Point3 = sample.transformed(tr);
        let (xn, yn): (f64, f64) = (pt.x(), pt.y());
        zn = pt.z();
        let terms: [f64; 5] = [xn * xn, xn * yn, yn * yn, xn, yn];
        for (col, term) in (1..=5).zip(terms) {
            dt[(row, col)] = term;
        }
    }
    let solver = Gauss::new(&dt)?;
    if !solver.is_done() {
        return Ok(None);
    }
    let mut sl: Vector<'static> = Vector::with_bounds(1, 5)?;
    solver.solve_into(&f, &mut sl)?;
    Ok(Some(([sl[1], sl[2], sl[3], sl[4], sl[5]], zn)))
}

// The ellipse of the conic `coefficients` (in the frame of `tr`, at height `zn`)
// moved back by `-a_trans`; None when the conic is not an ellipse, or is a circle.
fn fitted_ellipse(
    [af, bf, cf, df, ef]: [f64; 5],
    zn: f64,
    (tr, ndir): (Transform3, Direction3),
    mut a_trans: Vector3,
) -> Result<Option<Ellipse3>, Failure> {
    let q1: f64 =
        af * cf + bf * ef * df / 4.0 - cf * df * df / 4.0 - bf * bf / 4.0 - af * ef * ef / 4.0;
    let q2: f64 = af * cf - bf * bf / 4.0;
    let q3: f64 = af + cf;
    // Hyperbolas (Q2 < 0) and parabolas (Q2 == 0) are not recognized.
    if !(q2 > 0.0 && q1 * q3 < 0.0) {
        return Ok(None);
    }
    let Some(axes) = conic_definition(af, [bf, cf, df, ef], 1.0)? else {
        return Ok(None);
    };
    if axes.rmax - axes.rmin < precision::confusion() {
        return Ok(None); // really a circle, which the circle method must recognize
    }
    a_trans *= -1.0;
    let tr2: Transform3 = tr.inverted()?;
    let center = Point3::new(axes.center.x(), axes.center.y(), zn);
    let new_center: Point3 = center.transformed(tr2);
    let (ax, ay, az): (f64, f64, f64) =
        (axes.main_axis.x(), axes.main_axis.y(), axes.main_axis.z());
    let ptmp = Point3::new(
        center.x() + ax * 10.0,
        center.y() + ay * 10.0,
        center.z() + az * 10.0,
    );
    let new_ptmp: Point3 = ptmp.transformed(tr2);
    let new_main_axis = Direction3::new(
        new_ptmp.x() - new_center.x(),
        new_ptmp.y() - new_center.y(),
        new_ptmp.z() - new_center.z(),
    )?;
    let ax2: Axis2 = Axis2::with_x_direction(new_center, ndir, new_main_axis)?;
    let mut ellipse = Ellipse3::new(ax2, axes.rmax, axes.rmin)?;
    ellipse.translate(a_trans);
    Ok(Some(ellipse))
}

// The ellipse when 20 samples of `[c1, c2]` are within `tol` of it, with its
// deviation and range written to `fit`.
fn finish_ellipse(
    ellipse: Ellipse3,
    curve: &dyn GeomCurve,
    (c1, c2): (f64, f64),
    tol: f64,
    fit: &mut AnaCurveFit,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let gell = GeomEllipse::from_elips(ellipse);
    let mut param2: f64 = 0.0;
    let dc: f64 = (c2 - c1) / 20.0;
    for i in 1..=20 {
        let pp: Point3 = curve.value(c1 + f64::from(i) * dc)?;
        let a_par: f64 = elclib::parameter(&ellipse, pp)?;
        let dist: f64 = gell.value(a_par)?.distance(&pp);
        if dist > tol {
            return Ok(Handle::default()); // not done
        }
        if dist > param2 {
            param2 = dist;
        }
    }
    fit.deviation = param2;
    (fit.first, fit.last) = periodic_range(&ellipse, curve, (c1, c2))?;
    Ok(Handle::new(gell).cast())
}
