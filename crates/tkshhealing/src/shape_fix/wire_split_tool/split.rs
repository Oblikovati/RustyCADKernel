//! Preserve the SplitTool tolerance-only vertex update and SameParameter repair.
use super::ShapeFixSplitTool;
use crate::{
    shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisTransferParametersProj, TransferParameters},
    shape_build::ShapeBuildEdge,
    shape_fix::ShapeFixEdge,
};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face, Vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
impl ShapeFixSplitTool {
    /// Split at a p-curve parameter, preserving rejected outputs; `tool.split_edge(&edge, p, &vertex, &face, &mut a, &mut b, 1e-7, 1e-9)?`.
    #[allow(clippy::too_many_arguments)] // Native SplitEdge output and tolerance parameters.
    pub fn split_edge(
        &self,
        edge: &Edge,
        parameter: f64,
        vertex: &Vertex,
        face: &Face,
        first: &mut Edge,
        second: &mut Edge,
        tolerance_3d: f64,
        tolerance_2d: f64,
    ) -> Result<bool, Failure> {
        let (_, a, b) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
        if (a - parameter).abs() < tolerance_2d || (b - parameter).abs() < tolerance_2d {
            return Ok(false);
        }
        let Some(point) = split_point(edge, face, parameter)? else {
            return Ok(false);
        };
        let distance = point.distance(&BRepTool::pnt(vertex)?);
        if distance > tolerance_3d {
            BRepBuilder::default().update_vertex_tolerance(vertex, distance)?;
        }
        let mut transfer = ShapeAnalysisTransferParametersProj::new();
        transfer.set_max_tolerance(tolerance_3d);
        transfer.init(edge, face)?;
        finish_split(
            edge,
            vertex,
            parameter,
            (a, b),
            &mut transfer,
            (first, second),
        )?;
        Ok(true)
    }
    /// Split and remove an interval around one vertex; `tool.split_edge_cut(&edge,p,q,&v,&face,&mut a,&mut b,1e-7,1e-9)?`.
    #[allow(clippy::too_many_arguments)] // Mirrors the two-parameter native overload.
    pub fn split_edge_cut(
        &self,
        edge: &Edge,
        parameter1: f64,
        parameter2: f64,
        vertex: &Vertex,
        face: &Face,
        first: &mut Edge,
        second: &mut Edge,
        tolerance_3d: f64,
        tolerance_2d: f64,
    ) -> Result<bool, Failure> {
        let middle = (parameter1 + parameter2) / 2.;
        if !self.split_edge(
            edge,
            middle,
            vertex,
            face,
            first,
            second,
            tolerance_3d,
            tolerance_2d,
        )? {
            return Ok(false);
        }
        cut_pair(self, first, second, face, middle, (parameter1, parameter2))?;
        Ok(true)
    }
}
fn finish_split(
    edge: &Edge,
    vertex: &Vertex,
    parameter: f64,
    range: (f64, f64),
    transfer: &mut ShapeAnalysisTransferParametersProj,
    outputs: (&mut Edge, &mut Edge),
) -> Result<(), Failure> {
    let mut forward = edge.clone();
    forward.set_orientation(Orientation::Forward);
    let analysis = ShapeAnalysisEdge::new();
    let first_vertex = analysis.first_vertex(&forward);
    let last_vertex = analysis.last_vertex(&forward);
    let (lower, upper) = if range.0 < range.1 {
        range
    } else {
        (range.1, range.0)
    };
    split_piece(
        outputs.0,
        &forward,
        &first_vertex,
        vertex,
        (lower, parameter),
        transfer,
    )?;
    split_piece(
        outputs.1,
        &forward,
        vertex,
        &last_vertex,
        (parameter, upper),
        transfer,
    )?;
    outputs.0.set_orientation(edge.orientation());
    outputs.1.set_orientation(edge.orientation());
    if edge.orientation() == Orientation::Reversed {
        std::mem::swap(outputs.0, outputs.1);
    }
    Ok(())
}
fn split_point(edge: &Edge, face: &Face, parameter: f64) -> Result<Option<Point3>, Failure> {
    if BRepTool::same_parameter(edge)? {
        let (curve, _, _) = BRepTool::curve(edge)?;
        if curve.is_null() {
            return Ok(None);
        }
        return Ok(Some(curve.value(parameter)?));
    }
    let (curve, _, _) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
    let pcurve = curve.get().ok_or_else(|| {
        Failure::new("ShapeFix_SplitTool::SplitEdge: null pcurve; expected a curve on the face")
    })?;
    let uv = pcurve.value(parameter)?;
    Ok(Some(BRepTool::surface(face)?.value(uv.x(), uv.y())?))
}
fn split_piece(
    piece: &mut Edge,
    edge: &Edge,
    first: &Vertex,
    last: &Vertex,
    range: (f64, f64),
    transfer: &mut ShapeAnalysisTransferParametersProj,
) -> Result<(), Failure> {
    *piece = ShapeBuildEdge.copy_replace_vertices(edge, first, last)?;
    ShapeBuildEdge.copy_pcurves(piece, edge)?;
    transfer.transfer_range(piece, range.0, range.1, true)?;
    BRepBuilder::default().same_range(piece, false)?;
    ShapeFixEdge::new().fix_same_parameter_default(piece)?;
    Ok(())
}
fn cut_pair(
    tool: &ShapeFixSplitTool,
    first: &Edge,
    second: &Edge,
    face: &Face,
    middle: f64,
    parameters: (f64, f64),
) -> Result<(), Failure> {
    let analysis = ShapeAnalysisEdge::new();
    let (curve1, a, b) = analysis.pcurve_on_face(first, face, false)?;
    if curve1.is_null() {
        return Ok(());
    }
    let (curve2, c, d) = analysis.pcurve_on_face(second, face, false)?;
    if curve2.is_null() {
        return Ok(());
    }
    let (endpoint1, endpoint2, order) = if b == middle {
        (a, d, (b - a) * (b - parameters.0) > 0.)
    } else {
        (b, c, (a - b) * (a - parameters.0) > 0.)
    };
    let cuts = if order {
        parameters
    } else {
        (parameters.1, parameters.0)
    };
    let mut cut_line = false;
    tool.cut_edge(first, endpoint1, cuts.0, face, &mut cut_line)?;
    tool.cut_edge(second, endpoint2, cuts.1, face, &mut cut_line)?;
    Ok(())
}
