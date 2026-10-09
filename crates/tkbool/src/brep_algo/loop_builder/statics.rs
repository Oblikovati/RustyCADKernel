// Rust translation of the static Bubble, UpdateClosedEdge, RemovePendingEdges, SamePnt2d,
// SelectEdge, PurgeNewEdges and StoreInMVE of
// ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo_Loop.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The helpers of the loop builder: the vertices on an edge sorted, the closing vertex of
//! closed edges, the vertex to edges map and the walk from edge to edge.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;

/// The vertices replaced by others (`NCollection_DataMap<TopoDS_Shape, TopoDS_Shape,
/// TopTools_ShapeMapHasher>`).
pub type VertexSubstitutes = DataMap<Shape, Shape, ShapeMapHasher>;

/// Standard_NullObject for a missing p-curve (a null dereference natively).
fn no_pcurve(e: &Edge) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!(
            "BRepAlgo_Loop: edge {:?} has no p-curve on the face; expected one",
            e.as_ref().orientation()
        ),
        "BRepAlgo_Loop",
    )
}

/// The p-curve point of `e` on the FORWARD face at its first (`first`) or last end.
fn pcurve_end(e: &Edge, ff: &Face, first: bool) -> Result<Point2, Failure> {
    let pc = BRepTool::curve_on_face(e, ff)?;
    let c = pc.pcurve.get().ok_or_else(|| no_pcurve(e))?;
    c.value(if first { pc.first } else { pc.last })
}

/// The parameter on `e` of the vertex `v` taken INTERNAL.
fn internal_parameter(v: &Shape, e: &Edge) -> Result<f64, Failure> {
    BRepTool::parameter(&topods::vertex(&v.oriented(Orientation::Internal))?, e)
}

/// `Bubble(E, Seq)` (static): the duplicates (IsEqual) removed, then the vertices
/// bubble sorted by their parameter on `e`.
pub fn bubble(e: &Edge, seq: &mut Vec<Shape>) -> Result<(), Failure> {
    let mut i = 0;
    while i + 1 < seq.len() {
        let mut j = i + 1;
        while j < seq.len() {
            if seq[i].is_equal(&seq[j]) {
                seq.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
    let mut invert = true;
    while invert {
        invert = false;
        for i in 0..seq.len().saturating_sub(1) {
            if internal_parameter(&seq[i + 1], e)? < internal_parameter(&seq[i], e)? {
                seq.swap(i, i + 1);
                invert = true;
            }
        }
    }
    Ok(())
}

/// `UpdateClosedEdge(E, SV)` (static): the end of `sv` on the closing vertex of the
/// closed edge (within the tolerance of its first vertex), removed from `sv` (both ends
/// when they are the same vertex); null when none.
pub fn update_closed_edge(e: &Edge, sv: &mut Vec<Shape>) -> Result<Vertex, Failure> {
    let (v1, _) = topexp::vertices(e, false);
    let tol = BRepTool::vertex_tolerance(&v1)?;
    let mut v_res = Vertex::default();
    let (Some(first), Some(last)) = (sv.first().cloned(), sv.last().cloned()) else {
        return Ok(v_res);
    };
    let vb = [topods::vertex(&first)?, topods::vertex(&last)?];
    let pc = BRepTool::pnt(&v1)?;
    let mut on = [false, false];
    for i in 0..2 {
        if BRepTool::pnt(&vb[i])?.is_equal(&pc, tol) {
            v_res = vb[i].clone();
            on[i] = true;
        }
    }
    match on {
        [true, true] if vb[0].as_ref().is_same(vb[1].as_ref()) => {
            sv.remove(0);
            sv.pop();
        }
        [true, false] => {
            sv.remove(0);
        }
        [false, true] => {
            sv.pop();
        }
        _ => {}
    }
    Ok(v_res)
}

/// One pass of RemovePendingEdges: the vertices without edge, or on one open edge
/// (then removed with it); whether something was removed.
fn remove_pending_pass(mve: &mut IndexedDataMapOfShapeListOfShape) -> Result<bool, Failure> {
    let mut v_to_remove: Vec<Shape> = Vec::new();
    let mut e_to_remove = MapOfShape::default();
    for iv in 1..=mve.len() {
        let edges = mve.find_from_index(iv)?;
        if edges.is_empty() {
            v_to_remove.push(mve.find_key(iv)?.clone());
        }
        if edges.len() == 1
            && let Some(first) = edges.first()
        {
            let (v1, v2) = topexp::vertices(&topods::edge(first)?, false);
            if !v1.as_ref().is_same(v2.as_ref()) {
                v_to_remove.push(mve.find_key(iv)?.clone());
                e_to_remove.add(first.clone());
            }
        }
    }
    if v_to_remove.is_empty() {
        return Ok(false);
    }
    for v in &v_to_remove {
        mve.remove_key(v);
    }
    if !e_to_remove.is_empty() {
        for iv in 1..=mve.len() {
            mve.change_from_index(iv)?
                .retain(|e| !e_to_remove.contains(e));
        }
    }
    Ok(true)
}

/// `RemovePendingEdges(MVE)` (static): the hanging edges and their free vertices
/// removed until none remains.
pub fn remove_pending_edges(mve: &mut IndexedDataMapOfShapeListOfShape) -> Result<(), Failure> {
    while remove_pending_pass(mve)? {}
    Ok(())
}

/// `SamePnt2d(V, E1, E2, F)` (static): the start of `e1` and the end of `e2` on the
/// FORWARD face are within 100 times the tolerance of `v`.
pub fn same_pnt2d(v: &Vertex, e1: &Edge, e2: &Edge, f: &Face) -> Result<bool, Failure> {
    let ff = topods::face(&f.as_ref().oriented(Orientation::Forward))?;
    let p1 = pcurve_end(e1, &ff, e1.as_ref().orientation() == Orientation::Forward)?;
    let p2 = pcurve_end(e2, &ff, e2.as_ref().orientation() != Orientation::Forward)?;
    Ok(p1.distance(p2) < 100. * BRepTool::vertex_tolerance(v)?)
}

/// The index (from 1) of the edge of `le` other than `ce` whose start is the nearest
/// (the last of equal ones) to the end of `ce`, within 100 times the tolerance of `cv`;
/// 0 when none.
fn nearest_next_edge(
    ff: &Face,
    ce: &Edge,
    cv: &Vertex,
    le: &List<Shape>,
) -> Result<usize, Failure> {
    let pv = pcurve_end(ce, ff, ce.as_ref().orientation() != Orientation::Forward)?;
    let (mut kmin, mut distmin) = (0, 100. * BRepTool::vertex_tolerance(cv)?);
    for (k, s) in le.iter().enumerate() {
        let e = topods::edge(s)?;
        if e.as_ref().is_same(ce.as_ref()) {
            continue;
        }
        let dist = pv.distance(pcurve_end(
            &e,
            ff,
            e.as_ref().orientation() == Orientation::Forward,
        )?);
        if dist <= distmin {
            (kmin, distmin) = (k + 1, dist);
        }
    }
    Ok(kmin)
}

/// `SelectEdge(F, CE, CV, NE, LE)` (static): `ce` (IsEqual) removed from `le`, then the
/// next edge taken from `le`: the only one, or the nearest in UV among several; `None`
/// when none qualifies.
pub fn select_edge(
    f: &Face,
    ce: &Edge,
    cv: &Vertex,
    le: &mut List<Shape>,
) -> Result<Option<Edge>, Failure> {
    let mut removed = false;
    le.retain(|s| {
        let drop = !removed && s.is_equal(ce.as_ref());
        removed |= drop;
        !drop
    });
    if le.len() > 1 {
        let ff = topods::face(&f.as_ref().oriented(Orientation::Forward))?;
        let kmin = nearest_next_edge(&ff, ce, cv, le)?;
        if kmin == 0 {
            return Ok(None);
        }
        let mut items: Vec<Shape> = le.iter().cloned().collect();
        let ne = items.remove(kmin - 1);
        *le = items.into_iter().collect();
        return Ok(Some(topods::edge(&ne)?));
    }
    let Some(first) = le.first().cloned() else {
        return Ok(None);
    };
    le.remove_first()?;
    Ok(Some(topods::edge(&first)?))
}

/// `PurgeNewEdges(NewEdges, UsedEdges)` (static): the new edges not used removed.
pub fn purge_new_edges(
    new_edges: &mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
    used_edges: &MapOfShape,
) {
    for lne in new_edges.values_mut() {
        lne.retain(|ne| used_edges.contains(ne));
    }
}

/// The substitution of `v1` by `v` recorded (chains of substitutes kept consistent).
fn record_substitute(subst: &mut VertexSubstitutes, v1: &Shape, v: &Shape) {
    if let Some(old_new_v) = subst.seek(v1).cloned() {
        if !old_new_v.is_same(v) {
            subst.bind(old_new_v, v.clone());
            subst.bind(v1.clone(), v.clone());
        }
        return;
    }
    if let Some(new_new_v) = subst.seek(v).cloned() {
        if !new_new_v.is_same(v1) {
            subst.bind(v1.clone(), new_new_v);
        }
        return;
    }
    subst.bind(v1.clone(), v.clone());
    for value in subst.values_mut() {
        if value.is_same(v1) {
            *value = v.clone();
        }
    }
}

/// The vertices of `e` at the point of a vertex `v` of the map (within `tol_conf`)
/// replaced by `v` (with their orientation), and the substitutions recorded.
fn substitute_vertices(
    e: &mut Shape,
    mve: &IndexedDataMapOfShapeListOfShape,
    subst: &mut VertexSubstitutes,
    tol_conf: f64,
) -> Result<(), Failure> {
    let bb = BRepBuilder::default();
    for iv in 1..=mve.len() {
        let mut v = mve.find_key(iv)?.clone();
        let p = BRepTool::pnt(&topods::vertex(&v)?)?;
        let v_list: Vec<Shape> = ShapeIterator::new(e, true, true).collect();
        for v1 in &v_list {
            let p1 = BRepTool::pnt(&topods::vertex(v1)?)?;
            if !p.is_equal(&p1, tol_conf) || v.is_same(v1) {
                continue;
            }
            v.set_orientation(v1.orientation());
            record_substitute(subst, v1, &v);
            e.set_free(true)?;
            bb.remove(e, v1)?;
            bb.add(e, &v)?;
        }
    }
    Ok(())
}

/// `E` appended to the edges of the vertex `v` (a new key when absent).
fn append_edge(
    mve: &mut IndexedDataMapOfShapeListOfShape,
    v: &Vertex,
    e: Shape,
) -> Result<(), Failure> {
    if !mve.contains(v.as_ref()) {
        mve.add(v.as_ref().clone(), List::default());
    }
    mve.change_from_key(v.as_ref())?.append(e);
    Ok(())
}

/// `StoreInMVE(F, E, MVE, YaCouture, VerticesForSubstitute, theTolConf)` (static): the
/// vertices of `e` merged into the coincident ones of the map, then `e` stored at its
/// vertices (and reversed again for a seam of the face, setting `ya_couture`; a
/// vertexless edge clears it).
pub fn store_in_mve(
    f: &Face,
    e: &mut Shape,
    mve: &mut IndexedDataMapOfShapeListOfShape,
    ya_couture: &mut bool,
    subst: &mut VertexSubstitutes,
    tol_conf: f64,
) -> Result<(), Failure> {
    substitute_vertices(e, mve, subst, tol_conf)?;
    let edge = topods::edge(e)?;
    let (v1, v2) = topexp::vertices(&edge, false);
    if v1.as_ref().is_null() && v2.as_ref().is_null() {
        *ya_couture = false;
        return Ok(());
    }
    append_edge(mve, &v1, e.clone())?;
    if !v1.as_ref().is_same(v2.as_ref()) {
        append_edge(mve, &v2, e.clone())?;
    }
    let (s, l) = BRepTool::surface_with_location(f)?;
    if BRepTool::is_closed_on_surface(&edge, &s, &l)? {
        mve.change_from_key(v2.as_ref())?.append(e.reversed());
        if !v1.as_ref().is_same(v2.as_ref()) {
            mve.change_from_key(v1.as_ref())?.append(e.reversed());
        }
        *ya_couture = true;
    }
    Ok(())
}
