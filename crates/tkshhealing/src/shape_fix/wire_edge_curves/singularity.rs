//! Edge splitting and p-curve adjustment at collapsed surface boundaries.
use super::super::ShapeFixWire;
use crate::shape_analysis::TransferParameters;
use crate::shape_analysis::{
    ShapeAnalysisCurve, ShapeAnalysisEdge, ShapeAnalysisTransferParameters,
};
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::ShapeExtendWireData;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::Orientation;

impl ShapeFixWire {
    pub(super) fn split_edge_at_surface_singularities(
        &self,
        data: &ShapeExtendWireData,
        index: i32,
        face: &Face,
    ) -> Result<bool, Failure> {
        if self.context().is_null() {
            return Ok(false);
        }
        let edge = data.edge(index)?;
        let curve_data = BRepTool::curve(&edge)?;
        if curve_data.0.is_null() {
            return Ok(false);
        }
        let parameters = singularity_parameters(
            &self.working_analyzer().surface(),
            &curve_data.0,
            (curve_data.1, curve_data.2),
            self.min_tolerance(),
        )?;
        if parameters.is_empty() {
            return Ok(false);
        }
        self.replace_with_singular_segments(data, edge, face, parameters)
    }

    pub(super) fn repair_overdegenerated_pcurve(
        &self,
        data: &ShapeExtendWireData,
        index: i32,
        face: &Face,
    ) -> Result<(), Failure> {
        let analyzer = self.working_analyzer().surface();
        if !analyzer.is_u_closed(self.precision())? {
            return Ok(());
        }
        if !wire_pcurve_winds_once(data, face, analyzer.bounds())? {
            return Ok(());
        }
        self.reproject_overdegenerated_edge(data, index, face)
    }

    fn replace_with_singular_segments(
        &self,
        data: &ShapeExtendWireData,
        mut edge: Edge,
        face: &Face,
        parameters: Vec<f64>,
    ) -> Result<bool, Failure> {
        let forward = edge.orientation() == Orientation::Forward;
        edge.set_orientation(Orientation::Forward);
        let segments = singularity_segments(&edge, face, &parameters)?;
        if segments.nb_edges() == 0 {
            return Ok(false);
        }
        self.replace_edge_with_segments(&mut edge, &segments, forward)?;
        self.update_wire()?;
        let _ = data;
        Ok(true)
    }

    fn replace_edge_with_segments(
        &self,
        edge: &mut Edge,
        segments: &ShapeExtendWireData,
        forward: bool,
    ) -> Result<(), Failure> {
        let context = self.context();
        if !forward {
            segments.reverse()?;
            edge.set_orientation(Orientation::Reversed);
        }
        context.replace(&edge.clone().into_shape(), &segments.wire()?.into_shape())
    }

    fn reproject_overdegenerated_edge(
        &self,
        data: &ShapeExtendWireData,
        index: i32,
        face: &Face,
    ) -> Result<(), Failure> {
        let edge = data.edge(index)?;
        ShapeBuildEdge.remove_pcurve_on_face(&edge, face)?;
        self.edge_fixer.projector().set_adjust_over_degen_mode(0);
        let analyzer = self.working_analyzer();
        self.edge_fixer.fix_add_pcurve_on_face_with_analysis(
            &edge,
            face,
            data.is_seam(index),
            &analyzer.surface(),
            self.precision(),
        )?;
        Ok(())
    }
}

fn singularity_parameters(
    analysis: &Handle<crate::shape_analysis::ShapeAnalysisSurface>,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    tolerance: f64,
) -> Result<Vec<f64>, Failure> {
    let count = analysis.nb_singularities(tolerance)?;
    let mut parameters = Vec::new();
    for index in 1..=count as i32 {
        add_singularity_projection(analysis, curve, range, tolerance, index, &mut parameters)?;
    }
    Ok(parameters)
}

fn add_singularity_projection(
    analysis: &Handle<crate::shape_analysis::ShapeAnalysisSurface>,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    tolerance: f64,
    index: i32,
    parameters: &mut Vec<f64>,
) -> Result<(), Failure> {
    let Some(singularity) = analysis.singularity(index)? else {
        return Ok(());
    };
    let (distance, _, parameter) =
        ShapeAnalysisCurve.project(curve, singularity.point, tolerance, range, true)?;
    if distance < singularity.precision.max(tolerance) {
        insert_split_parameter(parameters, parameter, range);
    }
    Ok(())
}

fn insert_split_parameter(parameters: &mut Vec<f64>, parameter: f64, range: (f64, f64)) {
    let margin = precision::p_confusion();
    if parameter - range.0 <= margin || range.1 - parameter <= margin {
        return;
    }
    let index = parameters.partition_point(|existing| *existing < parameter - margin);
    if parameters
        .get(index)
        .is_some_and(|existing| (parameter - existing).abs() <= margin)
    {
        return;
    }
    parameters.insert(index, parameter);
}

fn singularity_segments(
    edge: &Edge,
    face: &Face,
    parameters: &[f64],
) -> Result<ShapeExtendWireData, Failure> {
    let cursor = split_cursor(edge)?;
    let (curve, _, _) = BRepTool::curve(edge)?;
    let segments = ShapeExtendWireData::new();
    append_split_segments(&segments, edge, face, &curve, parameters, cursor)?;
    Ok(segments)
}

struct SplitCursor {
    range: (f64, f64),
    first_vertex: Vertex,
    last_vertex: Vertex,
}

fn split_cursor(edge: &Edge) -> Result<SplitCursor, Failure> {
    Ok(SplitCursor {
        range: BRepTool::range(edge)?,
        first_vertex: ShapeAnalysisEdge::new().first_vertex(edge),
        last_vertex: ShapeAnalysisEdge::new().last_vertex(edge),
    })
}

fn append_split_segments(
    segments: &ShapeExtendWireData,
    edge: &Edge,
    face: &Face,
    curve: &Handle<dyn GeomCurve>,
    parameters: &[f64],
    mut cursor: SplitCursor,
) -> Result<(), Failure> {
    for parameter in parameters {
        let vertex = make_singularity_vertex(edge, curve, *parameter)?;
        if cursor.absorb_near_vertex(&vertex, *parameter)? {
            continue;
        }
        cursor.add_segment(segments, edge, face, &vertex, *parameter)?;
    }
    cursor.add_segment(
        segments,
        edge,
        face,
        &cursor.last_vertex.clone(),
        cursor.range.1,
    )
}

impl SplitCursor {
    fn absorb_near_vertex(&mut self, vertex: &Vertex, parameter: f64) -> Result<bool, Failure> {
        absorb_near_vertex(
            &mut self.first_vertex,
            &mut self.last_vertex,
            vertex,
            &mut self.range,
            parameter,
        )
    }

    fn add_segment(
        &mut self,
        segments: &ShapeExtendWireData,
        edge: &Edge,
        face: &Face,
        vertex: &Vertex,
        end: f64,
    ) -> Result<(), Failure> {
        add_edge_segment(
            segments,
            edge,
            face,
            &self.first_vertex,
            vertex,
            &mut self.range,
            end,
        )?;
        self.first_vertex = vertex.clone();
        Ok(())
    }
}

fn make_singularity_vertex(
    edge: &Edge,
    curve: &Handle<dyn GeomCurve>,
    parameter: f64,
) -> Result<Vertex, Failure> {
    let point = curve.value(parameter)?;
    let tolerance = BRepTool::edge_tolerance(edge)?;
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, point, tolerance)?;
    Ok(vertex)
}

fn absorb_near_vertex(
    first: &mut Vertex,
    last: &mut Vertex,
    split: &Vertex,
    range: &mut (f64, f64),
    parameter: f64,
) -> Result<bool, Failure> {
    let builder = BRepBuilder::default();
    let first_tolerance = BRepTool::vertex_tolerance(first)?;
    let first_distance = vertex_distance(first, split)?;
    if first_distance < first_tolerance * 1.01 {
        builder.update_vertex_tolerance(first, first_distance.max(first_tolerance))?;
        range.0 = parameter;
        *first = split.clone();
        return Ok(true);
    }
    let last_tolerance = BRepTool::vertex_tolerance(last)?;
    let last_distance = vertex_distance(last, split)?;
    if last_distance < last_tolerance * 1.01 {
        builder.update_vertex_tolerance(split, last_distance.max(last_tolerance))?;
        range.1 = parameter;
        *last = split.clone();
        return Ok(true);
    }
    Ok(false)
}

fn vertex_distance(first: &Vertex, last: &Vertex) -> Result<f64, Failure> {
    Ok(BRepTool::pnt(first)?.distance(&BRepTool::pnt(last)?))
}

fn add_edge_segment(
    segments: &ShapeExtendWireData,
    edge: &Edge,
    face: &Face,
    first_vertex: &Vertex,
    last_vertex: &Vertex,
    range: &mut (f64, f64),
    end: f64,
) -> Result<(), Failure> {
    let piece = ShapeBuildEdge.copy_replace_vertices(edge, first_vertex, last_vertex)?;
    if BRepTool::same_parameter(edge)? {
        BRepBuilder::default().range(&piece, range.0, end, false)?;
    } else {
        transfer_split_range(edge, face, &piece, range.0, end)?;
    }
    segments.add_edge(&piece, 0)?;
    range.0 = end;
    Ok(())
}

fn transfer_split_range(
    edge: &Edge,
    face: &Face,
    piece: &Edge,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let transfer = ShapeAnalysisTransferParameters::from_edge(edge, face)?;
    transfer.transfer_range(piece, first, last, false)
}

fn wire_pcurve_winds_once(
    data: &ShapeExtendWireData,
    face: &Face,
    bounds: (f64, f64, f64, f64),
) -> Result<bool, Failure> {
    let mut total_u = 0.0;
    for index in 1..=data.nb_edges() {
        let Some(delta) = pcurve_delta(data, face, index)? else {
            return Ok(false);
        };
        total_u += delta;
    }
    Ok((total_u.abs() - (bounds.1 - bounds.0).abs()).abs() < 0.1 * (bounds.1 - bounds.0).abs())
}

fn pcurve_delta(
    data: &ShapeExtendWireData,
    face: &Face,
    index: i32,
) -> Result<Option<f64>, Failure> {
    let edge = data.edge(index)?;
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(&edge, face, true)?;
    if curve.is_null() {
        return Ok(None);
    }
    let first_u = curve.value(first)?.x();
    let last_u = curve.value(last)?.x();
    Ok(Some(last_u - first_u))
}
