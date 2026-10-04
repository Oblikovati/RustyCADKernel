use super::{CandidatePair, MergePolicy};
use crate::{
    shape_analysis::ShapeAnalysisWire,
    shape_build::ShapeBuildReShape,
    shape_extend::{ShapeExtend, ShapeExtendStatus, ShapeExtendWireData},
    shape_fix::{FixRoot, ShapeFixWire, wireframe::ShapeFixWireframe},
};
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topods::{Edge, Shape},
    toptools::MapOfShape,
};
use rustycad_tkernel::{
    message::Msg,
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn drop_small_edge(
    fixer: &ShapeFixWireframe,
    wire_fixer: &ShapeFixWire,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    pair: &CandidatePair,
    policy: &mut MergePolicy,
) -> Result<Option<i32>, Failure> {
    let index = pair.middle_index;
    let new_index = checked_drop_index(wire_data, index, fixer.precision())?;
    let Some(new_index) = new_index else {
        return Ok(None);
    };
    remove_middle_edge(fixer, wire_fixer, wire_data, small, pair, new_index, policy)?;
    Ok(Some(index))
}

fn checked_drop_index(
    wire_data: &Handle<ShapeExtendWireData>,
    index: i32,
    precision: f64,
) -> Result<Option<i32>, Failure> {
    let temporary = ShapeExtendWireData::new();
    temporary.init(wire_data)?;
    temporary.remove(index)?;
    if temporary.nb_edges() == 0 {
        return Ok(None);
    }
    let new_index = if index <= temporary.nb_edges() {
        index
    } else {
        1
    };
    let analyzer = ShapeAnalysisWire::new();
    analyzer.load_wire_data(&Handle::new(temporary));
    analyzer.check_connected_at(new_index, precision)?;
    if analyzer.last_check_status(ShapeExtendStatus::Fail) {
        return Ok(None);
    }
    Ok(Some(new_index))
}

fn remove_middle_edge(
    fixer: &ShapeFixWireframe,
    wire_fixer: &ShapeFixWire,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    pair: &CandidatePair,
    new_index: i32,
    policy: &mut MergePolicy,
) -> Result<(), Failure> {
    let index = pair.middle_index;
    let old_count = wire_data.nb_edges();
    let before = wire_data.edge(if index == 1 { old_count } else { index - 1 })?;
    let after = wire_data.edge(if index == old_count { 1 } else { index + 1 })?;
    let removed = wire_data.edge(index)?;
    wire_data.remove(index)?;
    if let Some(face) = pair.faces.first() {
        wire_fixer.set_face(face)?;
    }
    wire_fixer.fix_connected_at(new_index, fixer.precision(), true)?;
    wire_fixer.fix_degenerated_at(new_index)?;
    publish_drop(fixer, small, &removed, &before, &after, policy)
}

fn publish_drop(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    removed: &Edge,
    before: &Edge,
    after: &Edge,
    policy: &mut MergePolicy,
) -> Result<(), Failure> {
    let context = fixer.context();
    let removed_shape = removed.clone().into_shape();
    ReShape::remove(&*context, &removed_shape)?;
    send_drop_warning(fixer, &removed_shape);
    small.remove(&removed_shape);
    policy
        .edge_faces
        .retain(|(edge, _)| !edge.is_same(&removed_shape));
    replace_neighbor_links(policy, &context, small, before, after)?;
    fixer.state.write().small |= ShapeExtend::encode_status(ShapeExtendStatus::Done3);
    Ok(())
}

fn replace_neighbor_links(
    policy: &mut MergePolicy,
    context: &Handle<ShapeBuildReShape>,
    small: &mut MapOfShape,
    before: &Edge,
    after: &Edge,
) -> Result<(), Failure> {
    let before_shape = before.clone().into_shape();
    let after_shape = after.clone().into_shape();
    let before_faces = take_edge_faces(policy, &before_shape);
    let after_faces = take_edge_faces(policy, &after_shape);
    bind_applied_edge(policy, context, small, &before_shape, before_faces)?;
    bind_applied_edge(policy, context, small, &after_shape, after_faces)
}

fn take_edge_faces(policy: &mut MergePolicy, edge: &Shape) -> Vec<Shape> {
    let position = policy
        .edge_faces
        .iter()
        .position(|(candidate, _)| candidate.is_same(edge));
    position.map_or_else(Vec::new, |index| policy.edge_faces.remove(index).1)
}

fn bind_applied_edge(
    policy: &mut MergePolicy,
    context: &Handle<ShapeBuildReShape>,
    small: &mut MapOfShape,
    source: &Shape,
    faces: Vec<Shape>,
) -> Result<(), Failure> {
    let applied = ReShape::apply(&**context, source, ShapeEnum::Shape)?;
    if applied.is_null() || applied.shape_type()? != ShapeEnum::Edge {
        return Ok(());
    }
    if small.contains(source) {
        small.remove(source);
        small.add(applied.clone());
    }
    policy.edge_faces.push((applied, faces));
    Ok(())
}

fn send_drop_warning(fixer: &ShapeFixWireframe, edge: &Shape) {
    let key = AsciiString::from("FixWireframe.FixSmallEdges.MSG0");
    fixer.send_warning(edge, &Msg::from_key(&key));
}
