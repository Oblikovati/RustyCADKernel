mod candidates;
mod non_manifold;
pub(super) use candidates::MergePolicy;
use candidates::{CandidatePair, candidate_pair, linked_face_order_allows_join};

use super::{FixRoot, ShapeFixWireframe, join_3d::join_edges_3d};
use crate::{
    shape_analysis::ShapeAnalysisWire,
    shape_build::ShapeBuildReShape,
    shape_extend::{ShapeExtend, ShapeExtendStatus, ShapeExtendWireData},
    shape_fix::{ShapeFix, ShapeFixWire},
};
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Edge, Wire},
    toptools::MapOfShape,
};
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Failure, Handle},
};

pub(super) fn merge_small_edges(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    multiple: &mut MapOfShape,
    angle_limit: Option<f64>,
    drop_mode: bool,
) -> Result<bool, Failure> {
    let mut policy = MergePolicy::free(angle_limit, drop_mode, multiple.iter().cloned().collect());
    merge_free_wire_candidates(fixer, small, &mut policy)
}

fn merge_free_wire_candidates(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    policy: &mut MergePolicy,
) -> Result<bool, Failure> {
    let context = fixer.context();
    let mut changed = false;
    for source in Explorer::new(
        &fixer.shape(),
        rustycad_tkg3d::top_abs::ShapeEnum::Wire,
        rustycad_tkg3d::top_abs::ShapeEnum::Face,
    )? {
        let wire = topods::wire(&source)?;
        changed |= merge_one_free_wire(fixer, &wire, &context, small, policy)?;
    }
    if !changed {
        return Ok(false);
    }
    finish_free_wire_merge(fixer, &context)
}

fn merge_one_free_wire(
    fixer: &ShapeFixWireframe,
    source: &Wire,
    context: &Handle<ShapeBuildReShape>,
    small: &mut MapOfShape,
    policy: &mut MergePolicy,
) -> Result<bool, Failure> {
    let applied = ReShape::apply(
        &**context,
        &source.clone().into_shape(),
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )?;
    let wire = topods::wire(&applied)?;
    merge_wire_with_faces(fixer, source, &wire, context, small, policy)
}

pub(super) fn merge_wire_with_faces(
    fixer: &ShapeFixWireframe,
    source: &Wire,
    wire: &Wire,
    context: &Handle<ShapeBuildReShape>,
    small: &mut MapOfShape,
    policy: &mut MergePolicy,
) -> Result<bool, Failure> {
    let tool = prepared_wire_fixer(wire, context)?;
    let wire_data = tool.wire_data()?;
    merge_wire_data(fixer, &tool, source, &wire_data, small, policy)
}

fn prepared_wire_fixer(
    wire: &Wire,
    context: &Handle<ShapeBuildReShape>,
) -> Result<ShapeFixWire, Failure> {
    let tool = ShapeFixWire::new();
    tool.set_context(context);
    tool.load(wire)?;
    tool.fix_reorder()?;
    Ok(tool)
}

fn merge_wire_data(
    fixer: &ShapeFixWireframe,
    wire_fixer: &ShapeFixWire,
    source: &Wire,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    policy: &mut MergePolicy,
) -> Result<bool, Failure> {
    let changed = merge_wire_candidates(fixer, wire_data, small, policy)?;
    if changed {
        publish_changed_wire(fixer, wire_fixer, source, wire_data, true)?;
        return Ok(true);
    }
    drop_single_small_edge(fixer, source, wire_data, small, policy)
}

fn drop_single_small_edge(
    fixer: &ShapeFixWireframe,
    source: &Wire,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    policy: &mut MergePolicy,
) -> Result<bool, Failure> {
    if !policy.drop_mode || wire_data.nb_edges() != 1 {
        return Ok(false);
    }
    let edge = wire_data.edge(1)?;
    if !small.contains(&edge.clone().into_shape()) {
        return Ok(false);
    }
    remove_dropped_wire_edge(fixer, source, &edge, small, policy)?;
    Ok(true)
}

fn remove_dropped_wire_edge(
    fixer: &ShapeFixWireframe,
    source: &Wire,
    edge: &Edge,
    small: &mut MapOfShape,
    policy: &mut MergePolicy,
) -> Result<(), Failure> {
    let context = fixer.context();
    ReShape::remove(&*context, &edge.clone().into_shape())?;
    ReShape::remove(&*context, &source.clone().into_shape())?;
    small.remove(&edge.clone().into_shape());
    policy
        .edge_faces
        .retain(|(candidate, _)| !candidate.is_same(&edge.clone().into_shape()));
    fixer.state.write().small |= ShapeExtend::encode_status(ShapeExtendStatus::Done2);
    Ok(())
}

fn merge_wire_candidates(
    fixer: &ShapeFixWireframe,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    policy: &mut MergePolicy,
) -> Result<bool, Failure> {
    let mut index = 1;
    let mut changed = false;
    while index <= wire_data.nb_edges() && wire_data.nb_edges() > 1 {
        let merged_index = merge_small_at_index(fixer, wire_data, small, index, policy)?;
        index = advance_merge_index(index, merged_index, &mut changed);
    }
    Ok(changed)
}

fn advance_merge_index(index: i32, merged: Option<i32>, changed: &mut bool) -> i32 {
    let Some(next_index) = merged else {
        return index + 1;
    };
    *changed = true;
    next_index
}

fn publish_changed_wire(
    fixer: &ShapeFixWireframe,
    wire_fixer: &ShapeFixWire,
    source: &Wire,
    wire_data: &Handle<ShapeExtendWireData>,
    changed: bool,
) -> Result<(), Failure> {
    if !changed {
        return Ok(());
    }
    wire_fixer.fix_connected_all(fixer.precision())?;
    let replacement = wire_data.wire()?;
    ReShape::replace(
        &*fixer.context(),
        &source.clone().into_shape(),
        &replacement.into_shape(),
    )
}

fn merge_small_at_index(
    fixer: &ShapeFixWireframe,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    index: i32,
    policy: &mut MergePolicy,
) -> Result<Option<i32>, Failure> {
    let count = wire_data.nb_edges();
    let previous = if index == 1 { count } else { index - 1 };
    let next = if index == count { 1 } else { index + 1 };
    let pair = candidate_pair(wire_data, small, index, previous, next, &*policy)?;
    let Some(pair) = pair else {
        return Ok(None);
    };
    merge_candidate_pair(fixer, wire_data, small, &pair, policy)
}

fn merge_candidate_pair(
    fixer: &ShapeFixWireframe,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    pair: &CandidatePair,
    policy: &mut MergePolicy,
) -> Result<Option<i32>, Failure> {
    if !linked_face_order_allows_join(fixer, pair, policy)? {
        fixer.state.write().small |= ShapeExtend::encode_status(ShapeExtendStatus::Fail1);
        return Ok(None);
    }
    let Some(joined) = join_edges_3d(&pair.first, &pair.second, &pair.faces)? else {
        return Ok(None);
    };
    remember_joined_edge(policy, pair, &joined.edge);
    install_joined_edge(fixer, wire_data, small, pair, joined)?;
    fixer.state.write().small |= ShapeExtend::encode_status(ShapeExtendStatus::Done1);
    Ok(Some(if pair.take_next {
        pair.middle_index
    } else {
        pair.middle_index.max(1)
    }))
}

fn remember_joined_edge(policy: &mut MergePolicy, pair: &CandidatePair, joined: &Edge) {
    if policy.edge_faces.is_empty() {
        return;
    }
    policy.face_small_changed = true;
    let faces = pair
        .faces
        .iter()
        .map(|face| face.clone().into_shape())
        .collect();
    let joined_shape = joined.clone().into_shape();
    policy.edge_faces.retain(|(edge, _)| {
        !edge.is_same(&pair.first.clone().into_shape())
            && !edge.is_same(&pair.second.clone().into_shape())
            && !edge.is_same(&joined_shape)
    });
    policy.edge_faces.push((joined_shape, faces));
}

fn install_joined_edge(
    fixer: &ShapeFixWireframe,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    pair: &CandidatePair,
    joined: super::join_3d::JoinedEdge3d,
) -> Result<(), Failure> {
    let mut joined = joined;
    non_manifold::copy_pair_vertices(fixer, pair, &mut joined.edge)?;
    replace_pair_in_context(fixer, pair, &joined)?;
    register_joined_edge(
        wire_data,
        small,
        fixer.precision(),
        pair,
        joined.edge,
        neighboring_index(pair),
    )
}

fn neighboring_index(pair: &CandidatePair) -> i32 {
    if pair.take_next {
        return pair.next_index;
    }
    pair.previous_index
}

fn register_joined_edge(
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    precision: f64,
    pair: &CandidatePair,
    joined: Edge,
    neighbor: i32,
) -> Result<(), Failure> {
    wire_data.set(&joined, neighbor)?;
    let remains_small = joined_edge_is_small(wire_data, neighbor, precision)?;
    wire_data.remove(pair.middle_index)?;
    update_small_edge_map(small, pair, joined, remains_small);
    Ok(())
}

fn update_small_edge_map(
    small: &mut MapOfShape,
    pair: &CandidatePair,
    joined: Edge,
    remains_small: bool,
) {
    small.remove(&pair.first.clone().into_shape());
    small.remove(&pair.second.clone().into_shape());
    if remains_small {
        small.add(joined.into_shape());
    }
}

fn joined_edge_is_small(
    wire_data: &Handle<ShapeExtendWireData>,
    index: i32,
    precision: f64,
) -> Result<bool, Failure> {
    let analyzer = ShapeAnalysisWire::new();
    analyzer.set_precision(precision);
    analyzer.load_wire_data(wire_data);
    analyzer.check_small_at(index, precision)
}

fn replace_pair_in_context(
    fixer: &ShapeFixWireframe,
    pair: &CandidatePair,
    joined: &super::join_3d::JoinedEdge3d,
) -> Result<(), Failure> {
    let context = fixer.context();
    let (kept, removed) = if joined.replace_first {
        (&pair.first, &pair.second)
    } else {
        (&pair.second, &pair.first)
    };
    ReShape::replace(
        &*context,
        &kept.clone().into_shape(),
        &joined.edge.clone().into_shape(),
    )?;
    ReShape::remove(&*context, &removed.clone().into_shape())
}

pub(super) fn finish_free_wire_merge(
    fixer: &ShapeFixWireframe,
    context: &Handle<ShapeBuildReShape>,
) -> Result<bool, Failure> {
    let applied = ReShape::apply(
        &**context,
        &fixer.shape(),
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )?;
    fixer.state.write().shape = applied;
    ShapeFix::same_parameter(
        &fixer.shape(),
        false,
        0.0,
        &ProgressRange::default(),
        &Handle::default(),
    )?;
    Ok(fixer.status_small_edges(ShapeExtendStatus::Done))
}
