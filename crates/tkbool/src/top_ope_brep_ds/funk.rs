// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_funk.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The states of a face on both sides of an edge at an inner point (FUN_staPinF3d,
//! FUN_UNKFstasta).
use super::export::config::dir;
use crate::top_ope_brep_tool::{
    TopOpeBRepToolShapeClassifier, fun_tool_nggeom_f, fun_tool_proj_ponbounded_f,
    fun_tool_value_on_face,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{uv_bounds, uv_bounds_of_edge};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

/// `FUN_staPinF3d(P, F)` (static): IN within the face tolerance, else IN/OUT by the
/// side of the (oriented) face normal at the projection; UNKNOWN when the projection
/// fails.
fn sta_p_in_f3d(p: Point3, f: &Face) -> Result<State, Failure> {
    let (mut uv, mut d) = (Point2::default(), 1.0e2);
    if !fun_tool_proj_ponbounded_f(p, f, &mut uv, &mut d)? {
        return Ok(State::Unknown);
    }
    if d < BRepTool::face_tolerance(f)? {
        return Ok(State::In);
    }
    let p_f = fun_tool_value_on_face(uv, f)?;
    let mut nt_f = dir(fun_tool_nggeom_f(uv, f)?)?;
    if f.orientation() == Orientation::Reversed {
        nt_f = nt_f.reversed();
    }
    let ppf = dir(Vector3::between(p, p_f))?;
    Ok(if nt_f.dot(ppf) > 0. {
        State::In
    } else {
        State::Out
    })
}

/// The p-curve of `ee` on `ff` (its range set in `range`), else its projection.
fn pcurve_on(
    ff: &Face,
    ee: &Edge,
    cee: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    sff: &Handle<dyn GeomSurface>,
    range: &mut (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let mut ceeffx: Handle<dyn Geom2dCurve> = Handle::default();
    for e in Explorer::new(ff, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if ee.is_same(&e) {
            let pc = BRepTool::curve_on_face(ee, ff)?;
            (ceeffx, *range) = (pc.pcurve, (pc.first, pc.last));
        }
    }
    if ceeffx.is_null() {
        ceeffx = geom_proj_lib::curve2d_range(cee, *range, sff)?;
    }
    Ok(ceeffx)
}

/// The UV bounds of `ee` (an edge of the other face) rebuilt on the surface of `ff`.
fn other_edge_bounds(
    ff: &Face,
    ee: &Edge,
    cee: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    sff: &Handle<dyn GeomSurface>,
    (fe, le): (f64, f64),
) -> Result<Option<(f64, f64, f64, f64)>, Failure> {
    let mut range = (fe, le);
    let ceeffx = pcurve_on(ff, ee, cee, sff, &mut range)?;
    if ceeffx.is_null() {
        return Ok(None);
    }
    let bb = BRepBuilder::default();
    let mut eex = Edge::default();
    bb.make_edge_with_curve(&mut eex, cee, BRepTool::edge_tolerance(ee)?)?;
    let (vf, vr) = topexp::vertices(ee, false);
    bb.add(&mut eex, &vf)?;
    bb.update_vertex_on_edge(&vf, range.0, &eex, BRepTool::vertex_tolerance(&vf)?)?;
    bb.add(&mut eex, &vr)?;
    bb.update_vertex_on_edge(&vr, range.1, &eex, BRepTool::vertex_tolerance(&vr)?)?;
    let mut ffx = Face::default();
    bb.make_face_on_surface(&mut ffx, sff, BRepTool::face_tolerance(ff)?)?;
    bb.update_edge_pcurve_on_face(&eex, &ceeffx, &ffx, BRepTool::face_tolerance(ff)?)?;
    Ok(Some(uv_bounds_of_edge(&ffx, &eex)?))
}

/// The points of the surface on both sides of the edge at `(u, v)`: (before, after).
#[allow(clippy::too_many_arguments)] // the native locals
fn side_points(
    sff: &Handle<dyn GeomSurface>,
    (u, v): (f64, f64),
    (eiso_u, eiso_v): (bool, bool),
    (paru, parv): (f64, f64),
    ve: Vector3,
    (n, d1u, d1v): (Vector3, Vector3, Vector3),
) -> Result<(Point3, Point3), Failure> {
    let s = sff
        .get()
        .ok_or_else(|| Failure::new("FUN_UNKFstasta: null surface"))?;
    let (mut up, mut vp, mut um, mut vm) = (u, v, u, v);
    if eiso_v {
        (up, um) = (up + paru, um - paru);
    } else if eiso_u {
        (vp, vm) = (vp + parv, vm - parv);
    } else {
        (up, vp, um, vm) = (up + paru, vp + parv, um - paru, vm - parv);
    }
    let (mut pb, mut pa) = (s.d0(up, vp)?, s.d0(um, vm)?);
    let ve_cro_n = ve.crossed(n);
    let dot = if eiso_v {
        ve_cro_n.dot(d1u)
    } else {
        ve_cro_n.dot(d1v)
    };
    if dot < 0. {
        std::mem::swap(&mut pa, &mut pb);
    }
    Ok((pb, pa))
}

/// `FUN_UNKFstasta(FF, FS, EE, EEofFF, stateb, statea, pClassif)`: the states before
/// and after of points of the surface of `ff` on both sides of an inner point of `ee`
/// (an edge of `ff`, or of `fs` when not `ee_of_ff`), classified by `p_classif` or
/// against `fs`; UNKNOWN when not computable. As natively, for an edge of `fs`, the
/// vertices of `ee` get a point on the edge rebuilt on the surface of `ff`;
/// `fun_unkf_stasta(&ff, &fs, &ee, true, None)?`.
pub fn fun_unkf_stasta(
    ff: &Face,
    fs: &Face,
    ee: &Edge,
    ee_of_ff: bool,
    p_classif: Option<&mut TopOpeBRepToolShapeClassifier>,
) -> Result<(State, State), Failure> {
    let unknown = (State::Unknown, State::Unknown);
    let (cee, fe, le) = BRepTool::curve(ee)?;
    let Some(c) = cee.get() else {
        return Ok(unknown);
    };
    let sff = BRepTool::surface(ff)?;
    let tt_e = 0.41237118973;
    let (pe, ve) = c.d1((1. - tt_e) * fe + tt_e * le)?;
    let pon_s = ProjectPointOnSurf::with_surface(pe, &sff, ExtAlgo::Grad)?;
    if !pon_s.extrema().is_done() || pon_s.nb_points()? == 0 {
        return Ok(unknown);
    }
    let (u, v) = pon_s.parameters(1)?;
    let d1 = sff
        .get()
        .ok_or_else(|| Failure::new("FUN_UNKFstasta: null surface"))?
        .d1(u, v)?;
    let n = d1.d1u.crossed(d1.d1v);
    let (fu_min, fu_max, fv_min, fv_max) = uv_bounds(ff)?;
    let ebounds = if ee_of_ff {
        uv_bounds_of_edge(ff, ee)?
    } else {
        match other_edge_bounds(ff, ee, &cee, &sff, (fe, le))? {
            Some(b) => b,
            None => return Ok(unknown),
        }
    };
    let (eu_min, eu_max, ev_min, ev_max) = ebounds;
    let eiso_u = (ev_max - ev_min).abs() < precision::confusion();
    let eiso_v = (eu_max - eu_min).abs() < precision::confusion();
    let (ttu, ttv) = (1.0e-2, 1.0e-2);
    let pars = (
        (ttu * (fu_max - fu_min)).abs(),
        (ttv * (fv_max - fv_min)).abs(),
    );
    let (pb, pa) = side_points(
        &sff,
        (u, v),
        (eiso_u, eiso_v),
        pars,
        ve,
        (n, d1.d1u, d1.d1v),
    )?;
    match p_classif {
        Some(pc) => {
            pc.state_p3d_reference(pb)?;
            let stateb = pc.state();
            pc.state_p3d_reference(pa)?;
            Ok((stateb, pc.state()))
        }
        None => Ok((sta_p_in_f3d(pb, fs)?, sta_p_in_f3d(pa, fs)?)),
    }
}
