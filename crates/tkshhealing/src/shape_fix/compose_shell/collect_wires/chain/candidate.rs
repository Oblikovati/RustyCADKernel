use super::super::coincident;
use super::{ChainState, ShapeFixComposeShell, patch, tangent_on_edge};
use crate::shape_extend::ShapeExtendCompositeSurface;
use crate::shape_fix::ShapeFixWireSegment;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Vector2};

#[derive(Clone, Copy, Default)]
pub(super) struct Candidate {
    pub(super) index: usize,
    pub(super) reverse: bool,
    pub(super) connected: bool,
    pub(super) misoriented: bool,
    pub(super) same_patch: bool,
    pub(super) angle: f64,
    pub(super) distance: f64,
    pub(super) weight: i32,
    pub(super) shift_u: f64,
    pub(super) shift_v: f64,
}

pub(super) fn choose_segment(
    composer: &ShapeFixComposeShell,
    segments: &Sequence<ShapeFixWireSegment>,
    grid: &Handle<ShapeExtendCompositeSurface>,
    face: &Face,
    shortness: &[i32],
    state: &ChainState,
    nu: i32,
    nv: i32,
) -> Result<Option<Candidate>, Failure> {
    let mut best = Candidate {
        angle: -std::f64::consts::PI,
        distance: f64::MAX,
        ..Candidate::default()
    };
    for index in 1..=segments.length() as usize {
        score_segment(
            composer, segments, grid, face, shortness, state, nu, nv, index, &mut best,
        )?;
        if state.wire_data.is_null() && best.index != 0 {
            break;
        }
    }
    Ok((best.index != 0).then_some(best))
}

fn score_segment(
    composer: &ShapeFixComposeShell,
    segments: &Sequence<ShapeFixWireSegment>,
    grid: &Handle<ShapeExtendCompositeSurface>,
    face: &Face,
    shortness: &[i32],
    state: &ChainState,
    nu: i32,
    nv: i32,
    index: usize,
    best: &mut Candidate,
) -> Result<(), Failure> {
    let segment = segments.value(index)?;
    if segment.is_vertex() || segment.orientation() == Orientation::Internal {
        return Ok(());
    }
    if state.wire_data.is_null() {
        return select_initial(segment, index, shortness, best);
    }
    score_attached_segment(
        composer, segment, index, grid, face, shortness, state, nu, nv, best,
    )
}

fn select_initial(
    segment: &ShapeFixWireSegment,
    index: usize,
    shortness: &[i32],
    best: &mut Candidate,
) -> Result<(), Failure> {
    if best.index != 0 {
        return Ok(());
    }
    if shortness.get(index - 1).copied().unwrap_or(0) > 0
        || segment.orientation() == Orientation::External
    {
        return Ok(());
    }
    *best = Candidate {
        index,
        reverse: segment.orientation() == Orientation::Forward,
        ..Candidate::default()
    };
    Ok(())
}

fn score_attached_segment(
    composer: &ShapeFixComposeShell,
    segment: &ShapeFixWireSegment,
    index: usize,
    grid: &Handle<ShapeExtendCompositeSurface>,
    face: &Face,
    shortness: &[i32],
    state: &ChainState,
    nu: i32,
    nv: i32,
    best: &mut Candidate,
) -> Result<(), Failure> {
    let same_patch = patch::extend_same_patch(segment, &mut state.patch.clone(), nu, nv, false);
    if !same_patch && (state.can_be_closed || (best.index != 0 && best.same_patch)) {
        return Ok(());
    }
    for endpoint in 0..=1 {
        score_endpoint(
            composer, segment, index, endpoint, grid, face, shortness, state, same_patch, best,
        )?;
    }
    Ok(())
}

fn score_endpoint(
    composer: &ShapeFixComposeShell,
    segment: &ShapeFixWireSegment,
    index: usize,
    endpoint: usize,
    grid: &Handle<ShapeExtendCompositeSurface>,
    face: &Face,
    shortness: &[i32],
    state: &ChainState,
    same_patch: bool,
    best: &mut Candidate,
) -> Result<(), Failure> {
    let (vertex, last_edge) = candidate_endpoint(segment, endpoint)?;
    if !state.end_vertex.is_same(&vertex) {
        return Ok(());
    }
    let reversed = endpoint != 0;
    let misoriented = segment.orientation() == endpoint_orientation(reversed);
    if state.last_edge.is_same(&last_edge) {
        select_return_edge(index, reversed, same_patch, misoriented, best);
        return Ok(());
    }
    evaluate_candidate(
        composer,
        segment,
        index,
        reversed,
        grid,
        face,
        shortness,
        state,
        same_patch,
        misoriented,
        best,
    )
}

fn candidate_endpoint(
    segment: &ShapeFixWireSegment,
    endpoint: usize,
) -> Result<(Vertex, Edge), Failure> {
    if endpoint == 0 {
        return Ok((segment.first_vertex()?, segment.edge(1)?));
    }
    let last_index = segment.nb_edges();
    Ok((segment.last_vertex()?, segment.edge(-last_index)?))
}

pub(super) fn endpoint_orientation(reverse: bool) -> Orientation {
    if reverse {
        Orientation::Reversed
    } else {
        Orientation::Forward
    }
}

fn select_return_edge(
    index: usize,
    reverse: bool,
    same_patch: bool,
    misoriented: bool,
    best: &mut Candidate,
) {
    if best.index != 0 {
        return;
    }
    *best = Candidate {
        index,
        reverse,
        connected: true,
        misoriented,
        same_patch,
        weight: i32::from(same_patch) * 16 + 8 + i32::from(!misoriented) * 4,
        ..Candidate::default()
    };
}

#[allow(clippy::too_many_arguments)]
fn evaluate_candidate(
    composer: &ShapeFixComposeShell,
    segment: &ShapeFixWireSegment,
    index: usize,
    reverse: bool,
    grid: &Handle<ShapeExtendCompositeSurface>,
    face: &Face,
    shortness: &[i32],
    state: &ChainState,
    same_patch: bool,
    misoriented: bool,
    best: &mut Candidate,
) -> Result<(), Failure> {
    let (point, tangent, edge_tolerance) = candidate_start_tangent(segment, face, reverse)?;
    let (point, shift_u, shift_v) = align_closed_mode(composer, point, state.end_point, grid)?;
    let short = shortness.get(index - 1).copied().unwrap_or(0) > 0;
    let angle = candidate_angle(composer.closed_mode(), short, state.end_tangent, tangent)?;
    let tolerance = edge_tolerance.max(BRepTool::vertex_tolerance(&state.end_vertex)?);
    let connected = coincident(state.end_point, point, composer.resolutions(), tolerance);
    let distance = state.end_point.square_distance(point);
    let (weight, tail) =
        candidate_weight(same_patch, connected, misoriented, angle, distance, best);
    if weight + tail <= best.weight + candidate_tail(best, angle, distance) {
        return Ok(());
    }
    *best = Candidate {
        index,
        reverse,
        connected,
        misoriented,
        same_patch,
        angle,
        distance,
        weight,
        shift_u,
        shift_v,
    };
    Ok(())
}

fn candidate_start_tangent(
    segment: &ShapeFixWireSegment,
    face: &Face,
    reverse: bool,
) -> Result<(Point2, Vector2, f64), Failure> {
    let count = segment.nb_edges();
    for position in 1..=count {
        let edge_index = if reverse {
            count + 1 - position
        } else {
            position
        };
        let edge = if reverse {
            segment.edge(-edge_index)?
        } else {
            segment.edge(edge_index)?
        };
        let (found, outputs) = tangent_on_edge(&edge, face, false, 1.0e-3)?;
        if found {
            return Ok((
                outputs.point,
                outputs.tangent,
                BRepTool::edge_tolerance(&edge)?,
            ));
        }
    }
    let edge = segment.edge(if reverse { -count } else { 1 })?;
    let (_, outputs) = tangent_on_edge(&edge, face, false, 1.0e-3)?;
    Ok((
        outputs.point,
        outputs.tangent,
        BRepTool::edge_tolerance(&edge)?,
    ))
}

fn align_closed_mode(
    composer: &ShapeFixComposeShell,
    point: Point2,
    end: Point2,
    grid: &Handle<ShapeExtendCompositeSurface>,
) -> Result<(Point2, f64, f64), Failure> {
    if !composer.closed_mode() {
        return Ok((point, 0.0, 0.0));
    }
    let fields = composer.fields.read();
    let mut aligned = point;
    let shift_u = if fields.u_closed {
        crate::shape_analysis::adjust_by_period(point.x(), end.x(), fields.u_period)
    } else {
        0.0
    };
    let shift_v = if fields.v_closed {
        crate::shape_analysis::adjust_by_period(point.y(), end.y(), fields.v_period)
    } else {
        0.0
    };
    aligned.set_x(point.x() + shift_u);
    aligned.set_y(point.y() + shift_v);
    let _ = grid;
    Ok((aligned, shift_u, shift_v))
}

fn candidate_angle(
    closed_mode: bool,
    short: bool,
    end_tangent: Vector2,
    candidate_tangent: Vector2,
) -> Result<f64, Failure> {
    if short {
        return Ok(std::f64::consts::PI);
    }
    let angle = end_tangent.angle(candidate_tangent)?.abs();
    Ok(
        if closed_mode && std::f64::consts::PI - angle < rustycad_tkernel::precision::angular() {
            0.0
        } else {
            angle
        },
    )
}

fn candidate_weight(
    same_patch: bool,
    connected: bool,
    misoriented: bool,
    angle: f64,
    distance: f64,
    best: &Candidate,
) -> (i32, i32) {
    let weight =
        i32::from(same_patch) * 16 + i32::from(!misoriented) * 8 + i32::from(connected) * 4;
    let tail =
        i32::from(!connected && distance < best.distance) * 2 + i32::from(angle > best.angle);
    (weight, tail)
}

fn candidate_tail(best: &Candidate, angle: f64, distance: f64) -> i32 {
    i32::from(!best.connected && distance > best.distance) * 2 + i32::from(angle < best.angle)
}
