use super::{
    ShapeFixWireframe, ShapeFixWireframeShapeLists,
    merge_free_wires::{MergePolicy, merge_wire_with_faces},
    repair::unported_small_edge_merge,
};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_fix::root::FixRoot;
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topods::{self, Face, Shape, ShapeIterator, Wire},
    toptools::MapOfShape,
};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn merge_face_small_edges(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    edge_faces: &ShapeFixWireframeShapeLists,
    face_small: &ShapeFixWireframeShapeLists,
    angle_limit: Option<f64>,
) -> Result<bool, Failure> {
    if face_small.is_empty() {
        return Ok(false);
    }
    let (source, edges) = single_face_entry(face_small)?;
    validate_face_edge_links(&source, edges, edge_faces)?;
    let links = edge_face_entries(edge_faces);
    merge_one_face(fixer, small, &source, &links, angle_limit)
}

fn edge_face_entries(edge_faces: &ShapeFixWireframeShapeLists) -> Vec<(Shape, Vec<Shape>)> {
    edge_faces
        .items()
        .map(|(edge, faces)| (edge.clone(), faces.iter().cloned().collect()))
        .collect()
}

fn single_face_entry(
    face_small: &ShapeFixWireframeShapeLists,
) -> Result<(Shape, &List<Shape>), Failure> {
    if face_small.len() != 1 {
        return Err(unported_small_edge_merge());
    }
    let (face, edges) = face_small
        .items()
        .next()
        .ok_or_else(unported_small_edge_merge)?;
    Ok((face.clone(), edges))
}

fn validate_face_edge_links(
    face: &Shape,
    edges: &List<Shape>,
    edge_faces: &ShapeFixWireframeShapeLists,
) -> Result<(), Failure> {
    for edge in edges.iter() {
        if !contains_face_link(face, edge, edge_faces)? {
            return Err(unported_small_edge_merge());
        }
    }
    Ok(())
}

fn contains_face_link(
    face: &Shape,
    edge: &Shape,
    edge_faces: &ShapeFixWireframeShapeLists,
) -> Result<bool, Failure> {
    let Ok(faces) = edge_faces.find(edge) else {
        return Ok(false);
    };
    Ok(faces.iter().any(|linked| linked.is_same(face)))
}

fn merge_one_face(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    source: &Shape,
    edge_faces: &[(Shape, Vec<Shape>)],
    angle_limit: Option<f64>,
) -> Result<bool, Failure> {
    let context = fixer.context();
    if context.is_null() {
        return Err(unported_small_edge_merge());
    }
    let face = applied_face(source, &context)?;
    let mut changed = false;
    for wire in face_wires(&face)? {
        changed |= merge_face_wire(
            fixer,
            &face,
            &wire,
            &context,
            small,
            edge_faces,
            angle_limit,
        )?;
    }
    if !changed {
        return Ok(false);
    }
    super::merge_free_wires::finish_free_wire_merge(fixer, &context)
}

fn applied_face(source: &Shape, context: &Handle<ShapeBuildReShape>) -> Result<Face, Failure> {
    let applied = ReShape::apply(&**context, source, ShapeEnum::Shape)?;
    topods::face(&applied)
}

fn face_wires(face: &Face) -> Result<Vec<Wire>, Failure> {
    ShapeIterator::new(&face.clone().into_shape(), true, true)
        .filter_map(|child| match child.shape_type() {
            Ok(ShapeEnum::Wire) => Some(topods::wire(&child)),
            _ => None,
        })
        .collect()
}

fn merge_face_wire(
    fixer: &ShapeFixWireframe,
    face: &Face,
    wire: &Wire,
    context: &Handle<ShapeBuildReShape>,
    small: &mut MapOfShape,
    edge_faces: &[(Shape, Vec<Shape>)],
    angle_limit: Option<f64>,
) -> Result<bool, Failure> {
    let source = wire.clone();
    let current = applied_wire(&source, context)?;
    let mut policy = MergePolicy::face(face, edge_faces.to_vec(), angle_limit);
    merge_wire_with_faces(fixer, &source, &current, context, small, &mut policy)
}

fn applied_wire(source: &Wire, context: &Handle<ShapeBuildReShape>) -> Result<Wire, Failure> {
    let shape = ReShape::apply(&**context, &source.clone().into_shape(), ShapeEnum::Shape)?;
    topods::wire(&shape)
}
