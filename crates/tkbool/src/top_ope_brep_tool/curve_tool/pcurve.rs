// Rust translation of TopOpeBRepTool_CurveTool::MakePCurveOnFace and of the static
// CheckPCurve (ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_CurveTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The p-curves on faces: the projection of a 3d curve shifted into the domain of the
//! face, and the check of a p-curve against the face bounds.
use super::{TopOpeBRepToolCurveTool, make_pcurve};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::proj_lib::ProjectedCurve;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Transform2, Vector2};
use std::f64::consts::PI;

/// The shift of a periodic bound so that the value falls in its period.
fn periodic_shift(value: f64, (min, max): (f64, f64), period: f64) -> (f64, f64) {
    let mut nshift: i32 = ((value - min) / period) as i32;
    if value < min + period * f64::from(nshift) {
        nshift -= 1;
    }
    (
        min + period * f64::from(nshift),
        max + period * f64::from(nshift),
    )
}

/// `CheckPCurve(aPC, aFace)`: whether 23 points of the p-curve are within 1% of the UV
/// bounds of the face (shifted by the periods of the surface at its middle).
pub(super) fn check_pcurve(a_pc: &Handle<dyn Geom2dCurve>, a_face: &Face) -> Result<bool, Failure> {
    const NPOINTS: i32 = 23;
    let (mut umin, mut umax, mut vmin, mut vmax) = brep_tools::uv_bounds(a_face)?;
    let tol_u: f64 = ((umax - umin) * 0.01).max(precision::confusion());
    let tol_v: f64 = ((vmax - vmin) * 0.01).max(precision::confusion());
    let pc = a_pc.get().ok_or_else(null_curve)?;
    let (fp, lp) = (pc.first_parameter(), pc.last_parameter());
    let step: f64 = (lp - fp) / f64::from(NPOINTS + 1);
    // The domain adjusted for periodic surfaces.
    let (a_surf_located, _) = BRepTool::surface_with_location(a_face)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = a_surf_located.cast();
    let a_surf: Handle<dyn GeomSurface> = trimmed
        .get()
        .map_or(a_surf_located.clone(), |t| t.basis_surface());
    let surf = a_surf.get().ok_or_else(null_curve)?;
    let mid: Point2 = pc.value((fp + lp) / 2.)?;
    if surf.is_u_periodic() {
        (umin, umax) = periodic_shift(mid.x(), (umin, umax), surf.u_period()?);
    }
    if surf.is_v_periodic() {
        (vmin, vmax) = periodic_shift(mid.y(), (vmin, vmax), surf.v_period()?);
    }
    for i in 1..=NPOINTS {
        let p = pc.value(fp + f64::from(i) * step)?;
        let (u, v) = (p.x(), p.y());
        if umin - u > tol_u || u - umax > tol_u || vmin - v > tol_v || v - vmax > tol_v {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The p-curve on a sphere mirrored about the apex iso (-PI/2 or PI/2) and moved by PI
/// along U when its middle is out of the V range (a quasi-periodic shift).
fn sphere_shift(
    c2d: Handle<dyn Geom2dCurve>,
    bas: &BRepAdaptorSurface,
    (t, u2, v2): (f64, f64, f64),
) -> Result<(Handle<dyn Geom2dCurve>, Point2), Failure> {
    let (v_first, v_last) = (bas.first_v_parameter()?, bas.last_v_parameter()?);
    let (mincond, maxcond) = (v2 < v_first, v2 > v_last);
    let curve = c2d.get().ok_or_else(null_curve)?;
    if !(mincond || maxcond) {
        return Ok((c2d.clone(), curve.value(t)?));
    }
    let pct: Handle<dyn Geom2dCurve> = curve.copy()?.cast();
    let p = pct.get().ok_or_else(null_curve)?;
    let po = Point2::new(0., if maxcond { PI / 2. } else { -PI / 2. });
    let mut a_trsf = Transform2::default();
    a_trsf.set_mirror_axis(Axis2d::new(po, Direction2::X));
    p.transform(a_trsf)?;
    // A translation along U by PI.
    let mut vec = Vector2::new(PI, 0.);
    if u2 - bas.first_u_parameter()? - PI > -1e-7 {
        vec.reverse();
    }
    p.translate(vec)?;
    let test: Point2 = p.value(t)?;
    Ok((pct, test))
}

/// The period shift of a coordinate out of its UV bounds by more than 1e-7.
fn period_shift(
    periodic: bool,
    value: f64,
    (min, max): (f64, f64),
    period: impl Fn() -> Result<f64, Failure>,
) -> Result<f64, Failure> {
    if !periodic {
        return Ok(0.);
    }
    let (mincond, maxcond) = (min - value > 1e-7, value - max > 1e-7);
    if !(mincond || maxcond) {
        return Ok(0.);
    }
    Ok(if mincond { period()? } else { -period()? })
}

impl TopOpeBRepToolCurveTool {
    /// `MakePCurveOnFace(S, C, TolReached2d, first, last)` (natively not trimmed): the
    /// projection of the curve (trimmed when first < last) on the face, its middle
    /// shifted into the UV bounds by the periods (and the apex mirror of a sphere);
    /// the reached 2d tolerance; `let (pc, tol) = TopOpeBRepToolCurveTool::make_pcurve_on_face(&face, &c, 0., 0.)?;`.
    pub fn make_pcurve_on_face(
        s: &Shape,
        c3d: &Handle<dyn GeomCurve>,
        first: f64,
        last: f64,
    ) -> Result<(Handle<dyn Geom2dCurve>, f64), Failure> {
        let f = topods::face(s)?;
        let bas = BRepAdaptorSurface::with_face(&f, false)?;
        let gac = if first < last {
            GeomAdaptorCurve::with_range(c3d, first, last)?
        } else {
            GeomAdaptorCurve::with_curve(c3d)?
        };
        let (gf, gl) = (gac.first_parameter()?, gac.last_parameter()?);
        let bahs: Handle<dyn Adaptor3dSurface> =
            Handle::new(BRepAdaptorSurface::with_face(&f, false)?).cast();
        let bahc: Handle<dyn Adaptor3dCurve> = Handle::new(gac).cast();
        let projcurv = ProjectedCurve::with_curve(&bahs, &bahc)?;
        let mut c2d = make_pcurve(&projcurv)?;
        let tol_reached2d: f64 = projcurv.get_tolerance();
        let (umin, umax, vmin, vmax) = brep_tools::uv_bounds(&f)?;
        let t: f64 = (gf + gl) * 0.5;
        let mut p_c2d: Point2 = c2d.get().ok_or_else(null_curve)?.value(t)?;
        if bas.get_type()? == SurfaceType::Sphere {
            (c2d, p_c2d) = sphere_shift(c2d, &bas, (t, p_c2d.x(), p_c2d.y()))?;
        }
        let surface = bahs.get().ok_or_else(null_curve)?;
        let du = period_shift(surface.is_u_periodic()?, p_c2d.x(), (umin, umax), || {
            surface.u_period()
        })?;
        let dv = period_shift(surface.is_v_periodic()?, p_c2d.y(), (vmin, vmax), || {
            surface.v_period()
        })?;
        if du != 0. || dv != 0. {
            let pct: Handle<dyn Geom2dCurve> = c2d.get().ok_or_else(null_curve)?.copy()?.cast();
            pct.get()
                .ok_or_else(null_curve)?
                .translate(Vector2::new(du, dv))?;
            c2d = pct;
        }
        Ok((c2d, tol_reached2d))
    }
}

/// Standard_NullObject of a null curve or surface (natively a crash).
fn null_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "TopOpeBRepTool_CurveTool: null curve or surface; expected a geometry",
        "TopOpeBRepTool_CurveTool",
    )
}
