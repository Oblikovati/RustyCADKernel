// Rust translation of the end of ChFi3d_PerformElSpine (reparametrization, C3
// extensions, smoothing) with CurveCleaner and GoodExt (ChFi3d_Builder_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The concatenated guide of an elementary spine reparametrized as the spine, extended
//! beyond the spine to its open ends, cleaned to C2 or C3 and fitted to the end tangents.
use super::elspine::ElSpineRange;
use crate::chfi_ds::ElSpine;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkgeombase::{geom_convert, geom_lib};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

fn live(bs: &Handle<GeomBSplineCurve>) -> Result<&GeomBSplineCurve, Failure> {
    bs.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi3d_PerformElSpine: null B-spline",
        )
    })
}

/// `CurveCleaner(BS, Tol, MultMin)`: the knots of each multiplicity from the degree down
/// to `MultMin + 1` lowered by one within a tolerance halved at each multiplicity.
pub(super) fn curve_cleaner(
    bs: &Handle<GeomBSplineCurve>,
    tol: f64,
    mult_min: i32,
) -> Result<(), Failure> {
    let curve = live(bs)?;
    let mut tol = tol;
    let nbk = curve.nb_knots();
    let mut mult = curve.degree();
    while mult > mult_min {
        tol *= 0.5;
        for ii in (2..=nbk).rev() {
            if curve.multiplicity(ii)? == mult {
                curve.remove_knot(ii, mult - 1, tol)?;
            }
        }
        mult -= 1;
    }
    Ok(())
}

/// `GoodExt(C, V, f, l, a)`: whether the tangent of the extension on `[f, l]` stays
/// within an angle growing from 0.002 to `a + 0.002` of `V`.
fn good_ext(
    c: &Handle<GeomBSplineCurve>,
    v: Vector3,
    (f, l): (f64, f64),
    a: f64,
) -> Result<bool, Failure> {
    let curve = live(c)?;
    for i in 0..6 {
        let t = f64::from(i) * 0.2;
        let (_, d1) = curve.d1((1.0 - t) * f + t * l)?;
        if d1.angle(v)? > a * t + 0.002 {
            return Ok(false);
        }
    }
    Ok(true)
}

/// One extension of the guide toward `bout` (`after` at the end): continuity 3 down to
/// 1 until the tangent stays close to the spine tangent; the end parameter `w` follows
/// the abscissa `rabdist` from `wref`.
fn extend(
    bspline: &Handle<GeomBSplineCurve>,
    bout: Point3,
    (rabdist, wref, w): (f64, f64, &mut f64),
    (vref, after): (Vector3, bool),
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let mut newc: Handle<GeomBSplineCurve> = Handle::default();
    for icont in (1..=3).rev() {
        let mut an_ext_curve: Handle<dyn GeomBoundedCurve> = bspline.clone().cast();
        geom_lib::extend_curve_to_point(&mut an_ext_curve, bout, icont, after)?;
        newc = an_ext_curve.cast();
        let gacurve = GeomAdaptorCurve::with_curve(&newc.clone().cast::<dyn GeomCurve>())?;
        let gcp = AbscissaPoint::from_start(&gacurve, rabdist, wref, *w)?;
        if gcp.is_done() {
            *w = gcp.parameter()?;
            if good_ext(&newc, vref, (wref, *w), PI * 0.75)? {
                break;
            }
        }
    }
    Ok(newc)
}

/// The extensions at the open ends of a non-periodic guide whose ends miss the spine
/// ends: `(bspline, caredeb, carefin, local_wf, local_wl)`, the knots added on rational
/// guides (to keep out of the smoothing) and the bounds before segmentation.
fn extensions(
    es: &ElSpine,
    bspline: Handle<GeomBSplineCurve>,
    range: &mut ElSpineRange,
    tol: f64,
) -> Result<(Handle<GeomBSplineCurve>, i32, i32, f64, f64), Failure> {
    let (mut bspline, mut caredeb, mut carefin) = (bspline, 0, 0);
    let (mut local_wf, mut local_wl) = (range.wf, range.wl);
    let periodic = es.is_periodic()?;
    if !periodic && !range.pdeb.is_equal(&live(&bspline)?.pole(1)?, tol) {
        let rational = live(&bspline)?.is_rational();
        let rabdist = range.wrefdeb - range.wf;
        let bout = range.pdeb.translated(range.vrefdeb * (-20.0 * rabdist));
        let newc = extend(
            &bspline,
            bout,
            (-rabdist, range.wrefdeb, &mut range.wf),
            (range.vrefdeb, false),
        )?;
        if rational {
            caredeb = live(&newc)?.nb_knots() - live(&bspline)?.nb_knots();
        }
        bspline = newc;
        local_wf = live(&bspline)?.first_parameter();
    }
    let last_pole = live(&bspline)?.pole(live(&bspline)?.nb_poles())?;
    if !periodic && !range.pfin.is_equal(&last_pole, tol) {
        let rational = live(&bspline)?.is_rational();
        let rabdist = range.wl - range.wreffin;
        let bout = range.pfin.translated(range.vreffin * (20.0 * rabdist));
        let newc = extend(
            &bspline,
            bout,
            (rabdist, range.wreffin, &mut range.wl),
            (range.vreffin, true),
        )?;
        if rational {
            carefin = live(&newc)?.nb_knots() - live(&bspline)?.nb_knots();
        }
        bspline = newc;
        local_wl = live(&bspline)?.last_parameter();
    }
    Ok((bspline, caredeb, carefin, local_wf, local_wl))
}

/// The guide segmented to `[WF, WL]` (snapped to its bounds within `tol`) when it
/// extends beyond, then made C1 when rational (unless its origin moves).
fn segment_and_c1(
    es: &ElSpine,
    bspline: Handle<GeomBSplineCurve>,
    range: &mut ElSpineRange,
    (local_wf, local_wl): (f64, f64),
    tol: f64,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let curve = live(&bspline)?;
    if (curve.first_parameter() - range.wf).abs() < tol {
        range.wf = curve.first_parameter();
    }
    if (curve.last_parameter() - range.wl).abs() < tol {
        range.wl = curve.last_parameter();
    }
    if local_wf < range.wf || local_wl > range.wl {
        curve.segment(range.wf, range.wl, precision::p_confusion())?;
        es.set_first_parameter(range.wf);
        es.set_last_parameter(range.wl);
    }
    if !curve.is_rational() {
        return Ok(bspline);
    }
    let mut c1: Handle<GeomBSplineCurve> = Handle::new(GeomBSplineCurve::copy_of(curve)?);
    geom_convert::c0_bspline_to_c1_bspline_curve(&mut c1, tol, 0.1)?;
    if live(&c1)?.first_parameter() == curve.first_parameter() {
        return Ok(c1);
    }
    Ok(bspline)
}

/// The smoothing to C2 (C3 when asked): the degree raised, the knots cleaned and their
/// multiplicities lowered between `fk` and `lk`; the maximal multiplicity.
fn smoothing(
    bspline: &Handle<GeomBSplineCurve>,
    (caredeb, carefin): (i32, i32),
    continuity: Continuity,
    wspan: f64,
) -> Result<i32, Failure> {
    let curve = live(bspline)?;
    if (caredeb != 0 || carefin != 0) && curve.degree() < 8 {
        curve.increase_degree(8)?;
    }
    let fk = (if curve.is_periodic() { 1 } else { 2 }) + caredeb;
    let lk = curve.nb_knots() - 1 - carefin;
    let (min_degree, drop) = if continuity == Continuity::C3 {
        (7, 3)
    } else {
        (5, 2)
    };
    if curve.degree() < min_degree {
        curve.increase_degree(min_degree)?;
    }
    let mult_max = curve.degree() - drop;
    curve_cleaner(bspline, wspan * 1.0e-4, 1)?;
    curve_cleaner(bspline, wspan * 1.0e-2, mult_max)?;
    let mult_min = if curve.degree() - 4 < 1 {
        1
    } else {
        curve.degree() - 4
    };
    for ii in fk..=lk {
        if curve.multiplicity(ii)? > mult_max {
            curve.remove_knot(ii, mult_max, wspan / 10.0)?;
        }
        if curve.multiplicity(ii)? > mult_min {
            curve.remove_knot(ii, mult_min, wspan * 1.0e-4)?;
        }
    }
    Ok(mult_max)
}

/// Whether an end of the guide must be moved onto the spine end: when it was not
/// extended and its tangent or point is not the spine one.
fn end_off(
    curve: &GeomBSplineCurve,
    w: f64,
    local_w: f64,
    (p, v): (Point3, Vector3),
    tol: f64,
) -> Result<bool, Failure> {
    let (pc, mut vc) = curve.d1(w)?;
    vc.normalize()?;
    let (scal, dis) = (v.dot(vc), p.distance(&pc));
    Ok((w - local_w).abs() < 1.0e-12 && (scal <= 0.9999999 || dis >= tol))
}

/// The periodic guide made periodic, or the open guide fitted to the end points and
/// tangents of the elementary spine when an end is off.
fn close_or_fit(
    es: &ElSpine,
    bspline: Handle<GeomBSplineCurve>,
    range: &ElSpineRange,
    (local_wf, local_wl): (f64, f64),
    (mult_max, approx_count, tol): (i32, i32, f64),
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let curve = live(&bspline)?;
    let wspan = (range.wl - range.wf).abs();
    if es.is_periodic()? {
        if !curve.is_periodic() {
            curve.set_periodic()?;
            if approx_count != 0 {
                curve.remove_knot(1, mult_max, wspan / 10.0)?;
            }
        }
        return Ok(bspline);
    }
    let deb = es.first_point_and_tgt();
    let adjust_deb = end_off(curve, range.wf, local_wf, deb, tol)?;
    let fin = es.last_point_and_tgt();
    let adjust_fin = end_off(curve, range.wl, local_wl, fin, tol)?;
    if !(adjust_deb || adjust_fin) {
        return Ok(bspline);
    }
    let mut an_ext_curve: Handle<dyn GeomBoundedCurve> = bspline.cast();
    geom_lib::adjust_extremity(&mut an_ext_curve, deb.0, fin.0, deb.1, fin.1)?;
    Ok(an_ext_curve.cast())
}

/// The end of `ChFi3d_PerformElSpine` on the concatenated guide: reparametrized on the
/// references of the spine, extended, segmented, smoothed, closed or fitted, and set
/// as the curve of the elementary spine.
pub(super) fn extend_and_smooth(
    es: &ElSpine,
    bspline: Handle<GeomBSplineCurve>,
    mut range: ElSpineRange,
    (continuity, tol): (Continuity, f64),
    approx_count: i32,
) -> Result<(), Failure> {
    let curve = live(&bspline)?;
    let mut knots = curve.knots();
    bsplclib::reparametrize(range.wrefdeb, range.wreffin, &mut knots)?;
    curve.set_knots(&knots)?;
    let (bspline, caredeb, carefin, local_wf, local_wl) = extensions(es, bspline, &mut range, tol)?;
    let bspline = segment_and_c1(es, bspline, &mut range, (local_wf, local_wl), tol)?;
    let wspan = (range.wl - range.wf).abs();
    let mult_max = smoothing(&bspline, (caredeb, carefin), continuity, wspan)?;
    let bspline = close_or_fit(
        es,
        bspline,
        &range,
        (local_wf, local_wl),
        (mult_max, approx_count, tol),
    )?;
    es.set_curve(&bspline.cast())
}
