//! Edge splitting and trimming primitives used by `ShapeFix_IntersectionTool`.
use crate::shape_analysis::{
    ShapeAnalysisEdge, ShapeAnalysisSurface, ShapeAnalysisTransferParametersProj,
    TransferParameters,
};
use crate::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dTrimmedCurve;
use rustycad_tkg3d::top_abs::Orientation;

/// Native `ShapeFix_IntersectionTool` edge primitives; `tool.cut_edge(&edge, 0.0, 0.4, &face, &mut cut_line)?`.
pub struct ShapeFixIntersectionTool {
    context: Handle<ShapeBuildReShape>,
    // Retained for the remaining wire aggregation routines in this native class.
    #[allow(dead_code)]
    precision: f64,
    // Retained for tolerance checks during native nonadjacent crossing repair.
    #[allow(dead_code)]
    max_tolerance: f64,
}

impl ShapeFixIntersectionTool {
    /// Creates the tool with native default `maxtol = 1.0`; `ShapeFixIntersectionTool::new(&context, 1e-7)`.
    pub fn new(context: &Handle<ShapeBuildReShape>, precision: f64) -> Self {
        Self::with_max_tolerance(context, precision, 1.0)
    }

    /// Creates the tool with an explicit vertex tolerance ceiling.
    pub fn with_max_tolerance(
        context: &Handle<ShapeBuildReShape>,
        precision: f64,
        max_tolerance: f64,
    ) -> Self {
        Self {
            context: context.clone(),
            precision,
            max_tolerance,
        }
    }

    /// Returns the shared replacement context; `tool.context().is_same(&context)`.
    pub fn context(&self) -> Handle<ShapeBuildReShape> {
        self.context.clone()
    }

    /// Splits an edge and preserves output parameters when native returns `false`.
    #[allow(clippy::too_many_arguments)] // Matches OCCT's seven-argument SplitEdge method.
    pub fn split_edge(
        &self,
        edge: &Edge,
        parameter: f64,
        vertex: &Vertex,
        face: &Face,
        first: &mut Edge,
        second: &mut Edge,
        precision: f64,
    ) -> Result<bool, Failure> {
        if split_rejected(edge, parameter, vertex, face, precision)? {
            return Ok(false);
        }
        let replacements = split_replacements(edge, parameter, vertex, face, precision)?;
        *first = replacements.0;
        *second = replacements.1;
        Ok(true)
    }

    /// Cuts an edge range and only raises `cut_line` when the native line case does so.
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

fn cut_edge(
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    face: &Face,
    cut_line: &mut bool,
) -> Result<bool, Failure> {
    let difference = (cut - endpoint).abs();
    if difference < 10.0 * precision::p_confusion() {
        return Ok(false);
    }
    let original_range = spatial_range(edge)?;
    if difference < 10.0 * precision::p_confusion() {
        return Ok(false);
    }
    cut_edge_by_pcurve(edge, endpoint, cut, face, original_range, cut_line)
}

fn spatial_range(edge: &Edge) -> Result<(f64, f64), Failure> {
    let curve = BRepTool::curve_with_location(edge)?;
    Ok((curve.first, curve.last))
}

fn cut_edge_by_pcurve(
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    face: &Face,
    original_range: (f64, f64),
    cut_line: &mut bool,
) -> Result<bool, Failure> {
    if BRepTool::same_parameter(edge)? {
        return cut_same_parameter(edge, endpoint, cut, original_range);
    }
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    let trimmed = curve.cast::<Geom2dTrimmedCurve>();
    let Some(trimmed) = trimmed.get() else {
        return Ok(false);
    };
    cut_trimmed_pcurve(
        edge,
        (endpoint, cut),
        (first, last),
        original_range,
        trimmed,
        cut_line,
    )
}

fn cut_trimmed_pcurve(
    edge: &Edge,
    parameters: (f64, f64),
    pcurve_range: (f64, f64),
    original_range: (f64, f64),
    trimmed: &Geom2dTrimmedCurve,
    cut_line: &mut bool,
) -> Result<bool, Failure> {
    let (endpoint, cut) = parameters;
    let builder = BRepBuilder::default();
    builder.range(edge, endpoint.min(cut), endpoint.max(cut), false)?;
    if trimmed
        .basis_curve()
        .cast::<rustycad_tkg2d::geom2d::Geom2dLine>()
        .is_null()
    {
        return Ok(true);
    }
    update_cut_line_range(
        &builder,
        edge,
        endpoint,
        cut,
        pcurve_range,
        original_range,
        cut_line,
    )?;
    Ok(true)
}

fn update_cut_line_range(
    builder: &BRepBuilder,
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    pcurve_range: (f64, f64),
    original_range: (f64, f64),
    cut_line: &mut bool,
) -> Result<(), Failure> {
    let confusion = precision::p_confusion();
    if (endpoint - pcurve_range.1).abs() < confusion {
        let distance = (cut - pcurve_range.0) * (original_range.1 - original_range.0)
            / (pcurve_range.1 - pcurve_range.0);
        builder.range(edge, original_range.0 + distance, original_range.1, true)?;
        *cut_line = true;
    } else if (endpoint - pcurve_range.0).abs() < confusion {
        let distance = (pcurve_range.1 - cut) * (original_range.1 - original_range.0)
            / (pcurve_range.1 - pcurve_range.0);
        builder.range(edge, original_range.0, original_range.1 - distance, true)?;
        *cut_line = true;
    }
    Ok(())
}

fn cut_same_parameter(
    edge: &Edge,
    endpoint: f64,
    cut: f64,
    original_range: (f64, f64),
) -> Result<bool, Failure> {
    let span = (original_range.1 - original_range.0).abs();
    if (span - (cut - endpoint).abs()).abs() < precision::p_confusion() {
        return Ok(false);
    }
    BRepBuilder::default().range(edge, endpoint.min(cut), endpoint.max(cut), false)?;
    Ok(true)
}

fn split_rejected(
    edge: &Edge,
    parameter: f64,
    vertex: &Vertex,
    face: &Face,
    precision: f64,
) -> Result<bool, Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let first = analysis.first_vertex(edge);
    let last = analysis.last_vertex(edge);
    if first.is_same(vertex) || last.is_same(vertex) {
        return Ok(true);
    }
    let (_, a, b) = analysis.pcurve_on_face(edge, face, true)?;
    Ok((a - parameter).abs() < 0.01 * precision || (b - parameter).abs() < 0.01 * precision)
}

fn split_replacements(
    edge: &Edge,
    parameter: f64,
    vertex: &Vertex,
    face: &Face,
    precision: f64,
) -> Result<(Edge, Edge), Failure> {
    raise_vertex_to_split_point(edge, parameter, vertex, face, precision)?;
    let (_, pcurve_first, pcurve_last) =
        ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
    let mut transfer = ShapeAnalysisTransferParametersProj::from_edge(edge, face)?;
    transfer.set_max_tolerance(precision);
    build_split_edges(
        edge,
        vertex,
        parameter,
        (pcurve_first, pcurve_last),
        &mut transfer,
    )
}

fn raise_vertex_to_split_point(
    edge: &Edge,
    parameter: f64,
    vertex: &Vertex,
    face: &Face,
    precision: f64,
) -> Result<(), Failure> {
    let point = split_point(edge, parameter, face)?;
    let distance = point.distance(&BRepTool::pnt(vertex)?);
    if distance > precision {
        BRepBuilder::default().update_vertex_point(vertex, point, distance)?;
    }
    Ok(())
}

fn split_point(
    edge: &Edge,
    parameter: f64,
    face: &Face,
) -> Result<rustycad_tkmath::gp::Point3, Failure> {
    if BRepTool::same_parameter(edge)? && !BRepTool::degenerated(edge)? {
        let (curve, _, _) = BRepTool::curve(edge)?;
        return curve.value(parameter);
    }
    let (curve, _, _) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
    let uv = curve
        .get()
        .ok_or_else(|| Failure::new("ShapeFix_IntersectionTool::SplitEdge has no p-curve"))?
        .value(parameter)?;
    let surface = BRepTool::surface(face)?;
    ShapeAnalysisSurface::new(&surface)?.value(uv.x(), uv.y())
}

fn build_split_edges(
    edge: &Edge,
    vertex: &Vertex,
    parameter: f64,
    pcurve_range: (f64, f64),
    transfer: &mut ShapeAnalysisTransferParametersProj,
) -> Result<(Edge, Edge), Failure> {
    let (first, last) = (
        pcurve_range.0.min(pcurve_range.1),
        pcurve_range.0.max(pcurve_range.1),
    );
    let orientation = edge.orientation();
    let mut forward = edge.clone();
    forward.set_orientation(Orientation::Forward);
    let analysis = ShapeAnalysisEdge::new();
    let builder = BRepBuilder::default();
    let build_edge = ShapeBuildEdge;
    let first_edge = split_piece(
        SplitPiece {
            source: &forward,
            start: &analysis.first_vertex(&forward),
            end: vertex,
            range: (first, parameter),
        },
        &build_edge,
        &builder,
        transfer,
    )?;
    let second_edge = split_piece(
        SplitPiece {
            source: &forward,
            start: vertex,
            end: &analysis.last_vertex(&forward),
            range: (parameter, last),
        },
        &build_edge,
        &builder,
        transfer,
    )?;
    orient_split_pair(first_edge, second_edge, orientation)
}

struct SplitPiece<'a> {
    source: &'a Edge,
    start: &'a Vertex,
    end: &'a Vertex,
    range: (f64, f64),
}

fn split_piece(
    piece: SplitPiece<'_>,
    build_edge: &ShapeBuildEdge,
    builder: &BRepBuilder,
    transfer: &mut ShapeAnalysisTransferParametersProj,
) -> Result<Edge, Failure> {
    let edge = build_edge.copy_replace_vertices(piece.source, piece.start, piece.end)?;
    build_edge.copy_pcurves(&edge, piece.source)?;
    transfer.transfer_range(&edge, piece.range.0, piece.range.1, true)?;
    builder.same_range(&edge, false)?;
    builder.same_parameter(&edge, false)?;
    Ok(edge)
}

fn orient_split_pair(
    mut first: Edge,
    mut second: Edge,
    orientation: Orientation,
) -> Result<(Edge, Edge), Failure> {
    first.set_orientation(orientation);
    second.set_orientation(orientation);
    if orientation == Orientation::Reversed {
        std::mem::swap(&mut first, &mut second);
    }
    Ok((first, second))
}
