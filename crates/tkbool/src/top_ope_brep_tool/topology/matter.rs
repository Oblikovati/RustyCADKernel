// Rust translation of FUN_tool_getgeomxx, FUN_tool_getxx and FUN_tool_getdxx (declared in
// TopOpeBRepTool_TOPOLOGY.hxx, defined in ModelingAlgorithms/TKBool/TopOpeBRepTool/
// TopOpeBRepTool_matter.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vectors normal to an edge of a face, tangent to the face and oriented inside it.
use super::bounds::fun_tool_isobounds;
use super::edges::fun_tool_orient_e_in_f_forward;
use super::params::{fun_nearest_iso, fun_tool_nggeom_f, fun_tool_par_f_tol, fun_tool_paron_ef};
use crate::top_ope_brep_tool::geometry::fun_tool_nc_inside_s;
use crate::top_ope_brep_tool::tool_iso::uviso_edge;
use crate::top_ope_brep_tool::tool_tangent::tggeom_e;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{self, Direction2, Direction3, Point2, Vector2, Vector3};

/// `FUN_tool_getgeomxx(Fi, Ei, parEi, ngFi)`: `ngFi ^ tg(Ei)` (FUN_tool_nCinsideS),
/// null when the edge has no tangent there; `fun_tool_getgeomxx_ng(&f, &e, 0.5, ng)?`.
pub fn fun_tool_getgeomxx_ng(
    _fi: &Face,
    ei: &Edge,
    par_ei: f64,
    ng_fi: Direction3,
) -> Result<Vector3, Failure> {
    let Some(tg_ei) = tggeom_e(par_ei, ei)? else {
        return Ok(Vector3::new(0., 0., 0.));
    };
    let xx = fun_tool_nc_inside_s(Direction3::from_xyz(*tg_ei.xyz())?, ng_fi)?;
    Ok(Vector3::from(xx))
}

/// `FUN_tool_getgeomxx(Fi, Ei, parOnEi)`: with the face normal at the point of the
/// edge parameter; (1, 0, 0) when it is not on the face;
/// `fun_tool_getgeomxx(&f, &e, 0.5)?`.
pub fn fun_tool_getgeomxx(fi: &Face, ei: &Edge, par_on_ei: f64) -> Result<Vector3, Failure> {
    let mut uvi = Point2::default();
    if !fun_tool_paron_ef(ei, par_on_ei, fi, &mut uvi)? {
        return Ok(Vector3::new(1., 0., 0.));
    }
    let ng_fi = Direction3::from_xyz(*fun_tool_nggeom_f(uvi, fi)?.xyz())?;
    fun_tool_getgeomxx_ng(fi, ei, par_on_ei, ng_fi)
}

/// `FUN_tool_getxx(Fi, Ei, parEi, ngFi, XX)`: the geometric vector inside the face,
/// reversed for an edge REVERSED in the face oriented FORWARD; `None` for a null vector
/// or an edge not of the face; `fun_tool_getxx_ng(&f, &e, 0.5, ng)?`.
pub fn fun_tool_getxx_ng(
    fi: &Face,
    ei: &Edge,
    par_ei: f64,
    ng_fi: Direction3,
) -> Result<Option<Direction3>, Failure> {
    let xx = fun_tool_getgeomxx_ng(fi, ei, par_ei, ng_fi)?;
    if xx.magnitude() < gp::resolution() {
        return Ok(None);
    }
    let mut dir = Direction3::from_xyz(*xx.xyz())?;
    let (ok, ori_e_in_f) = fun_tool_orient_e_in_f_forward(ei, fi)?;
    if !ok {
        return Ok(None);
    }
    if ori_e_in_f == Orientation::Reversed {
        dir.reverse();
    }
    Ok(Some(dir))
}

/// `FUN_tool_getxx(Fi, Ei, parEi, XX)`: with the face normal at the projection of the
/// edge point (within 100 times the face tolerance); `fun_tool_getxx(&f, &e, 0.5)?`.
pub fn fun_tool_getxx(fi: &Face, ei: &Edge, par_ei: f64) -> Result<Option<Direction3>, Failure> {
    let tol_fi = BRepTool::face_tolerance(fi)? * 1.0e2;
    let mut uv = Point2::default();
    if !fun_tool_par_f_tol(ei, par_ei, fi, &mut uv, tol_fi)? {
        return Ok(None);
    }
    let ng = Direction3::from_xyz(*fun_tool_nggeom_f(uv, fi)?.xyz())?;
    fun_tool_getxx_ng(fi, ei, par_ei, ng)
}

/// The iso flags, direction and origin of an iso edge of a face.
struct IsoEdge {
    iso_u: bool,
    iso_v: bool,
    d2d: Direction2,
    o2d: Point2,
}

/// The iso edge of the face, `None` when it is not iso.
fn iso_edge(e: &Edge, f: &Face) -> Result<Option<IsoEdge>, Failure> {
    let (mut d2d, mut o2d) = (Direction2::X, Point2::default());
    let (iso, iso_u, iso_v) = uviso_edge(e, f, &mut d2d, &mut o2d)?;
    Ok(iso.then_some(IsoEdge {
        iso_u,
        iso_v,
        d2d,
        o2d,
    }))
}

/// `FUN_tool_getdxx(F, E, parE, dxx)`: for a boundary iso edge of a face of iso edges,
/// the UV step from the edge to the nearest iso of the face on the side of the matter;
/// `None` otherwise; `fun_tool_getdxx(&f, &e, 0.5)?`.
pub fn fun_tool_getdxx(f: &Face, e: &Edge, _par_e: f64) -> Result<Option<Vector2>, Failure> {
    let (ok, o_e_in_ff) = fun_tool_orient_e_in_f_forward(e, f)?;
    if !ok || matches!(o_e_in_ff, Orientation::Internal | Orientation::External) {
        return Ok(None);
    }
    let Some(iso) = iso_edge(e, f)? else {
        return Ok(None);
    };
    let mut b = (0., 0., 0., 0.);
    if !fun_tool_isobounds(f, &mut b)? {
        return Ok(None);
    }
    let (u1, u2, v1, v2) = b;
    let xpar = if iso.iso_u { iso.o2d.x() } else { iso.o2d.y() };
    let mut xinf = if iso.iso_u { u1 } else { v1 };
    let mut xsup = if iso.iso_u { u2 } else { v2 };
    if !fun_nearest_iso(f, xpar, iso.iso_u, &mut xinf, &mut xsup)? {
        return Ok(None);
    }
    let ypar = if iso.iso_u { iso.d2d.y() } else { iso.d2d.x() };
    let mut matter_after_xpar = false;
    if iso.iso_u {
        matter_after_xpar = ypar < 0.;
    }
    if iso.iso_v {
        matter_after_xpar = ypar > 0.;
    }
    if o_e_in_ff == Orientation::Reversed {
        matter_after_xpar = !matter_after_xpar;
    }
    let dx = if matter_after_xpar {
        xsup - xpar
    } else {
        xinf - xpar
    };
    let mut dxx = Vector2::new(0., 0.);
    if iso.iso_u {
        dxx = Vector2::new(dx, 0.);
    }
    if iso.iso_v {
        dxx = Vector2::new(0., dx);
    }
    Ok(Some(dxx))
}
