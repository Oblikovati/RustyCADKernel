// Rust translation of the static helpers CheckSameParameter, CheckSameParameterExact,
// SameParameter and CorrectSameParameter
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The SameParameter checks and corrections of the edges built by BRepFill_Sweep.
use super::helpers::has_pcurves;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::approx::{CurveOnSurface, CurveOnSurfaceOptions, SameParameter};
use rustycad_tkgeombase::geom_lib::CheckCurveOnSurface;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// Standard_NullObject for a missing curve or surface (a crash natively).
fn null(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepFill_Sweep SameParameter: null {what}; expected a {what}"),
        "BRepFill_Sweep",
    )
}

/// `CheckSameParameter(C3d, Pcurv, S, tol3d, tolreached)`: the largest distance at 45
/// samples between the curve and the p-curve on the surface; twice it (at least
/// Precision::Confusion() on success) is the reached tolerance.
pub(super) fn check_same_parameter(
    c3d: &dyn Adaptor3dCurve,
    pcurv: &dyn Geom2dCurve,
    s: &dyn Adaptor3dSurface,
    tol3d: f64,
) -> Result<(bool, f64), Failure> {
    let mut tolreached: f64 = 0.;
    let (f, l) = (c3d.first_parameter()?, c3d.last_parameter()?);
    let nbp: i32 = 45;
    let step: f64 = 1. / f64::from(nbp - 1);
    for i in 0..nbp {
        let mut t: f64 = step * f64::from(i);
        t = (1. - t) * f + t * l;
        let uv = pcurv.value(t)?;
        let p_s = s.value(uv.x(), uv.y())?;
        let p_c = c3d.value(t)?;
        tolreached = tolreached.max(p_s.square_distance(&p_c));
    }
    tolreached = tolreached.sqrt();
    if tolreached > tol3d {
        return Ok((false, tolreached * 2.));
    }
    Ok((true, (tolreached * 2.).max(precision::confusion())))
}

/// `CheckSameParameterExact(C3d, curveOnSurface, tol3d, tolreached)`: the exact
/// maximum distance (GeomLib_CheckCurveOnSurface), times 1.05 on success.
fn check_same_parameter_exact(
    c3d: &Handle<dyn Adaptor3dCurve>,
    curve_on_surface: &Handle<Adaptor3dCurveOnSurface>,
    tol3d: f64,
) -> (bool, f64) {
    let mut check = CheckCurveOnSurface::with_curve(c3d, precision::p_confusion());
    check.set_parallel(false);
    check.perform(curve_on_surface);
    let tolreached: f64 = check.max_distance();
    if tolreached > tol3d {
        return (false, tolreached);
    }
    (true, tolreached.max(precision::confusion()) * 1.05)
}

/// The 3d curve of an edge approximated from its p-curve when the edge has no other
/// p-curve; the reached tolerance when it succeeds.
fn approximated_curve(
    e: &Edge,
    pcurv: &Handle<dyn Geom2dCurve>,
    s: &Handle<dyn Adaptor3dSurface>,
) -> Result<Option<f64>, Failure> {
    if has_pcurves(e)? {
        return Ok(None);
    }
    let hc2d = Geom2dAdaptorCurve::with_curve(pcurv)?;
    let (first, last) = (hc2d.first_parameter()?, hc2d.last_parameter()?);
    let hc2d: Handle<dyn Adaptor2dCurve2d> = Handle::new(hc2d).cast();
    let mut app = CurveOnSurface::without_perform(&hc2d, s, first, last, precision::confusion());
    app.perform(CurveOnSurfaceOptions {
        max_segments: 10,
        max_degree: 10,
        continuity: Continuity::C1,
        only_3d: true,
        only_2d: false,
    })?;
    if !(app.is_done() && app.has_result()) {
        return Ok(None);
    }
    let tolreached: f64 = app.max_error_3d();
    BRepBuilder::default().update_edge_curve(
        e,
        &app.curve_3d().cast::<dyn GeomCurve>(),
        tolreached,
    )?;
    Ok(Some(tolreached))
}

/// `SameParameter(E, Pcurv, Surf, tol3d, tolreached)`: whether the p-curve (possibly
/// reparametrized, or the 3d curve approximated) follows the 3d curve of the edge, and
/// the reached tolerance.
pub(super) fn same_parameter(
    e: &Edge,
    pcurv: &mut Handle<dyn Geom2dCurve>,
    surf: &Handle<dyn GeomSurface>,
    tol3d: f64,
) -> Result<(bool, f64), Failure> {
    let (c3d, f, l) = BRepTool::curve(e)?;
    let hc3d: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(&c3d, f, l)?).cast();
    let s: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(surf)?).cast();
    let (c, p, ad) = (
        hc3d.get().ok_or_else(|| null("curve"))?,
        pcurv.get().ok_or_else(|| null("pcurve"))?,
        s.get().ok_or_else(|| null("surface"))?,
    );
    let (ok, tolreached) = check_same_parameter(c, p, ad, tol3d)?;
    if ok {
        return Ok((true, tolreached));
    }
    if let Some(reached) = approximated_curve(e, pcurv, &s)? {
        return Ok((true, reached));
    }
    let sp = SameParameter::from_adaptors_and_pcurve(&hc3d, pcurv, &s, tol3d)?;
    if sp.is_done() && !sp.is_same_parameter() {
        *pcurv = sp.curve_2d();
    } else if !sp.is_done() && !sp.is_same_parameter() {
        return Ok((false, tolreached));
    }
    let (exact, res_tol) =
        check_same_parameter_exact(&sp.curve_3d(), &sp.curve_on_surface(), tol3d);
    if !exact && res_tol > tolreached {
        return Ok((false, tolreached));
    }
    if sp.is_done() && !sp.is_same_parameter() {
        *pcurv = sp.curve_2d();
    }
    Ok((true, 1.1 * res_tol))
}

/// `CorrectSameParameter(theEdge, theFace1, theFace2)`: the edge tolerance raised to the
/// largest distance at 24 samples between its 3d curve and its curves on the faces (the
/// edge is reversed when both faces are the same, as natively).
pub(super) fn correct_same_parameter(
    the_edge: &mut Edge,
    face1: &Face,
    face2: &Face,
) -> Result<(), Failure> {
    if BRepTool::degenerated(the_edge)? {
        return Ok(());
    }
    let (a_curve, fpar, lpar) = BRepTool::curve(the_edge)?;
    let curve = a_curve.get().ok_or_else(|| null("curve"))?;
    let mut ba_curves: Vec<BRepAdaptorCurve> = Vec::new();
    if !face1.is_null() {
        ba_curves.push(BRepAdaptorCurve::with_edge_on_face(the_edge, face1)?);
    }
    if !face1.is_null() && face1.is_same(face2) {
        the_edge.reverse();
    }
    if !face2.is_null() {
        ba_curves.push(BRepAdaptorCurve::with_edge_on_face(the_edge, face2)?);
    }
    const NCONTROL: i32 = 23;
    let delta: f64 = (lpar - fpar) / f64::from(NCONTROL);
    let mut max_sq_dist: f64 = 0.;
    for i in 0..=NCONTROL {
        let a_param: f64 = fpar + f64::from(i) * delta;
        let a_pnt = curve.value(a_param)?;
        for ba in &ba_curves {
            let a_sq_dist: f64 = a_pnt.square_distance(&ba.value(a_param)?);
            if a_sq_dist > max_sq_dist {
                max_sq_dist = a_sq_dist;
            }
        }
    }
    BRepBuilder::default().update_edge_tolerance(the_edge, max_sq_dist.sqrt())
}
