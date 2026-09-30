// Rust translation of the static replaceNMVertices and ReplaceEdge
// (BRepBuilderAPI_Sewing.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The replacement of a boundary by its merged edge (or wire of sections) in a
//! re-shaper, with the vertices it replaces and its non-manifold vertices.
use super::vertex_tolerance::find_nm_vertices;
use rustycad_tkbrep::brep::access::vertex_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepPointOnCurve, BRepPointRepresentation, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::Location;

/// A copy of a non-manifold vertex with a point on the edge's curve at `par`.
fn vertex_on_curve(
    vertex: &Shape,
    par: f64,
    (c3d, location): (&Handle<dyn GeomCurve>, &Location),
) -> Result<Shape, Failure> {
    let new_vertex = vertex.empty_copied()?;
    let point: Handle<dyn BRepPointRepresentation> =
        Handle::new(BRepPointOnCurve::new(par, c3d, location)).cast();
    vertex_tshape(&new_vertex, "BRep_TVertex::ChangePoints")?
        .change_points(|points| points.push(point));
    Ok(new_vertex)
}

/// The static `replaceNMVertices(theEdge, theV1, theV2, theReShape)`: each
/// non-manifold vertex of the edge at an end is replaced by that end, one at the
/// parameter of an earlier one by it, any other by a copy with a point on the curve
/// (the copies kept ordered by parameter); when vertices were merged, the edge is
/// replaced by a copy with the two ends and the copies.
fn replace_nm_vertices(
    edge: &Edge,
    v1: &Vertex,
    v2: &Vertex,
    reshape: &dyn ReShape,
) -> Result<(), Failure> {
    let (mut nm_vertices, mut nm_pars) = (Sequence::default(), Sequence::default());
    if !find_nm_vertices(edge, &mut nm_vertices, &mut nm_pars)? {
        return Ok(());
    }
    BRepTool::range(edge)?;
    let found = BRepTool::curve_with_location(edge)?;
    if found.curve.is_null() {
        return Ok(());
    }
    let (first, last) = (found.first, found.last);
    let curve = (&found.curve, &found.location);
    let (mut ed_vert, mut ed_params): (Sequence<Shape>, Sequence<f64>) =
        (Sequence::default(), Sequence::default());
    let nb = nm_pars.len();
    for i in 1..=nb {
        let par = *nm_pars.value(i)?;
        let vertex = nm_vertices.value(i)?.clone();
        if (par - first).abs() <= precision::p_confusion() {
            reshape.replace(&vertex, v1)?;
            continue;
        }
        if (par - last).abs() <= precision::p_confusion() {
            reshape.replace(&vertex, v2)?;
            continue;
        }
        place_nm_vertex(&vertex, par, curve, (&mut ed_vert, &mut ed_params), reshape)?;
    }
    if ed_params.len() >= nb {
        return Ok(());
    }
    let b = BRepBuilder::default();
    let mut new_edge = edge.empty_copied()?;
    let orientation = edge.orientation();
    new_edge.set_orientation(Orientation::Forward);
    b.add(&mut new_edge, v1)?;
    b.add(&mut new_edge, v2)?;
    for vertex in ed_vert.iter() {
        b.add(&mut new_edge, vertex)?;
    }
    new_edge.set_orientation(orientation);
    reshape.replace(edge, &new_edge)
}

/// One inner non-manifold vertex: merged into a copy at the same parameter, or
/// replaced by a new copy inserted in parameter order.
fn place_nm_vertex(
    vertex: &Shape,
    par: f64,
    curve: (&Handle<dyn GeomCurve>, &Location),
    (ed_vert, ed_params): (&mut Sequence<Shape>, &mut Sequence<f64>),
    reshape: &dyn ReShape,
) -> Result<(), Failure> {
    for j in 1..=ed_params.len() {
        let par2 = *ed_params.value(j)?;
        if (par - par2).abs() <= precision::p_confusion() {
            return reshape.replace(vertex, ed_vert.value(j)?);
        }
        if par < par2 {
            let new_vertex = vertex_on_curve(vertex, par, curve)?;
            ed_vert.insert_before(j, new_vertex.clone())?;
            ed_params.insert_before(j, par)?;
            return reshape.replace(vertex, &new_vertex);
        }
    }
    let new_vertex = vertex_on_curve(vertex, par, curve)?;
    ed_vert.append(new_vertex.clone());
    ed_params.append(par);
    reshape.replace(vertex, &new_vertex)
}

/// The ends of the new shape (an edge, or the first and last edges of a wire), its
/// last orientation, after its non-manifold vertices are placed.
fn new_ends(
    new_shape: &Shape,
    orientation: Orientation,
    reshape: &dyn ReShape,
) -> Result<(Vertex, Vertex, Orientation), Failure> {
    let (mut v1_new, mut v2_new, mut or_new) = (Vertex::default(), Vertex::default(), orientation);
    match new_shape.shape_type()? {
        ShapeEnum::Edge => {
            let en = topods::edge(new_shape)?;
            (v1_new, v2_new) = topexp::vertices(&en, false);
            or_new = en.orientation();
            replace_nm_vertices(&en, &v1_new, &v2_new, reshape)?;
        }
        ShapeEnum::Wire => {
            for e in Explorer::new(new_shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let ed = topods::edge(&e)?;
                or_new = ed.orientation();
                let (v1, v2) = topexp::vertices(&ed, false);
                replace_nm_vertices(&ed, &v1, &v2, reshape)?;
                if v1_new.is_null() {
                    v1_new = v1;
                }
                v2_new = v2;
            }
        }
        _ => {}
    }
    Ok((v1_new, v2_new, or_new))
}

/// The static `ReplaceEdge(oldEdge, theNewShape, aReShape)`: the images of the old
/// edge and of the new shape; unless they are the same or the new one is already
/// recorded, the old one is replaced, then its vertices by the ends of the new one (in
/// the same order when the orientations agree, swapped otherwise), except those
/// already the same or recorded.
pub(super) fn replace_edge(
    old_edge: &Shape,
    new_shape: &Shape,
    reshape: &dyn ReShape,
) -> Result<(), Failure> {
    let old_shape = reshape.apply(old_edge, ShapeEnum::Shape)?;
    let new_shape = reshape.apply(new_shape, ShapeEnum::Shape)?;
    if old_shape.is_same(&new_shape) || reshape.is_recorded(&new_shape) {
        return Ok(());
    }
    reshape.replace(&old_shape, &new_shape)?;
    let (v1_old, v2_old) = topexp::vertices(&topods::edge(&old_shape)?, false);
    let or_old = old_shape.orientation();
    let (mut v1_new, mut v2_new, or_new) = new_ends(&new_shape, or_old, reshape)?;
    v1_new.set_orientation(v1_old.orientation());
    v2_new.set_orientation(v2_old.orientation());
    if v1_old.is_same(&v2_old) && !v1_old.is_same(&v1_new) && !reshape.is_recorded(&v1_new) {
        return reshape.replace(&v1_old, &v1_new);
    }
    if or_old != or_new {
        v1_new.set_orientation(v2_old.orientation());
        v2_new.set_orientation(v1_old.orientation());
        (v1_new, v2_new) = (v2_new, v1_new);
    }
    if !v1_old.is_same(&v1_new) && !v1_old.is_same(&v2_new) && !reshape.is_recorded(&v1_new) {
        reshape.replace(&v1_old, &v1_new)?;
    }
    if !v2_old.is_same(&v2_new) && !v2_old.is_same(&v1_new) && !reshape.is_recorded(&v2_new) {
        reshape.replace(&v2_old, &v2_new)?;
    }
    Ok(())
}
