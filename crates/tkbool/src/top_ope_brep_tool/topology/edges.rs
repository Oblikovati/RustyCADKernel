//! TopOpeBRepTool_TOPOLOGY.cxx: the orientation of sub-shapes, closing and shared
//! edges, the same orientation of edges, p-curves on faces and the `FUN_ds_*` edge
//! helpers.
use super::params::{fun_tool_par_e, fun_tool_value_on_edge};
use crate::top_ope_brep_ds::TopOpeBRepDSBuildTool;
use crate::top_ope_brep_tool::curve_tool::TopOpeBRepToolCurveTool;
use crate::top_ope_brep_tool::project::{fun_tool_bounds, fun_tool_proj_pone};
use crate::top_ope_brep_tool::tool_tangent::tggeom_e;
use crate::top_ope_brep_tool::two_d::fc2d_curve_on_surface;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Direction3;

/// `FUN_tool_orientVinE(v, e)`: 1 (FIRST) or 2 (LAST) when the vertex is the first or
/// last vertex of the edge, 3 (CLOSING) when both, 0 otherwise;
/// `fun_tool_orient_v_in_e(&v, &e)`.
pub fn fun_tool_orient_v_in_e(v: &Vertex, e: &Edge) -> i32 {
    let (vf, vl) = topexp::vertices(e, false);
    let (visf, visl) = (v.is_same(&vf), v.is_same(&vl));
    match (visf, visl) {
        (true, true) => 3,
        (false, true) => 2,
        (true, false) => 1,
        _ => 0,
    }
}

/// `FUN_tool_orientEinF(E, F, oriEinF)`: the orientation of the first edge of the face
/// same as `E` (FORWARD when there is none) and whether there is one;
/// `let (found, ori) = fun_tool_orient_e_in_f(&e, &f)?;`.
pub fn fun_tool_orient_e_in_f(e: &Edge, f: &Face) -> Result<(bool, Orientation), Failure> {
    for ef in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if ef.is_same(e) {
            return Ok((true, ef.orientation()));
        }
    }
    Ok((false, Orientation::Forward))
}

/// `FUN_tool_orientEinFFORWARD(E, F, oriEinF)`: the orientation of the edge in the
/// face oriented FORWARD (EXTERNAL when it is not an edge of the face);
/// `let (found, ori) = fun_tool_orient_e_in_f_forward(&e, &f)?;`.
pub fn fun_tool_orient_e_in_f_forward(e: &Edge, f: &Face) -> Result<(bool, Orientation), Failure> {
    let ff = topods::face(&f.oriented(Orientation::Forward))?;
    let (ok, reso) = fun_tool_orient_e_in_f(e, &ff)?;
    Ok((ok, if ok { reso } else { Orientation::External }))
}

/// `FUN_tool_EboundF(E, F)`: whether the edge of the face is a closing edge or an
/// INTERNAL or EXTERNAL edge (not a boundary edge); false for an edge not of the face;
/// `fun_tool_ebound_f(&e, &f)?`.
pub fn fun_tool_ebound_f(e: &Edge, f: &Face) -> Result<bool, Failure> {
    let (ok, ori) = fun_tool_orient_e_in_f_forward(e, f)?;
    if !ok {
        return Ok(false);
    }
    if BRepTool::is_closed_on_face(e, f)? {
        return Ok(true);
    }
    Ok(matches!(ori, Orientation::Internal | Orientation::External))
}

/// `FUN_tool_IsClosingE(E, S, F)`: whether the edge occurs twice in `S` and is closed
/// on the face; `fun_tool_is_closing_e(&e, &w, &f)?`.
pub fn fun_tool_is_closing_e(e: &Edge, s: &Shape, f: &Face) -> Result<bool, Failure> {
    let mut nbocc = 0;
    for ex in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if ex.is_same(e) {
            nbocc += 1;
        }
    }
    if nbocc != 2 {
        return Ok(false);
    }
    BRepTool::is_closed_on_face(e, f)
}

/// `FUN_tool_ClosingE(E, W, F)`: `FUN_tool_IsClosingE(E, W, F)`;
/// `fun_tool_closing_e(&e, &w, &f)?`.
pub fn fun_tool_closing_e(e: &Edge, w: &Wire, f: &Face) -> Result<bool, Failure> {
    fun_tool_is_closing_e(e, w, f)
}

/// `FUN_tool_inS(subshape, shape)`: whether `subshape` is a sub-shape of `shape`;
/// `fun_tool_in_s(&e, &f)?`.
pub fn fun_tool_in_s(subshape: &Shape, shape: &Shape) -> Result<bool, Failure> {
    let mut m: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::default();
    topexp::map_shapes_of_type(shape, subshape.shape_type()?, &mut m)?;
    Ok(m.contains(subshape))
}

/// `FUN_tool_Eshared(v, F1, F2, Eshared)`: the first edge of `F2` bounded by `v` and
/// shared with `F1`; `fun_tool_eshared(&v, &f1, &f2)?`.
pub fn fun_tool_eshared(v: &Shape, f1: &Shape, f2: &Shape) -> Result<Option<Shape>, Failure> {
    let mut e1s: List<Shape> = List::default();
    for e1 in Explorer::new(f1, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let mut exv = Explorer::new(&e1, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        if exv.any(|cv| cv.is_same(v)) {
            e1s.append(e1);
        }
    }
    for e2 in Explorer::new(f2, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if e1s.iter().any(|e1| e1.is_same(&e2)) {
            return Ok(Some(e2));
        }
    }
    Ok(None)
}

/// The unit tangent of an edge at a parameter, reversed for a REVERSED edge.
fn oriented_tangent(par: f64, e: &Edge) -> Result<Option<Direction3>, Failure> {
    let Some(tmp) = tggeom_e(par, e)? else {
        return Ok(None);
    };
    let mut tg = Direction3::from_xyz(*tmp.xyz())?;
    if e.orientation() == Orientation::Reversed {
        tg.reverse();
    }
    Ok(Some(tg))
}

/// `FUN_tool_SameOri(E1, E2)`: whether the oriented tangents of the edges agree at
/// the point of `E2` at 0.345 of its range, projected on `E1` (false when the
/// projection is farther than ten times the greater tolerance);
/// `fun_tool_same_ori(&e1, &e2)?`.
pub fn fun_tool_same_ori(e1: &Edge, e2: &Edge) -> Result<bool, Failure> {
    let (f, l) = fun_tool_bounds(e2)?;
    let x: f64 = 0.345;
    let mid = x * f + (1. - x) * l;
    let pmid = fun_tool_value_on_edge(mid, e2)?.unwrap_or_default();
    let Some(tg_e2) = oriented_tangent(mid, e2)? else {
        return Ok(false);
    };
    let (mut p_e1, mut dist) = (0., 0.);
    let ok = fun_tool_proj_pone(pmid, e1, &mut p_e1, &mut dist)?;
    let tol = std_max(BRepTool::edge_tolerance(e1)?, BRepTool::edge_tolerance(e2)?) * 10.;
    if dist > tol || !ok {
        return Ok(false);
    }
    let Some(tg_e1) = oriented_tangent(p_e1, e1)? else {
        return Ok(false);
    };
    Ok(tg_e2.dot(tg_e1) > 0.)
}

/// `FUN_tool_haspc(E, F)`: whether the edge has (or gets) a p-curve on the face;
/// `fun_tool_haspc(&e, &f)?`.
pub fn fun_tool_haspc(e: &Edge, f: &Face) -> Result<bool, Failure> {
    Ok(!fc2d_curve_on_surface(e, f, false)?.0.is_null())
}

/// `FUN_tool_pcurveonF(F, E)`: a p-curve of the 3d curve of the edge made on the face
/// (TopOpeBRepTool_CurveTool::MakePCurveOnFace) and stored with the edge tolerance;
/// `fun_tool_pcurve_on_f(&f, &e)?`.
pub fn fun_tool_pcurve_on_f(f: &Face, e: &Edge) -> Result<bool, Failure> {
    let (c3d, first, last) = BRepTool::curve(e)?;
    if c3d.is_null() {
        return Ok(false);
    }
    let (c2d, _) = TopOpeBRepToolCurveTool::make_pcurve_on_face(f, &c3d, first, last)?;
    if c2d.is_null() {
        return Ok(false);
    }
    let tol_e = BRepTool::edge_tolerance(e)?;
    BRepBuilder::default().update_edge_pcurve_on_face(e, &c2d, f, tol_e)?;
    Ok(true)
}

/// The edges of a wire with the faulty edge given the p-curve (in place: the edge
/// shares its TEdge); `None` when the wire does not hold it.
fn wire_with_pcurve(
    w: &Shape,
    ff: &Face,
    faulty_e: &Edge,
    c2d: &Handle<dyn Geom2dCurve>,
) -> Result<Option<List<Shape>>, Failure> {
    let mut loe: List<Shape> = List::default();
    let mut hasneww = false;
    for es in Explorer::new(w, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e = topods::edge(&es)?;
        if !e.is_equal(faulty_e) {
            loe.append(e.into_shape());
            continue;
        }
        let tole = BRepTool::edge_tolerance(&e)?;
        let mut newe = faulty_e.clone();
        BRepBuilder::default().update_edge_pcurve_on_face(&newe, c2d, ff, tole)?;
        newe.set_orientation(e.orientation());
        loe.append(newe.into_shape());
        hasneww = true;
    }
    Ok(hasneww.then_some(loe))
}

/// `FUN_tool_pcurveonF(fF, faultyE, C2d, newf)`: the p-curve set on every occurrence
/// of the faulty edge in the wires of the face, and a copy of the face on the new wires
/// (true only when the edge occurs); `fun_tool_pcurve_on_f_faulty(&f, &e, &c2d, &mut newf)?`.
pub fn fun_tool_pcurve_on_f_faulty(
    f_f: &Face,
    faulty_e: &Edge,
    c2d: &Handle<dyn Geom2dCurve>,
    newf: &mut Face,
) -> Result<bool, Failure> {
    let mut low: List<Shape> = List::default();
    let mut hasnewf = false;
    for w in Explorer::new(f_f, ShapeEnum::Wire, ShapeEnum::Shape)? {
        match wire_with_pcurve(&w, f_f, faulty_e, c2d)? {
            Some(loe) => {
                hasnewf = true;
                low.append(fun_tool_make_wire(&loe)?.into_shape());
            }
            None => {
                low.append(w);
            }
        }
    }
    if !hasnewf {
        return Ok(false);
    }
    *newf = topods::face(&f_f.empty_copied()?)?;
    for w in low.iter() {
        BRepBuilder::default().add(newf, w)?;
    }
    Ok(true)
}

/// `FUN_tool_curvesSO(E1, p1, E2, p2, so)`: whether the geometric tangents at the
/// parameters are parallel (`so` true) or opposite (`so` false) within 1e3 times
/// Precision::Angular(); `so` untouched otherwise;
/// `fun_tool_curves_so_at_both(&e1, 0.5, &e2, 0.5, &mut so)?`.
pub fn fun_tool_curves_so_at_both(
    e1: &Edge,
    p1: f64,
    e2: &Edge,
    p2: f64,
    so: &mut bool,
) -> Result<bool, Failure> {
    let Some(tg1) = tggeom_e(p1, e1)? else {
        return Ok(false);
    };
    let Some(tg2) = tggeom_e(p2, e2)? else {
        return Ok(false);
    };
    let tola = precision::angular() * 1.0e3;
    if tg1.is_opposite(tg2, tola)? {
        *so = false;
    } else if tg1.is_parallel(tg2, tola)? {
        *so = true;
    } else {
        return Ok(false);
    }
    Ok(true)
}

/// `FUN_tool_curvesSO(E1, p1, E2, so)`: with the parameter of the point of `E1` on
/// `E2` (FUN_tool_parE); `fun_tool_curves_so_at(&e1, 0.5, &e2, &mut so)?`.
pub fn fun_tool_curves_so_at(
    e1: &Edge,
    p1: f64,
    e2: &Edge,
    so: &mut bool,
) -> Result<bool, Failure> {
    let mut p2: f64 = 0.;
    if !fun_tool_par_e(e1, p1, e2, &mut p2)? {
        return Ok(false);
    }
    fun_tool_curves_so_at_both(e1, p1, e2, p2, so)
}

/// `FUN_tool_curvesSO(E1, E2, so)`: from the shared vertices of two open edges, else
/// from the tangents at 0.45678 of the range of `E1`;
/// `fun_tool_curves_so(&e1, &e2, &mut so)?`.
pub fn fun_tool_curves_so(e1: &Edge, e2: &Edge, so: &mut bool) -> Result<bool, Failure> {
    let (vf1, vl1) = topexp::vertices(e1, false);
    let (vf2, vl2) = topexp::vertices(e2, false);
    let closed = vf1.is_same(&vl1) || vf2.is_same(&vl2);
    let shared = if closed {
        None
    } else if vf1.is_same(&vf2) || vl1.is_same(&vl2) {
        Some(true)
    } else if vf1.is_same(&vl2) || vl1.is_same(&vf2) {
        Some(false)
    } else {
        None
    };
    if let Some(s) = shared {
        *so = s;
        return Ok(true);
    }
    let (f, l) = fun_tool_bounds(e1)?;
    let x: f64 = 0.45678;
    fun_tool_curves_so_at(e1, (1. - x) * l + x * f, e2, so)
}

/// `FUN_tool_findAncestor(lF, E, Fanc)`: the first face of the list holding the edge;
/// `fun_tool_find_ancestor(&faces, &e)?`.
pub fn fun_tool_find_ancestor(lf: &List<Shape>, e: &Edge) -> Result<Option<Face>, Failure> {
    for s in lf.iter() {
        let f = topods::face(s)?;
        if fun_tool_orient_e_in_f(e, &f)?.0 {
            return Ok(Some(f));
        }
    }
    Ok(None)
}

/// `FUN_tool_nbshapes(S, typ)`: the count of explored sub-shapes of the type;
/// `fun_tool_nbshapes(&s, ShapeEnum::Edge)?`.
pub fn fun_tool_nbshapes(s: &Shape, typ: ShapeEnum) -> Result<i32, Failure> {
    let mut i = 0;
    for _ in Explorer::new(s, typ, ShapeEnum::Shape)? {
        i += 1;
    }
    Ok(i)
}

/// `FUN_tool_shapes(S, typ, ltyp)`: the explored sub-shapes of the type appended;
/// `fun_tool_shapes(&s, ShapeEnum::Edge, &mut l)?`.
pub fn fun_tool_shapes(s: &Shape, typ: ShapeEnum, ltyp: &mut List<Shape>) -> Result<(), Failure> {
    for c in Explorer::new(s, typ, ShapeEnum::Shape)? {
        ltyp.append(c);
    }
    Ok(())
}

/// `FUN_ds_CopyEdge(Ein, Eou)`: an empty copy of the edge with its range
/// (TopOpeBRepDS_BuildTool::CopyEdge); `fun_ds_copy_edge(&e, &mut copy)?`.
pub fn fun_ds_copy_edge(ein: &Shape, eou: &mut Shape) -> Result<(), Failure> {
    TopOpeBRepDSBuildTool::default().copy_edge(ein, eou)
}

/// `FUN_ds_Parameter(E, V, P)`: the vertex parameter on the edge
/// (TopOpeBRepDS_BuildTool::Parameter); `fun_ds_parameter(&e, &v, 1.)?`.
pub fn fun_ds_parameter(e: &Shape, v: &Shape, p: f64) -> Result<(), Failure> {
    TopOpeBRepDSBuildTool::default().parameter(e, v, p)
}

/// `FUN_tool_MakeWire(loE, newW)`: a new wire of the edges; `fun_tool_make_wire(&edges)?`.
pub fn fun_tool_make_wire(lo_e: &List<Shape>) -> Result<Wire, Failure> {
    let bb = BRepBuilder::default();
    let mut new_w = Wire::default();
    bb.make_wire(&mut new_w);
    for e in lo_e.iter() {
        bb.add(&mut new_w, &topods::edge(e)?.into_shape())?;
    }
    Ok(new_w)
}
