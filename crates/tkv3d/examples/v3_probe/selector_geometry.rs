//! Exact counterparts of native private point/frustum helpers and protected layer grouping.
#[path = "../../tests/support/geometry_sensitive.rs"]
mod geometry_sensitive;
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use geometry_sensitive::GeometrySensitivePoint;
pub(super) use geometry_sensitive::geometry_transform;
use rustycad_tkernel::{
    ncollection::{DataMap, Vec3},
    standard::Handle,
};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{Axis1, Direction3, Point2, Point3, Xy},
};
use rustycad_tkservice::{
    graphic3d::{Graphic3dCamera, Graphic3dCameraProjection, Graphic3dSequenceOfHClipPlane},
    null_driver::NullGraphicDriver,
};
use rustycad_tkv3d::{select_basics::*, select_mgr::*, select3d::*, v3d::V3dViewer};
pub(super) fn geometry_volume(kind: i32, projection: i32) -> SelectMgrSelectingVolumeManager {
    let mut volume = SelectMgrSelectingVolumeManager::new();
    match kind {
        0 => return volume,
        1 => volume.init_point_selecting_volume(Point2::from(Xy::new(4., 3.))),
        2 => volume.init_box_selecting_volume(
            Point2::from(Xy::new(1., 1.)),
            Point2::from(Xy::new(7., 5.)),
        ),
        3 => volume.init_axis_selecting_volume(Axis1::new(Point3::new(1., 2., 3.), Direction3::Z)),
        _ => unreachable!(),
    }
    let camera = Handle::new(Graphic3dCamera::default());
    camera.set_projection_type(if projection == 2 {
        Graphic3dCameraProjection::Perspective
    } else {
        Graphic3dCameraProjection::Orthographic
    });
    volume.set_camera(&camera).unwrap();
    volume.set_window_size(8, 6).unwrap();
    volume.set_pixel_tolerance(10).unwrap();
    volume.build_selecting_volume().unwrap();
    if projection == 0 {
        volume.set_camera(&Handle::default()).unwrap();
    }
    volume
}
fn tolerance_kind(kind: i32) -> SelectMgrTypeOfDepthTolerance {
    match kind {
        0 => SelectMgrTypeOfDepthTolerance::Uniform,
        1 => SelectMgrTypeOfDepthTolerance::UniformPixels,
        2 => SelectMgrTypeOfDepthTolerance::SensitivityFactor,
        _ => unreachable!(),
    }
}
fn point_criterion_state(criterion: &SelectMgrSortCriterion) {
    flag(criterion.entity.is_null());
    pnt(criterion.point);
    for value in [criterion.normal.x, criterion.normal.y, criterion.normal.z] {
        real(f64::from(value));
    }
    for value in [criterion.depth, criterion.min_dist, criterion.tolerance] {
        real(value);
    }
    for value in [
        criterion.selection_priority,
        criterion.display_priority,
        criterion.z_layer_position,
        criterion.nb_owner_matches,
    ] {
        int(value);
    }
    flag(criterion.is_prefer_priority);
}
pub fn selector_point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, projection, pick_kind, transform, initial, factor, tol_kind) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let normal = Vec3::new(r.real() as f32, r.real() as f32, r.real() as f32);
    let (tolerance, scale, depth) = (r.real(), r.real(), r.real());
    let base = SelectMgrViewerSelectorBase::new();
    {
        let mut fields = base.fields.write();
        let settings = &mut fields.settings;
        settings.depth_tolerance_type = tolerance_kind(tol_kind);
        settings.depth_tolerance = tolerance;
        settings.camera_scale = scale;
        settings.camera_eye = Point3::new(1., 2., 3.);
        settings.camera_direction = Direction3::Z;
    }
    let located = GeometrySensitivePoint::new(initial >= 0, geometry_transform(initial.max(0)));
    let argument = GeometrySensitivePoint::new(false, geometry_transform(0));
    argument.set_sensitivity_factor(factor).unwrap();
    let mut criterion = SelectMgrSortCriterion {
        entity: located.cast(),
        point: Point3::new(8., 9., 10.),
        normal: Vec3::new(3., 4., 5.),
        tolerance: 7.,
        depth,
        ..Default::default()
    };
    let mut pick = SelectBasicsPickResult::default();
    if pick_kind == 1 || pick_kind == 2 {
        pick.set_depth(100.);
    }
    if pick_kind >= 2 {
        pick.set_picked_point(Point3::new(5., 8., 11.));
    }
    pick.set_surface_normal(normal);
    attempt(|| {
        select_mgr_update_point3d(
            &base,
            &mut criterion,
            &pick,
            &argument.cast(),
            &geometry_transform(transform),
            &geometry_volume(kind, projection),
        )
    });
    point_criterion_state(&criterion);
    end();
}
fn geometry_volume_state(volume: &SelectMgrSelectingVolumeManager) {
    int(volume.get_active_selection_type() as i32);
    flag(volume.camera().is_null());
    flag(volume.view_clipping().is_null());
    flag(volume.object_clipping().is_null());
    for depth in [1., 2., 3., 4., 5.] {
        flag(volume.view_clip_ranges().is_clipped(depth));
    }
    if !volume.active_volume().is_null() {
        attempt(|| {
            pnt(volume.detected_point(2.)?);
            Ok(())
        });
    }
}
pub fn selector_frustum_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, projection, initial, transform, factor, adaptive, custom) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
        r.integer(),
    );
    let base = SelectMgrViewerSelectorBase::new();
    let global = geometry_volume(kind, projection);
    let mut object = geometry_volume(kind, projection);
    *base.selecting_volume.write() = global.clone();
    base.tolerances.write().add(adaptive);
    base.tolerances.write().set_custom_tolerance(custom);
    let entity = GeometrySensitivePoint::new(initial >= 0, geometry_transform(initial.max(0)));
    entity.set_sensitivity_factor(factor).unwrap();
    let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    object
        .set_view_clipping(&planes, &Handle::default(), None)
        .unwrap();
    let mut range = SelectMgrViewClipRange::default();
    *range.change_unclip_range() = Range1::new(2., 4.).unwrap();
    object.set_view_clip_ranges(&range);
    let mut cache = DataMap::default();
    let mut result = SelectMgrSelectingVolumeManager::new();
    attempt(|| {
        select_mgr_compute_frustum(
            &base,
            &entity.cast(),
            &global,
            &object,
            &geometry_transform(transform),
            &mut cache,
            &mut result,
        )
    });
    int(cache.len() as i32);
    flag(result.active_volume() == object.active_volume());
    geometry_volume_state(&result);
    let first = result.active_volume().clone();
    object
        .set_view_clipping(&Handle::default(), &Handle::default(), None)
        .unwrap();
    attempt(|| {
        select_mgr_compute_frustum(
            &base,
            &entity.cast(),
            &global,
            &object,
            &geometry_transform(transform),
            &mut cache,
            &mut result,
        )
    });
    flag(result.active_volume() == &first);
    int(cache.len() as i32);
    geometry_volume_state(&result);
    for (key, value) in cache.items() {
        int(*key);
        geometry_volume_state(value);
    }
    end();
}
pub fn selector_layers_case(values: &[f64]) {
    let seed = values[0] as i32;
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let selector = SelectMgrViewerSelector::new();
    for (index, layer) in viewer.get_all_z_layers().into_iter().enumerate() {
        let mut settings = viewer.z_layer_settings(layer).unwrap();
        settings.set_clear_depth(seed & (1 << index) != 0);
        settings.set_enable_depth_write(seed & (1 << (index + 5)) != 0);
        viewer.set_z_layer_settings(layer, &settings).unwrap();
    }
    selector
        .viewer_selector_base()
        .fields
        .write()
        .z_layer_order
        .bind(99, 88);
    selector.update_z_layers(&view).unwrap();
    let fields = selector.viewer_selector_base().fields.read();
    flag(fields.z_layer_order.is_bound(&99));
    let layers = viewer.get_all_z_layers();
    int(layers.len() as i32);
    for layer in layers {
        int(layer);
        int(*fields.z_layer_order.find(&layer).unwrap());
    }
    end();
}
pub fn selector_clear_case(values: &[f64]) {
    use rustycad_tkservice::graphic3d::{
        Graphic3dStructure, Graphic3dStructureMembers, Graphic3dStructureSource,
    };
    let count = values[0] as i32;
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let selector = SelectMgrViewerSelector::new();
    let mut structures = Vec::new();
    for _ in 0..count {
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
        structures.push(structure);
    }
    let view = if values[1] != 0. {
        viewer.create_view().unwrap()
    } else {
        Handle::default()
    };
    selector.clear_sensitive(&view).unwrap();
    int(selector
        .viewer_selector_base()
        .fields
        .read()
        .debug_structures
        .len() as i32);
    for structure in structures {
        flag(structure.is_deleted());
        flag(structure.is_displayed());
    }
    selector.clear_sensitive(&view).unwrap();
    int(selector
        .viewer_selector_base()
        .fields
        .read()
        .debug_structures
        .len() as i32);
    end();
}
