use super::{CoupleOfShape, get_edge_on_face, get_face_off};
use crate::int_tools::Context;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::{ncollection::List, standard::Failure};
use rustycad_tkg3d::top_abs::Orientation;

/// Return 0 outside, 1 inside, or 2 ambiguous between bounding faces; `internal_face_between(&face,&edge,&first,&last,&context)?`.
pub fn internal_face_between(
    face: &Face,
    edge: &Edge,
    first: &Face,
    last: &Face,
    context: &Context,
) -> Result<i32, Failure> {
    let (first_edge, last_edge) = boundary_edges(edge, first, last)?;
    let mut pairs = List::default();
    pairs.append(edge_face_pair(edge, face));
    pairs.append(edge_face_pair(&last_edge, last));
    let mut selected = Face::default();
    if !get_face_off(&first_edge, first, &pairs, &mut selected, context)? {
        return Ok(2);
    }
    Ok(i32::from(face.is_equal(&selected)))
}
fn boundary_edges(edge: &Edge, first: &Face, last: &Face) -> Result<(Edge, Edge), Failure> {
    let mut first_edge = Edge::default();
    get_edge_on_face(edge, first, &mut first_edge)?;
    if first_edge.orientation() == Orientation::Internal || first.is_equal(last) {
        let mut last_edge = first_edge.clone();
        first_edge.set_orientation(Orientation::Forward);
        last_edge.set_orientation(Orientation::Reversed);
        return Ok((first_edge, last_edge));
    }
    let mut last_edge = Edge::default();
    get_edge_on_face(edge, last, &mut last_edge)?;
    Ok((first_edge, last_edge))
}
pub(super) fn edge_face_pair(edge: &Edge, face: &Face) -> CoupleOfShape {
    let mut pair = CoupleOfShape::new();
    pair.set_shape1(edge);
    pair.set_shape2(face);
    pair
}
