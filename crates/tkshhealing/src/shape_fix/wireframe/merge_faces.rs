use super::{
    ShapeFixWireframe, ShapeFixWireframeShapeLists,
    merge_free_wires::{MergePolicy, merge_wire_with_faces},
    repair::unported_small_edge_merge,
};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_fix::root::FixRoot;
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Face, Shape, ShapeIterator, Wire},
    toptools::MapOfShape,
};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::ShapeEnum;

struct FaceMergeSettings {
    multiple_edges: Vec<Shape>,
    angle_limit: Option<f64>,
    drop_mode: bool,
}

pub(super) fn merge_face_small_edges(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    edge_faces: &mut ShapeFixWireframeShapeLists,
    face_small: &mut ShapeFixWireframeShapeLists,
    multiple: &MapOfShape,
    angle_limit: Option<f64>,
    drop_mode: bool,
) -> Result<bool, Failure> {
    if face_small.is_empty() {
        return Ok(false);
    }
    let settings = face_merge_settings(multiple, angle_limit, drop_mode);
    let mut links = edge_face_entries(edge_faces);
    let (changed, face_small_changed) =
        merge_face_groups(fixer, small, face_small, &mut links, &settings)?;
    publish_face_maps(
        edge_faces,
        face_small,
        small,
        &links,
        changed,
        face_small_changed,
    );
    finish_face_groups(fixer, changed)
}

fn merge_face_groups(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    face_small: &ShapeFixWireframeShapeLists,
    links: &mut Vec<(Shape, Vec<Shape>)>,
    settings: &FaceMergeSettings,
) -> Result<(bool, bool), Failure> {
    let mut changed = false;
    let mut face_small_changed = false;
    for source in source_faces(fixer)? {
        let (group_changed, group_map_changed) =
            merge_face_group(fixer, small, &source, links, face_small, settings)?;
        changed |= group_changed;
        face_small_changed |= group_map_changed;
    }
    Ok((changed, face_small_changed))
}

fn publish_face_maps(
    edge_faces: &mut ShapeFixWireframeShapeLists,
    face_small: &mut ShapeFixWireframeShapeLists,
    small: &MapOfShape,
    links: &[(Shape, Vec<Shape>)],
    changed: bool,
    face_small_changed: bool,
) {
    if !changed {
        return;
    }
    replace_edge_face_entries(edge_faces, links);
    if face_small_changed {
        refresh_face_small_edges(face_small, edge_faces, small);
    }
}

fn face_merge_settings(
    multiple: &MapOfShape,
    angle_limit: Option<f64>,
    drop_mode: bool,
) -> FaceMergeSettings {
    FaceMergeSettings {
        multiple_edges: multiple.iter().cloned().collect(),
        angle_limit,
        drop_mode,
    }
}

fn source_faces(fixer: &ShapeFixWireframe) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(&fixer.shape(), ShapeEnum::Face, ShapeEnum::Shape)?.collect())
}

fn merge_face_group(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    source: &Shape,
    links: &mut Vec<(Shape, Vec<Shape>)>,
    face_small: &ShapeFixWireframeShapeLists,
    settings: &FaceMergeSettings,
) -> Result<(bool, bool), Failure> {
    let Ok(edges) = face_small.find(source) else {
        return Ok((false, false));
    };
    if edges.is_empty() {
        return Ok((false, false));
    }
    merge_one_face(fixer, small, source, links, settings)
}

fn replace_edge_face_entries(
    edge_faces: &mut ShapeFixWireframeShapeLists,
    links: &[(Shape, Vec<Shape>)],
) {
    edge_faces.clear(false);
    for (edge, faces) in links {
        edge_faces.bind(edge.clone(), shape_list(faces));
    }
}

fn shape_list(shapes: &[Shape]) -> List<Shape> {
    let mut list = List::default();
    for shape in shapes {
        list.append(shape.clone());
    }
    list
}

fn refresh_face_small_edges(
    face_small: &mut ShapeFixWireframeShapeLists,
    edge_faces: &ShapeFixWireframeShapeLists,
    small: &MapOfShape,
) {
    let mut empty_faces = Vec::new();
    for (face, edges) in face_small.items_mut() {
        refresh_small_edge_list(face, edges, edge_faces, small);
        if edges.is_empty() {
            empty_faces.push(face.clone());
        }
    }
    for face in empty_faces {
        face_small.unbind(&face);
    }
}

fn refresh_small_edge_list(
    face: &Shape,
    edges: &mut List<Shape>,
    edge_faces: &ShapeFixWireframeShapeLists,
    small: &MapOfShape,
) {
    edges.retain(|edge| small.contains(edge));
    for (edge, faces) in edge_faces.items() {
        if small.contains(edge) && faces.iter().any(|linked| linked.is_same(face)) {
            append_missing_edge(edges, edge);
        }
    }
}

fn append_missing_edge(edges: &mut List<Shape>, edge: &Shape) {
    if !edges.iter().any(|existing| existing.is_same(edge)) {
        edges.append(edge.clone());
    }
}

fn finish_face_groups(fixer: &ShapeFixWireframe, changed: bool) -> Result<bool, Failure> {
    if !changed {
        return Ok(false);
    }
    super::merge_free_wires::finish_free_wire_merge(fixer, &fixer.context())
}

fn edge_face_entries(edge_faces: &ShapeFixWireframeShapeLists) -> Vec<(Shape, Vec<Shape>)> {
    edge_faces
        .items()
        .map(|(edge, faces)| (edge.clone(), faces.iter().cloned().collect()))
        .collect()
}

fn merge_one_face(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    source: &Shape,
    edge_faces: &mut Vec<(Shape, Vec<Shape>)>,
    settings: &FaceMergeSettings,
) -> Result<(bool, bool), Failure> {
    let context = fixer.context();
    if context.is_null() {
        return Err(unported_small_edge_merge());
    }
    let face = applied_face(source, &context)?;
    let (mut changed, map_changed) =
        merge_face_wires(fixer, &face, &context, small, edge_faces, settings)?;
    changed |= remove_empty_face(fixer, source, &context)?;
    Ok((changed, map_changed))
}

fn merge_face_wires(
    fixer: &ShapeFixWireframe,
    face: &Face,
    context: &Handle<ShapeBuildReShape>,
    small: &mut MapOfShape,
    edge_faces: &mut Vec<(Shape, Vec<Shape>)>,
    settings: &FaceMergeSettings,
) -> Result<(bool, bool), Failure> {
    let mut changed = false;
    let mut map_changed = false;
    for wire in face_wires(face)? {
        let (wire_changed, wire_map_changed) =
            merge_face_wire(fixer, face, &wire, context, small, edge_faces, settings)?;
        changed |= wire_changed;
        map_changed |= wire_map_changed;
    }
    Ok((changed, map_changed))
}

fn remove_empty_face(
    fixer: &ShapeFixWireframe,
    source: &Shape,
    context: &Handle<ShapeBuildReShape>,
) -> Result<bool, Failure> {
    let applied = ReShape::apply(&**context, source, ShapeEnum::Shape)?;
    if applied.is_null() || ShapeIterator::new(&applied, true, true).more() {
        return Ok(false);
    }
    ReShape::remove(&**context, &applied)?;
    send_empty_face_warning(fixer, source);
    Ok(true)
}

fn send_empty_face_warning(fixer: &ShapeFixWireframe, source: &Shape) {
    use rustycad_tkernel::{message::Msg, tcollection::AsciiString};
    let key = AsciiString::from("FixWireframe.FixSmallEdges.MSG2");
    fixer.send_warning(source, &Msg::from_key(&key));
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
    edge_faces: &mut Vec<(Shape, Vec<Shape>)>,
    settings: &FaceMergeSettings,
) -> Result<(bool, bool), Failure> {
    let source = wire.clone();
    let current = applied_wire(&source, context)?;
    let mut policy = face_merge_policy(face, edge_faces, settings);
    let changed = merge_wire_with_faces(fixer, &source, &current, context, small, &mut policy)?;
    *edge_faces = policy.edge_faces;
    Ok((changed, policy.face_small_changed))
}

fn face_merge_policy(
    face: &Face,
    edge_faces: &[(Shape, Vec<Shape>)],
    settings: &FaceMergeSettings,
) -> MergePolicy {
    MergePolicy::face(
        face,
        edge_faces.to_vec(),
        settings.angle_limit,
        settings.multiple_edges.clone(),
        settings.drop_mode,
    )
}

fn applied_wire(source: &Wire, context: &Handle<ShapeBuildReShape>) -> Result<Wire, Failure> {
    let shape = ReShape::apply(&**context, &source.clone().into_shape(), ShapeEnum::Shape)?;
    topods::wire(&shape)
}
