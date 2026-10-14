//! The reducers of the edge interferences (FDS_reduceONFACEinterferences,
//! FUN_ReducerEdge3d, CheckInterferenceIsValid, FUN_ReducerEdge, FUN_ReducerSDEdge).
use crate::top_ope_brep_ds::export::complete_se::tki_groups;
use crate::top_ope_brep_ds::export::{fds_data, fds_tdata, fun_ds_get_vsdm, fun_ds_sdm, iref};
use crate::top_ope_brep_ds::process_interferences_tool::fun_has_state_shape;
use crate::top_ope_brep_ds::process_interferences_tool::select::{to_list, to_vec};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSConfig, TopOpeBRepDSCurvePointInterference,
    TopOpeBRepDSDataStructure, TopOpeBRepDSEdge3dInterferenceTool,
    TopOpeBRepDSEdgeVertexInterference, TopOpeBRepDSInterference, TopOpeBRepDSKind,
    TopOpeBRepDSTransition, fds_repvg,
};
use crate::top_ope_brep_tool::{
    fun_tool_eshared, fun_tool_orient_e_in_f_forward, fun_tool_proj_pone,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

type H = Handle<TopOpeBRepDSInterference>;

/// `FDS_reduceONFACEinterferences(LI, BDS, SIX)` (static): when a POINT interference
/// is ON a FACE, the other POINT interferences are removed.
pub(crate) fn fds_reduce_onface_interferences(li: &mut ListOfInterference) -> Result<(), Failure> {
    let on_face = |i: &H| -> Result<bool, Failure> {
        Ok(fun_has_state_shape(
            &iref(i)?.transition(),
            State::On,
            ShapeEnum::Face,
        ))
    };
    let mut has_onface = false;
    for i in li.iter() {
        if fds_data(i).0 == TopOpeBRepDSKind::Point {
            has_onface = on_face(i)?;
            if has_onface {
                break;
            }
        }
    }
    if !has_onface {
        return Ok(());
    }
    let mut kept = Vec::new();
    for i in to_vec(li) {
        if fds_data(&i).0 != TopOpeBRepDSKind::Point || on_face(&i)? {
            kept.push(i);
        }
    }
    *li = to_list(kept);
    Ok(())
}

/// `CheckInterferenceIsValid(I, theEdge, theSupportEdge, theVertex)` (static): the
/// point of the curve/point or edge/vertex interference on the edge projects on the
/// support edge within the greatest tolerance of the edges (and vertex).
pub(crate) fn check_interference_is_valid(
    i: &H,
    the_edge: &Edge,
    the_support_edge: &Edge,
    the_vertex: &Vertex,
) -> Result<bool, Failure> {
    let bc = BRepAdaptorCurve::with_edge(the_edge)?;
    let cpi: Handle<TopOpeBRepDSCurvePointInterference> = i.cast();
    let evi: Handle<TopOpeBRepDSEdgeVertexInterference> = i.cast();
    let pref = match (cpi.get(), evi.get()) {
        (Some(c), _) => c.parameter(),
        (None, Some(e)) => e.parameter(),
        (None, None) => return Ok(false),
    };
    let p3d1 = bc.value(pref)?;
    let (mut paron, mut dist) = (0., 0.);
    if !fun_tool_proj_pone(p3d1, the_support_edge, &mut paron, &mut dist)? {
        return Ok(false);
    }
    let p3d2 = BRepAdaptorCurve::with_edge(the_support_edge)?.value(paron)?;
    let (te, ts) = (
        BRepTool::edge_tolerance(the_edge)?,
        BRepTool::edge_tolerance(the_support_edge)?,
    );
    let mut tolerance = if te > ts { te } else { ts };
    if !the_vertex.is_null() {
        let tv = BRepTool::vertex_tolerance(the_vertex)?;
        tolerance = if tv > tolerance { tv } else { tolerance };
    }
    Ok(p3d1.distance(&p3d2) <= tolerance)
}

/// The vertex of a VERTEX geometry (null otherwise).
fn vertex_of(
    bds: &TopOpeBRepDSDataStructure,
    gt: TopOpeBRepDSKind,
    g: i32,
) -> Result<Vertex, Failure> {
    if gt == TopOpeBRepDSKind::Vertex && g != 0 {
        return topods::vertex(bds.shape(g, true));
    }
    Ok(Vertex::default())
}

/// Whether an interference is still valid on the edge (`CheckInterferenceIsValid`
/// with its support edge and vertex).
fn valid_on(bds: &TopOpeBRepDSDataStructure, e: &Edge, i: &H) -> Result<bool, Failure> {
    let (gt, g, _, s) = fds_data(i);
    let v = vertex_of(bds, gt, g)?;
    check_interference_is_valid(i, e, &topods::edge(bds.shape(s, true))?, &v)
}

/// The geometry status kept from one interference to the next by `FUN_ReducerEdge3d`.
struct Walk3d {
    oov: Shape,
    gsta: i32,
}

/// The status of the vertex `g1` relative to the edge of rank `rank_e`.
fn vertex_status(
    bds: &TopOpeBRepDSDataStructure,
    g1: i32,
    rank_e: i32,
    w: &mut Walk3d,
) -> Result<(), Failure> {
    let vg1: Shape = topods::vertex(bds.shape(g1, true))?.into_shape();
    let sd = fun_ds_get_vsdm(bds, g1);
    if bds.ancestor_rank(&vg1) != rank_e {
        w.oov = vg1;
        w.gsta = if sd.is_some() { 3 } else { 2 };
    } else {
        if let Some(sd) = sd {
            w.oov = bds.shape(sd, true).clone();
        }
        w.gsta = if sd.is_some() { 3 } else { 1 };
    }
    Ok(())
}

/// The removal of the FACE/EDGE interferences on an EXTERNAL edge of their face.
fn remove_external(bds: &TopOpeBRepDSDataStructure, v: &mut Vec<H>) -> Result<(), Failure> {
    let mut kept = Vec::with_capacity(v.len());
    for i in v.drain(..) {
        let (_, _, st1, s1) = fds_data(&i);
        let (_, ib1, _, _) = fds_tdata(&i);
        let mut torem = false;
        if st1 == TopOpeBRepDSKind::Edge {
            let ee = topods::edge(bds.shape(s1, true))?;
            let ff = topods::face(bds.shape(ib1, true))?;
            let (ok, o) = fun_tool_orient_e_in_f_forward(&ee, &ff)?;
            torem = ok && o == Orientation::External;
        }
        if !torem {
            kept.push(i);
        }
    }
    *v = kept;
    Ok(())
}

/// The reduced interference of `i1` with the transition of the tool (`None` when `i1`
/// is neither edge/vertex nor curve/point).
fn reduced_3d(
    tool: &TopOpeBRepDSEdge3dInterferenceTool,
    i1: &H,
    ib1: i32,
    g1: i32,
) -> Result<Option<H>, Failure> {
    let iibid: H = Handle::new(TopOpeBRepDSInterference::default());
    tool.transition(&iibid)?;
    let mut t = iref(&iibid)?.transition();
    t.set_index(ib1);
    let evi: Handle<TopOpeBRepDSEdgeVertexInterference> = i1.cast();
    let cpi: Handle<TopOpeBRepDSCurvePointInterference> = i1.cast();
    let face = TopOpeBRepDSKind::Face;
    if let Some(e) = evi.get() {
        let unsh = TopOpeBRepDSConfig::UnshGeometry;
        let n = TopOpeBRepDSEdgeVertexInterference::new(
            &t,
            face,
            ib1,
            g1,
            e.gbound(),
            unsh,
            e.parameter(),
        );
        return Ok(Some(Handle::new(n).cast()));
    }
    if let Some(c) = cpi.get() {
        let p = TopOpeBRepDSKind::Point;
        let n = TopOpeBRepDSCurvePointInterference::new(&t, face, ib1, p, g1, c.parameter());
        return Ok(Some(Handle::new(n).cast()));
    }
    Ok(None)
}

/// The comparison of the interferences after `v[a]` of same geometry around the
/// support edge (`FUN_ReducerEdge3d` inner loop): whether the transition got complex;
/// `None` for the native `return` out of the whole reducer.
fn compare_3d(
    bds: &TopOpeBRepDSDataStructure,
    (six, e, rank_e): (i32, &Edge, i32),
    v: &mut Vec<H>,
    a: usize,
    w: &mut Walk3d,
    tool: &mut TopOpeBRepDSEdge3dInterferenceTool,
) -> Result<Option<bool>, Failure> {
    let i1 = v[a].clone();
    let (gt1, g1, st1, mut s1) = fds_data(&i1);
    let (_, ib1, _, _) = fds_tdata(&i1);
    let mut is_complex = false;
    let mut b = a + 1;
    while b < v.len() {
        let i2 = v[b].clone();
        let (gt2, g2, st2, mut s2) = fds_data(&i2);
        let (_, ib2, _, _) = fds_tdata(&i2);
        if !valid_on(bds, e, &i2)? {
            v.remove(b);
            continue;
        }
        if !(gt2 == gt1 && g2 == g1) {
            break;
        }
        if gt1 == TopOpeBRepDSKind::Vertex {
            vertex_status(bds, g1, rank_e, w)?;
        }
        let f1 = topods::face(bds.shape(ib1, true))?.into_shape();
        let f2 = topods::face(bds.shape(ib2, true))?.into_shape();
        if !(st2 == st1 && s2 == s1) {
            let Some(eshared) = fun_tool_eshared(&w.oov, &f1, &f2)? else {
                return Ok(None);
            };
            if !bds.has_shape(&eshared, true) {
                return Ok(None);
            }
            s1 = bds.shape_index(&eshared, true);
            s2 = s1;
        }
        let e1 = topods::edge(bds.shape(s1, true))?.into_shape();
        let e2 = topods::edge(bds.shape(s2, true))?.into_shape();
        if fun_ds_sdm(bds, e, &e1) {
            b += 1;
            continue;
        }
        if !is_complex || gt1 == TopOpeBRepDSKind::Vertex {
            if is_complex {
                let ibid: H = Handle::new(TopOpeBRepDSInterference::default());
                tool.transition(&ibid)?;
                let o = iref(&ibid)?
                    .transition()
                    .orientation(State::In, ShapeEnum::Face);
                iref(&i1)?.change_transition(|t| t.set_orientation(o));
            } else {
                tool.init_point_vertex(w.gsta, &w.oov);
            }
            is_complex = true;
            tool.init(e, &e1, &f1, &i1)?;
            tool.add(e, &e1, &f1, &i1)?;
        }
        tool.add(e, &e2, &f2, &i2)?;
        v.remove(b);
    }
    let _ = six;
    Ok(Some(is_complex))
}

/// The walk of `FUN_ReducerEdge3d` over `v` (returns early as natively).
fn reduce_3d(
    six: i32,
    bds: &TopOpeBRepDSDataStructure,
    v: &mut Vec<H>,
    reduced_li: &mut ListOfInterference,
) -> Result<(), Failure> {
    let e = topods::edge(bds.shape(six, true))?;
    let rank_e = bds.ancestor_rank(&e);
    let mut w = Walk3d {
        oov: Shape::default(),
        gsta: 0,
    };
    remove_external(bds, v)?;
    if v.len() <= 1 {
        return Ok(());
    }
    let mut a = 0;
    while a < v.len() {
        let mut tool = TopOpeBRepDSEdge3dInterferenceTool::new();
        let i1 = v[a].clone();
        if !valid_on(bds, &e, &i1)? {
            v.remove(a);
            continue;
        }
        let Some(is_complex) = compare_3d(bds, (six, &e, rank_e), v, a, &mut w, &mut tool)? else {
            return Ok(());
        };
        if !is_complex {
            a += 1;
            continue;
        }
        let (_, g1, _, _) = fds_data(&i1);
        let (_, ib1, _, _) = fds_tdata(&i1);
        if let Some(new_i) = reduced_3d(&tool, &i1, ib1, g1)? {
            reduced_li.append(new_i);
            v.remove(a);
        }
    }
    Ok(())
}

/// `FUN_ReducerEdge3d(SIX, BDS, LI, reducedLI)` (static): the 3d FACE/EDGE
/// interferences at a geometry around a support edge reduced to one FACE interference
/// of the accumulated 3d transition (in `reduced_li`).
pub(crate) fn fun_reducer_edge3d(
    six: i32,
    bds: &TopOpeBRepDSDataStructure,
    li: &mut ListOfInterference,
    reduced_li: &mut ListOfInterference,
) -> Result<(), Failure> {
    reduced_li.clear();
    if li.extent() <= 1 {
        return Ok(());
    }
    let mut v = to_vec(li);
    let r = reduce_3d(six, bds, &mut v, reduced_li);
    *li = to_list(v);
    r
}

/// `FUN_ReducerEdge(SIX, BDS, LI, reducedLI)` (static): the VERTEX then (after the ON
/// FACE reduction) POINT interferences reduced by `FDS_repvg`.
pub(crate) fn fun_reducer_edge(
    six: i32,
    bds: &TopOpeBRepDSDataStructure,
    li: &mut ListOfInterference,
    reduced_li: &mut ListOfInterference,
) -> Result<(), Failure> {
    fds_repvg(bds, six, TopOpeBRepDSKind::Vertex, li, reduced_li)?;
    fds_reduce_onface_interferences(li)?;
    fds_repvg(bds, six, TopOpeBRepDSKind::Point, li, reduced_li)
}

/// The 1d EDGE vertex interference of a FORWARD/REVERSED pair on same domain edges.
fn sd_edge_interference(
    bds: &TopOpeBRepDSDataStructure,
    (six, g, ib1): (i32, i32, i32),
    t: &TopOpeBRepDSTransition,
    i1: &H,
) -> Result<H, Failure> {
    let evi: Handle<TopOpeBRepDSEdgeVertexInterference> = i1.cast();
    let evi = evi.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "FUN_ReducerSDEdge: no edge/vertex interference",
            "TopOpeBRepDS_EIR",
        )
    })?;
    let (c_eix, c1) = (
        bds.same_domain_ori_index(six),
        bds.same_domain_ori_index(ib1),
    );
    let conf = if c_eix == c1 {
        TopOpeBRepDSConfig::SameOriented
    } else {
        TopOpeBRepDSConfig::DiffOriented
    };
    let edge = TopOpeBRepDSKind::Edge;
    let n = TopOpeBRepDSEdgeVertexInterference::new(
        t,
        edge,
        ib1,
        g,
        evi.gbound(),
        conf,
        evi.parameter(),
    );
    Ok(Handle::new(n).cast())
}

/// The pair of `FUN_ReducerSDEdge` for `v[a]`: the transition of the pair (the other
/// interference removed), the unpaired ones appended to `new_li` as natively.
fn sd_pair(
    bds: &TopOpeBRepDSDataStructure,
    eix: &Shape,
    v: &mut Vec<H>,
    a: usize,
    new_li: &mut ListOfInterference,
) -> Result<Option<TopOpeBRepDSTransition>, Failure> {
    let t1 = iref(&v[a])?.transition();
    let (o1, ib1) = (t1.orientation(State::In, ShapeEnum::Face), t1.index()?);
    let mut t = TopOpeBRepDSTransition::new(State::In, State::In, ShapeEnum::Edge, ShapeEnum::Edge);
    let mut b = a + 1;
    while b < v.len() {
        let t2 = iref(&v[b])?.transition();
        let (o2, ib2) = (t2.orientation(State::In, ShapeEnum::Face), t2.index()?);
        let complex1d = fun_ds_sdm(bds, eix, bds.shape(ib2, true))
            && ((o1 == Orientation::Forward && o2 == Orientation::Reversed)
                || (o2 == Orientation::Forward && o1 == Orientation::Reversed));
        if !complex1d {
            new_li.append(v[b].clone());
            b += 1;
            continue;
        }
        let rev1 = o1 == Orientation::Reversed;
        t.set_index_before(if rev1 { ib1 } else { ib2 });
        t.set_index_after(if rev1 { ib2 } else { ib1 });
        v.remove(b);
        return Ok(Some(t));
    }
    Ok(None)
}

/// `FUN_ReducerSDEdge(SIX, BDS, LI, reducedLI)` (static): per geometry, a FORWARD and
/// a REVERSED 1d interference on edges same domain as `six` reduced to one IN/IN EDGE
/// interference (in `reduced_li`); the others kept (unpaired later ones appended once
/// per earlier one, as natively).
pub(crate) fn fun_reducer_sd_edge(
    six: i32,
    bds: &TopOpeBRepDSDataStructure,
    li: &mut ListOfInterference,
    reduced_li: &mut ListOfInterference,
) -> Result<(), Failure> {
    reduced_li.clear();
    if li.extent() <= 1 {
        return Ok(());
    }
    let mut new_li = ListOfInterference::default();
    let eix = bds.shape(six, true).clone();
    for (_, g, loi) in tki_groups(li)? {
        let mut v = to_vec(&loi);
        let mut a = 0;
        while a < v.len() {
            let ib1 = iref(&v[a])?.transition().index()?;
            if !fun_ds_sdm(bds, &eix, bds.shape(ib1, true)) {
                new_li.append(v[a].clone());
                a += 1;
                continue;
            }
            match sd_pair(bds, &eix, &mut v, a, &mut new_li)? {
                Some(t) => {
                    reduced_li.append(sd_edge_interference(bds, (six, g, ib1), &t, &v[a])?);
                    a += 1;
                }
                None => {
                    new_li.append(v.remove(a));
                }
            }
        }
    }
    *li = new_li;
    Ok(())
}
