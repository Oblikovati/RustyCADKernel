//! TopOpeBRepTool_TOPOLOGY.cxx: the UV tolerances, bounds and closures of faces, the
//! maximal tolerances of sub-shapes and the comparison of bounding boxes.
use crate::top_ope_brep_tool::geometry::{fun_tool_closed, fun_tool_update_bnd2d};
use crate::top_ope_brep_tool::shape_tool::TopOpeBRepToolShapeTool;
use crate::top_ope_brep_tool::tool_iso::uviso;
use crate::top_ope_brep_tool::two_d::{fc2d_curve_on_surface, fc2d_has_curve_on_surface};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction2, Point2};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// `FUN_tool_tolUV(F, tolu, tolv)`: the U and V resolutions of the face tolerance;
/// `let (tolu, tolv) = fun_tool_tol_uv(&face)?;`.
pub fn fun_tool_tol_uv(f: &Face) -> Result<(f64, f64), Failure> {
    let tol_f = BRepTool::face_tolerance(f)?;
    let bs = BRepAdaptorSurface::with_face(f, true)?;
    Ok((bs.u_resolution(tol_f)?, bs.v_resolution(tol_f)?))
}

/// `FUN_tool_direct(F, direct)`: whether the surface of the face is elementary; its
/// frame directness is written only then; `fun_tool_direct(&face, &mut direct)?`.
pub fn fun_tool_direct(f: &Face, direct: &mut bool) -> Result<bool, Failure> {
    let bs = BRepAdaptorSurface::with_face(f, true)?;
    let found = match bs.get_type()? {
        SurfaceType::Plane => Some(bs.plane()?.direct()?),
        SurfaceType::Cylinder => Some(bs.cylinder()?.direct()?),
        SurfaceType::Cone => Some(bs.cone()?.direct()?),
        SurfaceType::Sphere => Some(bs.sphere()?.direct()?),
        SurfaceType::Torus => Some(bs.torus()?.direct()?),
        _ => None,
    };
    if let Some(d) = found {
        *direct = d;
    }
    Ok(found.is_some())
}

/// `FUN_tool_geombounds(F, u1, u2, v1, v2)`: the natural bounds of the surface of the
/// face (without its location); `None` without surface; `fun_tool_geombounds(&face)?`.
pub fn fun_tool_geombounds(f: &Face) -> Result<Option<(f64, f64, f64, f64)>, Failure> {
    let s = BRepTool::surface(f)?;
    let Some(surface) = s.get() else {
        return Ok(None);
    };
    let b = surface.bounds();
    Ok(Some(b))
}

/// `FUN_tool_bounds(Sh, u1, u2, v1, v2)` of a face: the box of the UV representations
/// of its wires. Natively the box is read by `Get(u1, v1, u2, v1)`: `v1` ends as the
/// maximal V and `v2` is never written (kept here); Standard_ConstructionError for a
/// face without wire (a void box); `fun_tool_bounds_face(&face, &mut u1, &mut u2, &mut v1, &mut v2)?`.
pub fn fun_tool_bounds_face(
    sh: &Shape,
    u1: &mut f64,
    u2: &mut f64,
    v1: &mut f64,
    _v2: &mut f64,
) -> Result<bool, Failure> {
    let mut b2d = BoundingBox2::default();
    let f = topods::face(sh)?;
    for w in Explorer::new(&f, ShapeEnum::Wire, ShapeEnum::Shape)? {
        let mut new_b2d = BoundingBox2::default();
        fun_tool_mk_bnd2d(&w, &f, &mut new_b2d)?;
        b2d.add_box(&new_b2d);
    }
    let [xmin, ymin, xmax, ymax] = b2d.get()?;
    (*u1, *v1, *u2, *v1) = (xmin, ymin, xmax, ymax);
    Ok(true)
}

/// The UV box of the ends of an iso p-curve added to the bounds.
fn add_iso_ends(ends: [Point2; 2], b: &mut (f64, f64, f64, f64)) {
    for p in ends {
        b.0 = std_min(p.x(), b.0);
        b.1 = std_max(p.x(), b.1);
        b.2 = std_min(p.y(), b.2);
        b.3 = std_max(p.y(), b.3);
    }
}

/// `FUN_tool_isobounds(Sh, u1, u2, v1, v2)`: the UV box of the ends of the p-curves of
/// a face made of iso lines; false at the first edge without p-curve or not iso. The
/// bounds start at (1e7, -1e7) and keep the edges already seen;
/// `fun_tool_isobounds(&face, &mut b)?` with `b = (u1, u2, v1, v2)`.
pub fn fun_tool_isobounds(sh: &Shape, b: &mut (f64, f64, f64, f64)) -> Result<bool, Failure> {
    let f = topods::face(sh)?;
    *b = (1.0e7, -1.0e7, 1.0e7, -1.0e7);
    if BRepTool::surface(&f)?.is_null() {
        return Ok(false);
    }
    for e in Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let (pc, first, last, _) = fc2d_curve_on_surface(&topods::edge(&e)?, &f, false)?;
        let Some(curve) = pc.get() else {
            return Ok(false);
        };
        let (mut d2d, mut o2d) = (Direction2::X, Point2::default());
        if !uviso(&pc, &mut d2d, &mut o2d)?.0 {
            return Ok(false);
        }
        add_iso_ends([curve.value(first)?, curve.value(last)?], b);
    }
    Ok(true)
}

/// `FUN_tool_outbounds(Sh, u1, u2, v1, v2, outbounds)`: the bounds of the face, or the
/// natural bounds of its basis surface when they span more than a period of a closed
/// direction (1e-6); `fun_tool_outbounds(&face, &mut b, &mut out)?` with
/// `b = (u1, u2, v1, v2)`.
pub fn fun_tool_outbounds(
    sh: &Shape,
    b: &mut (f64, f64, f64, f64),
    outbounds: &mut bool,
) -> Result<bool, Failure> {
    let f = topods::face(sh)?;
    let s = TopOpeBRepToolShapeTool::basis_surface_of_face(&f)?;
    let Some(surface) = s.get() else {
        return Ok(false);
    };
    let nb = surface.bounds();
    *outbounds = false;
    fun_tool_bounds_face(sh, &mut b.0, &mut b.1, &mut b.2, &mut b.3)?;
    let (uclosed, uperiod, vclosed, vperiod) = fun_tool_closed(&s)?.unwrap_or_default();
    let tolp: f64 = 1.0e-6;
    let out_u = uclosed && b.1 - b.0 > uperiod + tolp;
    let out_v = vclosed && b.3 - b.2 > vperiod + tolp;
    if out_u || out_v {
        *b = nb;
        *outbounds = true;
    }
    Ok(true)
}

/// `FUN_tool_closedS(F, uclosed, uperiod, vclosed, vperiod)`: the closure of the basis
/// surface of the face; `None` without surface; `fun_tool_closed_s(&face)?`.
pub fn fun_tool_closed_s(
    f: &Shape,
) -> Result<Option<crate::top_ope_brep_tool::SurfaceClosure>, Failure> {
    let s = TopOpeBRepToolShapeTool::basis_surface_of_face(&topods::face(f)?)?;
    fun_tool_closed(&s)
}

/// `FUN_tool_closedS(F)`: whether the basis surface of the face is closed (and
/// periodic) in U or V; `fun_tool_closed_s_flag(&face)?`.
pub fn fun_tool_closed_s_flag(f: &Shape) -> Result<bool, Failure> {
    Ok(fun_tool_closed_s(f)?.is_some_and(|c| c.0 || c.2))
}

/// `FUN_tool_closedS(F, inU, xmin, xper)`: for a closed surface, whether it is closed
/// in U, the first bound and the period of that direction; natively the function
/// returns false in every case (kept); `fun_tool_closed_s_in_u(&face, &mut inu, &mut xmin, &mut xper)?`.
pub fn fun_tool_closed_s_in_u(
    f: &Shape,
    in_u: &mut bool,
    xmin: &mut f64,
    xper: &mut f64,
) -> Result<bool, Failure> {
    let s = TopOpeBRepToolShapeTool::basis_surface_of_face(&topods::face(f)?)?;
    let Some(surface) = s.get() else {
        return Ok(false);
    };
    let Some((uclosed, uperiod, vclosed, vperiod)) = fun_tool_closed(&s)? else {
        return Ok(false);
    };
    if !(uclosed || vclosed) {
        return Ok(false);
    }
    let b = surface.bounds();
    *in_u = uclosed;
    *xper = if uclosed { uperiod } else { vperiod };
    *xmin = if uclosed { b.0 } else { b.2 };
    Ok(false)
}

/// `FUN_tool_mkBnd2d(W, FF, B2d)`: the box of the UV representations of the edges of
/// the wire on the face (1e-8), the missing p-curves made and stored on the edges with
/// the greater of the edge and p-curve tolerances; the box then replaces `b2d`
/// (FUN_tool_UpdateBnd2d); `fun_tool_mk_bnd2d(&w, &face, &mut b2d)?`.
pub fn fun_tool_mk_bnd2d(w: &Shape, ff: &Shape, b2d: &mut BoundingBox2) -> Result<(), Failure> {
    let tol: f64 = 1.0e-8;
    let mut new_b2d = BoundingBox2::default();
    let f = topods::face(ff)?;
    for es in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e = topods::edge(&es)?;
        if !fc2d_has_curve_on_surface(&e, &f)? {
            let tol_e = BRepTool::edge_tolerance(&e)?;
            let (pc, _, _, tolpc) = fc2d_curve_on_surface(&e, &f, false)?;
            BRepBuilder::default().update_edge_pcurve_on_face(
                &e,
                &pc,
                &f,
                std_max(tol_e, tolpc),
            )?;
        }
        let bc2d = BRepAdaptorCurve2d::with_edge_on_face(&e, &f)?;
        Add2dCurve::add_adaptor(&bc2d, tol, &mut new_b2d)?;
    }
    fun_tool_update_bnd2d(b2d, &new_b2d);
    Ok(())
}

/// `FUN_tool_maxtol(S, typ, maxtol)`: `maxtol` raised to the tolerances of the
/// faces, edges or vertices of the shape; whether it has any of that type;
/// `fun_tool_maxtol_of_type(&s, ShapeEnum::Edge, &mut maxtol)?`.
pub fn fun_tool_maxtol_of_type(
    s: &Shape,
    typ: ShapeEnum,
    maxtol: &mut f64,
) -> Result<bool, Failure> {
    let mut hasshatyp = false;
    for ss in Explorer::new(s, typ, ShapeEnum::Shape)? {
        hasshatyp = true;
        let tolss = match typ {
            ShapeEnum::Face => BRepTool::face_tolerance(&topods::face(&ss)?)?,
            ShapeEnum::Edge => BRepTool::edge_tolerance(&topods::edge(&ss)?)?,
            ShapeEnum::Vertex => BRepTool::vertex_tolerance(&topods::vertex(&ss)?)?,
            _ => 0.,
        };
        if tolss > *maxtol {
            *maxtol = tolss;
        }
    }
    Ok(hasshatyp)
}

/// `FUN_tool_maxtol(S)`: the greatest tolerance of the faces, edges and vertices of
/// the shape (0 without any); `fun_tool_maxtol(&s)?`.
pub fn fun_tool_maxtol(s: &Shape) -> Result<f64, Failure> {
    let mut maxtol: f64 = 0.;
    for typ in [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex] {
        fun_tool_maxtol_of_type(s, typ, &mut maxtol)?;
    }
    Ok(maxtol)
}

/// The box of a shape without gap.
fn tight_box(sh: &Shape) -> Result<BoundingBox3, Failure> {
    let mut b = BoundingBox3::default();
    BRepBndLib::add(sh, &mut b, true)?;
    b.set_gap(0.);
    Ok(b)
}

/// The count of the bounds of the second box outside the first and of the equal ones
/// (Precision::Confusion()).
fn count_box_bounds(xyz1: [f64; 6], xyz2: [f64; 6]) -> (i32, i32) {
    let tol = precision::confusion();
    let (mut neq, mut n2sup) = (0, 0);
    for i in 0..6 {
        let d = xyz2[i] - xyz1[i];
        if d.abs() < tol {
            neq += 1;
            continue;
        }
        if (i < 3 && d < 0.) || (i >= 3 && d > 0.) {
            n2sup += 1;
        }
    }
    (neq, n2sup)
}

/// `FUN_tool_comparebndkole(sh1, sh2)`: 2 when the box of `sh1` lies in the box of
/// `sh2` and is smaller, 0 otherwise (disjoint, equal, crossing or containing boxes);
/// the native branch returning 1 is unreachable (it needs six equal bounds, which
/// return 0 first); `fun_tool_comparebndkole(&s1, &s2)?`.
pub fn fun_tool_comparebndkole(sh1: &Shape, sh2: &Shape) -> Result<i32, Failure> {
    let (bnd1, bnd2) = (tight_box(sh1)?, tight_box(sh2)?);
    if bnd1.is_out_box(&bnd2) {
        return Ok(0);
    }
    let (neq, n2sup) = count_box_bounds(bnd1.get()?, bnd2.get()?);
    if n2sup + neq != 6 || neq == 6 {
        return Ok(0);
    }
    Ok(if n2sup > 0 { 2 } else { 1 })
}
