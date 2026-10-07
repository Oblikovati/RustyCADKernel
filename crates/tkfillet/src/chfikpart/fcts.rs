// Rust translation of ChFiKPart_InPeriod, ChFiKPart_PCurve and ChFiKPart_ProjPC
// (ChFiKPart_ComputeData_Fcts.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameters and pcurves of the particular cases.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::proj_lib::ProjectedCurve;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Point2;

/// `ChFiKPart_InPeriod(U, UFirst, ULast, Eps)`: `U` moved by whole periods
/// `ULast - UFirst` into the period (within `Eps`), then at least `UFirst`.
///
/// ```
/// use rustycad_tkfillet::chfikpart::in_period;
/// assert_eq!(in_period(-1.0, 0.0, 4.0, 1e-9), 3.0);
/// ```
pub fn in_period(u: f64, u_first: f64, u_last: f64, eps: f64) -> f64 {
    let (mut u, period) = (u, u_last - u_first);
    while eps < (u_first - u) {
        u += period;
    }
    while eps > (u_last - u) {
        u -= period;
    }
    if u < u_first { u_first } else { u }
}

/// `ChFiKPart_PCurve(UV1, UV2, Pardeb, Parfin)`: the straight pcurve from `UV1` to `UV2`
/// as a degree 1 B-spline on `[Pardeb, Parfin]`; `pcurve(p1, p2, 0.0, 1.0)?`.
pub fn pcurve(
    uv1: Point2,
    uv2: Point2,
    pardeb: f64,
    parfin: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let (mut p, mut k, mut m) = ([uv1, uv2], [pardeb, parfin], [2, 2]);
    let (p, k, m) = (
        Array1::from_slice(&mut p),
        Array1::from_slice(&mut k),
        Array1::from_slice(&mut m),
    );
    Ok(Handle::new(Geom2dBSplineCurve::new(&p, &k, &m, 1, false)?))
}

/// The copy of a projected Bezier pcurve (with its weights when rational).
fn bezier_copy(projc: &ProjectedCurve) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let bez = projc.bezier()?;
    let bez = bez.get().ok_or_else(|| {
        Failure::with_message("Standard_NullObject", "ChFiKPart_ProjPC: null Bezier")
    })?;
    let tp = bez.poles();
    if bez.is_rational() {
        let weights = bez.weights_array(|w| w.clone());
        return Ok(Handle::new(Geom2dBezierCurve::with_weights(&tp, &weights)?).cast());
    }
    Ok(Handle::new(Geom2dBezierCurve::new(&tp)?).cast())
}

/// The copy of a projected B-spline pcurve (with its weights when rational).
fn bspline_copy(projc: &ProjectedCurve) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let bsp = projc.bspline()?;
    let bsp = bsp.get().ok_or_else(|| {
        Failure::with_message("Standard_NullObject", "ChFiKPart_ProjPC: null B-spline")
    })?;
    let (tp, tk, tm) = (bsp.poles(), bsp.knots(), bsp.multiplicities());
    if bsp.is_rational() {
        let weights = bsp.weights_array(|w| w.clone());
        return Ok(Handle::new(Geom2dBSplineCurve::with_weights(
            &tp,
            &weights,
            &tk,
            &tm,
            bsp.degree(),
            false,
        )?)
        .cast());
    }
    Ok(Handle::new(Geom2dBSplineCurve::new(&tp, &tk, &tm, bsp.degree(), false)?).cast())
}

/// `ChFiKPart_ProjPC(Cg, Sg, Pcurv)`: the pcurve of the curve projected on an
/// elementary surface (a line, a Bezier or a B-spline copied);
/// Standard_NotImplemented otherwise; `proj_pc(&cg, &sg)?`.
pub fn proj_pc(
    cg: &GeomAdaptorCurve,
    sg: &GeomAdaptorSurface,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if (sg.get_type()? as i32) >= (SurfaceType::BezierSurface as i32) {
        return Err(Failure::with_message(
            "Standard_NotImplemented",
            "approximate pcurve on the left surface",
        ));
    }
    let hcg: Handle<dyn Adaptor3dCurve> = Handle::new(cg.shallow_adaptor()?).cast();
    let hsg: Handle<dyn Adaptor3dSurface> = Handle::new(sg.shallow_adaptor()?).cast();
    let projc = ProjectedCurve::with_curve(&hsg, &hcg)?;
    match projc.get_type()? {
        CurveType::Line => Ok(Handle::new(Geom2dLine::from_lin2d(projc.line()?)).cast()),
        CurveType::BezierCurve => bezier_copy(&projc),
        CurveType::BSplineCurve => bspline_copy(&projc),
        _ => Err(Failure::with_message(
            "Standard_NotImplemented",
            "failed approximation of the pcurve ",
        )),
    }
}
