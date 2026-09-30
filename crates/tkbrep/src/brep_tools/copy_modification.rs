// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_CopyModification.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_CopyModification`: the modification that copies the geometry (and,
//! optionally, the mesh) of a shape.
use super::modification::{BRepToolsModification, NewCurve, NewSurface};
use crate::brep::BRepTool;
use crate::topods::{Edge, Face, Vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation, mesh_purpose};

/// `BRepTools_CopyModification`: every face, edge and vertex keeps its geometry,
/// copied when `copy_geom` is set; the mesh is kept on geometric shapes only with
/// `copy_mesh` (always on shapes without geometry), copied with the geometry.
///
/// ```
/// use rustycad_tkbrep::brep_tools::CopyModification;
/// let modification = CopyModification::new(true, false);
/// assert!(modification.is_copy_geom());
/// ```
pub struct CopyModification {
    parent: StandardTransient,
    copy_geom: bool,
    copy_mesh: bool,
}
impl_transient!(
    CopyModification,
    "BRepTools_CopyModification",
    dyn BRepToolsModification,
    parent,
    interfaces: [dyn BRepToolsModification]
);

impl Default for CopyModification {
    /// `BRepTools_CopyModification()`: copy the geometry and the mesh (both default
    /// arguments are true).
    fn default() -> Self {
        Self::new(true, true)
    }
}

impl CopyModification {
    /// `BRepTools_CopyModification(theCopyGeom, theCopyMesh)`;
    /// `CopyModification::new(true, false)`.
    pub fn new(copy_geom: bool, copy_mesh: bool) -> Self {
        Self {
            parent: StandardTransient::default(),
            copy_geom,
            copy_mesh,
        }
    }

    /// Whether the geometry is copied (`myCopyGeom`); `modification.is_copy_geom()`.
    pub fn is_copy_geom(&self) -> bool {
        self.copy_geom
    }

    /// Whether the mesh of geometric shapes is kept (`myCopyMesh`);
    /// `modification.is_copy_mesh()`.
    pub fn is_copy_mesh(&self) -> bool {
        self.copy_mesh
    }
}

impl BRepToolsModification for CopyModification {
    /// The surface and location of the face (the surface copied), its tolerance, no
    /// reversal; always true.
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        result.surface = surface;
        result.location = location;
        result.tolerance = BRepTool::face_tolerance(face)?;
        result.rev_wires = false;
        result.rev_face = false;
        if let Some(stored) = result.surface.get().filter(|_| self.copy_geom) {
            result.surface = stored.copy()?.cast();
        }
        Ok(true)
    }

    /// The active triangulation (copied with the geometry); false for a geometric face
    /// without `copy_mesh` and for a face without triangulation.
    fn new_triangulation(
        &self,
        face: &Face,
        triangulation: &mut Handle<Triangulation>,
    ) -> Result<bool, Failure> {
        if !self.copy_mesh && BRepTool::is_geometric_face(face)? {
            return Ok(false);
        }
        *triangulation = BRepTool::triangulation(face, mesh_purpose::NONE)?.0;
        let Some(stored) = triangulation.get() else {
            return Ok(false);
        };
        if self.copy_geom {
            *triangulation = stored.copy();
        }
        Ok(true)
    }

    /// The 3D curve and location of the edge (the curve copied) and its tolerance;
    /// always true.
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        let found = BRepTool::curve_with_location(edge)?;
        result.curve = found.curve;
        result.location = found.location;
        result.tolerance = BRepTool::edge_tolerance(edge)?;
        if let Some(stored) = result.curve.get().filter(|_| self.copy_geom) {
            result.curve = stored.copy()?.cast();
        }
        Ok(true)
    }

    /// The 3D polygon (copied with the geometry); false for a geometric edge without
    /// `copy_mesh` and for an edge without polygon.
    fn new_polygon(&self, edge: &Edge, polygon: &mut Handle<Polygon3D>) -> Result<bool, Failure> {
        if !self.copy_mesh && BRepTool::is_geometric_edge(edge)? {
            return Ok(false);
        }
        *polygon = BRepTool::polygon3d(edge)?.0;
        let Some(stored) = polygon.get() else {
            return Ok(false);
        };
        if self.copy_geom {
            *polygon = stored.copy()?;
        }
        Ok(true)
    }

    /// The polygon on the face's active triangulation (copied with the geometry); false
    /// for a geometric edge without `copy_mesh` and when there is none.
    fn new_polygon_on_triangulation(
        &self,
        edge: &Edge,
        face: &Face,
        polygon: &mut Handle<PolygonOnTriangulation>,
    ) -> Result<bool, Failure> {
        if !self.copy_mesh && BRepTool::is_geometric_edge(edge)? {
            return Ok(false);
        }
        let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
        *polygon = BRepTool::polygon_on_triangulation(edge, &mesh, &location)?;
        let Some(stored) = polygon.get() else {
            return Ok(false);
        };
        if self.copy_geom {
            *polygon = stored.copy()?;
        }
        Ok(true)
    }

    /// The point and tolerance of the vertex; always true.
    fn new_point(
        &self,
        vertex: &Vertex,
        point: &mut Point3,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        *point = BRepTool::pnt(vertex)?;
        *tolerance = BRepTool::vertex_tolerance(vertex)?;
        Ok(true)
    }

    /// The p-curve on the face (copied with the geometry) and the edge tolerance;
    /// always true.
    fn new_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        _new_edge: &Edge,
        _new_face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        *tolerance = BRepTool::edge_tolerance(edge)?;
        *curve = BRepTool::curve_on_face(edge, face)?.pcurve;
        if let Some(stored) = curve.get().filter(|_| self.copy_geom) {
            *curve = stored.copy()?.cast();
        }
        Ok(true)
    }

    /// The parameter of the vertex on the edge and its tolerance; false for a null
    /// vertex (an infinite edge).
    fn new_parameter(
        &self,
        vertex: &Vertex,
        edge: &Edge,
        parameter: &mut f64,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        if vertex.is_null() {
            return Ok(false);
        }
        *tolerance = BRepTool::vertex_tolerance(vertex)?;
        *parameter = BRepTool::parameter(vertex, edge)?;
        Ok(true)
    }

    /// The continuity of the edge between the original faces.
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
