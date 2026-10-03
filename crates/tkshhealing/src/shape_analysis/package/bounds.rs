//! Native endpoint and p-curve bounds from ShapeAnalysis.cxx at 3d097a0328e71b8.
use super::{ShapeAnalysis, ShapeAnalysisCurve};
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::{self, Explorer},
    topods::{self, Face, Shape, Vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox2;
impl ShapeAnalysis {
    /// Return oriented endpoints of a vertex, edge or wire; other shapes have null endpoints.
    /// Example: `let (first,last) = ShapeAnalysis::find_bounds(&shape)?;`.
    pub fn find_bounds(shape: &Shape) -> Result<(Vertex, Vertex), Failure> {
        match shape.shape_type()? {
            ShapeEnum::Wire => Ok(topexp::wire_vertices(&topods::wire(shape)?)),
            ShapeEnum::Edge => {
                let edge = topods::edge(shape)?;
                let analyzer = ShapeAnalysisEdge::new();
                Ok((analyzer.first_vertex(&edge), analyzer.last_vertex(&edge)))
            }
            ShapeEnum::Vertex => {
                let vertex = topods::vertex(shape)?;
                Ok((vertex.clone(), vertex))
            }
            _ => Ok((Vertex::default(), Vertex::default())),
        }
    }
    /// Bound all face p-curves, or use natural surface bounds when no edges exist.
    /// Example: `let (umin,umax,vmin,vmax) = ShapeAnalysis::get_face_uv_bounds(&face)?;`.
    pub fn get_face_uv_bounds(face: &Face) -> Result<(f64, f64, f64, f64), Failure> {
        let forward = face.oriented(Orientation::Forward);
        let mut edges = Explorer::new(&forward, ShapeEnum::Edge, ShapeEnum::Shape)?;
        if !edges.more() {
            return Ok(BRepTool::surface_with_location(face)?.0.bounds());
        }
        let mut bounds = BoundingBox2::default();
        while edges.more() {
            include_pcurve(&topods::edge(edges.current())?, face, &mut bounds)?;
            edges.next()?;
        }
        let [umin, vmin, umax, vmax] = bounds.get()?;
        Ok((umin, umax, vmin, vmax))
    }
}
fn include_pcurve(
    edge: &rustycad_tkbrep::topods::Edge,
    face: &Face,
    bounds: &mut BoundingBox2,
) -> Result<(), Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
    if curve.is_null() {
        return Ok(());
    }
    ShapeAnalysisCurve.fill_bnd_box(&curve, first, last, 20, true, bounds)
}
