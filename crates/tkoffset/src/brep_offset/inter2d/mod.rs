// Rust translation of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Inter2d.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepOffset_Inter2d`: the 2d intersections of the edges of the offset faces (new
//! intersection edges and extended offset edges), their vertices stored in the 2d
//! AsDes, and the fusion of the coinciding vertices.
mod connex;
mod extent;

use rustycad_tkbo::bop_tools::make_vertex;
use rustycad_tkbo::int_tools::Tools as IntToolsTools;
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeVertex};
use std::f64::consts::PI;

/// The vertices coinciding with each vertex (`theDMVV`).
pub type CoincidentVertices = IndexedDataMapOfShapeListOfShape;
/// The edges each edge was intersected with (`theEdgeIntEdges`).
pub type EdgeIntersections = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `BRepOffset_Inter2d`: the static 2d intersection services of the offset
/// algorithm; `BRepOffsetInter2d::fuse_vertices(&dmvv, &as_des, &mut image)?`.
pub struct BRepOffsetInter2d;

/// `Store(theEdge, theLV, theTol, IsToUpdate, theAsDes2d, theDMVV)`: the vertices (their
/// tolerance raised) added as descendants of the edge, each recorded as coinciding with
/// the vertices already on the edge within the tolerances; with `IsToUpdate` only the
/// vertices projecting on the edge, which get their parameter on it.
pub(super) fn store(
    the_edge: &Edge,
    the_lv: &[Shape],
    the_tol: f64,
    is_to_update: bool,
    the_as_des2d: &BRepAlgoAsDes,
    the_dmvv: &mut CoincidentVertices,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    // The update of the vertices.
    for v in the_lv {
        builder.update_vertex_tolerance(&topods::vertex(v)?, the_tol)?;
    }
    // The vertices already on the edge: the live descendant list when the edge has one
    // (it grows with the additions below), the static empty list otherwise.
    let bound = the_as_des2d.has_descendant(the_edge.as_ref());
    if !is_to_update && !bound {
        if !the_lv.is_empty() {
            for v in the_lv {
                the_as_des2d.add_shared(the_edge.as_ref(), v);
            }
        }
        return Ok(());
    }
    let mut a_proj_pc = ProjectPointOnCurve::new();
    let mut a_tol_e = 0.;
    if is_to_update {
        let (a_c, a_t1, a_t2) = BRepTool::curve(the_edge)?;
        a_proj_pc.init_curve(&a_c, a_t1, a_t2)?;
        a_tol_e = BRepTool::edge_tolerance(the_edge)?;
    }
    let mut a_mv = MapOfShape::default();
    for v in the_lv {
        let a_v = topods::vertex(v)?;
        if !a_mv.add(v.clone()) {
            continue;
        }
        let a_p = BRepTool::pnt(&a_v)?;
        let a_tol = BRepTool::vertex_tolerance(&a_v)?;
        let a_lv_ex: Vec<Shape> = if bound {
            the_as_des2d.descendant(the_edge.as_ref()).collect()
        } else {
            Vec::new()
        };
        let mut a_lvc = List::default();
        let mut already = false;
        for s_ex in &a_lv_ex {
            let a_v_ex = topods::vertex(s_ex)?;
            if a_v.is_same(&a_v_ex) {
                already = true;
                break;
            }
            let a_p_ex = BRepTool::pnt(&a_v_ex)?;
            let a_tol_v_ex = BRepTool::vertex_tolerance(&a_v_ex)?;
            if a_p.is_equal(&a_p_ex, a_tol + a_tol_v_ex) {
                a_lvc.append(s_ex.clone());
            }
        }
        if already {
            continue;
        }
        if is_to_update {
            // The parameter of the vertex on the edge.
            a_proj_pc.perform(a_p)?;
            if a_proj_pc.nb_points() == 0 {
                continue;
            }
            if a_proj_pc.lower_distance()? > a_tol + a_tol_e {
                continue;
            }
            let a_t = a_proj_pc.lower_distance_parameter()?;
            builder.update_vertex_on_edge(
                &topods::vertex(&a_v.oriented(Orientation::Internal))?,
                a_t,
                the_edge,
                a_tol,
            )?;
        }
        if !a_lvc.is_empty() {
            for a_vc in a_lvc.iter() {
                if the_dmvv.seek(a_vc).is_none() {
                    the_dmvv.add(a_vc.clone(), List::default());
                }
                the_dmvv.change_seek(a_vc).expect("bound").append(v.clone());
            }
            if the_dmvv.seek(v).is_none() {
                the_dmvv.add(v.clone(), List::default());
            }
            let p_lv = the_dmvv.change_seek(v).expect("bound");
            for a_vc in a_lvc.iter() {
                p_lv.append(a_vc.clone());
            }
        }
        the_as_des2d.add_shared(the_edge.as_ref(), v);
    }
    Ok(())
}

/// `Store(theE1, theE2, theLV1, theLV2, theTol, theAsDes2d, theDMVV)`: the vertices of
/// each edge.
fn store_pair(
    (the_e1, the_lv1): (&Edge, &[Shape]),
    (the_e2, the_lv2): (&Edge, &[Shape]),
    the_tol: f64,
    the_as_des2d: &BRepAlgoAsDes,
    the_dmvv: &mut CoincidentVertices,
) -> Result<(), Failure> {
    store(the_e1, the_lv1, the_tol, false, the_as_des2d, the_dmvv)?;
    store(the_e2, the_lv2, the_tol, false, the_as_des2d, the_dmvv)
}

/// The point of a degenerated edge among two: its vertex, or its first curve point.
fn degenerated_point(f: &Face, e1: &Edge, e2: &Edge) -> Result<Point3, Failure> {
    let edeg = if BRepTool::degenerated(e1)? { e1 } else { e2 };
    let iter = ShapeIterator::new(edeg.as_ref(), true, true);
    if iter.more() {
        return BRepTool::pnt(&topods::vertex(iter.value()?)?);
    }
    let ce_deg = BRepAdaptorCurve::with_edge_on_face(edeg, f)?;
    ce_deg.value(ce_deg.first_parameter()?)
}

/// The intersection points of two p-curves: the 3d point (the degenerated point with a
/// degenerated edge) and the parameters on both curves.
fn intersection_points(
    inter2d: &GInter,
    b_asurf: &BRepAdaptorSurface,
    deg_point: Option<Point3>,
) -> Result<Vec<(Point3, f64, f64)>, Failure> {
    let mut res = Vec::new();
    for i in 1..=inter2d.nb_points()? {
        let ip = inter2d.point(i)?;
        let p3d = match deg_point {
            Some(p) => p,
            None => {
                let p2d = ip.value();
                b_asurf.value(p2d.x(), p2d.y())?
            }
        };
        res.push((p3d, ip.param_on_first(), ip.param_on_second()));
    }
    Ok(res)
}

/// The INTERNAL vertex of an intersection point with its parameters on both edges,
/// its tolerance raised to the spread of the three points, and its orientations on
/// both edges (by the 2d tangents when `with_ori`, REVERSED otherwise).
#[allow(clippy::too_many_arguments)]
fn intersection_vertex(
    f: &Face,
    (e1, e2): (&Edge, &Edge),
    (ce1, ce2): (&BRepAdaptorCurve, &BRepAdaptorCurve),
    (p, a_t1, a_t2): (Point3, f64, f64),
    tol: f64,
    with_ori: bool,
) -> Result<(Shape, Orientation, Orientation), Failure> {
    let builder = BRepBuilder::default();
    let mut a_new_vertex = BRepLibMakeVertex::new(p)?.vertex()?;
    a_new_vertex.set_orientation(Orientation::Internal);
    builder.update_vertex_on_edge(&a_new_vertex, a_t1, e1, tol)?;
    builder.update_vertex_on_edge(&a_new_vertex, a_t2, e2, tol)?;
    let p1 = ce1.value(a_t1)?;
    let p2 = ce2.value(a_t2)?;
    let dist1 = p1.distance(&p).max(p2.distance(&p)).max(p1.distance(&p2));
    builder.update_vertex_tolerance(&a_new_vertex, dist1)?;
    // The orientation of the new vertex.
    let mut oo1 = Orientation::Reversed;
    let mut oo2 = Orientation::Reversed;
    if with_ori {
        let pce1 = BRepAdaptorCurve2d::with_edge_on_face(e1, f)?;
        let pce2 = BRepAdaptorCurve2d::with_edge_on_face(e2, f)?;
        let (_, v1) = pce1.d1(a_t1)?;
        let (_, v2) = pce2.d1(a_t2)?;
        let (mut v1or, mut v2or) = (v1, v2);
        if e1.orientation() == Orientation::Reversed {
            v1or.reverse();
        }
        if e2.orientation() == Orientation::Reversed {
            v2or.reverse();
        }
        if v2or.crossed(v1) > 0. {
            oo1 = Orientation::Forward;
        }
        if v1or.crossed(v2) > 0. {
            oo2 = Orientation::Forward;
        }
    }
    Ok((a_new_vertex.as_ref().clone(), oo1, oo2))
}

/// The "test at end" of the edge intersections: a new vertex for each pair of ends
/// closer than `tol_conf` (except a common vertex already in the AsDes), prepended to
/// the lists with the orientations of the ends.
#[allow(clippy::too_many_arguments)]
fn end_vertices(
    (e1, e2): (&Edge, &Edge),
    (f, l): ([f64; 2], [f64; 2]),
    as_des: &BRepAlgoAsDes,
    tol_conf: f64,
    vertex_tol: Option<f64>,
    lv1: &mut Vec<Shape>,
    lv2: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let (v1a, v1b) = topexp::vertices(e1, false);
    let (v2a, v2b) = topexp::vertices(e2, false);
    for (j, v1j) in [&v1a, &v1b].into_iter().enumerate() {
        if v1j.is_null() {
            continue;
        }
        for (k, v2k) in [&v2a, &v2b].into_iter().enumerate() {
            if v2k.is_null() {
                continue;
            }
            if v1j.is_same(v2k) && as_des.has_ascendant(v1j.as_ref()) {
                continue;
            }
            let p1 = BRepTool::pnt(v1j)?;
            let p2 = BRepTool::pnt(v2k)?;
            if p1.distance(&p2) < tol_conf {
                let a_tol = match vertex_tol {
                    Some(t) => t,
                    None => BRepTool::vertex_tolerance(v1j)?.max(BRepTool::vertex_tolerance(v2k)?),
                };
                let v = BRepLibMakeVertex::new(p1)?.vertex()?;
                let u1 = if j == 0 { f[0] } else { l[0] };
                let u2 = if k == 0 { f[1] } else { l[1] };
                let internal = topods::vertex(&v.oriented(Orientation::Internal))?;
                builder.update_vertex_on_edge(&internal, u1, e1, a_tol)?;
                builder.update_vertex_on_edge(&internal, u2, e2, a_tol)?;
                lv1.insert(0, v.oriented(v1j.orientation()));
                lv2.insert(0, v.oriented(v2k.orientation()));
            }
        }
    }
    Ok(())
}

/// The removal of the doubles of the lists: a vertex equal (within the tolerance, the
/// greater vertex tolerance when `tol` is `None`) to an earlier one is removed with its
/// counterpart, and the search restarts.
fn purge_doubles(
    lv1: &mut Vec<Shape>,
    lv2: &mut Vec<Shape>,
    tol: Option<f64>,
) -> Result<(), Failure> {
    let mut purge = true;
    while purge {
        purge = false;
        'outer: for i in 0..lv1.len() {
            let a_v1 = topods::vertex(&lv1[i])?;
            let p1 = BRepTool::pnt(&a_v1)?;
            let a_tol1 = BRepTool::vertex_tolerance(&a_v1)?;
            for j in 0..i {
                let a_v2 = topods::vertex(&lv1[j])?;
                let p2 = BRepTool::pnt(&a_v2)?;
                let a_tol = match tol {
                    Some(t) => t,
                    None => a_tol1.max(BRepTool::vertex_tolerance(&a_v2)?),
                };
                if p1.is_equal(&p2, a_tol) {
                    lv1.remove(i);
                    lv2.remove(i);
                    purge = true;
                    break 'outer;
                }
            }
        }
    }
    Ok(())
}

/// `EdgeInter(F, BAsurf, E1, E2, AsDes, Tol, WithOri, aDMVV)`: the intersection vertices
/// of two edges without common vertex on the face, and the vertices of their close
/// ends; stored for both edges.
#[allow(clippy::too_many_arguments)]
fn edge_inter(
    f: &Face,
    b_asurf: &BRepAdaptorSurface,
    e1: &Edge,
    e2: &Edge,
    as_des: &BRepAlgoAsDes,
    tol: f64,
    with_ori: bool,
    a_dmvv: &mut CoincidentVertices,
) -> Result<(), Failure> {
    if e1.is_same(e2) {
        return Ok(());
    }
    let tol_dub = 1.0e-7;
    let (mut f1, mut l1) = BRepTool::range(e1)?;
    let (mut f2, mut l2) = BRepTool::range(e2)?;
    let ce1 = BRepAdaptorCurve::with_edge_on_face(e1, f)?;
    let ce2 = BRepAdaptorCurve::with_edge_on_face(e2, f)?;
    let (mut lv1, mut lv2) = (Vec::new(), Vec::new());
    if topexp::common_vertex(e1, e2).is_none() {
        let (continuity, max_degree, max_segment) = brep_lib::BUILD_CURVE3D_DEFAULTS;
        brep_lib::build_curve3d(e1, 1.0e-5, continuity, max_degree, max_segment)?;
        brep_lib::build_curve3d(e2, 1.0e-5, continuity, max_degree, max_segment)?;
        let with_degen = BRepTool::degenerated(e1)? || BRepTool::degenerated(e2)?;
        let deg_point = if with_degen {
            Some(degenerated_point(f, e1, e2)?)
        } else {
            None
        };
        let pc1 = BRepTool::curve_on_face(e1, f)?;
        let pc2 = BRepTool::curve_on_face(e2, f)?;
        (f1, l1, f2, l2) = (pc1.first, pc1.last, pc2.first, pc2.last);
        let gac1 = Geom2dAdaptorCurve::with_range(&pc1.pcurve, f1, l1)?;
        let gac2 = Geom2dAdaptorCurve::with_range(&pc2.pcurve, f2, l2)?;
        let inter2d = GInter::between(&gac1, &gac2, tol_dub, tol_dub)?;
        for (p, a_t1, a_t2) in intersection_points(&inter2d, b_asurf, deg_point)? {
            if precision::is_infinite(a_t1) || precision::is_infinite(a_t2) {
                continue;
            }
            let (v, oo1, oo2) =
                intersection_vertex(f, (e1, e2), (&ce1, &ce2), (p, a_t1, a_t2), tol, with_ori)?;
            lv1.push(v.oriented(oo1));
            lv2.push(v.oriented(oo2));
        }
    }
    // The test at the ends.
    end_vertices(
        (e1, e2),
        ([f1, f2], [l1, l2]),
        as_des,
        tol,
        None,
        &mut lv1,
        &mut lv2,
    )?;
    if !lv1.is_empty() {
        // The removal of the doubles.
        purge_doubles(&mut lv1, &mut lv2, None)?;
        // The storage of the vertices.
        let tol_store = (BRepTool::edge_tolerance(e1)? + BRepTool::edge_tolerance(e2)?).max(tol);
        store_pair((e1, &lv1), (e2, &lv2), tol_store, as_des, a_dmvv)?;
    }
    Ok(())
}

/// `RefEdgeInter(F, BAsurf, E1, E2, theOr1, theOr2, AsDes, Tol, WithOri, theVref,
/// theImageVV, aDMVV, theCoincide)`: the intersection vertices of two edges on the
/// face (coinciding lines reported, not intersected), with the given orientations
/// (unless EXTERNAL) and the close ends; only the one nearest the reference vertex is
/// kept, recorded as its image and stored for both edges; whether the edges coincide.
#[allow(clippy::too_many_arguments)]
pub(super) fn ref_edge_inter(
    f: &Face,
    b_asurf: &BRepAdaptorSurface,
    e1: &Edge,
    e2: &Edge,
    (the_or1, the_or2): (Orientation, Orientation),
    as_des: &BRepAlgoAsDes,
    tol: f64,
    with_ori: bool,
    the_vref: &Vertex,
    the_image_vv: &mut BRepAlgoImage,
    a_dmvv: &mut CoincidentVertices,
) -> Result<bool, Failure> {
    if e1.is_same(e2) || e1.is_null() || e2.is_null() {
        return Ok(false);
    }
    let tol_dub = 1.0e-7;
    let mut tol_ll = 0.0;
    let pc1 = BRepTool::curve_on_face(e1, f)?;
    let pc2 = BRepTool::curve_on_face(e2, f)?;
    if pc1.pcurve.is_null() || pc2.pcurve.is_null() {
        return Ok(false);
    }
    let (f1, l1, f2, l2) = (pc1.first, pc1.last, pc2.first, pc2.last);
    let ce1 = BRepAdaptorCurve::with_edge_on_face(e1, f)?;
    let ce2 = BRepAdaptorCurve::with_edge_on_face(e2, f)?;
    let (continuity, max_degree, max_segment) = brep_lib::BUILD_CURVE3D_DEFAULTS;
    brep_lib::build_curve3d(e1, 1.0e-5, continuity, max_degree, max_segment)?;
    brep_lib::build_curve3d(e2, 1.0e-5, continuity, max_degree, max_segment)?;
    let with_degen = BRepTool::degenerated(e1)? || BRepTool::degenerated(e2)?;
    let deg_point = if with_degen {
        Some(degenerated_point(f, e1, e2)?)
    } else {
        None
    };
    let gac1 = Geom2dAdaptorCurve::with_range(&pc1.pcurve, f1, l1)?;
    let gac2 = Geom2dAdaptorCurve::with_range(&pc2.pcurve, f2, l2)?;
    let lines = gac1.get_type()? == CurveType::Line && gac2.get_type()? == CurveType::Line;
    if lines {
        // A quick check of coinciding lines.
        let an_angle = gac1
            .line()?
            .direction()
            .angle(gac2.line()?.direction())
            .abs();
        if an_angle <= 1.0e-8 || PI - an_angle <= 1.0e-8 {
            return Ok(true);
        }
        // The intersection range of the line-line intersection (the smaller the angle
        // between the curves, the bigger the range).
        tol_ll = IntToolsTools::compute_int_range(tol_dub, tol_dub, an_angle).min(1.0e-5);
    }
    let inter2d = GInter::between(&gac1, &gac2, tol_dub, tol_dub)?;
    if !inter2d.is_done() || inter2d.nb_points()? == 0 {
        return Ok(inter2d.nb_segments()? > 0 && lines);
    }
    let (mut lv1, mut lv2) = (Vec::new(), Vec::new());
    for (p, a_t1, a_t2) in intersection_points(&inter2d, b_asurf, deg_point)? {
        if precision::is_infinite(a_t1) || precision::is_infinite(a_t2) {
            continue;
        }
        let (v, mut oo1, mut oo2) =
            intersection_vertex(f, (e1, e2), (&ce1, &ce2), (p, a_t1, a_t2), tol, with_ori)?;
        if the_or1 != Orientation::External {
            oo1 = the_or1;
        }
        if the_or2 != Orientation::External {
            oo2 = the_or2;
        }
        lv1.push(v.oriented(oo1));
        lv2.push(v.oriented(oo2));
    }
    // The test at the ends.
    end_vertices(
        (e1, e2),
        ([f1, f2], [l1, l2]),
        as_des,
        tol,
        Some(tol),
        &mut lv1,
        &mut lv2,
    )?;
    if !lv1.is_empty() {
        // The removal of the doubles.
        purge_doubles(&mut lv1, &mut lv2, Some(tol))?;
        if lv1.len() > 1 {
            keep_nearest(the_vref, &mut lv1, &mut lv2)?;
        }
        for s in &lv1 {
            let a_new_vertex = s.oriented(Orientation::Forward);
            let vref = the_vref.oriented(Orientation::Forward);
            if the_image_vv.has_image(the_vref.as_ref()) {
                the_image_vv.add(&vref, &a_new_vertex)?;
            } else {
                the_image_vv.bind(&vref, &a_new_vertex)?;
            }
        }
        let tol_store = (BRepTool::edge_tolerance(e1)? + BRepTool::edge_tolerance(e2)?)
            .max(tol)
            .max(tol_ll);
        store_pair((e1, &lv1), (e2, &lv2), tol_store, as_des, a_dmvv)?;
    }
    Ok(false)
}

/// The vertices other than the nearest to the reference one removed, as the native
/// loop does: a removal moves on to the next element, which the loop increment then
/// skips.
fn keep_nearest(
    the_vref: &Vertex,
    lv1: &mut Vec<Shape>,
    lv2: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let pref = BRepTool::pnt(the_vref)?;
    let mut dmin = f64::MAX;
    let mut vmin = Shape::default();
    for s in lv1.iter() {
        let d = BRepTool::pnt(&topods::vertex(s)?)?.square_distance(&pref);
        if d < dmin {
            dmin = d;
            vmin = s.clone();
        }
    }
    let mut i = 0;
    while i < lv1.len() {
        if !vmin.is_same(&lv1[i]) {
            lv1.remove(i);
            lv2.remove(i);
            if i >= lv1.len() {
                break;
            }
        }
        i += 1;
    }
    Ok(())
}

/// `MakeChain(theV, theDMVV, theMDone, theChain)`: the vertices coinciding with the
/// vertex, transitively, in depth-first order.
fn make_chain(
    the_v: &Shape,
    the_dmvv: &CoincidentVertices,
    the_m_done: &mut MapOfShape,
    the_chain: &mut Vec<Shape>,
) {
    if the_m_done.add(the_v.clone()) {
        the_chain.push(the_v.clone());
        if let Some(p_lv) = the_dmvv.seek(the_v) {
            for v in p_lv.iter() {
                make_chain(v, the_dmvv, the_m_done, the_chain);
            }
        }
    }
}

impl BRepOffsetInter2d {
    /// `Compute(AsDes, F, NewEdges, Tol, theEdgeIntEdges, theDMVV, Range)`: the 2d
    /// intersections of the descendant edges of the face two by two (when one is new
    /// and one is not an edge of the face, unless already intersected together);
    /// `BRepOffsetInter2d::compute(&as_des, &face, &new_edges, 1e-7, &edge_int, &mut dmvv, &range)?`.
    pub fn compute(
        as_des: &BRepAlgoAsDes,
        f: &Face,
        new_edges: &IndexedMapOfShape,
        tol: f64,
        the_edge_int_edges: &EdgeIntersections,
        the_dmvv: &mut CoincidentVertices,
        the_range: &ProgressRange,
    ) -> Result<(), Failure> {
        // The edges of the face are not intersected.
        let mut edges_of_face = MapOfShape::default();
        for e in Explorer::new(f.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
            edges_of_face.add(e);
        }
        // The intersections of the edges two by two.
        let le: Vec<Shape> = as_des.descendant(f.as_ref()).collect();
        let b_asurf = BRepAdaptorSurface::with_face(f, true)?;
        let forward = topods::face(&f.oriented(Orientation::Forward))?;
        let a_ps = ProgressScope::with_literal(
            the_range,
            "Intersecting edges on faces",
            le.len() as f64,
            false,
        );
        for (i, s1) in le.iter().enumerate() {
            if !a_ps.more() {
                return Ok(());
            }
            let e1 = topods::edge(s1)?;
            for s2 in &le[..i] {
                let e2 = topods::edge(s2)?;
                let mut to_intersect = true;
                if let Some(a_elist) = the_edge_int_edges.seek(s1) {
                    if a_elist.iter().any(|x| e2.is_same(x)) {
                        to_intersect = false;
                    }
                    if to_intersect {
                        for an_edge in a_elist.iter() {
                            if let Some(a_elist2) = the_edge_int_edges.seek(an_edge)
                                && a_elist2.iter().any(|x| e2.is_same(x))
                            {
                                to_intersect = false;
                            }
                        }
                    }
                }
                // The intersections of the new edges together and with the edges of
                // the restrictions.
                if to_intersect
                    && (!edges_of_face.contains(s1) || !edges_of_face.contains(s2))
                    && (new_edges.contains(s1) || new_edges.contains(s2))
                {
                    edge_inter(&forward, &b_asurf, &e1, &e2, as_des, tol, true, the_dmvv)?;
                }
            }
            let _ = a_ps.next(1.);
        }
        Ok(())
    }

    /// `FuseVertices(theDMVV, theAsDes, theImageVV)`: each chain of coinciding vertices
    /// replaced in the AsDes by one new vertex (with the parameters of the old ones on
    /// their edges), recorded as an image of the origins of the old ones; false when an
    /// old vertex has no parameter on an edge;
    /// `BRepOffsetInter2d::fuse_vertices(&dmvv, &as_des, &mut image)?`.
    pub fn fuse_vertices(
        the_dmvv: &CoincidentVertices,
        the_as_des: &BRepAlgoAsDes,
        the_image_vv: &mut BRepAlgoImage,
    ) -> Result<bool, Failure> {
        let builder = BRepBuilder::default();
        let mut a_mv_done = MapOfShape::default();
        for i in 1..=the_dmvv.extent() {
            let a_v = the_dmvv.find_key(i)?.clone();
            // The chain of vertices.
            let mut a_lv_chain = Vec::new();
            make_chain(&a_v, the_dmvv, &mut a_mv_done, &mut a_lv_chain);
            if a_lv_chain.len() < 2 {
                continue;
            }
            // The new vertex.
            let mut chain = List::default();
            for v in &a_lv_chain {
                chain.append(v.clone());
            }
            let mut a_v_new = Vertex::default();
            make_vertex(&chain, &mut a_v_new)?;
            let a_v_new_int = topods::vertex(&a_v_new.oriented(Orientation::Internal))?;
            for a_v_old in &a_lv_chain {
                // The parameters on the edges.
                let a_v_old_int = topods::vertex(&a_v_old.oriented(Orientation::Internal))?;
                for a_es in the_as_des.ascendant(a_v_old) {
                    let a_e = topods::edge(&a_es)?;
                    let a_tol_e = BRepTool::edge_tolerance(&a_e)?;
                    let Some(a_t) = BRepTool::find_parameter(&a_v_old_int, &a_e)? else {
                        return Ok(false);
                    };
                    builder.update_vertex_on_edge(&a_v_new_int, a_t, &a_e, a_tol_e)?;
                }
                // The replacement of the vertex.
                the_as_des.replace_shared(a_v_old, a_v_new.as_ref());
                if the_image_vv.is_image(a_v_old) {
                    let a_pro_vertex = the_image_vv.image_from(a_v_old)?;
                    the_image_vv.add(&a_pro_vertex, &a_v_new.oriented(Orientation::Forward))?;
                }
            }
        }
        Ok(true)
    }
}
