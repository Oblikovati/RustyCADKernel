// Rust translation of ChFi3d_ReparamPcurv, ChFi3d_ProjectPCurv,
// ChFi3d_CheckSameParameter, ChFi3d_SameParameter and ChFi3d_ComputePCurv
// (ChFi3d_Builder_0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pcurves of the fillet edges: projected, straight or reparametrized, and
//! brought to the same parameter as their 3d curve.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dLine,
    Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::approx::SameParameter;
use rustycad_tkgeombase::proj_lib::ProjectedCurve;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Direction2, Point2};

/// `ChFi3d_ReparamPcurv(Uf, Ul, Pcurv)`: a B-spline pcurve (or the B-spline basis of a
/// trimmed one) segmented to the range of the pcurve, then reparametrized on
/// `[Uf, Ul]`; other pcurves are kept; `reparam_pcurv(0.0, 1.0, &mut pcurve)?`.
pub fn reparam_pcurv(uf: f64, ul: f64, pcurv: &mut Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    let Some(curve) = pcurv.get() else {
        return Ok(());
    };
    let (upcf, upcl) = (curve.first_parameter(), curve.last_parameter());
    let mut basis = pcurv.clone();
    if let Some(trimmed) = pcurv.cast::<Geom2dTrimmedCurve>().get() {
        basis = trimmed.basis_curve();
    }
    let pc_handle = basis.cast::<Geom2dBSplineCurve>();
    let Some(pc) = pc_handle.get() else {
        return Ok(());
    };
    let pconf = precision::p_confusion();
    if (upcf - pc.first_parameter()).abs() > pconf || (upcl - pc.last_parameter()).abs() > pconf {
        pc.segment(upcf, upcl, pconf)?;
    }
    if (uf - pc.first_parameter()).abs() <= pconf && (ul - pc.last_parameter()).abs() <= pconf {
        *pcurv = pc_handle.cast();
        return Ok(());
    }
    let mut knots = pc.knots();
    bsplclib::reparametrize(uf, ul, &mut knots)?;
    let rebuilt = Geom2dBSplineCurve::new(
        &pc.poles(),
        &knots,
        &pc.multiplicities(),
        pc.degree(),
        false,
    )?;
    *pcurv = Handle::new(rebuilt).cast();
    Ok(())
}

/// `ChFi3d_ProjectPCurv(HCg, HSg, Pcurv, tol, tolreached)`: the pcurve projected on a
/// surface that is neither Bezier nor B-spline, with the reached tolerance (both kept
/// on such surfaces); Standard_NotImplemented when the projection is not a conic or a
/// polynomial curve; `project_pcurv(&c, &s, &mut pcurve, 1e-6, &mut tolreached)?`.
pub fn project_pcurv(
    hcg: &Handle<dyn Adaptor3dCurve>,
    hsg: &Handle<dyn Adaptor3dSurface>,
    pcurv: &mut Handle<dyn Geom2dCurve>,
    tol: f64,
    tolreached: &mut f64,
) -> Result<(), Failure> {
    let surface_type = hsg.get_type()?;
    if surface_type == SurfaceType::BezierSurface || surface_type == SurfaceType::BSplineSurface {
        return Ok(());
    }
    let projc = ProjectedCurve::with_tolerance(hsg, hcg, tol)?;
    *tolreached = projc.get_tolerance();
    *pcurv = match projc.get_type()? {
        CurveType::Line => Handle::new(Geom2dLine::from_lin2d(projc.line()?)).cast(),
        CurveType::Circle => Handle::new(Geom2dCircle::from_circ2d(projc.circle()?)).cast(),
        CurveType::Ellipse => Handle::new(Geom2dEllipse::from_elips2d(projc.ellipse()?)).cast(),
        CurveType::Hyperbola => {
            Handle::new(Geom2dHyperbola::from_hypr2d(projc.hyperbola()?)).cast()
        }
        CurveType::Parabola => Handle::new(Geom2dParabola::from_parab2d(projc.parabola()?)).cast(),
        CurveType::BezierCurve => projc.bezier()?.cast(),
        CurveType::BSplineCurve => projc.bspline()?.cast(),
        _ => {
            return Err(Failure::with_message(
                "Standard_NotImplemented",
                "echec approximation de la pcurve ",
            ));
        }
    };
    Ok(())
}

/// `ChFi3d_CheckSameParameter(C3d, Pcurv, S, tol3d, tolreached)`: `(ok, tolreached)`,
/// twice the largest distance between the curve and the pcurve on 45 parameters (at
/// least the confusion when within `tol3d`); `check_same_parameter(&c, &pc, &s, 1e-6)?`.
pub fn check_same_parameter(
    c3d: &Handle<dyn Adaptor3dCurve>,
    pcurv: &Handle<dyn Geom2dCurve>,
    s: &Handle<dyn Adaptor3dSurface>,
    tol3d: f64,
) -> Result<(bool, f64), Failure> {
    let (f, l) = (c3d.first_parameter()?, c3d.last_parameter()?);
    let nbp = 45;
    let step = 1.0 / f64::from(nbp - 1);
    let mut tolreached: f64 = 0.0;
    for i in 0..nbp {
        let t = step * f64::from(i);
        let t = (1.0 - t) * f + t * l;
        let uv = pcurv.value(t)?;
        let d2 = s.value(uv.x(), uv.y())?.square_distance(&c3d.value(t)?);
        tolreached = if tolreached < d2 { d2 } else { tolreached };
    }
    tolreached = tolreached.sqrt();
    if tolreached > tol3d {
        return Ok((false, tolreached * 2.0));
    }
    tolreached *= 2.0;
    let confusion = precision::confusion();
    Ok((
        true,
        if tolreached < confusion {
            confusion
        } else {
            tolreached
        },
    ))
}

/// `ChFi3d_SameParameter(C3d, Pcurv, S, tol3d, tolreached)` on adaptors: the check, else
/// `Approx_SameParameter` (the pcurve replaced when it changed); whether it succeeded;
/// `same_parameter(&c, &mut pcurve, &s, 1e-6, &mut tolreached)?`.
pub fn same_parameter(
    c3d: &Handle<dyn Adaptor3dCurve>,
    pcurv: &mut Handle<dyn Geom2dCurve>,
    s: &Handle<dyn Adaptor3dSurface>,
    tol3d: f64,
    tolreached: &mut f64,
) -> Result<bool, Failure> {
    let (ok, reached) = check_same_parameter(c3d, pcurv, s, tol3d)?;
    *tolreached = reached;
    if ok {
        return Ok(true);
    }
    let sp = SameParameter::from_adaptors_and_pcurve(c3d, pcurv, s, tol3d)?;
    if sp.is_done() && !sp.is_same_parameter() {
        *pcurv = sp.curve_2d();
    } else if !sp.is_done() && !sp.is_same_parameter() {
        return Ok(false);
    }
    *tolreached = sp.tol_reached();
    Ok(true)
}

/// `ChFi3d_SameParameter(C3d, Pcurv, S, Pardeb, Parfin, tol3d, tolreached)`: the same on
/// the curve restricted to `[Pardeb, Parfin]`;
/// `same_parameter_geom(&c, &mut pcurve, &s, (0.0, 1.0), 1e-6, &mut tolreached)?`.
pub fn same_parameter_geom(
    c3d: &Handle<dyn GeomCurve>,
    pcurv: &mut Handle<dyn Geom2dCurve>,
    s: &Handle<dyn GeomSurface>,
    (pardeb, parfin): (f64, f64),
    tol3d: f64,
    tolreached: &mut f64,
) -> Result<bool, Failure> {
    let hs: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(s)?).cast();
    let hc: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(c3d, pardeb, parfin)?).cast();
    same_parameter(&hc, pcurv, &hs, tol3d, tolreached)
}

/// `ChFi3d_ComputePCurv(UV1, UV2, Pcurv, Pardeb, Parfin, reverse)`: the straight pcurve
/// from `UV1` to `UV2` (reversed when asked) on `[Pardeb, Parfin]`: an iso line when it
/// runs along U or V at unit speed, else a degree 1 B-spline; trimmed to the range;
/// `compute_pcurv(uv1, uv2, (0.0, 1.0), false)?`.
pub fn compute_pcurv(
    uv1: Point2,
    uv2: Point2,
    (pardeb, parfin): (f64, f64),
    reverse: bool,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let (p1, p2) = if reverse { (uv2, uv1) } else { (uv1, uv2) };
    let pcurv = match iso_line(p1, p2, (pardeb, parfin))? {
        Some(line) => Handle::new(line).cast(),
        None => {
            let (mut poles, mut knots, mut mults) = ([p1, p2], [pardeb, parfin], [2, 2]);
            let (poles, knots, mults) = (
                Array1::from_slice(&mut poles),
                Array1::from_slice(&mut knots),
                Array1::from_slice(&mut mults),
            );
            Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 1, false)?).cast()
        }
    };
    Ok(Handle::new(Geom2dTrimmedCurve::new(&pcurv, pardeb, parfin, true, true)?).cast())
}

/// The iso line of `ChFi3d_ComputePCurv` when the points differ by the range along V
/// (up or down) or U, as natively tested in that order.
fn iso_line(
    p1: Point2,
    p2: Point2,
    (pardeb, parfin): (f64, f64),
) -> Result<Option<Geom2dLine>, Failure> {
    let tol = precision::p_confusion();
    let range = parfin - pardeb;
    let same_u = (p1.x() - p2.x()).abs() <= tol;
    let same_v = (p1.y() - p2.y()).abs() <= tol;
    let (dx, dy) = (Direction2::new(1.0, 0.0)?, Direction2::new(0.0, 1.0)?);
    let line = if same_u && ((p2.y() - p1.y()) - range).abs() <= tol {
        Geom2dLine::from_point(Point2::new(p1.x(), p1.y() - pardeb), dy)
    } else if same_u && ((p1.y() - p2.y()) - range).abs() <= tol {
        Geom2dLine::from_point(Point2::new(p1.x(), p1.y() + pardeb), dy.reversed())
    } else if same_v && ((p2.x() - p1.x()) - range).abs() <= tol {
        Geom2dLine::from_point(Point2::new(p1.x() - pardeb, p1.y()), dx)
    } else if same_v && ((p1.x() - p2.x()) - range).abs() <= tol {
        Geom2dLine::from_point(Point2::new(p1.x() + pardeb, p1.y()), dx.reversed())
    } else {
        return Ok(None);
    };
    Ok(Some(line))
}

/// `ChFi3d_ComputePCurv(C3d, UV1, UV2, Pcurv, S, Pardeb, Parfin, tol3d, tolreached,
/// reverse)` on adaptors: the straight pcurve brought to the same parameter as the
/// curve (`tolreached` written by `ChFi3d_SameParameter`);
/// `compute_pcurv_on(&c, (uv1, uv2), &s, (0.0, 1.0), 1e-6, &mut tolreached, false)?`.
pub fn compute_pcurv_on(
    c3d: &Handle<dyn Adaptor3dCurve>,
    (uv1, uv2): (Point2, Point2),
    s: &Handle<dyn Adaptor3dSurface>,
    range: (f64, f64),
    tol3d: f64,
    tolreached: &mut f64,
    reverse: bool,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let mut pcurv = compute_pcurv(uv1, uv2, range, reverse)?;
    same_parameter(c3d, &mut pcurv, s, tol3d, tolreached)?;
    Ok(pcurv)
}

/// `ChFi3d_ComputePCurv(C3d, UV1, UV2, Pcurv, S, Pardeb, Parfin, tol3d, tolreached,
/// reverse)` on Geom objects (the curve restricted to the range);
/// `compute_pcurv_geom(&c, (uv1, uv2), &s, (0.0, 1.0), 1e-6, &mut tolreached, false)?`.
pub fn compute_pcurv_geom(
    c3d: &Handle<dyn GeomCurve>,
    uvs: (Point2, Point2),
    s: &Handle<dyn GeomSurface>,
    (pardeb, parfin): (f64, f64),
    tol3d: f64,
    tolreached: &mut f64,
    reverse: bool,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let hs: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(s)?).cast();
    let hc: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(c3d, pardeb, parfin)?).cast();
    compute_pcurv_on(&hc, uvs, &hs, (pardeb, parfin), tol3d, tolreached, reverse)
}
