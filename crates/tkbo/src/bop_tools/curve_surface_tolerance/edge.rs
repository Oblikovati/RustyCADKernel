use super::super::shape_tolerances::{correct_vertices, update_tolerance};
use super::{
    plane::plane_projection,
    reference::{ReferenceCurve, curve_on_surface, reference_curve},
};
use rustycad_tkbrep::{
    brep::{
        BRepCurveRepresentation, BRepGCurve, BRepTool,
        access::{edge_tshape, face_tshape},
    },
    topods::{Edge, Face},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{adaptor3d::Adaptor3dCurveOnSurface, geom::GeomSurface};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::BRepLibValidateEdge;
struct EdgeValidation<'a> {
    edge: &'a Edge,
    reference: ReferenceCurve,
    avoided: &'a IndexedMapOfShape,
    base_tolerance: f64,
    tolerance: f64,
    maximum: f64,
    same_parameter: bool,
    same_range: bool,
}
pub(super) fn correct_edge(
    edge: &Edge,
    face: &Face,
    maximum: f64,
    avoided: &IndexedMapOfShape,
) -> Result<(), Failure> {
    let Some(reference) = reference_curve(edge)? else {
        return Ok(());
    };
    let base_tolerance = BRepTool::edge_tolerance(edge)?;
    let mut validation = EdgeValidation {
        edge,
        reference,
        avoided,
        base_tolerance,
        tolerance: base_tolerance,
        maximum,
        same_parameter: BRepTool::same_parameter(edge)?,
        same_range: BRepTool::same_range(edge)?,
    };
    let topology = face_tshape(face, "CorrectCurveOnSurface")?;
    let surface = topology.surface();
    let location = face
        .location()
        .multiplied(&topology.location())?
        .predivided(edge.location())?;
    validation.check_representations(&surface, &location)
}
impl EdgeValidation<'_> {
    fn check_representations(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(), Failure> {
        let mut found = false;
        for representation in edge_tshape(self.edge, "CorrectCurveOnSurface")?.curves() {
            if representation == self.reference.representation
                || !representation.is_curve_on_surface_at(surface, location)
            {
                continue;
            }
            found = true;
            let parametric: Handle<dyn BRepGCurve> = representation.cast();
            let range = parametric.range();
            if self.same_range && range != self.reference.range {
                return Ok(());
            }
            self.check_representation(&representation, surface, location, range)?;
        }
        if !found && let Some(projected) = plane_projection(&self.reference, surface, location)? {
            self.validate(&projected, true)?;
        }
        Ok(())
    }
    fn check_representation(
        &mut self,
        representation: &Handle<dyn BRepCurveRepresentation>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        range: (f64, f64),
    ) -> Result<(), Failure> {
        let surface: Handle<dyn GeomSurface> =
            surface.transformed(location.transformation())?.cast();
        let curve = curve_on_surface(&representation.pcurve()?, &surface, range)?;
        self.validate(&curve, self.same_parameter)?;
        if representation.is_curve_on_closed_surface() {
            let second = curve_on_surface(&representation.pcurve2()?, &surface, range)?;
            self.validate(&second, self.same_parameter)?;
        }
        Ok(())
    }
    fn validate(
        &mut self,
        curve: &Handle<Adaptor3dCurveOnSurface>,
        same_parameter: bool,
    ) -> Result<(), Failure> {
        let mut validation =
            BRepLibValidateEdge::new(&self.reference.adaptor, curve, same_parameter);
        validation.process()?;
        validation.update_tolerance(&mut self.tolerance);
        if !validation.is_done()
            || validation.check_tolerance(self.base_tolerance)?
            || self.tolerance.partial_cmp(&self.maximum) != Some(std::cmp::Ordering::Less)
        {
            return Ok(());
        }
        update_tolerance(self.edge, self.tolerance, self.avoided)?;
        correct_vertices(self.edge, self.avoided)
    }
}
