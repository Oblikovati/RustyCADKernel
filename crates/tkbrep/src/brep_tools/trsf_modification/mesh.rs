// Rust translation of BRepTools_TrsfModification::NewTriangulation, NewPolygon and
// NewPolygonOnTriangulation (ModelingData/TKBRep/BRepTools/BRepTools_TrsfModification.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The mesh members of `BRepTools_TrsfModification`.
use super::{TrsfModification, local_transformation, representation_transformation};
use crate::brep::BRepTool;
use crate::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkgeombase::geom_lib::{Tool, gtransform};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{GeneralTransform2, TransformForm};
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;

impl TrsfModification {
    /// `NewTriangulation(F, T)`: with the mesh flag, a transformed copy of the active
    /// triangulation (in the face frame): nodes moved, UV nodes remapped on a surface,
    /// triangles flipped for a negative scale, normals transformed, deflection scaled,
    /// cached box cleared; false without the flag or a triangulation.
    pub(super) fn transformed_triangulation(
        &self,
        face: &Face,
        triangulation: &mut Handle<Triangulation>,
    ) -> Result<bool, Failure> {
        if !self.is_copy_mesh() {
            return Ok(false);
        }
        let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
        *triangulation = mesh;
        let Some(source) = triangulation.get() else {
            return Ok(false);
        };
        let trsf = representation_transformation(self.trsf(), &location, face.location())?;
        let copy = source.copy();
        let Some(mesh) = copy.get() else {
            return Ok(false);
        };
        mesh.set_cached_min_max(&BoundingBox3::default());
        mesh.set_deflection(mesh.deflection() * self.trsf().scale_factor().abs());
        for index in 1..=mesh.nb_nodes() {
            mesh.set_node(index, mesh.node(index)?.transformed(trsf))?;
        }
        self.remap_uv_nodes(face, mesh)?;
        if self.trsf().scale_factor() < 0.0 {
            flip_triangles(mesh)?;
        }
        if mesh.has_normals() {
            for index in 1..=mesh.nb_nodes() {
                mesh.set_normal(index, mesh.normal(index)?.transformed(trsf))?;
            }
        }
        *triangulation = copy;
        Ok(true)
    }

    /// The UV nodes through `TransformParameters` of the face's surface, at the
    /// surface's location.
    fn remap_uv_nodes(&self, face: &Face, mesh: &Triangulation) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        let Some(surface) = surface.get().filter(|_| mesh.has_uv_nodes()) else {
            return Ok(());
        };
        let local = local_transformation(self.trsf(), &location)?;
        for index in 1..=mesh.nb_nodes() {
            let uv = mesh.uv_node(index)?;
            let (u, v) = surface.transform_parameters(uv.x(), uv.y(), local);
            mesh.set_uv_node(index, rustycad_tkmath::gp::Point2::new(u, v))?;
        }
        Ok(())
    }

    /// `NewPolygon(E, P)`: with the mesh flag, a transformed copy of the 3D polygon (in
    /// the edge frame) with a scaled deflection and, when the 3D curve changes its
    /// parametrisation, rescaled parameters; false without the flag or a polygon.
    pub(super) fn transformed_polygon(
        &self,
        edge: &Edge,
        polygon: &mut Handle<Polygon3D>,
    ) -> Result<bool, Failure> {
        if !self.is_copy_mesh() {
            return Ok(false);
        }
        let (found, location) = BRepTool::polygon3d(edge)?;
        *polygon = found;
        let Some(source) = polygon.get() else {
            return Ok(false);
        };
        let trsf = representation_transformation(self.trsf(), &location, edge.location())?;
        let copy = source.copy()?;
        let Some(moved) = copy.get() else {
            return Ok(false);
        };
        moved.set_deflection(moved.deflection() * self.trsf().scale_factor().abs());
        moved.change_nodes(|nodes| nodes.iter_mut().for_each(|node| node.transform(trsf)));
        if moved.has_parameters() {
            self.rescale_parameters(edge, moved)?;
        }
        *polygon = copy;
        Ok(true)
    }

    /// The parameters multiplied by `ParametricTransformation` of the 3D curve when it
    /// differs from 1 by more than `Precision::PConfusion()`.
    fn rescale_parameters(&self, edge: &Edge, polygon: &Polygon3D) -> Result<(), Failure> {
        let found = BRepTool::curve_with_location(edge)?;
        let Some(curve) = found.curve.get() else {
            return Ok(());
        };
        let local = local_transformation(self.trsf(), &found.location)?;
        let ratio = curve.parametric_transformation(local);
        if (ratio - 1.0).abs() > precision::p_confusion() {
            polygon.change_parameters(|parameters| {
                parameters.iter_mut().for_each(|value| *value *= ratio)
            })?;
        }
        Ok(())
    }

    /// `NewPolygonOnTriangulation(E, F, P)`: with the mesh flag, a copy of the polygon on
    /// the face's active triangulation with a scaled deflection; for a scaling
    /// transformation of a surface with a non-identity parametric transformation, its
    /// parameters are remapped onto the transformed p-curve. `polygon` becomes null when
    /// the face has no triangulation; false then and when there is no polygon.
    pub(super) fn transformed_polygon_on_triangulation(
        &self,
        edge: &Edge,
        face: &Face,
        polygon: &mut Handle<PolygonOnTriangulation>,
    ) -> Result<bool, Failure> {
        if !self.is_copy_mesh() {
            return Ok(false);
        }
        let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
        if mesh.is_null() {
            *polygon = Handle::default();
            return Ok(false);
        }
        *polygon = BRepTool::polygon_on_triangulation(edge, &mesh, &location)?;
        let Some(source) = polygon.get() else {
            return Ok(false);
        };
        let copy = source.copy()?;
        let Some(moved) = copy.get() else {
            return Ok(false);
        };
        moved.set_deflection(moved.deflection() * self.trsf().scale_factor().abs());
        self.remap_polygon_parameters(edge, face, moved)?;
        *polygon = copy;
        Ok(true)
    }

    /// The parameter remapping of `NewPolygonOnTriangulation`.
    fn remap_polygon_parameters(
        &self,
        edge: &Edge,
        face: &Face,
        polygon: &PolygonOnTriangulation,
    ) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        let pcurve = BRepTool::curve_on_face(edge, face)?.pcurve;
        let scaling = (self.trsf().scale_factor().abs() - 1.0).abs() > Location::scale_prec();
        let Some(surface) = surface.get() else {
            return Ok(());
        };
        if !polygon.has_parameters() || pcurve.is_null() || !scaling {
            return Ok(());
        }
        let local = local_transformation(self.trsf(), &location)?;
        let gtrsf = surface.parametric_transformation(local)?;
        if gtrsf.form() == TransformForm::Identity {
            return Ok(());
        }
        remap_on_transformed_pcurve(polygon, &pcurve, &gtrsf)
    }
}

/// Each parameter mapped to the point of the p-curve, transformed, then projected on
/// the transformed p-curve within the polygon deflection (unchanged when too far).
fn remap_on_transformed_pcurve(
    polygon: &PolygonOnTriangulation,
    pcurve: &Handle<dyn Geom2dCurve>,
    gtrsf: &GeneralTransform2,
) -> Result<(), Failure> {
    let new_pcurve = gtransform(pcurve, gtrsf)?;
    let Some(source) = pcurve.get() else {
        return Ok(());
    };
    for index in 1..=polygon.nb_nodes() {
        let mut parameter = polygon.parameter(index)?;
        let mut point = source.value(parameter)?;
        gtrsf.transforms(point.change_coord());
        if let Some(found) = Tool::parameter_2d(&new_pcurve, point, polygon.deflection())? {
            parameter = found;
        }
        polygon.set_parameter(index, parameter)?;
    }
    Ok(())
}

/// Reverses the orientation of every triangle (`Set(N1, N3, N2)`).
pub(crate) fn flip_triangles(mesh: &Triangulation) -> Result<(), Failure> {
    for index in 1..=mesh.nb_triangles() {
        let (n1, n2, n3) = mesh.triangle(index)?.get();
        mesh.set_triangle(index, rustycad_tkmath::poly::Triangle::new(n1, n3, n2))?;
    }
    Ok(())
}
