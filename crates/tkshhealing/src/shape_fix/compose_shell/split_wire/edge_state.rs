use super::super::{ComposeShellFields, ShapeFixComposeShell};
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Shape, Vertex},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Point3};

pub(super) struct EdgeSplitState {
    pub(super) first_vertex: Vertex,
    pub(super) last_vertex: Vertex,
    pub(super) first_point: Point3,
    pub(super) last_point: Point3,
    pub(super) previous_grid_point: Point3,
    pub(super) last_grid_point: Point3,
    pub(super) previous_uv: Point2,
    pub(super) last_uv: Point2,
    pub(super) curve_3d: Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    pub(super) pcurve: Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    pub(super) first_parameter: f64,
    pub(super) previous_parameter: f64,
    pub(super) current_parameter: f64,
    pub(super) last_parameter: f64,
    pub(super) first_3d_parameter: f64,
    pub(super) last_3d_parameter: f64,
    pub(super) edge_tolerance: f64,
    pub(super) first_vertex_tolerance: f64,
    pub(super) last_vertex_tolerance: f64,
    pub(super) same_parameter: bool,
    pub(super) non_manifold_vertices: Vec<Shape>,
    pub(super) non_manifold_parameters: Vec<f64>,
    pub(super) periodic: bool,
    pub(super) period: f64,
}

impl EdgeSplitState {
    pub(super) fn new(
        composer: &ShapeFixComposeShell,
        fields: &ComposeShellFields,
        edge: &Edge,
    ) -> Result<Self, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let (curve_3d, first_3d_parameter, last_3d_parameter) = analysis.curve_3d(edge, true)?;
        let (pcurve, first_parameter, last_parameter) =
            analysis.pcurve_on_face(edge, &fields.face, true)?;
        if pcurve.is_null() {
            super::edge_projection::mark_fail2(composer);
            return Err(super::null_pcurve());
        }
        let first_vertex = analysis.first_vertex(edge);
        let last_vertex = analysis.last_vertex(edge);
        let inverse = fields.location.inverted()?.transformation();
        let first_point = BRepTool::pnt(&first_vertex)?.transformed(inverse);
        let last_point = BRepTool::pnt(&last_vertex)?.transformed(inverse);
        let first_uv = pcurve.value(first_parameter)?;
        let last_uv = pcurve.value(last_parameter)?;
        let previous_grid_point = fields.grid.value_at(first_uv)?;
        let last_grid_point = fields.grid.value_at(last_uv)?;
        let non_manifold_vertices = super::edge_projection::non_manifold_vertices(edge);
        let non_manifold_parameters = super::edge_projection::non_manifold_parameters(
            &non_manifold_vertices,
            &curve_3d,
            &pcurve,
            &fields.face,
            (first_3d_parameter, last_3d_parameter),
            (first_parameter, last_parameter),
        )?;
        let (first_vertex_tolerance, last_vertex_tolerance) =
            super::edge_projection::vertex_tolerances(composer, &first_vertex, &last_vertex)?;
        Ok(Self {
            first_vertex,
            last_vertex,
            first_point,
            last_point,
            previous_grid_point,
            last_grid_point,
            previous_uv: first_uv,
            last_uv,
            curve_3d,
            pcurve: pcurve.clone(),
            first_parameter,
            previous_parameter: first_parameter,
            current_parameter: first_parameter,
            last_parameter,
            first_3d_parameter,
            last_3d_parameter,
            edge_tolerance: BRepTool::edge_tolerance(edge)?,
            first_vertex_tolerance,
            last_vertex_tolerance,
            same_parameter: first_3d_parameter == first_parameter
                && last_3d_parameter == last_parameter,
            non_manifold_vertices,
            non_manifold_parameters,
            periodic: pcurve.is_periodic(),
            period: pcurve.period().unwrap_or(0.0),
        })
    }
}
