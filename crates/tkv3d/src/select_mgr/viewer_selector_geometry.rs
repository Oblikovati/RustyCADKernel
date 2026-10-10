//! Native geometry helpers used by the viewer's two-level selection traversal.
use super::*;
use crate::{
    select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager},
    select3d::{Select3DSensitiveEntity, Select3DSensitiveEntityMembers},
    v3d::V3dView,
};
use rustycad_tkernel::{
    ncollection::{DataMap, Mat4, Vec3, Vec4},
    standard::{Failure, Handle, short_real::short_real_epsilon},
};
use rustycad_tkmath::gp::{GeneralTransform3, TransformForm, Xyz};

/// Refresh depth-write layer groups in driver order; `selector.update_z_layers(&view)?`.
pub trait SelectMgrViewerSelectorGeometryMembers: SelectMgrViewerSelectorSource {
    /// Clear first, retaining the native partial prefix if layer lookup fails.
    /// Example: `selector.update_z_layers(&view)?`.
    fn update_z_layers(&self, view: &V3dView) -> Result<(), Failure> {
        let base = self.viewer_selector_base();
        base.fields.write().z_layer_order.clear(true);
        let viewer = view.viewer();
        let mut position = 0;
        let mut previous_write = true;
        for layer in viewer.get_all_z_layers() {
            let settings = viewer.z_layer_settings(layer)?;
            if settings.to_clear_depth() || previous_write != settings.to_enable_depth_write() {
                position += 1;
            }
            previous_write = settings.to_enable_depth_write();
            base.fields.write().z_layer_order.bind(layer, position);
        }
        Ok(())
    }
}
impl<T: SelectMgrViewerSelectorSource + ?Sized> SelectMgrViewerSelectorGeometryMembers for T {}

/// Native private updatePoint3d body, shared with traversal and differential probes.
/// Example: `select_mgr_update_point3d(&base,&mut criterion,&pick,&entity,&inverse,&volume)?`.
pub fn select_mgr_update_point3d(
    base: &SelectMgrViewerSelectorBase,
    criterion: &mut SelectMgrSortCriterion,
    pick: &SelectBasicsPickResult,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    inverse: &GeneralTransform3,
    volume: &SelectMgrSelectingVolumeManager,
) -> Result<(), Failure> {
    if volume.get_active_selection_type() != SelectMgrSelectionType::Point {
        return Ok(());
    }
    let has_normal = assign_picked_position(criterion, pick, volume)?;
    if !pick.has_picked_point() && !pick.is_valid() {
        return Ok(());
    }
    transform_picked_position(criterion, inverse, has_normal)?;
    criterion.tolerance = picked_depth_tolerance(base, criterion, entity, volume);
    Ok(())
}
fn assign_picked_position(
    criterion: &mut SelectMgrSortCriterion,
    pick: &SelectBasicsPickResult,
    volume: &SelectMgrSelectingVolumeManager,
) -> Result<bool, Failure> {
    if !pick.has_picked_point() {
        criterion.point = if pick.is_valid() {
            volume.detected_point(criterion.depth)?
        } else {
            *pick.picked_point()
        };
        return Ok(false);
    }
    criterion.point = *pick.picked_point();
    criterion.normal = *pick.surface_normal();
    let length_squared = criterion.normal.square_modulus();
    let has_normal = length_squared > short_real_epsilon();
    if has_normal {
        criterion.normal *= 1.0_f32 / length_squared.sqrt();
    }
    Ok(has_normal)
}
fn transform_picked_position(
    criterion: &mut SelectMgrSortCriterion,
    inverse: &GeneralTransform3,
    has_normal: bool,
) -> Result<(), Failure> {
    let transform = entity_inverse_transform(&criterion.entity, inverse);
    if transform.form() == TransformForm::Identity {
        return Ok(());
    }
    let forward = transform.inverted()?;
    criterion.point = forward.apply(Xyz::from(criterion.point)).into();
    if has_normal {
        criterion.normal = transformed_picked_normal(criterion.normal, &forward);
    }
    Ok(())
}
fn transformed_picked_normal(normal: Vec3<f32>, forward: &GeneralTransform3) -> Vec3<f32> {
    let mut matrix = Mat4::<f64>::default();
    forward.get_mat4(&mut matrix);
    let transformed = matrix
        * Vec4::new(
            f64::from(normal.x),
            f64::from(normal.y),
            f64::from(normal.z),
            0.,
        );
    Vec3::new(
        transformed.x as f32,
        transformed.y as f32,
        transformed.z as f32,
    )
}
fn picked_depth_tolerance(
    base: &SelectMgrViewerSelectorBase,
    criterion: &SelectMgrSortCriterion,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    volume: &SelectMgrSelectingVolumeManager,
) -> f64 {
    let settings = base.fields.read().settings;
    let factor =
        if settings.depth_tolerance_type == SelectMgrTypeOfDepthTolerance::SensitivityFactor {
            f64::from(entity.sensitivity_factor())
        } else {
            settings.depth_tolerance
        };
    if settings.depth_tolerance_type == SelectMgrTypeOfDepthTolerance::Uniform {
        return settings.depth_tolerance;
    }
    pixel_depth_tolerance(&settings, criterion.point, volume, factor)
}
fn pixel_depth_tolerance(
    settings: &SelectMgrViewerSelectorSettings,
    point: rustycad_tkmath::gp::Point3,
    volume: &SelectMgrSelectingVolumeManager,
    factor: f64,
) -> f64 {
    let camera = volume.camera();
    if camera.is_null() {
        return factor / 33.;
    }
    if camera.is_orthographic() {
        return settings.camera_scale * factor;
    }
    let distance = (Xyz::from(point) - Xyz::from(settings.camera_eye))
        .dot(Xyz::from(settings.camera_direction))
        .abs();
    distance * settings.camera_scale * factor
}

/// Native private computeFrustum body, retaining cached volume identity and clipping copies.
/// Example: `select_mgr_compute_frustum(&base,&entity,&global,&object,&inverse,&mut cache,&mut result)?`.
pub fn select_mgr_compute_frustum(
    base: &SelectMgrViewerSelectorBase,
    entity: &Handle<dyn Select3DSensitiveEntity>,
    global: &SelectMgrSelectingVolumeManager,
    object: &SelectMgrSelectingVolumeManager,
    inverse: &GeneralTransform3,
    cache: &mut DataMap<i32, SelectMgrSelectingVolumeManager>,
    result: &mut SelectMgrSelectingVolumeManager,
) -> Result<(), Failure> {
    let scale = selector_frustum_scale(base, entity);
    let transform = entity_inverse_transform(entity, inverse);
    if scale == 1 && transform.form() == TransformForm::Identity {
        *result = object.clone();
        return Ok(());
    }
    *result = scaled_entity_volume(global, scale, &transform, cache)?;
    result.copy_view_clipping(object);
    Ok(())
}
fn entity_inverse_transform(
    entity: &Handle<dyn Select3DSensitiveEntity>,
    inverse: &GeneralTransform3,
) -> GeneralTransform3 {
    if entity.has_init_location() {
        return entity.inv_init_location() * *inverse;
    }
    *inverse
}
fn selector_frustum_scale(
    base: &SelectMgrViewerSelectorBase,
    entity: &Handle<dyn Select3DSensitiveEntity>,
) -> i32 {
    let volume = base.selecting_volume.read().clone();
    if !volume.is_scalable_active_volume() {
        return 1;
    }
    let tolerances = base.tolerances.read();
    let sensitivity = entity.sensitivity_factor()
        + if tolerances.is_custom_tol_set() {
            tolerances.custom_tolerance()
        } else {
            0
        };
    if sensitivity < tolerances.tolerance() {
        sensitivity
    } else {
        1
    }
}
fn scaled_entity_volume(
    global: &SelectMgrSelectingVolumeManager,
    scale: i32,
    transform: &GeneralTransform3,
    cache: &mut DataMap<i32, SelectMgrSelectingVolumeManager>,
) -> Result<SelectMgrSelectingVolumeManager, Failure> {
    if transform.form() != TransformForm::Identity {
        return global.scale_and_transform(scale, transform, &Handle::default());
    }
    if let Some(existing) = cache.seek(&scale) {
        return Ok(existing.clone());
    }
    let scaled = global.scale_and_transform(scale, transform, &Handle::default())?;
    cache.bind(scale, scaled.clone());
    Ok(scaled)
}
