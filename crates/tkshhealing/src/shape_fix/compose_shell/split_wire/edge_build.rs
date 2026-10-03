use super::edge_state::EdgeSplitState;
use super::{SplitEdgeWork, SplitWireState, null_context, null_pcurve};
use crate::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
use crate::shape_fix::FixRoot;
use crate::shape_fix::ShapeFixWireSegment;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_tools::ReShape,
    topods::{self, Edge, Vertex},
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn infer_external_code(cut_by_u: bool, point_index: i32) -> i32 {
    if cut_by_u == (point_index == 1) { 2 } else { 1 }
}

pub(super) fn infer_external_code_by_parameter(
    cut_by_u: bool,
    first: f64,
    last: f64,
    current: f64,
) -> i32 {
    let first_is_nearer = (first - current).abs() < (last - current).abs();
    if cut_by_u == first_is_nearer { 2 } else { 1 }
}

pub(super) fn copy_as_external_edge(
    edge: &Edge,
    state: &EdgeSplitState,
    face: &rustycad_tkbrep::topods::Face,
    context: &Handle<ShapeBuildReShape>,
) -> Result<Edge, Failure> {
    let source = forward_internal_edge(edge);
    let copy = ShapeBuildEdge.copy(&source, false)?;
    let first = copy_pcurve(&state.pcurve)?;
    let second = copy_pcurve(&state.pcurve)?;
    BRepBuilder::default().update_edge_pcurves_on_face(&copy, (&first, &second), face, 0.0)?;
    let mut external = copy;
    external.set_orientation(Orientation::External);
    context
        .get()
        .ok_or_else(null_context)?
        .replace(&edge.clone().into_shape(), &external.clone().into_shape())?;
    Ok(external)
}

fn forward_internal_edge(edge: &Edge) -> Edge {
    let mut source = edge.clone();
    if source.orientation() == Orientation::Internal {
        source.set_orientation(Orientation::Forward);
    }
    source
}

fn copy_pcurve(
    curve: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
) -> Result<Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>, Failure> {
    curve
        .get()
        .ok_or_else(null_pcurve)?
        .copy()
        .map(|copy| copy.cast())
}

pub(super) fn build_split_edge(
    state: &mut SplitWireState<'_>,
    work: &mut SplitEdgeWork,
    vertex: &Vertex,
    parameter: f64,
    point_index: i32,
) -> Result<Edge, Failure> {
    let (mut first, mut last, mut cut) = (
        work.state.first_vertex.clone(),
        work.state.last_vertex.clone(),
        vertex.clone(),
    );
    protect_edge_endpoints(
        state,
        &mut work.state,
        &mut first,
        &mut last,
        &mut cut,
        work.split,
    )?;
    first.set_orientation(Orientation::Forward);
    cut.set_orientation(Orientation::Reversed);
    let manifold = matches!(
        work.edge.orientation(),
        Orientation::Forward | Orientation::Reversed
    );
    let original = oriented_source_edge(&work.edge, manifold)?;
    let mut result = ShapeBuildEdge.copy_replace_vertices(&original, &first, &cut)?;
    add_internal_vertices(state, &mut work.state, &mut result, &first, &cut, parameter)?;
    ShapeBuildEdge.copy_pcurves(&result, &original)?;
    transfer_split_range(state, &original, &result, &work.state, parameter)?;
    set_split_orientation(
        &mut result,
        work.edge.orientation(),
        manifold,
        work,
        point_index,
        work.state.same_parameter,
    )?;
    update_split_cursor(state, &mut work.state, &cut, parameter, &mut work.split)?;
    Ok(result)
}

fn protect_edge_endpoints(
    state: &SplitWireState<'_>,
    edge_state: &mut EdgeSplitState,
    first: &mut Vertex,
    last: &mut Vertex,
    cut: &mut Vertex,
    split: bool,
) -> Result<(), Failure> {
    if split {
        return Ok(());
    }
    let copied_first = topods::vertex(&first.clone().into_shape().empty_copied()?)?;
    state
        .composer
        .context()
        .get()
        .ok_or_else(null_context)?
        .replace(
            &first.clone().into_shape(),
            &copied_first.clone().into_shape(),
        )?;
    let copied_last = if first.is_same(last) {
        topods::vertex(
            &copied_first
                .clone()
                .into_shape()
                .oriented(last.orientation()),
        )?
    } else {
        let copy = topods::vertex(&last.clone().into_shape().empty_copied()?)?;
        state
            .composer
            .context()
            .get()
            .ok_or_else(null_context)?
            .replace(&last.clone().into_shape(), &copy.clone().into_shape())?;
        copy
    };
    replace_split_endpoint(cut, first, last, &copied_first, &copied_last);
    edge_state.first_vertex = copied_first.clone();
    edge_state.last_vertex = copied_last.clone();
    *first = copied_first;
    *last = copied_last;
    Ok(())
}

fn replace_split_endpoint(
    cut: &mut Vertex,
    first: &Vertex,
    last: &Vertex,
    copied_first: &Vertex,
    copied_last: &Vertex,
) {
    if cut.is_same(last) {
        *cut = copied_last.clone();
    } else if cut.is_same(first) {
        *cut = copied_first.clone();
    }
}

fn oriented_source_edge(edge: &Edge, manifold: bool) -> Result<Edge, Failure> {
    if manifold {
        return Ok(edge.clone());
    }
    let mut source = edge.clone();
    source.set_orientation(Orientation::Forward);
    Ok(source)
}

fn add_internal_vertices(
    state: &SplitWireState<'_>,
    edge_state: &mut EdgeSplitState,
    edge: &mut Edge,
    first: &Vertex,
    last: &Vertex,
    parameter: f64,
) -> Result<(), Failure> {
    let mut edge_shape = edge.clone().into_shape();
    let mut index = 0;
    while index < edge_state.non_manifold_parameters.len() {
        let cut_parameter = edge_state.non_manifold_parameters[index];
        let vertex = topods::vertex(&edge_state.non_manifold_vertices[index])?;
        let applied = state
            .composer
            .context()
            .get()
            .ok_or_else(null_context)?
            .apply(&vertex.clone().into_shape(), ShapeEnum::Shape)?;
        let applied_vertex = topods::vertex(&applied)?;
        if (cut_parameter - edge_state.previous_parameter).abs() <= precision::p_confusion() {
            state
                .composer
                .context()
                .get()
                .ok_or_else(null_context)?
                .replace(
                    &applied_vertex.clone().into_shape(),
                    &first.clone().into_shape(),
                )?;
            remove_internal_vertex(edge_state, index);
            continue;
        }
        if (cut_parameter - parameter).abs() <= precision::p_confusion() {
            state
                .composer
                .context()
                .get()
                .ok_or_else(null_context)?
                .replace(
                    &applied_vertex.clone().into_shape(),
                    &last.clone().into_shape(),
                )?;
            remove_internal_vertex(edge_state, index);
            continue;
        }
        if cut_parameter > edge_state.previous_parameter && cut_parameter < parameter {
            BRepBuilder::default().add(&mut edge_shape, &applied_vertex.clone().into_shape())?;
            remove_internal_vertex(edge_state, index);
            continue;
        }
        index += 1;
    }
    *edge = topods::edge(&edge_shape)?;
    Ok(())
}

fn remove_internal_vertex(state: &mut EdgeSplitState, index: usize) {
    state.non_manifold_parameters.remove(index);
    state.non_manifold_vertices.remove(index);
}

fn transfer_split_range(
    state: &SplitWireState<'_>,
    source: &Edge,
    result: &Edge,
    edge_state: &EdgeSplitState,
    parameter: f64,
) -> Result<(), Failure> {
    let tool = state.fields.transfer_tool.clone();
    tool.set_max_tolerance(state.composer.max_tolerance());
    tool.init(source, &state.fields.face)?;
    tool.transfer_range(result, edge_state.previous_parameter, parameter, true)
}

fn set_split_orientation(
    edge: &mut Edge,
    source_orientation: Orientation,
    manifold: bool,
    work: &mut SplitEdgeWork,
    point_index: i32,
    same_parameter: bool,
) -> Result<(), Failure> {
    if !manifold {
        edge.set_orientation(if work.code == 0 {
            Orientation::External
        } else {
            source_orientation
        });
    }
    if work.code == 0 && work.wire_orientation == Orientation::External {
        work.code = infer_external_code(work.cut_by_u, point_index);
    }
    if !same_parameter && !BRepTool::degenerated(edge)? {
        BRepBuilder::default().same_range(edge, false)?;
    }
    Ok(())
}

fn update_split_cursor(
    state: &SplitWireState<'_>,
    edge_state: &mut EdgeSplitState,
    vertex: &Vertex,
    parameter: f64,
    split: &mut bool,
) -> Result<(), Failure> {
    *split = true;
    edge_state.first_vertex = vertex.clone();
    edge_state.first_vertex_tolerance = state
        .composer
        .limit_tolerance(BRepTool::vertex_tolerance(vertex)?);
    edge_state.first_point =
        BRepTool::pnt(vertex)?.transformed(state.fields.location.inverted()?.transformation());
    edge_state.previous_uv = edge_state.pcurve.value(parameter)?;
    edge_state.previous_grid_point = state.fields.grid.value_at(edge_state.previous_uv)?;
    edge_state.previous_parameter = parameter;
    Ok(())
}

pub(super) fn result_edges_as_wire(
    segment: &ShapeFixWireSegment,
    initial_count: i32,
    orientation: Orientation,
) -> Result<rustycad_tkbrep::topods::Wire, Failure> {
    let mut result = rustycad_tkbrep::topods::Wire::default();
    rustycad_tkbrep::topods::Builder.make_wire(&mut result);
    let mut result_shape = result.into_shape();
    for rank in 1..=(segment.nb_edges() - initial_count) {
        let index = if matches!(orientation, Orientation::Forward | Orientation::Internal) {
            initial_count + rank
        } else {
            segment.nb_edges() - rank + 1
        };
        rustycad_tkbrep::topods::Builder
            .add(&mut result_shape, &segment.edge(index)?.into_shape())?;
    }
    topods::wire(&result_shape)
}
