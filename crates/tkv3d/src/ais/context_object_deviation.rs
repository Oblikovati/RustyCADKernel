//! The per-object deviation and triangulation-isoline members of `AIS_InteractiveContext`
//! (`SetDeviationCoefficient`, `SetDeviationAngle` and `IsoOnTriangulation` taking an
//! object); the default-drawer overloads live in `context_isoline_policies`.
use super::context_attributes::required_shape;
use super::{
    AisInteractiveContext, AisInteractiveObject, AisKindOfInteractive, AisShapeParameters,
    AisShapeSource,
};
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrPresentableObjectUpdates};
use rustycad_tkernel::standard::{Failure, Handle};

impl AisInteractiveContext {
    /// `SetDeviationCoefficient(object, coefficient, update)`: the own deviation coefficient
    /// of an AIS_Shape (kind Object or Shape, signature 0); other objects are left alone.
    /// Example: `context.set_object_deviation_coefficient(&object, 0.01, true)?`.
    pub fn set_object_deviation_coefficient(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        coefficient: f64,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let accepted = [AisKindOfInteractive::Object, AisKindOfInteractive::Shape];
        let Some(shape) = self.deviation_target(object, &accepted)? else {
            return Ok(());
        };
        let shape = required_shape(&shape)?;
        shape.set_own_deviation_coefficient(coefficient);
        self.finish_deviation(shape, update_viewer)
    }

    /// `SetDeviationAngle(object, angle, update)`: the own deviation angle of an AIS_Shape
    /// (kind Shape, signature 0); other objects are left alone.
    /// Example: `context.set_object_deviation_angle(&object, 0.2, true)?`.
    pub fn set_object_deviation_angle(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        angle: f64,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(shape) = self.deviation_target(object, &[AisKindOfInteractive::Shape])? else {
            return Ok(());
        };
        let shape = required_shape(&shape)?;
        shape.set_own_deviation_angle(angle);
        self.finish_deviation(shape, update_viewer)
    }

    /// `IsoOnTriangulation(enabled, object)`: the object's own isolines-on-triangulation flag
    /// (a null object is ignored); `context.set_object_iso_on_triangulation(true, &object)`.
    pub fn set_object_iso_on_triangulation(
        &self,
        is_enabled: bool,
        object: &Handle<dyn AisInteractiveObject>,
    ) {
        if let Some(value) = object.get() {
            value.set_iso_on_triangulation(is_enabled);
        }
    }

    /// The AIS_Shape of a non-null object of an accepted kind and signature 0, after binding
    /// it to this context (native order: the context is set before the kind is checked).
    fn deviation_target(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        accepted: &[AisKindOfInteractive],
    ) -> Result<Option<Handle<dyn AisShapeSource>>, Failure> {
        let Some(value) = object.get() else {
            return Ok(None);
        };
        self.set_context_to_object(object)?;
        if !accepted.contains(&value.kind_of_interactive()) || value.signature() != 0 {
            return Ok(None);
        }
        Ok(Some(object.cast::<dyn AisShapeSource>()))
    }

    /// Recomputes the shape presentations, then redraws the viewer when asked.
    fn finish_deviation(
        &self,
        shape: &dyn AisShapeSource,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        shape.update_presentations(false)?;
        if update_viewer {
            self.update_current_viewer();
        }
        Ok(())
    }
}
