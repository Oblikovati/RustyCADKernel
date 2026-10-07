// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_GEOMETRY.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871 (FUN_tool_nCinsideS and
// FUN_tool_nC2dINSIDES, declared there, are defined in TopOpeBRepTool_matter.cxx).

//! The geometric `FUN_tool_*` helpers: basis curves, curve tangents, surface normals
//! (with the apices of cones and spheres), line and quadric tests and closures.
use super::shape_tool::{TopOpeBRepToolShapeTool, null_geometry};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dOffsetCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Vector3};
use std::f64::consts::PI;

/// `BASISCURVE2D(C)`: the basis of offset and trimmed 2d curves, recursively;
/// Standard_NullObject for a null curve (a null dereference natively);
/// `basis_curve2d(&pc)?`.
pub fn basis_curve2d(c: &Handle<dyn Geom2dCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if c.is_null() {
        return Err(null_geometry("BASISCURVE2D"));
    }
    let offset: Handle<Geom2dOffsetCurve> = c.clone().cast();
    if let Some(o) = offset.get() {
        return basis_curve2d(&o.basis_curve());
    }
    let trimmed: Handle<Geom2dTrimmedCurve> = c.clone().cast();
    if let Some(t) = trimmed.get() {
        return basis_curve2d(&t.basis_curve());
    }
    Ok(c.clone())
}

/// `FUN_tool_dirC(par, C)`: the unit tangent of the curve at `par`
/// (Standard_ConstructionError for a null derivative); `fun_tool_dir_c(0.5, &c)?`.
pub fn fun_tool_dir_c(par: f64, c: &Handle<dyn GeomCurve>) -> Result<Direction3, Failure> {
    let curve = c.get().ok_or_else(|| null_geometry("FUN_tool_dirC"))?;
    let (_, tg_e) = curve.d1(par)?;
    Direction3::from_xyz(*tg_e.xyz())
}

/// `FUN_tool_onapex(p2d, S)`: whether the UV point is the apex of a cone (within
/// Precision::Confusion()) or a pole of a sphere (V = +-PI/2 within 1e-8);
/// `fun_tool_onapex(uv, &s)?`.
pub fn fun_tool_onapex(p2d: Point2, s: &Handle<dyn GeomSurface>) -> Result<bool, Failure> {
    let gs = GeomAdaptorSurface::with_surface(s)?;
    let (tol, toluv): (f64, f64) = (precision::confusion(), 1.0e-8);
    let mut isapex = false;
    let st = gs.get_type()?;
    if st == SurfaceType::Cone {
        let apex = gs.cone()?.apex();
        isapex = gs.value(p2d.x(), p2d.y())?.distance(&apex) < tol;
    }
    if st == SurfaceType::Sphere {
        let (pisur2, v) = (PI * 0.5, p2d.y());
        isapex = (v - pisur2).abs() < toluv || (v + pisur2).abs() < toluv;
    }
    Ok(isapex)
}

/// The normal of a degenerated point of a cone (its apex or a null U derivative) or a
/// sphere pole; `None` elsewhere.
fn kpart_normal(
    p2d: Point2,
    s: &Handle<dyn GeomSurface>,
    du: f64,
    tol: f64,
) -> Result<Option<Vector3>, Failure> {
    let gs = GeomAdaptorSurface::with_surface(s)?;
    let toluv: f64 = 1.0e-8;
    let st = gs.get_type()?;
    if st == SurfaceType::Cone {
        let apex = p2d.x().abs() < toluv && p2d.y().abs() < toluv;
        if apex {
            return Ok(Some(
                Vector3::from(gs.cone()?.axis().direction()).reversed(),
            ));
        }
        if du < tol {
            let vf = gs.first_v_parameter()?;
            let y = if (p2d.y() - vf).abs() < toluv {
                p2d.y() + 1.
            } else {
                p2d.y() - 1.
            };
            let surface = s.get().ok_or_else(|| null_geometry("FUN_tool_ngS"))?;
            let d1 = surface.d1(p2d.x(), y)?;
            return Ok(Some(d1.d1u.crossed(d1.d1v)));
        }
    }
    if st == SurfaceType::Sphere {
        let pisur2 = PI * 0.5;
        let (u, v) = (p2d.x(), p2d.y());
        if (v - pisur2).abs() < toluv || (v + pisur2).abs() < toluv {
            let center = gs.sphere()?.location();
            return Ok(Some(Vector3::between(center, gs.value(u, v)?)));
        }
    }
    Ok(None)
}

/// `FUN_tool_ngS(p2d, S)`: the unit normal `d1u ^ d1v` of the surface; at a
/// degenerated point, the normal of the cone apex or of a nearby iso, or the sphere
/// radius, else +Z (NYI natively); `fun_tool_ng_s(uv, &s)?`.
pub fn fun_tool_ng_s(p2d: Point2, s: &Handle<dyn GeomSurface>) -> Result<Direction3, Failure> {
    let surface = s.get().ok_or_else(|| null_geometry("FUN_tool_ngS"))?;
    let d1 = surface.d1(p2d.x(), p2d.y())?;
    let (du, dv) = (d1.d1u.magnitude(), d1.d1v.magnitude());
    let tol: f64 = precision::confusion();
    if du < tol || dv < tol {
        return match kpart_normal(p2d, s, du, tol)? {
            Some(ng) => Direction3::from_xyz(*ng.xyz()),
            None => Ok(Direction3::Z),
        };
    }
    let udir = Direction3::from_xyz(*d1.d1u.xyz())?;
    let vdir = Direction3::from_xyz(*d1.d1v.xyz())?;
    udir.crossed(vdir)
}

/// `FUN_tool_line(C3d)`: whether the basis curve is a line; `fun_tool_line(&c)?`.
pub fn fun_tool_line(c3d: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
    let c = TopOpeBRepToolShapeTool::basis_curve(c3d)?;
    Ok(GeomAdaptorCurve::with_curve(&c)?.get_type()? == CurveType::Line)
}

/// `FUN_tool_line(C2d)`: whether the basis 2d curve is a line; `fun_tool_line2d(&pc)?`.
pub fn fun_tool_line2d(pc: &Handle<dyn Geom2dCurve>) -> Result<bool, Failure> {
    let pcb = basis_curve2d(pc)?;
    Ok(Geom2dAdaptorCurve::with_curve(&pcb)?.get_type()? == CurveType::Line)
}

/// `FUN_quadCT(CT)`: a line or a conic; `fun_quad_ct(CurveType::Circle)`.
pub fn fun_quad_ct(ct: CurveType) -> bool {
    matches!(
        ct,
        CurveType::Line
            | CurveType::Circle
            | CurveType::Ellipse
            | CurveType::Hyperbola
            | CurveType::Parabola
    )
}

/// `FUN_tool_quad(C3d)`: whether the basis curve is a line or a conic;
/// `fun_tool_quad(&c)?`.
pub fn fun_tool_quad(c3d: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
    let c = TopOpeBRepToolShapeTool::basis_curve(c3d)?;
    Ok(fun_quad_ct(GeomAdaptorCurve::with_curve(&c)?.get_type()?))
}

/// `FUN_tool_quad(pc)`: whether the basis 2d curve is a line or a conic;
/// `fun_tool_quad2d(&pc)?`.
pub fn fun_tool_quad2d(pc: &Handle<dyn Geom2dCurve>) -> Result<bool, Failure> {
    let pcb = basis_curve2d(pc)?;
    Ok(fun_quad_ct(
        Geom2dAdaptorCurve::with_curve(&pcb)?.get_type()?,
    ))
}

/// `FUN_tool_quad(S)`: a plane, cylinder, cone, sphere or torus, false for a null
/// surface; `fun_tool_quad_surface(&s)?`.
pub fn fun_tool_quad_surface(s: &Handle<dyn GeomSurface>) -> Result<bool, Failure> {
    if s.is_null() {
        return Ok(false);
    }
    let typ = GeomAdaptorSurface::with_surface(s)?.get_type()?;
    Ok(matches!(
        typ,
        SurfaceType::Plane
            | SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
    ))
}

/// The closure of a surface in U and V: `(uclosed, uperiod, vclosed, vperiod)`.
pub type SurfaceClosure = (bool, f64, bool, f64);

/// `FUN_tool_closed(S, uclosed, uperiod, vclosed, vperiod)`: the closed and periodic
/// directions with their periods (0 otherwise); `None` for a null surface (the closure
/// flags untouched natively); `fun_tool_closed(&s)?`.
pub fn fun_tool_closed(s: &Handle<dyn GeomSurface>) -> Result<Option<SurfaceClosure>, Failure> {
    let Some(surface) = s.get() else {
        return Ok(None);
    };
    // xpu261098 (BUC60382): closed means closed and periodic.
    let uclosed = surface.is_u_closed()? && surface.is_u_periodic();
    let uperiod = if uclosed { surface.u_period()? } else { 0. };
    let vclosed = surface.is_v_closed()? && surface.is_v_periodic();
    let vperiod = if vclosed { surface.v_period()? } else { 0. };
    Ok(Some((uclosed, uperiod, vclosed, vperiod)))
}

/// `FUN_tool_UpdateBnd2d(B2d, newB2d)`: the box replaced; `fun_tool_update_bnd2d(&mut b, &nb)`.
pub fn fun_tool_update_bnd2d(b2d: &mut BoundingBox2, new_b2d: &BoundingBox2) {
    *b2d = new_b2d.clone();
}

/// `FUN_tool_nCinsideS(tgC, ngS)` (defined in TopOpeBRepTool_matter.cxx): the normal
/// to the curve tangent to the surface, `ngS ^ tgC` (oriented inside the surface);
/// Standard_ConstructionError for parallel directions; `fun_tool_nc_inside_s(tg, ng)?`.
pub fn fun_tool_nc_inside_s(tg_c: Direction3, ng_s: Direction3) -> Result<Direction3, Failure> {
    ng_s.crossed(tg_c)
}

/// `FUN_tool_nC2dINSIDES(tgC2d)` (defined in TopOpeBRepTool_matter.cxx): the 2d normal
/// on the left of the tangent, `(Z ^ (tgC2d, 0))` projected on the plane;
/// `fun_tool_nc2d_inside_s(tg2d)?`.
pub fn fun_tool_nc2d_inside_s(tg_c2d: Direction2) -> Result<Direction2, Failure> {
    let z = Direction3::Z;
    let x = Direction3::new(tg_c2d.x(), tg_c2d.y(), 0.)?;
    let y = z.crossed(x)?;
    Direction2::new(y.x(), y.y())
}
