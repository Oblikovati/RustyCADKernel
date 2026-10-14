//! TopOpeBRepTool_TOPOLOGY.cxx: points and parameters on edges and faces, normals and
//! tangents at edge parameters, curve and surface types and the nearest iso lines.
use super::edges::fun_tool_orient_e_in_f_forward;
use crate::top_ope_brep_tool::geometry::{fun_quad_ct, fun_tool_ng_s, fun_tool_quad_surface};
use crate::top_ope_brep_tool::project::{
    fun_tool_getindex, fun_tool_proj_pone_tol, fun_tool_proj_ponf,
};
use crate::top_ope_brep_tool::shape_tool::TopOpeBRepToolShapeTool;
use crate::top_ope_brep_tool::tool_iso::uviso_edge;
use crate::top_ope_brep_tool::tool_tangent::tggeom_e;
use crate::top_ope_brep_tool::two_d::fc2d_curve_on_surface;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPC};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3, Vector3};

/// The native default tolerance of `Extrema_ExtPC(P, C)` and `Extrema_ExtPC(P, C, Uinf, Usup)`.
const EXT_PC_TOLF: f64 = 1.0e-10;

/// `FUN_tool_PinC(P, BAC, pmin, pmax, tol)`: whether the nearest extremum of the point
/// on the range is within `tol`; `fun_tool_pin_c_range(p, &bac, 0., 1., 1e-7)?`.
pub fn fun_tool_pin_c_range(
    p: Point3,
    bac: &BRepAdaptorCurve,
    pmin: f64,
    pmax: f64,
    tol: f64,
) -> Result<bool, Failure> {
    let ponc = ExtPC::with_range(p, bac, pmin, pmax, EXT_PC_TOLF)?;
    if !ponc.is_done() || ponc.nb_ext()? == 0 {
        return Ok(false);
    }
    let i = fun_tool_getindex(&ponc)?;
    Ok(ponc.square_distance(i)? <= tol * tol)
}

/// `FUN_tool_PinC(P, BAC, tol)`: on the whole adaptor range;
/// `fun_tool_pin_c(p, &bac, 1e-7)?`.
pub fn fun_tool_pin_c(p: Point3, bac: &BRepAdaptorCurve, tol: f64) -> Result<bool, Failure> {
    let (pmin, pmax) = (bac.first_parameter()?, bac.last_parameter()?);
    fun_tool_pin_c_range(p, bac, pmin, pmax, tol)
}

/// `FUN_tool_value(par, E, P)`: the point of the edge at a parameter of its range;
/// `None` outside; `fun_tool_value_on_edge(0.5, &e)?`.
pub fn fun_tool_value_on_edge(par: f64, e: &Edge) -> Result<Option<Point3>, Failure> {
    let bac = BRepAdaptorCurve::with_edge(e)?;
    let (f, l) = (bac.first_parameter()?, bac.last_parameter()?);
    if !(f <= par && par <= l) {
        return Ok(None);
    }
    Ok(Some(bac.value(par)?))
}

/// `FUN_tool_value(UV, F, P)`: the point of the face at UV;
/// `fun_tool_value_on_face(uv, &f)?`.
pub fn fun_tool_value_on_face(uv: Point2, f: &Face) -> Result<Point3, Failure> {
    BRepAdaptorSurface::with_face(f, true)?.value(uv.x(), uv.y())
}

/// `FUN_tool_staPinE(P, E, tol)`: IN when the nearest projection of the point on the
/// edge is within `tol`, OUT otherwise, UNKNOWN when the projection fails;
/// `fun_tool_sta_pin_e_tol(p, &e, 1e-5)?`.
pub fn fun_tool_sta_pin_e_tol(p: Point3, e: &Edge, tol: f64) -> Result<State, Failure> {
    let bac = BRepAdaptorCurve::with_edge(e)?;
    let (umin, umax) = (bac.first_parameter()?, bac.last_parameter()?);
    let proj = ExtPC::with_range(p, &bac, umin, umax, EXT_PC_TOLF)?;
    if !(proj.is_done() && proj.nb_ext()? > 0) {
        return Ok(State::Unknown);
    }
    let i = fun_tool_getindex(&proj)?;
    let pnear = *proj.point(i)?.value();
    Ok(if pnear.distance(&p) < tol {
        State::In
    } else {
        State::Out
    })
}

/// `FUN_tool_staPinE(P, E)`: with 100 times the edge tolerance;
/// `fun_tool_sta_pin_e(p, &e)?`.
pub fn fun_tool_sta_pin_e(p: Point3, e: &Edge) -> Result<State, Failure> {
    fun_tool_sta_pin_e_tol(p, e, BRepTool::edge_tolerance(e)? * 1.0e2)
}

/// `FUN_tool_nggeomF(p2d, F)`: the geometric normal of the surface of the face at UV
/// (FUN_tool_ngS); `fun_tool_nggeom_f(uv, &f)?`.
pub fn fun_tool_nggeom_f(p2d: Point2, f: &Face) -> Result<Vector3, Failure> {
    let s = BRepTool::surface(f)?;
    Ok(Vector3::from(fun_tool_ng_s(p2d, &s)?))
}

/// The UV point of an edge parameter: on the p-curve of an edge of the face, else the
/// projection of the 3d point (or the p-curve made when the projection fails) checked
/// within `tol`.
fn uv_on_face(paron_e: f64, e: &Edge, f: &Face, tol: f64) -> Result<Option<Point2>, Failure> {
    if fun_tool_orient_e_in_f_forward(e, f)?.0 {
        let (c2d, _, _, _) = fc2d_curve_on_surface(e, f, false)?;
        if let Some(c) = c2d.get() {
            return Ok(Some(c.value(paron_e)?));
        }
    }
    let p3d = BRepAdaptorCurve::with_edge(e)?.value(paron_e)?;
    let (mut p2d, mut d) = (Point2::default(), 0.);
    let mut ok = fun_tool_proj_ponf(p3d, f, &mut p2d, &mut d, ExtFlag::MinMax, ExtAlgo::Grad)?;
    if !ok {
        // NIZHNY-MZV 01/12/1999: if !ok try to compute new pcurve
        let (c2d, _, _, _) = fc2d_curve_on_surface(e, f, false)?;
        if let Some(c) = c2d.get() {
            (ok, p2d) = (true, c.value(paron_e)?);
        }
    }
    if !ok || fun_tool_value_on_face(p2d, f)?.distance(&p3d) > tol {
        return Ok(None);
    }
    Ok(Some(p2d))
}

/// `FUN_tool_nggeomF(paronE, E, F, nggeomF, tol)`: the geometric normal of the face at
/// the point of the edge parameter; `None` when the point is not on the face within
/// `tol`; `fun_tool_nggeom_f_on_edge_tol(0.5, &e, &f, 1e-5)?`.
pub fn fun_tool_nggeom_f_on_edge_tol(
    paron_e: f64,
    e: &Edge,
    f: &Face,
    tol: f64,
) -> Result<Option<Vector3>, Failure> {
    match uv_on_face(paron_e, e, f, tol)? {
        Some(p2d) => Ok(Some(fun_tool_nggeom_f(p2d, f)?)),
        None => Ok(None),
    }
}

/// `FUN_tool_nggeomF(paronE, E, F, nggeomF)`: with 100 times the face tolerance;
/// `fun_tool_nggeom_f_on_edge(0.5, &e, &f)?`.
pub fn fun_tool_nggeom_f_on_edge(
    paron_e: f64,
    e: &Edge,
    f: &Face,
) -> Result<Option<Vector3>, Failure> {
    let tol3d = BRepTool::face_tolerance(f)? * 1.0e2;
    fun_tool_nggeom_f_on_edge_tol(paron_e, e, f, tol3d)
}

/// `FUN_tool_EtgF(paronE, E, p2d, F, tola)`: whether the edge tangent is normal to the
/// face normal at UV within `tola`; `fun_tool_etg_f(0.5, &e, uv, &f, 1e-9)?`.
pub fn fun_tool_etg_f(
    paron_e: f64,
    e: &Edge,
    p2d: Point2,
    f: &Face,
    tola: f64,
) -> Result<bool, Failure> {
    let Some(tg_e) = tggeom_e(paron_e, e)? else {
        return Ok(false);
    };
    let ng_f = fun_tool_nggeom_f(p2d, f)?;
    Ok(tg_e.dot(ng_f).abs() < tola)
}

/// `FUN_tool_EtgOOE(paronE, E, paronOOE, OOE, tola)`: whether the edge tangents are
/// parallel or opposite within `tola`; `fun_tool_etg_ooe(0.5, &e, 0.5, &ooe, 1e-9)?`.
pub fn fun_tool_etg_ooe(
    paron_e: f64,
    e: &Edge,
    paron_ooe: f64,
    ooe: &Edge,
    tola: f64,
) -> Result<bool, Failure> {
    let Some(tg_ooe) = tggeom_e(paron_ooe, ooe)? else {
        return Ok(false);
    };
    let Some(tg_e) = tggeom_e(paron_e, e)? else {
        return Ok(false);
    };
    Ok((1. - tg_ooe.dot(tg_e).abs()).abs() < tola)
}

/// The iso value of an edge of the face with the iso direction asked; `None` when the
/// edge is not iso or not of that direction.
fn iso_value(e: &Edge, f: &Face, iso_u: bool) -> Result<Option<f64>, Failure> {
    let (mut d2d, mut o2d) = (Direction2::X, Point2::default());
    let (isouv, isou, isov) = uviso_edge(e, f, &mut d2d, &mut o2d)?;
    if !isouv || !((iso_u && isou) || (!iso_u && isov)) {
        return Ok(None);
    }
    Ok(Some(if isou { o2d.x() } else { o2d.y() }))
}

/// `FUN_nearestISO(F, xpar, isoU, xinf, xsup)`: `xinf` raised and `xsup` lowered to the
/// nearest iso values of the edges of the face around `xpar` (true at once when `xpar`
/// is on a bound within 1e-6); false at the first edge not iso in that direction;
/// `fun_nearest_iso(&f, 0.5, true, &mut xinf, &mut xsup)?`.
pub fn fun_nearest_iso(
    f: &Face,
    xpar: f64,
    iso_u: bool,
    xinf: &mut f64,
    xsup: &mut f64,
) -> Result<bool, Failure> {
    let tol2d: f64 = 1.0e-6;
    if (xpar - *xinf).abs() < tol2d || (xpar - *xsup).abs() < tol2d {
        return Ok(true);
    }
    for es in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let Some(xvalue) = iso_value(&topods::edge(&es)?, f, iso_u)? else {
            return Ok(false);
        };
        if xvalue < xpar && *xinf < xvalue {
            *xinf = xvalue;
        }
        if xpar < xvalue && xvalue < *xsup {
            *xsup = xvalue;
        }
    }
    Ok(true)
}

/// `FUN_tool_EitangenttoFe(ngFe, Ei, parOnEi)`: whether the edge tangent is normal to
/// `ngFe` within the parametric confusion; `fun_tool_eitangentto_fe(ng, &e, 0.5)?`.
pub fn fun_tool_eitangentto_fe(
    ng_fe: Direction3,
    ei: &Edge,
    par_on_ei: f64,
) -> Result<bool, Failure> {
    let Some(tg_ei) = tggeom_e(par_on_ei, ei)? else {
        return Ok(false);
    };
    let tol: f64 = precision::parametric(precision::confusion());
    Ok(Vector3::from(ng_fe).dot(tg_ei).abs() <= tol)
}

/// `FUN_tool_typ(E)`: the curve type of the edge adaptor; `fun_tool_typ(&e)?`.
pub fn fun_tool_typ(e: &Edge) -> Result<CurveType, Failure> {
    BRepAdaptorCurve::with_edge(e)?.get_type()
}

/// `FUN_tool_line(E)`: whether the edge is a line; `fun_tool_line_edge(&e)?`.
pub fn fun_tool_line_edge(e: &Edge) -> Result<bool, Failure> {
    Ok(fun_tool_typ(e)? == CurveType::Line)
}

/// The type of the basis surface of a face.
fn basis_surface_type(f: &Shape) -> Result<SurfaceType, Failure> {
    let s = TopOpeBRepToolShapeTool::basis_surface_of_face(&topods::face(f)?)?;
    GeomAdaptorSurface::with_surface(&s)?.get_type()
}

/// `FUN_tool_plane(F)`: whether the basis surface of the face is a plane;
/// `fun_tool_plane_face(&f)?`.
pub fn fun_tool_plane_face(f: &Shape) -> Result<bool, Failure> {
    Ok(basis_surface_type(f)? == SurfaceType::Plane)
}

/// `FUN_tool_cylinder(F)`: whether the basis surface of the face is a cylinder;
/// `fun_tool_cylinder_face(&f)?`.
pub fn fun_tool_cylinder_face(f: &Shape) -> Result<bool, Failure> {
    Ok(basis_surface_type(f)? == SurfaceType::Cylinder)
}

/// `FUN_tool_parVonE(v, E, par)`: the parameter of the vertex on the edge, of a vertex
/// of the edge at the same point (Precision::Confusion()), or of its projection;
/// `None` when the projection fails; `fun_tool_par_v_on_e(&v, &e)?`.
pub fn fun_tool_par_v_on_e(v: &Vertex, e: &Edge) -> Result<Option<f64>, Failure> {
    let vertices: Vec<Shape> = Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)?.collect();
    if let Some(found) = vertices.iter().find(|c| c.is_same(v)) {
        return Ok(Some(BRepTool::parameter(&topods::vertex(found)?, e)?));
    }
    let pt = BRepTool::pnt(v)?;
    for c in &vertices {
        let vex = topods::vertex(c)?;
        if BRepTool::pnt(&vex)?.is_equal(&pt, precision::confusion()) {
            return Ok(Some(BRepTool::parameter(&vex, e)?));
        }
    }
    let bac = BRepAdaptorCurve::with_edge(e)?;
    let (umin, umax) = (bac.first_parameter()?, bac.last_parameter()?);
    let pro = ExtPC::with_range(pt, &bac, umin, umax, EXT_PC_TOLF)?;
    if !(pro.is_done() && pro.nb_ext()? > 0) {
        return Ok(None);
    }
    let i = fun_tool_getindex(&pro)?;
    Ok(Some(pro.point(i)?.parameter()))
}

/// `FUN_tool_parE(E0, par0, E, par, tol)`: the parameter on `E` of the point of `E0`
/// at `par0` (written by the projection even when farther than `tol`); whether it is
/// within `tol`; `fun_tool_par_e_tol(&e0, 0.5, &e, &mut par, 1e-5)?`.
pub fn fun_tool_par_e_tol(
    e0: &Edge,
    par0: f64,
    e: &Edge,
    par: &mut f64,
    tol: f64,
) -> Result<bool, Failure> {
    let Some(p) = fun_tool_value_on_edge(par0, e0)? else {
        return Ok(false);
    };
    let mut dist: f64 = 0.;
    let tole = BRepTool::edge_tolerance(e)?;
    if !fun_tool_proj_pone_tol(p, tole, e, par, &mut dist)? {
        return Ok(false);
    }
    Ok(dist < tol)
}

/// `FUN_tool_parE(E0, par0, E, par)`: with 100 times the tolerance of `E`;
/// `fun_tool_par_e(&e0, 0.5, &e, &mut par)?`.
pub fn fun_tool_par_e(e0: &Edge, par0: f64, e: &Edge, par: &mut f64) -> Result<bool, Failure> {
    let tol3d = BRepTool::edge_tolerance(e)? * 1.0e2;
    fun_tool_par_e_tol(e0, par0, e, par, tol3d)
}

/// `FUN_tool_parF(E, par, F, UV, tol3d)`: the UV of the projection on the face of the
/// point of the edge (written even when farther than `tol3d`); whether it is within
/// `tol3d`; `fun_tool_par_f_tol(&e, 0.5, &f, &mut uv, 1e-5)?`.
pub fn fun_tool_par_f_tol(
    e: &Edge,
    par: f64,
    f: &Face,
    uv: &mut Point2,
    tol3d: f64,
) -> Result<bool, Failure> {
    let Some(p) = fun_tool_value_on_edge(par, e)? else {
        return Ok(false);
    };
    let mut dist: f64 = 0.;
    if !fun_tool_proj_ponf(p, f, uv, &mut dist, ExtFlag::MinMax, ExtAlgo::Grad)? {
        return Ok(false);
    }
    Ok(dist < tol3d)
}

/// `FUN_tool_parF(E, par, F, UV)`: with 100 times the face tolerance;
/// `fun_tool_par_f(&e, 0.5, &f, &mut uv)?`.
pub fn fun_tool_par_f(e: &Edge, par: f64, f: &Face, uv: &mut Point2) -> Result<bool, Failure> {
    let tol3d = BRepTool::face_tolerance(f)? * 1.0e2;
    fun_tool_par_f_tol(e, par, f, uv, tol3d)
}

/// `FUN_tool_paronEF(E, par, F, UV, tol3d)`: the UV of an edge parameter on the
/// p-curve of the edge on the face (false outside its range), else by FUN_tool_parF;
/// `fun_tool_paron_ef_tol(&e, 0.5, &f, &mut uv, 1e-5)?`.
pub fn fun_tool_paron_ef_tol(
    e: &Edge,
    par: f64,
    f: &Face,
    uv: &mut Point2,
    tol3d: f64,
) -> Result<bool, Failure> {
    let (pc, first, last, _) = fc2d_curve_on_surface(e, f, false)?;
    let Some(c) = pc.get() else {
        return fun_tool_par_f_tol(e, par, f, uv, tol3d);
    };
    if !(first <= par && par <= last) {
        return Ok(false);
    }
    *uv = c.value(par)?;
    Ok(true)
}

/// `FUN_tool_paronEF(E, par, F, UV)`: with 100 times the face tolerance;
/// `fun_tool_paron_ef(&e, 0.5, &f, &mut uv)?`.
pub fn fun_tool_paron_ef(e: &Edge, par: f64, f: &Face, uv: &mut Point2) -> Result<bool, Failure> {
    let tol3d = BRepTool::face_tolerance(f)? * 1.0e2;
    fun_tool_paron_ef_tol(e, par, f, uv, tol3d)
}

/// `FUN_tool_dirC(par, BAC)`: the unit tangent of the adaptor (Standard_ConstructionError
/// for a null derivative); `fun_tool_dir_c_on_curve(0.5, &bac)?`.
pub fn fun_tool_dir_c_on_curve(par: f64, bac: &BRepAdaptorCurve) -> Result<Direction3, Failure> {
    let (_, tg_e) = bac.d1(par)?;
    Direction3::from_xyz(*tg_e.xyz())
}

/// `FUN_tool_tggeomE(paronE, E)`: the unit tangent of the edge, null for a degenerated
/// edge; `fun_tool_tggeom_e_vec(0.5, &e)?`.
pub fn fun_tool_tggeom_e_vec(paron_e: f64, e: &Edge) -> Result<Vector3, Failure> {
    if BRepTool::degenerated(e)? {
        return Ok(Vector3::new(0., 0., 0.));
    }
    let bac = BRepAdaptorCurve::with_edge(e)?;
    Ok(Vector3::from(fun_tool_dir_c_on_curve(paron_e, &bac)?))
}

/// `FUN_tool_line(BAC)`: whether the adaptor is a line; `fun_tool_line_bac(&bac)?`.
pub fn fun_tool_line_bac(bac: &BRepAdaptorCurve) -> Result<bool, Failure> {
    Ok(bac.get_type()? == CurveType::Line)
}

/// `FUN_tool_quad(E)`: whether the edge curve is a line or a conic;
/// `fun_tool_quad_edge(&e)?`.
pub fn fun_tool_quad_edge(e: &Edge) -> Result<bool, Failure> {
    Ok(fun_quad_ct(fun_tool_typ(e)?))
}

/// `FUN_tool_quad(BAC)`: whether the adaptor is a line or a conic;
/// `fun_tool_quad_bac(&bac)?`.
pub fn fun_tool_quad_bac(bac: &BRepAdaptorCurve) -> Result<bool, Failure> {
    Ok(matches!(
        bac.get_type()?,
        CurveType::Line
            | CurveType::Circle
            | CurveType::Ellipse
            | CurveType::Hyperbola
            | CurveType::Parabola
    ))
}

/// `FUN_tool_quad(F)`: whether the basis surface of the face is a quadric;
/// `fun_tool_quad_face(&f)?`.
pub fn fun_tool_quad_face(f: &Face) -> Result<bool, Failure> {
    fun_tool_quad_surface(&TopOpeBRepToolShapeTool::basis_surface_of_face(f)?)
}

/// `FUN_tool_findparinBAC(BAC, par)`: the parameter at 0.34567237 of the adaptor range;
/// `fun_tool_find_par_in_bac(&bac)?`.
pub fn fun_tool_find_par_in_bac(bac: &BRepAdaptorCurve) -> Result<f64, Failure> {
    let (f_e, l_e) = (bac.first_parameter()?, bac.last_parameter()?);
    let t: f64 = 0.34567237;
    Ok((1. - t) * f_e + t * l_e)
}

/// `FUN_tool_findPinBAC(BAC, P, par)`: that parameter and its point;
/// `let (p, par) = fun_tool_find_pin_bac(&bac)?;`.
pub fn fun_tool_find_pin_bac(bac: &BRepAdaptorCurve) -> Result<(Point3, f64), Failure> {
    let par = fun_tool_find_par_in_bac(bac)?;
    Ok((bac.value(par)?, par))
}

/// `FUN_tool_findparinE(E, par)`: on the adaptor of the edge;
/// `fun_tool_find_par_in_e(&e)?`.
pub fn fun_tool_find_par_in_e(e: &Shape) -> Result<f64, Failure> {
    fun_tool_find_par_in_bac(&BRepAdaptorCurve::with_edge(&topods::edge(e)?)?)
}

/// `FUN_tool_findPinE(E, P, par)`: on the adaptor of the edge;
/// `let (p, par) = fun_tool_find_pin_e(&e)?;`.
pub fn fun_tool_find_pin_e(e: &Shape) -> Result<(Point3, f64), Failure> {
    fun_tool_find_pin_bac(&BRepAdaptorCurve::with_edge(&topods::edge(e)?)?)
}
