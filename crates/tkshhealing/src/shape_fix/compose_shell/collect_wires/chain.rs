mod candidate;
use super::coincident;
use super::{ShapeFixComposeShell, patch};
use crate::shape_analysis::{EdgeTangent2d, ShapeAnalysisEdge};
use crate::shape_extend::{
    ShapeExtend, ShapeExtendCompositeSurface, ShapeExtendStatus, ShapeExtendWireData,
};
use crate::shape_fix::{ShapeFixWireSegment, WirePatchIndex};
use candidate::{Candidate, choose_segment, endpoint_orientation};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Vector2};

pub(super) fn collect_chains(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
    loops: &mut Sequence<ShapeFixWireSegment>,
    grid: &Handle<ShapeExtendCompositeSurface>,
    face: &Face,
    shortness: &[i32],
) -> Result<(), Failure> {
    let (nu, nv) = patch_counts(grid)?;
    let mut state = ChainState::default();
    loop {
        let Some(candidate) =
            choose_segment(composer, segments, grid, face, shortness, &state, nu, nv)?
        else {
            if state.wire_data.is_null() {
                break;
            }
            close_current_loop(composer, loops, face, &mut state)?;
            continue;
        };
        append_candidate(
            composer, segments, grid, face, &mut state, candidate, nu, nv,
        )?;
        update_chain_end(composer, segments, face, shortness, &mut state, candidate)?;
        close_if_ready(composer, loops, face, &mut state)?;
    }
    Ok(())
}

fn patch_counts(grid: &Handle<ShapeExtendCompositeSurface>) -> Result<(i32, i32), Failure> {
    let grid = grid.get().ok_or_else(null_grid)?;
    Ok((grid.nb_u_patches()?, grid.nb_v_patches()?))
}

#[derive(Default)]
struct ChainState {
    wire_data: Handle<ShapeExtendWireData>,
    first_vertex: Vertex,
    end_vertex: Vertex,
    first_edge: Edge,
    last_edge: Edge,
    first_point: Point2,
    end_point: Point2,
    first_tangent: Vector2,
    end_tangent: Vector2,
    tolerance: f64,
    patch: WirePatchIndex,
    can_be_closed: bool,
    shift_u: f64,
    shift_v: f64,
}

fn append_candidate(
    composer: &ShapeFixComposeShell,
    segments: &mut Sequence<ShapeFixWireSegment>,
    grid: &Handle<ShapeExtendCompositeSurface>,
    face: &Face,
    state: &mut ChainState,
    candidate: Candidate,
    nu: i32,
    nv: i32,
) -> Result<(), Failure> {
    let segment = segments.value(candidate.index)?.clone();
    if candidate.misoriented {
        composer.fields.write().invert_edge_status = true;
    }
    if state.wire_data.is_null() {
        state.wire_data = Handle::new(ShapeExtendWireData::new());
        state.patch = segment.patch_index(1)?;
    } else if candidate.same_patch {
        patch::extend_same_patch(&segment, &mut state.patch, nu, nv, true);
    }
    if composer.closed_mode() {
        state.patch = segment.patch_index(1)?;
    }
    add_segment_data(state, &segment, face, candidate.reverse)?;
    mark_segment_consumed(
        segments,
        candidate.index,
        segment.orientation(),
        candidate.reverse,
    )?;
    if state.end_vertex.is_null() {
        initialize_chain_start(state, face)?;
    }
    state.shift_u = candidate.shift_u;
    state.shift_v = candidate.shift_v;
    let _ = grid;
    Ok(())
}

fn add_segment_data(
    state: &ChainState,
    segment: &ShapeFixWireSegment,
    face: &Face,
    reverse: bool,
) -> Result<(), Failure> {
    if reverse {
        let reversed = ShapeExtendWireData::new();
        reversed.set_manifold_mode(false);
        reversed.add_wire_data(segment.wire_data().get().ok_or_else(null_wire)?, 0)?;
        reversed.reverse_on_face(face)?;
        state
            .wire_data
            .get()
            .ok_or_else(null_wire)?
            .add_wire_data(&reversed, 0)?;
        return Ok(());
    }
    state
        .wire_data
        .get()
        .ok_or_else(null_wire)?
        .add_wire_data(segment.wire_data().get().ok_or_else(null_wire)?, 0)
}

fn mark_segment_consumed(
    segments: &mut Sequence<ShapeFixWireSegment>,
    index: usize,
    orientation: Orientation,
    reverse: bool,
) -> Result<(), Failure> {
    let segment = segments.change_value(index)?;
    segment.set_orientation(if orientation == Orientation::External {
        endpoint_orientation(reverse)
    } else {
        Orientation::Internal
    });
    Ok(())
}

fn initialize_chain_start(state: &mut ChainState, face: &Face) -> Result<(), Failure> {
    state.first_edge = state.wire_data.get().ok_or_else(null_wire)?.edge(1)?;
    state.first_vertex = ShapeAnalysisEdge::new().first_vertex(&state.first_edge);
    let (_, outputs) = tangent_on_edge(&state.first_edge, face, false, 1.0e-3)?;
    state.first_point = outputs.point;
    state.first_tangent = outputs.tangent;
    Ok(())
}

fn update_chain_end(
    composer: &ShapeFixComposeShell,
    segments: &Sequence<ShapeFixWireSegment>,
    face: &Face,
    shortness: &[i32],
    state: &mut ChainState,
    candidate: Candidate,
) -> Result<(), Failure> {
    if candidate.index == 0
        || (shortness.get(candidate.index - 1).copied().unwrap_or(0) > 0
            && !state.end_vertex.is_null())
    {
        return Ok(());
    }
    let wire = state.wire_data.get().ok_or_else(null_wire)?;
    state.last_edge = wire.edge(wire.nb_edges())?;
    state.end_vertex = ShapeAnalysisEdge::new().last_vertex(&state.last_edge);
    state.tolerance = BRepTool::vertex_tolerance(&state.end_vertex)?;
    let mut outputs = EdgeTangent2d {
        point: state.end_point,
        tangent: state.end_tangent,
    };
    if !last_tangent(wire, face, &mut outputs)? {
        set_status(composer, ShapeExtendStatus::Fail2);
    }
    state.end_point = outputs.point;
    state.end_tangent = outputs.tangent;
    let fields = composer.fields.read();
    if fields.u_closed {
        state.end_point.set_x(state.end_point.x() + state.shift_u);
    }
    if fields.v_closed {
        state.end_point.set_y(state.end_point.y() + state.shift_v);
    }
    let _ = segments;
    Ok(())
}

fn last_tangent(
    wire: &ShapeExtendWireData,
    face: &Face,
    outputs: &mut EdgeTangent2d,
) -> Result<bool, Failure> {
    for edge_index in (1..=wire.nb_edges()).rev() {
        let edge = wire.edge(edge_index)?;
        if ShapeAnalysisEdge::new()
            .get_end_tangent_2d_on_face_into(&edge, face, true, 1.0e-3, outputs)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn close_if_ready(
    composer: &ShapeFixComposeShell,
    loops: &mut Sequence<ShapeFixWireSegment>,
    face: &Face,
    state: &mut ChainState,
) -> Result<(), Failure> {
    state.can_be_closed = state.end_vertex.is_same(&state.first_vertex);
    let closed_tangency = state.can_be_closed
        && !state.last_edge.is_same(&state.first_edge)
        && coincident(
            state.end_point,
            state.first_point,
            composer.resolutions(),
            2.0 * state.tolerance,
        );
    if !(state.can_be_closed && closed_tangency) {
        return Ok(());
    }
    close_current_loop(composer, loops, face, state)
}

fn close_current_loop(
    composer: &ShapeFixComposeShell,
    loops: &mut Sequence<ShapeFixWireSegment>,
    face: &Face,
    state: &mut ChainState,
) -> Result<(), Failure> {
    if !state
        .end_vertex
        .is_same(&ShapeAnalysisEdge::new().first_vertex(&state.first_edge))
    {
        set_status(composer, ShapeExtendStatus::Fail5);
    }
    let loop_segment = ShapeFixWireSegment::from_wire_data(&state.wire_data, Orientation::Forward)?;
    loop_segment.set_patch_index(1, state.patch)?;
    loops.append(loop_segment);
    *state = ChainState::default();
    let _ = face;
    Ok(())
}

fn tangent_on_edge(
    edge: &Edge,
    face: &Face,
    at_end: bool,
    dparam: f64,
) -> Result<(bool, EdgeTangent2d), Failure> {
    let mut output = EdgeTangent2d::default();
    let found = ShapeAnalysisEdge::new().get_end_tangent_2d_on_face_into(
        edge,
        face,
        at_end,
        dparam,
        &mut output,
    )?;
    Ok((found, output))
}

fn set_status(composer: &ShapeFixComposeShell, status: ShapeExtendStatus) {
    composer.fields.write().status |= ShapeExtend::encode_status(status);
}

fn null_grid() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::CollectWires: null grid",
    )
}

fn null_wire() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::CollectWires: null wire data",
    )
}
