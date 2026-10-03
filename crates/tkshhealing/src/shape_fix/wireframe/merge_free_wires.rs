use super::{
    FixRoot, ShapeFixWireframe, join_3d::join_edges_3d, repair::unported_small_edge_merge,
};
use crate::{
    shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire},
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
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{geom::GeomCurve, top_abs::Orientation};
use rustycad_tkmath::gp::Vector3;

struct CandidatePair {
    first: Edge,
    second: Edge,
    middle_index: i32,
    previous_index: i32,
    next_index: i32,
    take_next: bool,
}

pub(super) fn merge_small_edges(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
    edge_faces: &mut super::ShapeFixWireframeShapeLists,
    face_small: &mut super::ShapeFixWireframeShapeLists,
    multiple: &mut MapOfShape,
) -> Result<bool, Failure> {
    if has_face_edge_lists(small, edge_faces, face_small, multiple)? {
        return Err(unported_small_edge_merge());
    }
    merge_free_wire_candidates(fixer, small)
}

fn has_face_edge_lists(
    small: &MapOfShape,
    edge_faces: &super::ShapeFixWireframeShapeLists,
    face_small: &super::ShapeFixWireframeShapeLists,
    multiple: &MapOfShape,
) -> Result<bool, Failure> {
    if !face_small.is_empty() || !multiple.is_empty() {
        return Ok(true);
    }
    for edge in small.iter() {
        if edge_faces.is_bound(edge) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn merge_free_wire_candidates(
    fixer: &ShapeFixWireframe,
    small: &mut MapOfShape,
) -> Result<bool, Failure> {
    let context = fixer.context();
    let mut changed = false;
    for source in Explorer::new(
        &fixer.shape(),
        rustycad_tkg3d::top_abs::ShapeEnum::Wire,
        rustycad_tkg3d::top_abs::ShapeEnum::Face,
    )? {
        let wire = topods::wire(&source)?;
        changed |= merge_one_free_wire(fixer, &wire, &context, small)?;
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
) -> Result<bool, Failure> {
    let applied = ReShape::apply(
        &**context,
        &source.clone().into_shape(),
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )?;
    let wire = topods::wire(&applied)?;
    let tool = prepared_wire_fixer(&wire, context)?;
    let wire_data = tool.wire_data()?;
    merge_wire_data(fixer, &tool, &wire, &wire_data, small)
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
) -> Result<bool, Failure> {
    let changed = merge_wire_candidates(fixer, wire_data, small)?;
    publish_changed_wire(fixer, wire_fixer, source, wire_data, changed)?;
    Ok(changed)
}

fn merge_wire_candidates(
    fixer: &ShapeFixWireframe,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
) -> Result<bool, Failure> {
    let mut index = 1;
    let mut changed = false;
    while index <= wire_data.nb_edges() && wire_data.nb_edges() > 1 {
        let merged_index = merge_small_at_index(fixer, wire_data, small, index)?;
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
) -> Result<Option<i32>, Failure> {
    let count = wire_data.nb_edges();
    let previous = if index == 1 { count } else { index - 1 };
    let next = if index == count { 1 } else { index + 1 };
    let pair = candidate_pair(wire_data, small, index, previous, next)?;
    let Some(pair) = pair else {
        return Ok(None);
    };
    merge_candidate_pair(fixer, wire_data, small, &pair)
}

fn candidate_pair(
    wire_data: &Handle<ShapeExtendWireData>,
    small: &MapOfShape,
    index: i32,
    previous: i32,
    next: i32,
) -> Result<Option<CandidatePair>, Failure> {
    let middle = wire_data.edge(index)?;
    if !is_mergeable_middle(small, &middle, index, previous, next) {
        return Ok(None);
    }
    let before = wire_data.edge(previous)?;
    let after = wire_data.edge(next)?;
    Ok(Some(select_candidate_pair(
        before, middle, after, index, previous, next,
    )?))
}

fn select_candidate_pair(
    before: Edge,
    middle: Edge,
    after: Edge,
    index: i32,
    previous: i32,
    next: i32,
) -> Result<CandidatePair, Failure> {
    let take_next = choose_next_pair(&before, &middle, &after)?;
    let (first, second) = if take_next {
        (middle, after)
    } else {
        (before, middle)
    };
    Ok(candidate_pair_from_edges(
        first, second, index, previous, next, take_next,
    ))
}

fn candidate_pair_from_edges(
    first: Edge,
    second: Edge,
    index: i32,
    previous: i32,
    next: i32,
    take_next: bool,
) -> CandidatePair {
    CandidatePair {
        first,
        second,
        middle_index: index,
        previous_index: previous,
        next_index: next,
        take_next,
    }
}

fn is_mergeable_middle(
    small: &MapOfShape,
    middle: &Edge,
    index: i32,
    previous: i32,
    next: i32,
) -> bool {
    small.contains(&middle.clone().into_shape())
        && index != previous
        && index != next
        && previous != next
}

fn choose_next_pair(before: &Edge, middle: &Edge, after: &Edge) -> Result<bool, Failure> {
    let angle_before = adjacent_angle(before, middle)?;
    let angle_after = adjacent_angle(middle, after)?;
    if (angle_after - angle_before).abs() > precision::angular() {
        return Ok(angle_after < angle_before);
    }
    Ok(false)
}

fn adjacent_angle(first: &Edge, second: &Edge) -> Result<f64, Failure> {
    let (first_curve, _, first_last) = ShapeAnalysisEdge::new().curve_3d(first, true)?;
    let (second_curve, second_first, _) = ShapeAnalysisEdge::new().curve_3d(second, true)?;
    let first_tangent = tangent_at(&first_curve, first_last)?;
    let second_tangent = tangent_at(&second_curve, second_first)?;
    let first_tangent = oriented_tangent(first_tangent, first.orientation());
    let second_tangent = oriented_tangent(second_tangent, second.orientation());
    angle_or_right_angle(first_tangent, second_tangent)
}

fn tangent_at(curve: &Handle<dyn GeomCurve>, parameter: f64) -> Result<Vector3, Failure> {
    Ok(curve.d1(parameter)?.1)
}

fn oriented_tangent(tangent: Vector3, orientation: Orientation) -> Vector3 {
    if orientation == Orientation::Reversed {
        return -tangent;
    }
    tangent
}

fn angle_or_right_angle(first: Vector3, second: Vector3) -> Result<f64, Failure> {
    if first.square_magnitude() < precision::square_confusion()
        || second.square_magnitude() < precision::square_confusion()
    {
        return Ok(std::f64::consts::FRAC_PI_2);
    }
    Ok(first.angle(second)?.abs())
}

fn merge_candidate_pair(
    fixer: &ShapeFixWireframe,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    pair: &CandidatePair,
) -> Result<Option<i32>, Failure> {
    let Some(joined) = join_edges_3d(&pair.first, &pair.second)? else {
        return Ok(None);
    };
    install_joined_edge(fixer, wire_data, small, pair, joined)?;
    fixer.state.write().small |= ShapeExtend::encode_status(ShapeExtendStatus::Done1);
    Ok(Some(if pair.take_next {
        pair.middle_index
    } else {
        pair.middle_index.max(1)
    }))
}

fn install_joined_edge(
    fixer: &ShapeFixWireframe,
    wire_data: &Handle<ShapeExtendWireData>,
    small: &mut MapOfShape,
    pair: &CandidatePair,
    joined: super::join_3d::JoinedEdge3d,
) -> Result<(), Failure> {
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

fn finish_free_wire_merge(
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
