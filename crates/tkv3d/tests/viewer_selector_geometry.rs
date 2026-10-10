//! Picking geometry preserves native early exits, rounding and cached identity.
#[path = "support/geometry_sensitive.rs"]
mod geometry_sensitive;
use geometry_sensitive::{GeometrySensitivePoint, geometry_transform};
use rustycad_tkernel::{
    ncollection::{DataMap, Vec3},
    standard::Handle,
};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{GeneralTransform3, Point2, Point3, Xy},
};
use rustycad_tkservice::{
    graphic3d::Graphic3dSequenceOfHClipPlane, null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::{select_basics::*, select_mgr::*, select3d::*, v3d::V3dViewer};
fn point_volume() -> SelectMgrSelectingVolumeManager {
    let mut volume = SelectMgrSelectingVolumeManager::new();
    volume.init_point_selecting_volume(Point2::from(Xy::new(4., 3.)));
    volume
        .set_camera(&Handle::new(
            rustycad_tkservice::graphic3d::Graphic3dCamera::default(),
        ))
        .unwrap();
    volume.set_window_size(8, 6).unwrap();
    volume.set_pixel_tolerance(10).unwrap();
    volume.build_selecting_volume().unwrap();
    volume
}
fn criterion(entity: &Handle<dyn Select3DSensitiveEntity>) -> SelectMgrSortCriterion {
    SelectMgrSortCriterion {
        entity: entity.clone(),
        point: Point3::new(8., 9., 10.),
        normal: Vec3::new(3., 4., 5.),
        tolerance: 7.,
        depth: 2.,
        ..Default::default()
    }
}
#[test]
fn non_point_and_invalid_picks_preserve_native_untouched_fields() {
    let base = SelectMgrViewerSelectorBase::new();
    let entity = GeometrySensitivePoint::new(false, geometry_transform(0)).cast();
    let mut result = criterion(&entity);
    let pick = SelectBasicsPickResult::default();
    select_mgr_update_point3d(
        &base,
        &mut result,
        &pick,
        &entity,
        &geometry_transform(4),
        &SelectMgrSelectingVolumeManager::new(),
    )
    .unwrap();
    assert_eq!(result.point, Point3::new(8., 9., 10.));
    select_mgr_update_point3d(
        &base,
        &mut result,
        &pick,
        &entity,
        &geometry_transform(4),
        &point_volume(),
    )
    .unwrap();
    assert_eq!(result.point.x(), f64::MAX);
    assert_eq!(result.normal, Vec3::new(3., 4., 5.));
    assert_eq!(result.tolerance, 7.);
}
#[test]
fn explicit_normal_uses_initial_location_but_argument_entity_sensitivity() {
    let base = SelectMgrViewerSelectorBase::new();
    let initial = GeometrySensitivePoint::new(true, geometry_transform(2)).cast();
    let factor = GeometrySensitivePoint::new(false, geometry_transform(0));
    factor.set_sensitivity_factor(33).unwrap();
    let mut result = criterion(&initial);
    let volume = point_volume();
    volume.set_camera(&Handle::default()).unwrap();
    let mut pick = SelectBasicsPickResult::new(2., 0., Point3::new(5., 8., 11.));
    pick.set_surface_normal(Vec3::new(3., 4., 0.));
    select_mgr_update_point3d(
        &base,
        &mut result,
        &pick,
        &factor.cast(),
        &geometry_transform(0),
        &volume,
    )
    .unwrap();
    assert_eq!(result.point, Point3::new(2., 2., 2.));
    assert_eq!(result.normal, Vec3::new(0.3, 0.8 / 3., 0.));
    assert_eq!(result.tolerance, 1.);
}
#[test]
fn tiny_normals_remain_untransformed_and_detected_points_use_criterion_depth() {
    let base = SelectMgrViewerSelectorBase::new();
    let entity = GeometrySensitivePoint::new(false, geometry_transform(0)).cast();
    let mut result = criterion(&entity);
    let mut pick = SelectBasicsPickResult::new(100., 0., Point3::new(5., 8., 11.));
    pick.set_surface_normal(Vec3::new(0.0001, 0., 0.));
    select_mgr_update_point3d(
        &base,
        &mut result,
        &pick,
        &entity,
        &geometry_transform(2),
        &point_volume(),
    )
    .unwrap();
    assert_eq!(result.normal, Vec3::new(0.0001, 0., 0.));
    pick.invalidate();
    pick.set_depth(100.);
    let volume = point_volume();
    select_mgr_update_point3d(
        &base,
        &mut result,
        &pick,
        &entity,
        &geometry_transform(0),
        &volume,
    )
    .unwrap();
    assert_eq!(result.point, volume.detected_point(result.depth).unwrap());
}
#[test]
fn scale_only_cache_keeps_volume_identity_and_object_clipping_is_not_cached() {
    let base = SelectMgrViewerSelectorBase::new();
    let global = point_volume();
    *base.selecting_volume.write() = global.clone();
    base.tolerances.write().add(10);
    let entity = GeometrySensitivePoint::new(false, geometry_transform(0));
    entity.set_sensitivity_factor(3).unwrap();
    let mut object = global.clone();
    let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    object
        .set_view_clipping(&planes, &Handle::default(), None)
        .unwrap();
    let mut range = SelectMgrViewClipRange::default();
    *range.change_unclip_range() = Range1::new(2., 4.).unwrap();
    object.set_view_clip_ranges(&range);
    let mut cache = DataMap::default();
    let mut result = SelectMgrSelectingVolumeManager::new();
    select_mgr_compute_frustum(
        &base,
        &entity.cast(),
        &global,
        &object,
        &GeneralTransform3::default(),
        &mut cache,
        &mut result,
    )
    .unwrap();
    assert_eq!(cache.len(), 1);
    assert_eq!(
        result.active_volume(),
        cache.find(&3).unwrap().active_volume()
    );
    assert_eq!(result.view_clipping(), &planes);
    assert!(cache.find(&3).unwrap().view_clipping().is_null());
    assert_eq!(result.view_clip_ranges(), &range);
    let first = result.active_volume().clone();
    select_mgr_compute_frustum(
        &base,
        &entity.cast(),
        &global,
        &global,
        &geometry_transform(0),
        &mut cache,
        &mut result,
    )
    .unwrap();
    assert_eq!(result.active_volume(), &first);
    assert!(result.view_clipping().is_null());
}
#[test]
fn unscaled_identity_reuses_object_and_transformed_scaling_bypasses_cache() {
    let base = SelectMgrViewerSelectorBase::new();
    let global = point_volume();
    let object = point_volume();
    let entity = GeometrySensitivePoint::new(true, geometry_transform(0));
    let mut cache = DataMap::default();
    let mut result = SelectMgrSelectingVolumeManager::new();
    select_mgr_compute_frustum(
        &base,
        &entity.cast(),
        &global,
        &object,
        &geometry_transform(0),
        &mut cache,
        &mut result,
    )
    .unwrap();
    assert_eq!(result.active_volume(), object.active_volume());
    *base.selecting_volume.write() = global.clone();
    base.tolerances.write().add(20);
    entity.set_sensitivity_factor(3).unwrap();
    select_mgr_compute_frustum(
        &base,
        &entity.cast(),
        &global,
        &object,
        &geometry_transform(1),
        &mut cache,
        &mut result,
    )
    .unwrap();
    assert!(cache.is_empty());
    assert_ne!(result.active_volume(), object.active_volume());
}
#[test]
fn layer_order_follows_clear_and_write_transitions_and_discards_old_entries() {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let selector = SelectMgrViewerSelector::new();
    selector
        .viewer_selector_base()
        .fields
        .write()
        .z_layer_order
        .bind(99, 88);
    let mut expected = Vec::new();
    for (index, layer) in viewer.get_all_z_layers().into_iter().enumerate() {
        let mut settings = viewer.z_layer_settings(layer).unwrap();
        settings.set_clear_depth(index == 2);
        settings.set_enable_depth_write(index < 3);
        viewer.set_z_layer_settings(layer, &settings).unwrap();
        expected.push((layer, [0, 0, 1, 2, 2][index]));
    }
    selector.update_z_layers(&view).unwrap();
    let fields = selector.viewer_selector_base().fields.read();
    assert!(!fields.z_layer_order.is_bound(&99));
    for (layer, position) in expected {
        assert_eq!(*fields.z_layer_order.find(&layer).unwrap(), position);
    }
}

#[test]
fn singular_transform_preserves_native_nonfinite_coordinates() {
    let base = SelectMgrViewerSelectorBase::new();
    let entity = GeometrySensitivePoint::new(false, geometry_transform(0)).cast();
    let mut result = criterion(&entity);
    let mut pick = SelectBasicsPickResult::new(2., 0., Point3::new(5., 8., 11.));
    pick.set_surface_normal(Vec3::new(3., 4., 0.));
    select_mgr_update_point3d(
        &base,
        &mut result,
        &pick,
        &entity,
        &geometry_transform(4),
        &point_volume(),
    )
    .unwrap();
    // The pinned release gp_Mat inversion deliberately has no singularity raise.
    assert!(result.point.x().is_nan());
    assert!(result.normal.x.is_nan());
    assert_eq!(result.tolerance, 12.);
}
#[test]
fn clear_sensitive_removes_retained_structures_then_releases_sequence() {
    use rustycad_tkservice::graphic3d::{
        Graphic3dStructure, Graphic3dStructureMembers, Graphic3dStructureSource,
    };
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let selector = SelectMgrViewerSelector::new();
    let structure =
        Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    structure.new_group().unwrap();
    structure.display();
    selector
        .viewer_selector_base()
        .fields
        .write()
        .debug_structures
        .append(structure.clone());
    selector.clear_sensitive(&Handle::default()).unwrap();
    assert!(structure.is_deleted());
    assert!(!structure.is_displayed());
    assert!(
        selector
            .viewer_selector_base()
            .fields
            .read()
            .debug_structures
            .is_empty()
    );
    selector
        .clear_sensitive(&viewer.create_view().unwrap())
        .unwrap();
}
#[test]
fn clear_sensitive_reports_invalid_null_structure_without_dropping_the_sequence() {
    let selector = SelectMgrViewerSelector::new();
    selector
        .viewer_selector_base()
        .fields
        .write()
        .debug_structures
        .append(Handle::default());
    assert_eq!(
        selector
            .clear_sensitive(&Handle::default())
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(
        selector
            .viewer_selector_base()
            .fields
            .read()
            .debug_structures
            .len(),
        1
    );
}
#[test]
fn failed_frustum_transform_preserves_output_and_does_not_populate_cache() {
    let base = SelectMgrViewerSelectorBase::new();
    let global = point_volume();
    *base.selecting_volume.write() = global.clone();
    base.tolerances.write().add(3);
    let entity = GeometrySensitivePoint::new(false, geometry_transform(0));
    entity.set_sensitivity_factor(2).unwrap();
    let mut result = point_volume();
    let previous = result.active_volume().clone();
    let mut cache = DataMap::default();
    let error = select_mgr_compute_frustum(
        &base,
        &entity.cast(),
        &global,
        &global,
        &geometry_transform(4),
        &mut cache,
        &mut result,
    )
    .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert_eq!(result.active_volume(), &previous);
    assert!(cache.is_empty());
}
