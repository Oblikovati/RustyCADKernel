use super::internal_face::edge_face_pair;
use super::{CoupleOfShape, get_edge_on_face, get_face_off, internal_face_between};
use crate::int_tools::Context;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::{ncollection::List, standard::Failure};
/// Classify against native ordered face pairs; `internal_face_among(&face,&edge,&faces,&context)?`.
pub fn internal_face_among(
    face: &Face,
    edge: &Edge,
    supports: &List<Shape>,
    context: &Context,
) -> Result<i32, Failure> {
    if supports.len() == 2 {
        let first = Face::unchecked(supports.first().expect("two supports").clone());
        let last = Face::unchecked(supports.last().expect("two supports").clone());
        return internal_face_between(face, edge, &first, &last, context);
    }
    for (first, last) in face_pairs(edge, supports, context)? {
        let state = internal_face_between(face, edge, &first, &last, context)?;
        if state != 0 {
            return Ok(state);
        }
    }
    Ok(0)
}
fn face_pairs(
    edge: &Edge,
    supports: &List<Shape>,
    context: &Context,
) -> Result<Vec<(Face, Face)>, Failure> {
    let Some(mut remaining) = edge_occurrences(edge, supports)? else {
        return Ok(Vec::new());
    };
    let mut paired: Vec<Shape> = Vec::new();
    let mut selected = Face::default();
    let mut result = Vec::new();
    while !remaining.is_empty() {
        let first = remaining[0].clone();
        let candidates = opposite_occurrences(&remaining, &mut paired);
        let edge = Edge::unchecked(first.shape1().clone());
        let face = Face::unchecked(first.shape2().clone());
        // Native retains selected across iterations and ignores GetFaceOff's ambiguity flag.
        get_face_off(&edge, &face, &candidates, &mut selected, context)?;
        result.push((face.clone(), selected.clone()));
        paired.extend([face.into_shape(), selected.clone().into_shape()]);
        remaining.retain(|pair| !paired.iter().any(|shape| shape.is_same(pair.shape2())));
    }
    Ok(result)
}
fn edge_occurrences(
    edge: &Edge,
    supports: &List<Shape>,
) -> Result<Option<Vec<CoupleOfShape>>, Failure> {
    let mut result = Vec::new();
    let mut occurrence = Edge::default();
    for shape in supports.iter() {
        let face = Face::unchecked(shape.clone());
        if !get_edge_on_face(edge, &face, &mut occurrence)? {
            return Ok(None);
        }
        result.push(edge_face_pair(&occurrence, &face));
    }
    Ok(Some(result))
}
fn opposite_occurrences(
    remaining: &[CoupleOfShape],
    paired: &mut Vec<Shape>,
) -> List<CoupleOfShape> {
    let first = &remaining[0];
    let orientation = rustycad_tkg3d::top_abs::reverse(first.shape1().orientation());
    paired.push(first.shape2().clone());
    let mut result = List::default();
    for pair in &remaining[1..] {
        if pair.shape1().orientation() == orientation {
            result.append(pair.clone());
            // OCCT fences every opposite face here, not only the eventual selected face.
            paired.push(pair.shape2().clone());
        }
    }
    result
}
