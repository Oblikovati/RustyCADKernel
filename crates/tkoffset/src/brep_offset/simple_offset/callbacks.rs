//! Preserve callback outputs on misses and source geometry for parameter callbacks.
use super::BRepOffsetSimpleOffset;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewCurve, NewSurface};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;

impl BRepToolsModification for BRepOffsetSimpleOffset {
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        if !self.faces.is_bound(face) {
            return Ok(false);
        }
        let geometry = self.faces.find(face)?;
        *result = NewSurface {
            surface: geometry.offset_surface.clone(),
            location: geometry.location.clone(),
            tolerance: geometry.tolerance,
            rev_wires: geometry.reverse_wires,
            rev_face: geometry.reverse_face,
        };
        Ok(true)
    }

    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        if !self.edges.is_bound(edge) {
            return Ok(false);
        }
        let geometry = self.edges.find(edge)?;
        *result = NewCurve {
            curve: geometry.curve.clone(),
            location: geometry.location.clone(),
            tolerance: geometry.tolerance,
        };
        Ok(true)
    }

    fn new_point(
        &self,
        vertex: &Vertex,
        point: &mut Point3,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        if !self.vertices.is_bound(vertex) {
            return Ok(false);
        }
        let geometry = self.vertices.find(vertex)?;
        *point = geometry.point;
        *tolerance = geometry.tolerance;
        Ok(true)
    }

    fn new_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        _new_edge: &Edge,
        _new_face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        *curve = BRepTool::curve_on_face(edge, face)?.pcurve;
        *tolerance = BRepTool::edge_tolerance(edge)?;
        if self.edges.is_bound(edge) {
            *tolerance = self.edges.find(edge)?.tolerance;
        }
        Ok(true)
    }

    fn new_parameter(
        &self,
        vertex: &Vertex,
        edge: &Edge,
        parameter: &mut f64,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        *parameter = BRepTool::parameter(vertex, edge)?;
        *tolerance = BRepTool::vertex_tolerance(vertex)?;
        if self.vertices.is_bound(vertex) {
            *tolerance = self.vertices.find(vertex)?.tolerance;
        }
        Ok(true)
    }

    fn continuity(
        &self,
        edge: &Edge,
        face1: &Face,
        face2: &Face,
        _new_edge: &Edge,
        _new_face1: &Face,
        _new_face2: &Face,
    ) -> Result<Continuity, Failure> {
        BRepTool::continuity_on_faces(edge, face1, face2)
    }
}
