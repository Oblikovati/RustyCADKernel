//! Set copies share input pixels but preserve distinct native child/boundary transitions.
use super::*;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{GeneralTransform3, Point3};
impl SelectMgrTriangularFrustumSet {
    pub(super) fn frustum_set_transformed(
        &self,
        scale: i32,
        transform: &GeneralTransform3,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Self, Failure> {
        self.frustum_set_initialized("ScaleAndTransform")?;
        let result = Self::new();
        result.set_camera(&self.camera());
        for child in self.frustums.read().iter() {
            result
                .frustums
                .write()
                .push(child.scale_and_transform(scale, transform, builder)?.cast());
        }
        self.frustum_set_transform_boundary(&result, transform)?;
        self.frustum_set_copy_selection(&result, builder);
        Ok(result)
    }
    fn frustum_set_transform_boundary(
        &self,
        result: &Self,
        transform: &GeneralTransform3,
    ) -> Result<(), Failure> {
        let source = self.boundary_points.read();
        let mut boundary = result.boundary_points.write();
        boundary
            .resize_bounds(source.lower(), source.upper(), false)
            .map_err(|e| e.into_failure("TriangularFrustumSet transformed boundary"))?;
        for index in source.lower()..=source.upper() {
            let mut xyz = source
                .value(index)
                .map_err(|e| e.into_failure("frustum source boundary"))?
                .xyz();
            transform.transforms(&mut xyz);
            boundary
                .set_value(index, Point3::from(xyz))
                .map_err(|e| e.into_failure("frustum transformed boundary"))?;
        }
        Ok(())
    }
    pub(super) fn frustum_set_copied(
        &self,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Self, Failure> {
        self.frustum_set_initialized("CopyWithBuilder")?;
        if builder.is_null() {
            return Err(Failure::with_message(
                "Standard_ProgramError",
                "TriangularFrustumSet::CopyWithBuilder builder=null; expected a non-null builder",
            ));
        }
        let result = Self::new();
        result.set_camera(&self.camera());
        for child in self.frustums.read().iter() {
            result
                .frustums
                .write()
                .push(child.copy_with_builder(builder)?.cast());
        }
        self.frustum_set_copy_selection(&result, builder);
        *result.allow_overlap.write() = *self.allow_overlap.read();
        Ok(result)
    }
    fn frustum_set_copy_selection(&self, result: &Self, builder: &Handle<SelectMgrFrustumBuilder>) {
        *result.base.intersector.selection_type.write() = self.get_selection_type();
        *result.selection_polyline.write() = self.selection_polyline.read().clone();
        result.set_builder(builder);
    }
}
