//! Native range-cut helpers for ShapeFix_Wire adjacent crossing repair.
use super::wire_intersection_fix::CrossingState;
use crate::shape_analysis::{ShapeAnalysisCurve, ShapeAnalysisEdge};
use crate::shape_fix::ShapeFixEdge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg2d::geom2d::{Geom2dLine, Geom2dTrimmedCurve};
pub(super) fn split_endpoint(state: &CrossingState, edge_index: usize) -> f64 {
    let forward = state.forwards[edge_index];
    let range = state.ranges[edge_index];
    match (edge_index, forward) {
        (0, true) | (1, false) => range.0,
        (0, false) | (1, true) => range.1,
        _ => unreachable!("adjacent edge index must be zero or one"),
    }
}

pub(super) fn same_vertex(shared: &Vertex, edge: &Edge, use_last: bool) -> bool {
    let analysis = ShapeAnalysisEdge::new();
    let endpoint = if use_last {
        analysis.last_vertex(edge)
    } else {
        analysis.first_vertex(edge)
    };
    shared.is_same(&endpoint)
}

pub(super) fn cut_edge(
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    face: &Face,
    cut_line: &mut bool,
) -> Result<bool, Failure> {
    if (cut - endpoint).abs() < 10.0 * precision::p_confusion() {
        return Ok(false);
    }
    let edge_range = BRepTool::range(edge)?;
    *cut_line = false;
    if (cut - endpoint).abs() < 10.0 * precision::p_confusion() {
        return Ok(false);
    }
    if !BRepTool::same_parameter(edge)? {
        return cut_without_same_parameter(edge, endpoint, cut, face, edge_range, cut_line);
    }
    cut_same_parameter(edge, endpoint, cut, face, edge_range)
}

pub(super) fn cut_adjacent_pair(
    state: &CrossingState,
    face: &Face,
    parameters: (f64, f64),
) -> Result<([bool; 2], bool), Failure> {
    let mut cut_line = false;
    let first = cut_edge(
        &state.edges[0],
        split_endpoint(state, 0),
        parameters.0,
        face,
        &mut cut_line,
    )?;
    let second = cut_edge(
        &state.edges[1],
        split_endpoint(state, 1),
        parameters.1,
        face,
        &mut cut_line,
    )?;
    Ok(([first, second], cut_line))
}

fn cut_without_same_parameter(
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    face: &Face,
    edge_range: (f64, f64),
    cut_line: &mut bool,
) -> Result<bool, Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    let trimmed_handle = curve.cast::<Geom2dTrimmedCurve>();
    let Some(trimmed) = trimmed_handle.get() else {
        return Ok(true);
    };
    if trimmed.basis_curve().cast::<Geom2dLine>().is_null() {
        return Ok(true);
    }
    BRepBuilder::default().range(edge, endpoint.min(cut), endpoint.max(cut), false)?;
    cut_trimmed_line(edge, endpoint, cut, first, last, edge_range, cut_line)
}

fn cut_trimmed_line(
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    first: f64,
    last: f64,
    edge_range: (f64, f64),
    cut_line: &mut bool,
) -> Result<bool, Failure> {
    let Some(((new_first, new_last), shift)) =
        trimmed_line_range(endpoint, cut, first, last, edge_range)
    else {
        return Ok(true);
    };
    if shift <= precision::p_confusion() {
        return Ok(false);
    }
    BRepBuilder::default().range(edge, new_first, new_last, true)?;
    *cut_line = true;
    Ok(true)
}
fn trimmed_line_range(
    endpoint: f64,
    cut: f64,
    first: f64,
    last: f64,
    edge_range: (f64, f64),
) -> Option<((f64, f64), f64)> {
    if (endpoint - last).abs() < precision::p_confusion() {
        let shift = (cut - first) * (edge_range.1 - edge_range.0) / (last - first);
        return Some(((edge_range.0 + shift, edge_range.1), shift));
    }
    if (endpoint - first).abs() < precision::p_confusion() {
        let shift = (last - cut) * (edge_range.1 - edge_range.0) / (last - first);
        return Some(((edge_range.0, edge_range.1 - shift), shift));
    }
    None
}

fn cut_same_parameter(
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    face: &Face,
    edge_range: (f64, f64),
) -> Result<bool, Failure> {
    if ranges_already_match(edge_range, endpoint, cut) {
        return Ok(false);
    }
    let range = (endpoint.min(cut), endpoint.max(cut));
    if let Some(validated) = validated_curve_range(edge, range)? {
        update_validated_range(edge, face, validated)?;
        return Ok(true);
    }
    BRepBuilder::default().range(edge, range.0, range.1, false)?;
    Ok(true)
}

fn ranges_already_match(edge_range: (f64, f64), endpoint: f64, cut: f64) -> bool {
    let span = (edge_range.1 - edge_range.0).abs();
    (span - (cut - endpoint).abs()).abs() < precision::p_confusion()
}

fn validated_curve_range(edge: &Edge, range: (f64, f64)) -> Result<Option<(f64, f64)>, Failure> {
    let (curve, _, _) = BRepTool::curve(edge)?;
    if BRepTool::degenerated(edge)? || curve.is_null() {
        return Ok(None);
    }
    validate_curve_range(curve, range)
}

fn validate_curve_range(
    curve: rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    range: (f64, f64),
) -> Result<Option<(f64, f64)>, Failure> {
    let mut validated = range;
    if !ShapeAnalysisCurve.validate_range(
        &curve,
        &mut validated.0,
        &mut validated.1,
        precision::p_confusion(),
    )? || validated == range
    {
        return Ok(None);
    }
    Ok(Some(validated))
}

fn update_validated_range(edge: &Edge, face: &Face, range: (f64, f64)) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    builder.range(edge, range.0, range.1, true)?;
    let (pcurve, _, _) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    if !pcurve.is_null() {
        builder.same_range(edge, false)?;
    }
    ShapeFixEdge::new().fix_same_parameter_default(edge)?;
    Ok(())
}

/// OCCT edge splitting and trimming utility; `ShapeFixSplitTool::new()`.
#[derive(Default)]
pub struct ShapeFixSplitTool;
impl ShapeFixSplitTool {
    /// Construct the stateless tool; `ShapeFixSplitTool::new()`.
    pub fn new() -> Self {
        Self
    }
    /// Trim the native edge ranges; `tool.cut_edge(&edge, 0., 0.5, &face, &mut cut_line)?`.
    pub fn cut_edge(
        &self,
        edge: &Edge,
        endpoint: f64,
        cut: f64,
        face: &Face,
        cut_line: &mut bool,
    ) -> Result<bool, Failure> {
        cut_edge(edge, endpoint, cut, face, cut_line)
    }
}

#[path = "wire_split_tool/split.rs"]
mod split;

#[path = "wire_split_tool/sequence.rs"]
mod sequence;
