//! Build/copy transitions retain native initialization and transform-form distinctions.
use super::rectangular_frustum::RectangularRay;
use super::rectangular_geometry::*;
use super::*;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, GeneralTransform3, Point2, Point3, TransformForm, Xy};

impl SelectMgrRectangularFrustum {
    pub(super) fn rectangular_build(&self) -> Result<(), Failure> {
        self.rectangular_initialized("Build")?;
        let builder = self.rectangular_builder()?;
        let rectangle = *self.selection_rectangle.read();
        let (center, bounds) = self.rectangular_build_pixels(rectangle);
        let mut ray = self.ray.write();
        ray.near = builder.project_pnt_on_view_plane(center.x(), center.y(), 0.);
        ray.far = builder.project_pnt_on_view_plane(center.x(), center.y(), 1.);
        ray.direction = Direction3::from_xyz(ray.far.xyz() - ray.near.xyz())?;
        let mut core = self.frustum_core.write();
        rectangular_compute_vertices(&mut core, bounds, &builder);
        rectangular_cache_geometry(&mut core, self.is_orthographic());
        ray.scale = 1.;
        Ok(())
    }
    fn rectangular_build_pixels(
        &self,
        rectangle: SelectMgrSelectionRectangle,
    ) -> (Point2, (Point2, Point2)) {
        if self.is_scalable() {
            let center = rectangle.mouse_pos();
            return (
                center,
                rectangular_pixel_bounds(center, *self.base.pixel_tolerance.read()),
            );
        }
        let (min, max) = (rectangle.min_pnt(), rectangle.max_pnt());
        (
            Point2::from(Xy::new(
                (min.x() + max.x()) * 0.5,
                (min.y() + max.y()) * 0.5,
            )),
            (min, max),
        )
    }
    fn rectangular_builder(&self) -> Result<Handle<SelectMgrFrustumBuilder>, Failure> {
        let builder = self.base.builder.read().clone();
        if !builder.is_null() {
            return Ok(builder);
        }
        Err(Failure::with_message(
            "Standard_NullObject",
            "RectangularFrustum builder=null; expected a builder for projection",
        ))
    }
    pub(super) fn rectangular_transformed(
        &self,
        scale: i32,
        transform: &GeneralTransform3,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Self, Failure> {
        self.rectangular_initialized("ScaleAndTransform")?;
        rectangular_check_scale(scale)?;
        let result = Self::new();
        if scale == 1 && transform.form() == TransformForm::Identity {
            result.set_builder(builder);
            return Ok(result);
        }
        result.set_camera(&self.camera());
        self.rectangular_transform_geometry(&result, scale, transform)?;
        result.set_builder(builder);
        rectangular_cache_geometry(&mut result.frustum_core.write(), result.is_orthographic());
        *result.base.intersector.selection_type.write() = self.get_selection_type();
        *result.selection_rectangle.write() = *self.selection_rectangle.read();
        Ok(result)
    }
    fn rectangular_transform_geometry(
        &self,
        result: &Self,
        scale: i32,
        transform: &GeneralTransform3,
    ) -> Result<(), Failure> {
        if scale != 1 {
            self.rectangular_rescale(result, scale)?;
        }
        if transform.form() == TransformForm::Identity {
            return Ok(());
        }
        let source = if scale == 1 { self } else { result };
        result.rectangular_apply_transform(source, transform)
    }
    fn rectangular_rescale(&self, result: &Self, scale: i32) -> Result<(), Failure> {
        let builder = self.rectangular_builder()?;
        *result.ray.write() = RectangularRay {
            scale: 1.,
            ..*self.ray.read()
        };
        let bounds = rectangular_pixel_bounds(self.selection_rectangle.read().mouse_pos(), scale);
        rectangular_compute_vertices(&mut result.frustum_core.write(), bounds, &builder);
        Ok(())
    }
    fn rectangular_apply_transform(
        &self,
        source: &Self,
        transform: &GeneralTransform3,
    ) -> Result<(), Failure> {
        let ray = *source.ray.read();
        let vertices = source.get_vertices();
        self.rectangular_transform_ray(ray, transform)?;
        let mut core = self.frustum_core.write();
        for (target, vertex) in core.vertices.iter_mut().zip(vertices) {
            *target = rectangular_transformed_point(vertex, transform);
        }
        rectangular_compute_edges(&mut core);
        let mut result_ray = self.ray.write();
        result_ray.scale = (ray.far.square_distance(&ray.near)
            / result_ray.far.square_distance(&result_ray.near))
        .sqrt();
        Ok(())
    }
    fn rectangular_transform_ray(
        &self,
        source: RectangularRay,
        transform: &GeneralTransform3,
    ) -> Result<(), Failure> {
        let mut ray = self.ray.write();
        ray.near = rectangular_transformed_point(source.near, transform);
        ray.far = rectangular_transformed_point(source.far, transform);
        ray.direction = Direction3::from_xyz(ray.far.xyz() - ray.near.xyz())?;
        Ok(())
    }
    pub(super) fn rectangular_copied(
        &self,
        builder: &Handle<SelectMgrFrustumBuilder>,
    ) -> Result<Self, Failure> {
        self.rectangular_initialized("CopyWithBuilder")?;
        if builder.is_null() {
            return Err(Failure::with_message(
                "Standard_ProgramError",
                "RectangularFrustum::CopyWithBuilder builder=null; expected a non-null builder",
            ));
        }
        let result = Self::new();
        *result.base.intersector.selection_type.write() = self.get_selection_type();
        *result.selection_rectangle.write() = *self.selection_rectangle.read();
        result.set_pixel_tolerance(*self.base.pixel_tolerance.read());
        result.set_builder(builder);
        Ok(result)
    }
}
fn rectangular_transformed_point(point: Point3, transform: &GeneralTransform3) -> Point3 {
    let mut xyz = point.xyz();
    transform.transforms(&mut xyz);
    Point3::from(xyz)
}

fn rectangular_check_scale(scale: i32) -> Result<(), Failure> {
    if scale < 0 {
        return Err(Failure::with_message(
            "Standard_ProgramError",
            &format!("RectangularFrustum scale={scale}; expected a nonnegative pixel tolerance"),
        ));
    }
    Ok(())
}
