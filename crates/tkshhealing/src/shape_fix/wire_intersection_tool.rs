//! Edge splitting and trimming primitives used by `ShapeFix_IntersectionTool`.
#[path = "wire_intersection_tool/aggregate.rs"]
mod aggregate;
#[path = "wire_intersection_tool/boxes.rs"]
mod boxes;
#[path = "wire_intersection_tool/cut.rs"]
mod cut;
#[path = "wire_intersection_tool/vertices.rs"]
mod vertices;
use crate::shape_analysis::{
    ShapeAnalysisEdge, ShapeAnalysisSurface, ShapeAnalysisTransferParametersProj,
    TransferParameters,
};
use crate::shape_build::{ShapeBuildEdge, ShapeBuildReShape};
use crate::shape_extend::ShapeExtendWireData;
use boxes::EdgeBox;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::bnd::BoundingBox2;

/// Native `ShapeFix_IntersectionTool` edge primitives; `tool.cut_edge(&edge, 0.0, 0.4, &face, &mut cut_line)?`.
pub struct ShapeFixIntersectionTool {
    context: Handle<ShapeBuildReShape>,
    // OCCT retains this configuration for other private aggregation routines.
    #[allow(dead_code)]
    precision: f64,
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
        cut::cut_edge(edge, endpoint, cut, face, cut_line)
    }

    /// Repairs nonadjacent self-intersections in a wire using this tool's replacement context.
    ///
    /// Example: `tool.fix_self_intersect_wire(&wire, &face, &mut split, &mut cut, &mut removed)?`.
    pub fn fix_self_intersect_wire(
        &self,
        wire: &ShapeExtendWireData,
        face: &Face,
        split_count: &mut i32,
        cut_count: &mut i32,
        removed_count: &mut i32,
    ) -> Result<bool, Failure> {
        aggregate::fix_self_intersect_wire(self, wire, face, split_count, cut_count, removed_count)
    }

    fn max_tolerance(&self) -> f64 {
        self.max_tolerance
    }

    #[allow(clippy::too_many_arguments)] // Mirrors native's private SplitEdge1 helper.
    fn split_edge1(
        &self,
        wire: &ShapeExtendWireData,
        face: &Face,
        index: i32,
        parameter: f64,
        vertex: &Vertex,
        tolerance: f64,
        boxes: &mut Vec<EdgeBox>,
    ) -> Result<bool, Failure> {
        let source = wire.edge(index)?;
        let (mut first, mut second) = (Edge::default(), Edge::default());
        if !self.split_edge(
            &source,
            parameter,
            vertex,
            face,
            &mut first,
            &mut second,
            tolerance,
        )? {
            return Ok(false);
        }
        boxes::replace_split_edge(self, wire, face, index, &source, &first, &second, boxes)?;
        Ok(true)
    }

    #[allow(clippy::too_many_arguments)] // Mirrors native's SplitEdge2 helper.
    fn split_edge2(
        &self,
        wire: &ShapeExtendWireData,
        face: &Face,
        index: i32,
        parameters: (f64, f64),
        vertex: &Vertex,
        tolerance: f64,
        boxes: &mut Vec<EdgeBox>,
    ) -> Result<bool, Failure> {
        let source = wire.edge(index)?;
        let (mut first, mut second) = (Edge::default(), Edge::default());
        let middle = 0.5 * (parameters.0 + parameters.1);
        if !self.split_edge(
            &source,
            middle,
            vertex,
            face,
            &mut first,
            &mut second,
            tolerance,
        )? {
            return Ok(false);
        }
        trim_overlap_pieces(self, &first, &second, parameters, face)?;
        boxes::replace_split_edge(self, wire, face, index, &source, &first, &second, boxes)?;
        Ok(true)
    }

    #[allow(clippy::too_many_arguments)] // Mirrors native's private FindVertAndSplitEdge helper.
    fn find_vertex_and_split_edge(
        &self,
        parameter: f64,
        edge: &Edge,
        other: &Edge,
        pcurve: &Handle<dyn Geom2dCurve>,
        max_vertex_tolerance: &mut f64,
        index: &mut i32,
        wire: &ShapeExtendWireData,
        face: &Face,
        boxes: &mut Vec<EdgeBox>,
        allow_shared_endpoint: bool,
    ) -> Result<bool, Failure> {
        let candidate = vertices::nearest_vertex(edge, other, parameter, pcurve, face)?;
        if !candidate.should_split && !allow_shared_endpoint {
            return Ok(false);
        }
        if !self.split_edge1(
            wire,
            face,
            *index,
            parameter,
            &candidate.vertex,
            candidate.tolerance,
            boxes,
        )? {
            return Ok(false);
        }
        BRepBuilder::default().update_vertex_tolerance(&candidate.vertex, candidate.tolerance)?;
        *max_vertex_tolerance = max_vertex_tolerance.max(candidate.tolerance);
        *index -= 1;
        Ok(true)
    }

    fn union_vertices(
        &self,
        wire: &ShapeExtendWireData,
        first: &Edge,
        second: &mut Edge,
        second_index: i32,
        boxes: &mut Vec<EdgeBox>,
        second_box: &BoundingBox2,
    ) -> Result<bool, Failure> {
        let Some(union) = vertices::closest_vertex_union(first, second)? else {
            return Ok(false);
        };
        if union.keep.is_same(&union.discard) || union.distance >= union.tolerance {
            return Ok(false);
        }
        BRepBuilder::default().update_vertex_tolerance(&union.keep, union.tolerance)?;
        vertices::replace_union_edges(self, wire, second, second_index, &union, boxes, second_box)?;
        Ok(true)
    }
}

fn point_on_edge(
    edge: &Edge,
    parameter: f64,
    pcurve: &Handle<dyn Geom2dCurve>,
    face: &Face,
) -> Result<rustycad_tkmath::gp::Point3, Failure> {
    if BRepTool::same_parameter(edge)? {
        let (curve, _, _) = BRepTool::curve(edge)?;
        if !curve.is_null() {
            return curve.value(parameter);
        }
    }
    let uv = pcurve.value(parameter)?;
    let surface = BRepTool::surface(face)?;
    ShapeAnalysisSurface::new(&surface)?.value(uv.x(), uv.y())
}

fn trim_overlap_pieces(
    tool: &ShapeFixIntersectionTool,
    first: &Edge,
    second: &Edge,
    parameters: (f64, f64),
    face: &Face,
) -> Result<(), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let (first_curve, first_start, first_end) = analysis.pcurve_on_face(first, face, false)?;
    let (second_curve, second_start, second_end) = analysis.pcurve_on_face(second, face, false)?;
    if first_curve.is_null() || second_curve.is_null() {
        return Ok(());
    }
    cut_overlap_pieces(
        tool,
        first,
        second,
        (first_start, first_end),
        (second_start, second_end),
        parameters,
        face,
    )
}

fn cut_overlap_pieces(
    tool: &ShapeFixIntersectionTool,
    first: &Edge,
    second: &Edge,
    first_range: (f64, f64),
    second_range: (f64, f64),
    parameters: (f64, f64),
    face: &Face,
) -> Result<(), Failure> {
    let first_at_end = first_range.1 == 0.5 * (parameters.0 + parameters.1);
    let parameter_order = if first_at_end {
        (first_range.1 - first_range.0) * (first_range.1 - parameters.0) > 0.0
    } else {
        (first_range.0 - first_range.1) * (first_range.0 - parameters.0) > 0.0
    };
    let (first_cut, second_cut) = overlap_cut_parameters(
        first_range,
        second_range,
        parameters,
        first_at_end,
        parameter_order,
    );
    let mut cut_line = false;
    tool.cut_edge(first, first_cut.0, first_cut.1, face, &mut cut_line)?;
    tool.cut_edge(second, second_cut.0, second_cut.1, face, &mut cut_line)?;
    Ok(())
}

fn overlap_cut_parameters(
    first_range: (f64, f64),
    second_range: (f64, f64),
    parameters: (f64, f64),
    first_at_end: bool,
    parameter_order: bool,
) -> ((f64, f64), (f64, f64)) {
    let (first_endpoint, second_endpoint) = if first_at_end {
        (first_range.0, second_range.1)
    } else {
        (first_range.1, second_range.0)
    };
    if parameter_order {
        (
            (first_endpoint, parameters.0),
            (second_endpoint, parameters.1),
        )
    } else {
        (
            (first_endpoint, parameters.1),
            (second_endpoint, parameters.0),
        )
    }
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
