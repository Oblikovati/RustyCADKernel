// Rust translation of IsoIsDeg, TrimC3d and ExtendC2d (ProjLib_ProjectedCurve.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Curves through a pole of the surface: the pole is detected (a degenerated iso
//! line), the curve is trimmed by 1% of its range at that end, and the projection
//! of the trimmed curve is extended by a segment to the singular bound.
use crate::geom_lib::same_range;
use crate::geom2d_convert::CompCurveToBSplineCurve;
use crate::native_failure::failure;
use crate::scan_step::next_scan_parameter;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Point3, Vector2};

/// `IsoIsDeg(S, Param, IT, TolMin, TolMax)`: whether the iso line of the surface at
/// `param` is degenerated (the largest first derivative along it, sampled in ten
/// steps, within `[tol_min, tol_max]`). A range the steps cannot advance over raises
/// Standard_DomainError where the native loop never ends (`crate::scan_step`).
pub(super) fn iso_is_deg(
    s: &dyn Adaptor3dSurface,
    param: f64,
    iso: IsoType,
    tol_min: f64,
    tol_max: f64,
) -> Result<bool, Failure> {
    let (u1, u2) = (s.first_u_parameter()?, s.last_u_parameter()?);
    let (v1, v2) = (s.first_v_parameter()?, s.last_v_parameter()?);
    let (first, last) = if iso == IsoType::IsoV {
        (u1, u2)
    } else {
        (v1, v2)
    };
    let step: f64 = (last - first) / 10.;
    let mut d1_norm_max: f64 = 0.;
    let mut t: f64 = first;
    while t <= last {
        let magnitude: f64 = if iso == IsoType::IsoV {
            s.d1(t, param)?.d1u.magnitude()
        } else {
            s.d1(param, t)?.d1v.magnitude()
        };
        d1_norm_max = d1_norm_max.max(magnitude);
        // Natively `T += Step` never moves `T` on a zero-length or few-ulp range (or at
        // an infinite bound), and the loop never ends.
        t = next_scan_parameter("ProjLib_ProjectedCurve IsoIsDeg", t, step, [first, last])?;
    }
    Ok(!(d1_norm_max > tol_max || d1_norm_max < tol_min))
}

/// The trimmed ends and their singular cases (1 to 4: U min, U max, V min, V max).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Trims {
    pub(super) is_trimmed: [bool; 2],
    pub(super) singular_case: [i32; 2],
}

/// `TrimC3d(myCurve, IsTrimmed, dt, Pole, SingularCase, N, TolConf)`: an end of
/// the curve within `tol_conf` of the pole is moved by `dt` inside.
pub(super) fn trim_c3d(
    curve: &mut Handle<dyn Adaptor3dCurve>,
    trims: &mut Trims,
    dt: f64,
    pole: Point3,
    case: i32,
    tol_conf: f64,
) -> Result<(), Failure> {
    let c: &dyn Adaptor3dCurve = live(curve)?;
    let (mut f, mut l) = (c.first_parameter()?, c.last_parameter()?);
    if c.value(f)?.distance(&pole) <= tol_conf {
        trims.is_trimmed[0] = true;
        f += dt;
        *curve = c.trim(f, l, precision::confusion())?;
        trims.singular_case[0] = case;
    }
    let c: &dyn Adaptor3dCurve = live(curve)?;
    if c.value(l)?.distance(&pole) <= tol_conf {
        trims.is_trimmed[1] = true;
        l -= dt;
        *curve = c.trim(f, l, precision::confusion())?;
        trims.singular_case[1] = case;
    }
    Ok(())
}

fn live(curve: &Handle<dyn Adaptor3dCurve>) -> Result<&dyn Adaptor3dCurve, Failure> {
    curve
        .get()
        .ok_or_else(|| failure("Standard_NullObject", "ProjLib_ProjectedCurve: null curve"))
}

/// The parameter bounds of the extension: `(u1, u2, v1, v2)`.
pub(super) type Box2d = [f64; 4];

// The pole and the bound direction of a singular case.
fn singular_bound(case: i32, b: Box2d) -> (Point2, Direction2) {
    let [u1, u2, v1, v2] = b;
    let unit = |x: f64, y: f64| Direction2::new(x, y).unwrap_or_default();
    match case {
        1 => (Point2::new(u1, v1), unit(0., 1.)),
        2 => (Point2::new(u2, v1), unit(0., 1.)),
        3 => (Point2::new(u1, v1), unit(1., 0.)),
        4 => (Point2::new(u1, v2), unit(1., 0.)),
        _ => (Point2::default(), Direction2::default()),
    }
}

/// `ExtendC2d(aRes, t, dt, u1, u2, v1, v2, FirstOrLast, NumberOfSingularCase)`: the
/// 2D curve extended from its first (`first_or_last` 0) or last end, along its
/// tangent, to the singular bound of `case`.
pub(super) fn extend_c2d(
    res: &mut Handle<Geom2dBSplineCurve>,
    b: Box2d,
    first_or_last: i32,
    case: i32,
) -> Result<(), Failure> {
    let spline: &Geom2dBSplineCurve = res.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "ProjLib_ProjectedCurve: null 2D curve",
        )
    })?;
    let param: f64 = if first_or_last == 0 {
        spline.first_parameter()
    } else {
        spline.last_parameter()
    };
    let bounded: Handle<dyn Geom2dBoundedCurve> = res.clone().cast();
    let mut comp = CompCurveToBSplineCurve::with_curve(&bounded, ParameterisationType::RationalC1)?;
    let (p_bnd, v_bnd): (Point2, Vector2) = spline.d1(param)?;
    let d_bnd: Direction2 = Direction2::from_xy(v_bnd.xy())?;
    let lin: Line2 = Line2::new(p_bnd, d_bnd);
    let (pole, bound_dir) = singular_bound(case, b);
    let bound_lin: Line2 = Line2::new(pole, bound_dir);
    let par_on_lin: f64 = if bound_dir.is_parallel(d_bnd, 100. * precision::angular()) {
        elclib::parameter(&lin, pole)?
    } else {
        let (u1x, u1y) = (bound_lin.direction().x(), bound_lin.direction().y());
        let (u2x, u2y) = (lin.direction().x(), lin.direction().y());
        let uo21x: f64 = lin.location().x() - bound_lin.location().x();
        let uo21y: f64 = lin.location().y() - bound_lin.location().y();
        let d: f64 = u1y * u2x - u1x * u2y;
        (uo21y * u1x - uo21x * u1y) / d
    };
    let seg_line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(lin)).cast();
    let (s1, s2) = if first_or_last == 0 {
        (par_on_lin, 0.)
    } else {
        (0., par_on_lin)
    };
    let segment: Handle<dyn Geom2dBoundedCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&seg_line, s1, s2, true, true)?).cast();
    comp.add(&segment, precision::confusion(), first_or_last != 0)?;
    *res = comp.bspline_curve();
    Ok(())
}

/// The extensions of the trimmed ends, then `GeomLib::SameRange` back to the
/// range of the original curve.
pub(super) fn extend_ends(
    res: &mut Handle<Geom2dBSplineCurve>,
    trims: &Trims,
    boxes: [Box2d; 2],
    cases: [i32; 2],
    range: (f64, f64),
) -> Result<(), Failure> {
    if trims.is_trimmed[0] {
        // Add a segment before the start of the curve.
        extend_c2d(res, boxes[0], 0, cases[0])?;
    }
    if trims.is_trimmed[1] {
        // Add a segment after the end of the curve.
        extend_c2d(res, boxes[1], 1, cases[1])?;
    }
    let spline: &Geom2dBSplineCurve = res.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "ProjLib_ProjectedCurve: null 2D curve",
        )
    })?;
    let bounds: (f64, f64) = (spline.first_parameter(), spline.last_parameter());
    let mut new_curve: Handle<dyn Geom2dCurve> = Handle::default();
    same_range(
        precision::p_confusion(),
        &res.clone().cast(),
        bounds,
        range,
        &mut new_curve,
    )?;
    *res = new_curve.cast();
    Ok(())
}
