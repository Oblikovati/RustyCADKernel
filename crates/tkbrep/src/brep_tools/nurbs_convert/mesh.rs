// Rust translation of BRepTools_NurbsConvertModification::NewTriangulation, NewPolygon and
// NewPolygonOnTriangulation (ModelingData/TKBRep/BRepTools/BRepTools_NurbsConvertModification.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The mesh members of the NURBS conversion: the mesh is copied and its parameters are
//! remapped onto the converted geometry.
use super::NurbsConvertModification;
use super::helpers::{
    new_curve_of, new_parameter_on_curve, new_parameter_on_pcurve, new_pcurve_of, new_surface_of,
    new_uv,
};
use crate::brep::BRepTool;
use crate::brep_tools::modification::BRepToolsModification;
use crate::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangulation};

impl NurbsConvertModification {
    /// `NewTriangulation(F, T)`: the triangulation of `BRepTools_CopyModification`, with
    /// each UV node moved to the projection of its surface point on the converted
    /// surface (within the face tolerance).
    pub(super) fn converted_triangulation(
        &self,
        face: &Face,
        triangulation: &mut Handle<Triangulation>,
    ) -> Result<bool, Failure> {
        if !self
            .copy_modification()
            .new_triangulation(face, triangulation)?
        {
            return Ok(false);
        }
        let Some(mesh) = triangulation.get().filter(|m| m.has_uv_nodes()) else {
            return Ok(true);
        };
        let (surface, _) = BRepTool::surface_with_location(face)?;
        let new_surface = new_surface_of(&self.conversions(), face)?;
        let Some(source) = surface.get().filter(|_| !new_surface.is_null()) else {
            return Ok(true);
        };
        let tolerance = BRepTool::face_tolerance(face)?;
        for index in 1..=mesh.nb_nodes() {
            let mut uv = mesh.uv_node(index)?;
            let point = source.value(uv.x(), uv.y())?;
            if new_uv(point, &new_surface, tolerance, &mut uv)? {
                mesh.set_uv_node(index, uv)?;
            }
        }
        Ok(true)
    }

    /// `NewPolygon(E, P)`: the polygon of `BRepTools_CopyModification` with its
    /// parameters projected on the converted curve; false for a polygon without
    /// parameters and for an edge whose curve was not converted (a degenerated edge).
    pub(super) fn converted_polygon(
        &self,
        edge: &Edge,
        polygon: &mut Handle<Polygon3D>,
    ) -> Result<bool, Failure> {
        if !self.copy_modification().new_polygon(edge, polygon)? {
            return Ok(false);
        }
        let Some(copy) = polygon.get().filter(|p| p.has_parameters()) else {
            return Ok(false);
        };
        let tolerance = BRepTool::edge_tolerance(edge)?;
        let (curve, _, _) = BRepTool::curve(edge)?;
        let (new_curve, first, last) = new_curve_of(&self.conversions(), edge)?;
        let Some(source) = curve.get().filter(|_| !new_curve.is_null()) else {
            return Ok(false);
        };
        let mut parameters = copy.parameters()?;
        for parameter in parameters.iter_mut() {
            let point = source.value(*parameter)?;
            new_parameter_on_curve(point, &new_curve, (first, last), tolerance, parameter)?;
        }
        copy.change_parameters(|stored| {
            stored.assign(&parameters);
        })?;
        Ok(true)
    }

    /// `NewPolygonOnTriangulation(E, F, P)`: the polygon of `BRepTools_CopyModification`
    /// with each parameter moved to the projection of its point on the converted
    /// surface and p-curve (within the larger of the edge tolerance and the polygon
    /// deflection, in 3D and in the surface resolution).
    pub(super) fn converted_polygon_on_triangulation(
        &self,
        edge: &Edge,
        face: &Face,
        polygon: &mut Handle<PolygonOnTriangulation>,
    ) -> Result<bool, Failure> {
        if !self
            .copy_modification()
            .new_polygon_on_triangulation(edge, face, polygon)?
        {
            return Ok(false);
        }
        let Some(copy) = polygon.get().filter(|p| p.has_parameters()) else {
            return Ok(true);
        };
        self.remap_polygon(edge, face, copy)?;
        Ok(true)
    }

    fn remap_polygon(
        &self,
        edge: &Edge,
        face: &Face,
        polygon: &PolygonOnTriangulation,
    ) -> Result<(), Failure> {
        let tolerance = BRepTool::edge_tolerance(edge)?.max(polygon.deflection());
        let conversions = self.conversions();
        let (surface, _) = BRepTool::surface_with_location(face)?;
        let new_surface = new_surface_of(&conversions, face)?;
        let found = BRepTool::curve_on_face(edge, face)?;
        let (new_pcurve, first, last) = new_pcurve_of(&conversions, edge, face)?;
        let (Some(source), Some(pcurve)) = (surface.get(), found.pcurve.get()) else {
            return Ok(());
        };
        if new_surface.is_null() || new_pcurve.is_null() {
            return Ok(());
        }
        let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
        let tolerance_2d = adaptor
            .u_resolution(tolerance)?
            .max(adaptor.v_resolution(tolerance)?);
        for index in 1..=polygon.nb_nodes() {
            let mut parameter = polygon.parameter(index)?;
            let mut uv = pcurve.value(parameter)?;
            let point = source.value(uv.x(), uv.y())?;
            if new_uv(point, &new_surface, tolerance, &mut uv)?
                && new_parameter_on_pcurve(
                    uv,
                    &new_pcurve,
                    (first, last),
                    tolerance_2d,
                    &mut parameter,
                )?
            {
                polygon.set_parameter(index, parameter)?;
            }
        }
        Ok(())
    }
}
