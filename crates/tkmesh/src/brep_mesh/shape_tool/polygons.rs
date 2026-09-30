//! Direct native BRep polygon storage, including inverse face placement.
use super::*;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation};
impl ShapeTool {
    /// Transform mesh nodes into face coordinates and store it; `ShapeTool::add_in_face(&face, &mesh)?`.
    pub fn add_in_face(face: &Face, triangulation: &Handle<Triangulation>) -> Result<(), Failure> {
        if !face.location().is_identity() {
            let inverse = face.location().transformation().inverted()?;
            for index in 1..=triangulation.nb_nodes() {
                triangulation.set_node(index, triangulation.node(index)?.transformed(inverse))?;
            }
        }
        BRepBuilder::default().update_face_triangulation(face, triangulation, true)
    }
    /// Remove face triangulation; `ShapeTool::nullify_face(&face)?`.
    pub fn nullify_face(face: &Face) -> Result<(), Failure> {
        BRepBuilder::default().update_face_triangulation(face, &Handle::default(), true)
    }
    /// Remove polygon associated with the mesh and location; `ShapeTool::nullify_edge_on_triangulation(&edge, &mesh, &location)?`.
    pub fn nullify_edge_on_triangulation(
        edge: &Edge,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> Result<(), Failure> {
        Self::update_edge_on_triangulation(edge, &Handle::default(), triangulation, location)
    }
    /// Remove 3D polygon at the supplied placement; `ShapeTool::nullify_edge_polygon3d(&edge, &location)?`.
    pub fn nullify_edge_polygon3d(edge: &Edge, location: &Location) -> Result<(), Failure> {
        BRepBuilder::default().update_edge_polygon3d(edge, &Handle::default(), location)
    }
    /// Store polygon on a triangulation; `ShapeTool::update_edge_on_triangulation(&edge, &polygon, &mesh, &location)?`.
    pub fn update_edge_on_triangulation(
        edge: &Edge,
        polygon: &Handle<PolygonOnTriangulation>,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> Result<(), Failure> {
        BRepBuilder::default().update_edge_polygon_on_triangulation(
            edge,
            polygon,
            triangulation,
            location,
        )
    }
    /// Store two representations of a seam; `ShapeTool::update_edge_seam(&edge, (&first, &last), &mesh, &location)?`.
    pub fn update_edge_seam(
        edge: &Edge,
        polygons: (
            &Handle<PolygonOnTriangulation>,
            &Handle<PolygonOnTriangulation>,
        ),
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> Result<(), Failure> {
        BRepBuilder::default().update_edge_polygons_on_triangulation(
            edge,
            polygons,
            triangulation,
            location,
        )
    }
    /// Store a 3D polygon in global coordinates; `ShapeTool::update_edge_polygon3d(&edge, &polygon)?`.
    pub fn update_edge_polygon3d(edge: &Edge, polygon: &Handle<Polygon3D>) -> Result<(), Failure> {
        BRepBuilder::default().update_edge_polygon3d(edge, polygon, &Location::default())
    }
}
