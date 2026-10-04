use super::ShapeFixWireframe;
use crate::{
    shape_analysis::ShapeAnalysisEdge,
    shape_extend::ShapeExtendWireData,
    shape_fix::{FixRoot, ShapeFixWire},
};
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topods::{self, Edge, Face, Shape, ShapeIterator, Wire},
    toptools::MapOfShape,
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::{geom::GeomCurve, top_abs::Orientation};
use rustycad_tkmath::gp::Vector3;

pub(super) struct CandidatePair {
    pub(super) first: Edge,
    pub(super) second: Edge,
    pub(super) middle_index: i32,
    pub(super) previous_index: i32,
    pub(super) next_index: i32,
    pub(super) take_next: bool,
    pub(super) faces: Vec<Face>,
    pub(super) minimum_angle: f64,
}

#[derive(Clone, Copy)]
struct PairIndexes {
    middle: i32,
    previous: i32,
    next: i32,
}

struct CandidateEdges {
    before: Edge,
    middle: Edge,
    after: Edge,
}

pub(in crate::shape_fix::wireframe) struct MergePolicy {
    pub(super) faces: Vec<Face>,
    pub(super) edge_faces: Vec<(Shape, Vec<Shape>)>,
    pub(super) angle_limit: Option<f64>,
    pub(super) drop_mode: bool,
}

impl MergePolicy {
    pub(in crate::shape_fix::wireframe) fn free(angle_limit: Option<f64>, drop_mode: bool) -> Self {
        Self {
            faces: Vec::new(),
            edge_faces: Vec::new(),
            angle_limit,
            drop_mode,
        }
    }

    pub(in crate::shape_fix::wireframe) fn face(
        face: &Face,
        edge_faces: Vec<(Shape, Vec<Shape>)>,
        angle_limit: Option<f64>,
    ) -> Self {
        Self {
            faces: vec![face.clone()],
            edge_faces,
            angle_limit,
            drop_mode: false,
        }
    }
}

pub(super) fn candidate_pair(
    wire_data: &Handle<ShapeExtendWireData>,
    small: &MapOfShape,
    index: i32,
    previous: i32,
    next: i32,
    policy: &MergePolicy,
) -> Result<Option<CandidatePair>, Failure> {
    let Some(edges) = candidate_edges(wire_data, small, index, previous, next)? else {
        return Ok(None);
    };
    let indexes = PairIndexes {
        middle: index,
        previous,
        next,
    };
    limited_candidate_pair(wire_data, edges, indexes, policy)
}

fn limited_candidate_pair(
    wire_data: &Handle<ShapeExtendWireData>,
    edges: CandidateEdges,
    indexes: PairIndexes,
    policy: &MergePolicy,
) -> Result<Option<CandidatePair>, Failure> {
    let Some((faces, forced_next)) = candidate_face_set(wire_data, &edges, indexes, policy) else {
        return Ok(None);
    };
    let pair = select_candidate_pair(edges, indexes, &faces, forced_next)?;
    if exceeds_pair_angle_limit(&pair, policy.angle_limit) {
        return Ok(None);
    }
    Ok(Some(pair))
}

fn candidate_face_set(
    wire_data: &Handle<ShapeExtendWireData>,
    edges: &CandidateEdges,
    indexes: PairIndexes,
    policy: &MergePolicy,
) -> Option<(Vec<Face>, Option<bool>)> {
    if policy.edge_faces.is_empty() {
        return Some((policy.faces.clone(), None));
    }
    let middle = edge_faces(&edges.middle, &policy.edge_faces);
    let before = edge_faces(&edges.before, &policy.edge_faces);
    let after = edge_faces(&edges.after, &policy.edge_faces);
    let same_before = matching_faces(&middle, &before)
        && wire_data.is_seam(indexes.middle) == wire_data.is_seam(indexes.previous);
    let same_after = matching_faces(&middle, &after)
        && wire_data.is_seam(indexes.middle) == wire_data.is_seam(indexes.next);
    let forced_next = match (same_before, same_after) {
        (true, true) => None,
        (true, false) => Some(false),
        (false, true) => Some(true),
        (false, false) => return None,
    };
    let faces = middle
        .iter()
        .map(topods::face)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    Some((faces, forced_next))
}

fn edge_faces(edge: &Edge, links: &[(Shape, Vec<Shape>)]) -> Vec<Shape> {
    links
        .iter()
        .find(|(candidate, _)| candidate.is_same(&edge.clone().into_shape()))
        .map(|(_, faces)| faces.clone())
        .unwrap_or_default()
}

fn matching_faces(first: &[Shape], second: &[Shape]) -> bool {
    first.len() == second.len()
        && first
            .iter()
            .all(|face| second.iter().any(|item| item.is_same(face)))
}

fn candidate_edges(
    wire_data: &Handle<ShapeExtendWireData>,
    small: &MapOfShape,
    index: i32,
    previous: i32,
    next: i32,
) -> Result<Option<CandidateEdges>, Failure> {
    let middle = wire_data.edge(index)?;
    if !is_mergeable_middle(small, &middle, index, previous, next) {
        return Ok(None);
    }
    Ok(Some(CandidateEdges {
        before: wire_data.edge(previous)?,
        middle,
        after: wire_data.edge(next)?,
    }))
}

fn exceeds_pair_angle_limit(pair: &CandidatePair, limit: Option<f64>) -> bool {
    limit.is_some_and(|maximum| pair.minimum_angle > maximum)
}

fn select_candidate_pair(
    edges: CandidateEdges,
    indexes: PairIndexes,
    faces: &[Face],
    forced_next: Option<bool>,
) -> Result<CandidatePair, Failure> {
    let (angle_next, minimum_angle) = choose_next_pair(&edges.before, &edges.middle, &edges.after)?;
    let take_next = forced_next.unwrap_or(angle_next);
    let (first, second) = selected_pair_edges(edges, take_next);
    Ok(CandidatePair {
        first,
        second,
        middle_index: indexes.middle,
        previous_index: indexes.previous,
        next_index: indexes.next,
        take_next,
        faces: faces.to_vec(),
        minimum_angle,
    })
}

fn selected_pair_edges(edges: CandidateEdges, take_next: bool) -> (Edge, Edge) {
    if take_next {
        (edges.middle, edges.after)
    } else {
        (edges.before, edges.middle)
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

fn choose_next_pair(before: &Edge, middle: &Edge, after: &Edge) -> Result<(bool, f64), Failure> {
    let angle_before = adjacent_angle(before, middle)?;
    let angle_after = adjacent_angle(middle, after)?;
    if (angle_after - angle_before).abs() > precision::angular() {
        return Ok((angle_after < angle_before, angle_before.min(angle_after)));
    }
    Ok((false, angle_before.min(angle_after)))
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

pub(super) fn linked_face_order_allows_join(
    fixer: &ShapeFixWireframe,
    pair: &CandidatePair,
    policy: &MergePolicy,
) -> Result<bool, Failure> {
    if policy.faces.is_empty() {
        return Ok(true);
    }
    for face in &pair.faces {
        if policy.faces.iter().any(|current| current.is_same(face)) {
            continue;
        }
        if !face_wire_allows_join(fixer, face, pair)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn face_wire_allows_join(
    fixer: &ShapeFixWireframe,
    face: &Face,
    pair: &CandidatePair,
) -> Result<bool, Failure> {
    let face_shape = ReShape::apply(
        &*fixer.context(),
        &face.clone().into_shape(),
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )?;
    for child in ShapeIterator::new(&face_shape, true, true) {
        if child.shape_type()? != rustycad_tkg3d::top_abs::ShapeEnum::Wire {
            continue;
        }
        if let Some(allowed) = wire_pair_allows_join(&topods::wire(&child)?, pair)? {
            return Ok(allowed);
        }
    }
    Ok(true)
}

fn wire_pair_allows_join(wire: &Wire, pair: &CandidatePair) -> Result<Option<bool>, Failure> {
    let tool = ShapeFixWire::new();
    tool.load(wire)?;
    tool.fix_reorder()?;
    let data = tool.wire_data()?;
    let first = data.index(&pair.first)?;
    let second = data.index(&pair.second)?;
    if first == 0 || second == 0 {
        return Ok(None);
    }
    Ok(Some(
        first - second == 1 || (second == data.nb_edges() && first == 1),
    ))
}
