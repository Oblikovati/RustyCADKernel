use super::BRepOffsetAnalyse;
use crate::brep_offset::BRepOffsetInterval;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorCurve,
    topexp::{self, Explorer},
    topods::{self, Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{self, Vector3};
impl BRepOffsetAnalyse {
    /// Read edge classification intervals; `analysis.type_of(&edge)?`.
    pub fn type_of(&self, edge: &Edge) -> Result<&[BRepOffsetInterval], Failure> {
        Ok(self.intervals.find(edge)?.as_slice())
    }
    /// Check incidence membership; `analysis.has_ancestor(&edge)`.
    pub fn has_ancestor(&self, shape: &Shape) -> bool {
        self.ancestors.is_bound(shape)
    }
    /// Read unique ancestors in native exploration order; `analysis.ancestors(&edge)?`.
    pub fn ancestors(&self, shape: &Shape) -> Result<&[Shape], Failure> {
        Ok(self.ancestors.find(shape)?.as_slice())
    }
    /// Replace output with matching face edges; `analysis.edges_on_face(&face, kind, &mut edges)?`.
    pub fn edges_on_face(
        &self,
        face: &Face,
        kind: TypeOfConcavity,
        edges: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        edges.clear();
        for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            for interval in self.type_of(&topods::edge(&shape)?)? {
                if interval.get_type()? == kind {
                    edges.push(shape.clone());
                }
            }
        }
        Ok(())
    }
    /// Replace output with endpoint-matching edges, retaining duplicates; `analysis.edges_on_vertex(&vertex, kind, &mut edges)?`.
    pub fn edges_on_vertex(
        &self,
        vertex: &Vertex,
        kind: TypeOfConcavity,
        edges: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        edges.clear();
        for shape in self.ancestors(vertex)? {
            if !self.intervals.is_bound(shape) {
                continue;
            }
            let intervals = self.intervals.find(shape)?;
            if intervals.is_empty() {
                continue;
            }
            let edge = topods::edge(shape)?;
            let (mut first, mut last) = topexp::vertices(&edge, false);
            if edge.orientation() == Orientation::Reversed {
                std::mem::swap(&mut first, &mut last);
            }
            if first.is_same(vertex) && intervals.last().unwrap().get_type()? == kind {
                edges.push(shape.clone());
            }
            if last.is_same(vertex) && intervals[0].get_type()? == kind {
                edges.push(shape.clone());
            }
        }
        Ok(())
    }
    /// Find opposite outgoing tangents; a null reference derivative preserves output; `analysis.tangent_edges(&edge, &vertex, &mut edges)?`.
    pub fn tangent_edges(
        &self,
        edge: &Edge,
        vertex: &Vertex,
        edges: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        let reference = outgoing_tangent(edge, vertex)?;
        if reference.square_magnitude() < gp::resolution() {
            return Ok(());
        }
        edges.clear();
        for shape in self.ancestors(vertex)? {
            if shape.is_same(edge) {
                continue;
            }
            let tangent = outgoing_tangent(&topods::edge(shape)?, vertex)?;
            if tangent.square_magnitude() < gp::resolution() {
                continue;
            }
            if tangent.is_opposite(reference, self.angle)? {
                edges.push(shape.clone());
            }
        }
        Ok(())
    }
}
fn outgoing_tangent(edge: &Edge, vertex: &Vertex) -> Result<Vector3, Failure> {
    let parameter = BRepTool::parameter(vertex, edge)?;
    let tangent = BRepAdaptorCurve::with_edge(edge)?.dn(parameter, 1)?;
    if topexp::last_vertex(edge, false).is_same(vertex) {
        return Ok(tangent.reversed());
    }
    Ok(tangent)
}
