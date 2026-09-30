// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_TrsfModification.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_TrsfModification`: the modification that applies a gp_Trsf.
mod curve2d;
mod mesh;

pub(crate) use mesh::flip_triangles;

use super::modification::{BRepToolsModification, NewCurve, NewSurface};
use crate::brep::BRepTool;
use crate::topods::{Edge, Face, Vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;

/// The anonymous-namespace `representationTransformation(theTrsf, theSource,
/// theTarget)`: `theTrsf` itself when both locations are the identity, else
/// `Target^-1 * theTrsf * Source`.
pub(super) fn representation_transformation(
    trsf: Transform3,
    source: &Location,
    target: &Location,
) -> Result<Transform3, Failure> {
    if source.is_identity() && target.is_identity() {
        return Ok(trsf);
    }
    Ok(target
        .transformation()
        .inverted()?
        .multiplied(trsf)
        .multiplied(source.transformation()))
}

/// `representationTransformation(theTrsf, theLocation)`: the same location on both
/// sides.
pub(super) fn local_transformation(
    trsf: Transform3,
    location: &Location,
) -> Result<Transform3, Failure> {
    representation_transformation(trsf, location, location)
}

/// The mutable fields: the transformation (`Trsf()` returns a reference) and the mesh
/// flag (`IsCopyMesh()` returns a reference).
#[derive(Clone, Copy)]
struct TrsfFields {
    trsf: Transform3,
    copy_mesh: bool,
}

/// `BRepTools_TrsfModification`: every point, curve and surface is transformed by a
/// gp_Trsf (tolerances scaled by `|ScaleFactor|`); with `IsCopyMesh`, the mesh is
/// transformed too.
///
/// ```
/// use rustycad_tkbrep::brep_tools::TrsfModification;
/// use rustycad_tkmath::gp::Transform3;
/// let modification = TrsfModification::new(Transform3::default());
/// modification.set_copy_mesh(true);
/// assert!(modification.is_copy_mesh());
/// ```
pub struct TrsfModification {
    parent: StandardTransient,
    fields: SharedState<TrsfFields>,
}
impl_transient!(
    TrsfModification,
    "BRepTools_TrsfModification",
    dyn BRepToolsModification,
    parent,
    interfaces: [dyn BRepToolsModification]
);

impl TrsfModification {
    /// `BRepTools_TrsfModification(T)`: without mesh copy; `TrsfModification::new(trsf)`.
    pub fn new(trsf: Transform3) -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(TrsfFields {
                trsf,
                copy_mesh: false,
            }),
        }
    }

    /// `Trsf()`: the transformation; `modification.trsf()`.
    pub fn trsf(&self) -> Transform3 {
        self.fields.read().trsf
    }

    /// `Trsf() = T`: replaces the transformation; `modification.set_trsf(trsf)`.
    pub fn set_trsf(&self, trsf: Transform3) {
        self.fields.write().trsf = trsf;
    }

    /// `IsCopyMesh()`; `modification.is_copy_mesh()`.
    pub fn is_copy_mesh(&self) -> bool {
        self.fields.read().copy_mesh
    }

    /// `IsCopyMesh() = flag`; `modification.set_copy_mesh(true)`.
    pub fn set_copy_mesh(&self, copy_mesh: bool) {
        self.fields.write().copy_mesh = copy_mesh;
    }

    /// `|myTrsf.ScaleFactor()|`, the factor of the tolerances and deflections.
    fn abs_scale(&self) -> f64 {
        self.trsf().scale_factor().abs()
    }
}

impl BRepToolsModification for TrsfModification {
    /// The transformed surface at the face's own location, the scaled tolerance, the
    /// face reversed for a negative transformation; false for a face without surface.
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        result.surface = surface;
        result.location = location;
        let Some(stored) = result.surface.get() else {
            return Ok(false);
        };
        result.tolerance = BRepTool::face_tolerance(face)? * self.abs_scale();
        result.rev_wires = false;
        result.rev_face = self.trsf().is_negative();
        let local = local_transformation(self.trsf(), &result.location)?;
        result.surface = stored.transformed(local)?.cast();
        Ok(true)
    }

    fn new_triangulation(
        &self,
        face: &Face,
        triangulation: &mut Handle<Triangulation>,
    ) -> Result<bool, Failure> {
        self.transformed_triangulation(face, triangulation)
    }

    /// The 3D curve transformed at its own location (null for an edge without one) and
    /// the scaled tolerance; always true.
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        let found = BRepTool::curve_with_location(edge)?;
        result.curve = found.curve;
        result.location = found.location;
        result.tolerance = BRepTool::edge_tolerance(edge)? * self.abs_scale();
        if let Some(stored) = result.curve.get() {
            let local = local_transformation(self.trsf(), &result.location)?;
            result.curve = stored.transformed(local)?.cast::<dyn GeomCurve>();
        }
        Ok(true)
    }

    fn new_polygon(&self, edge: &Edge, polygon: &mut Handle<Polygon3D>) -> Result<bool, Failure> {
        self.transformed_polygon(edge, polygon)
    }

    fn new_polygon_on_triangulation(
        &self,
        edge: &Edge,
        face: &Face,
        polygon: &mut Handle<PolygonOnTriangulation>,
    ) -> Result<bool, Failure> {
        self.transformed_polygon_on_triangulation(edge, face, polygon)
    }

    /// The transformed point and the scaled tolerance; always true.
    fn new_point(
        &self,
        vertex: &Vertex,
        point: &mut Point3,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        *point = BRepTool::pnt(vertex)?;
        *tolerance = BRepTool::vertex_tolerance(vertex)? * self.abs_scale();
        point.transform(self.trsf());
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
        self.transformed_curve2d(edge, face, curve, tolerance)
    }

    /// The parameter of the vertex on the edge, mapped by the transformation of the 3D
    /// curve, and the scaled tolerance; false for a null vertex (an infinite edge).
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
        *tolerance = BRepTool::vertex_tolerance(vertex)? * self.abs_scale();
        *parameter = BRepTool::parameter(vertex, edge)?;
        let found = BRepTool::curve_with_location(edge)?;
        if let Some(curve) = found.curve.get() {
            let local = local_transformation(self.trsf(), &found.location)?;
            *parameter = curve.transformed_parameter(*parameter, local);
        }
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
