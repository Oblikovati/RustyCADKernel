//! Martensite point cloud alias/copy contracts, native selection modes and live aspects.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    ncollection::HArray1,
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::{
    aspect::{AspectTypeOfFacingModel, AspectTypeOfMarker},
    graphic3d::*,
};
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, select3d::*, v3d::V3dViewer,
};
fn buffer() -> Handle<Graphic3dArrayOfPoints> {
    let points = Handle::new(Graphic3dArrayOfPoints::new(4, 0).unwrap());
    points.add_vertex_pnt(&Point3::new(0.1, -2., 3.)).unwrap();
    points.add_vertex_pnt(&Point3::new(4., 5., -6.)).unwrap();
    points
}
#[test]
fn native_defaults_and_empty_presentations() {
    let cloud = AisPointCloud::new().unwrap();
    assert_eq!(cloud.kind_of_interactive(), AisKindOfInteractive::None);
    assert_eq!(cloud.signature(), -1);
    assert_eq!(
        cloud.display_mode(),
        AisPointCloudDisplayMode::Points as i32
    );
    assert_eq!(
        cloud.hilight_mode(),
        AisPointCloudDisplayMode::BndBox as i32
    );
    assert_eq!(
        cloud.attributes().shading_aspect().aspect().marker_type(),
        AspectTypeOfMarker::Point
    );
    let dynamic = cloud.dynamic_hilight_attributes().point_aspect().aspect();
    assert_eq!(dynamic.marker_kind(), AspectTypeOfMarker::Plus);
    assert_eq!(dynamic.interior_color(), Color::named(NamedColor::CYAN1));
    assert_eq!(dynamic.scale(), 1.);
    assert!(cloud.get_points().is_null());
    assert!(cloud.get_bounding_box().is_void());
    for mode in [-1, 0, 1, 2, 99] {
        assert!(cloud.accept_display_mode(mode));
    }
    assert_eq!(AisPointCloudSelectionMode::SubsetOfPoints as i32, 1);
    assert_eq!(AisPointCloudSelectionMode::BndBox as i32, 2);
}
#[test]
fn aliased_buffer_mutations_do_not_refresh_cached_bounds_or_schedule_updates() {
    let cloud = AisPointCloud::new().unwrap();
    let points = buffer();
    cloud.set_points(&points).unwrap();
    assert_eq!(cloud.get_points(), points);
    let bounds = cloud.get_bounding_box().get().unwrap();
    assert_eq!(bounds[0], f64::from(0.1_f32));
    points
        .set_vertice_pnt(1, &Point3::new(-9., -8., -7.))
        .unwrap();
    assert_eq!(
        cloud.get_points().vertice(1).unwrap(),
        Point3::new(-9., -8., -7.)
    );
    assert_eq!(cloud.get_bounding_box().get().unwrap(), bounds);
    cloud.set_points(&points).unwrap();
    assert_eq!(cloud.get_bounding_box().get().unwrap()[0], -9.);
    assert!(cloud.presentations().is_empty());
    cloud.set_points(&Handle::default()).unwrap();
    assert!(cloud.get_points().is_null());
    assert!(cloud.get_bounding_box().is_void());
}
#[test]
fn coordinate_copy_uses_array_offsets_and_copies_color_normal_attributes() {
    let cloud = AisPointCloud::new().unwrap();
    let coords = Handle::new(HArray1::with_value(-7, -6, Point3::new(0.1, 2., 3.)).unwrap());
    let colors = Handle::new(HArray1::with_value(8, 9, Color::named(NamedColor::RED)).unwrap());
    let normals = Handle::new(HArray1::with_value(42, 43, Direction3::Z).unwrap());
    colors
        .change_array1()
        .set_value(9, Color::named(NamedColor::GREEN))
        .unwrap();
    normals
        .change_array1()
        .set_value(43, Direction3::X)
        .unwrap();
    cloud.set_points_arrays(&coords, &colors, &normals).unwrap();
    let points = cloud.get_points();
    assert_eq!(points.vertex_number(), 2);
    assert_eq!(
        points.vertex_color(2).unwrap(),
        Color::named(NamedColor::GREEN)
    );
    assert_eq!(points.vertex_normal(2).unwrap(), Direction3::X);
    coords
        .change_array1()
        .set_value(-7, Point3::new(99., 99., 99.))
        .unwrap();
    assert_eq!(points.vertice(1).unwrap().x(), f64::from(0.1_f32));
    assert_eq!(
        cloud.get_bounding_box().get().unwrap()[0],
        f64::from(0.1_f32)
    );
}
#[test]
fn invalid_attribute_lengths_and_null_coordinates_clear_previous_buffers() {
    let cloud = AisPointCloud::new().unwrap();
    let coords = Handle::new(HArray1::with_value(1, 2, Point3::default()).unwrap());
    let colors = Handle::new(HArray1::with_value(1, 1, Color::default()).unwrap());
    let normals = Handle::new(HArray1::with_value(1, 3, Direction3::Z).unwrap());
    for (coords, colors, normals) in [
        (coords.clone(), colors, Handle::default()),
        (coords, Handle::default(), normals),
        (Handle::default(), Handle::default(), Handle::default()),
    ] {
        cloud.set_points(&buffer()).unwrap();
        cloud.set_points_arrays(&coords, &colors, &normals).unwrap();
        assert!(cloud.get_points().is_null());
        assert!(cloud.get_bounding_box().is_void());
    }
}
#[test]
fn selection_uses_native_owners_modes_sensitivity_and_does_not_clear() {
    let cloud = AisPointCloud::new().unwrap();
    cloud.set_points(&buffer()).unwrap();
    for mode in [-1, 0, 1, 2, 99] {
        let selection = Handle::new(SelectMgrSelection::new(mode));
        cloud.compute_selection(&selection, mode).unwrap();
        if !(0..=2).contains(&mode) {
            assert!(selection.entities().is_empty());
            continue;
        }
        let sensitive = selection.entities()[0].base_sensitive();
        let owner = sensitive.owner_id();
        assert_eq!(owner.selectable().object_address(), cloud.object_address());
        assert_eq!(owner.priority(), if mode == 1 { 5 } else { 0 });
        assert_eq!(owner.is_forced_hilight(), mode == 1);
        assert_eq!(!owner.cast::<AisPointCloudOwner>().is_null(), mode == 1);
        if mode != 2 {
            assert_eq!(sensitive.sensitivity_factor(), 8);
        }
        if mode == 2 {
            assert!(!sensitive.cast::<Select3DSensitiveBox>().is_null());
        }
        cloud.compute_selection(&selection, mode).unwrap();
        assert_eq!(selection.entities().len(), 2);
    }
}
#[test]
fn presentations_use_aliased_arrays_and_box_edges_only_for_native_modes() {
    let cloud = AisPointCloud::new().unwrap();
    cloud.attributes().setup_own_defaults().unwrap();
    cloud.set_points(&buffer()).unwrap();
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    for mode in [-1, 0, 1, 2, 99] {
        let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
        cloud
            .compute(&Handle::default(), &prs.cast(), mode)
            .unwrap();
        assert_eq!(prs.groups().len(), usize::from(mode == 0 || mode == 2));
    }
}
#[test]
fn color_material_restoration_preserves_link_and_facing_policy() {
    let cloud = AisPointCloud::new().unwrap();
    let linked = Handle::new(Prs3dDrawer::new());
    linked.setup_own_defaults().unwrap();
    linked
        .shading_aspect()
        .set_color(
            Color::named(NamedColor::GREEN),
            AspectTypeOfFacingModel::BothSide,
        )
        .unwrap();
    cloud.attributes().link_to(&linked);
    cloud.set_color(Color::named(NamedColor::RED)).unwrap();
    cloud.set_transparency(0.25).unwrap();
    cloud
        .set_material(&Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold))
        .unwrap();
    assert!(cloud.has_color() && cloud.has_material());
    cloud.unset_color().unwrap();
    let shading = cloud.attributes().shading_aspect();
    assert_eq!(
        shading.aspect().interior_color(),
        Color::named(NamedColor::GREEN)
    );
    assert_eq!(
        shading.transparency(AspectTypeOfFacingModel::BothSide),
        0.25
    );
    cloud.unset_material().unwrap();
    assert!(!cloud.has_material());
    assert!(cloud.attributes().has_own_shading_aspect());
    assert_eq!(
        shading.material(AspectTypeOfFacingModel::BothSide).name(),
        linked
            .shading_aspect()
            .material(AspectTypeOfFacingModel::BothSide)
            .name()
    );
    cloud.unset_color().unwrap();
    cloud.unset_material().unwrap();
}

#[test]
fn empty_coordinate_arrays_retain_native_empty_buffer_selection() {
    let cloud = AisPointCloud::new().unwrap();
    cloud
        .set_points_arrays(
            &Handle::new(HArray1::<Point3>::default()),
            &Handle::default(),
            &Handle::default(),
        )
        .unwrap();
    assert!(!cloud.get_points().is_null());
    assert_eq!(cloud.get_points().vertex_number(), 0);
    assert!(cloud.get_bounding_box().is_void());
    let selection = Handle::new(SelectMgrSelection::new(0));
    cloud.compute_selection(&selection, 0).unwrap();
    assert_eq!(selection.entities().len(), 1);
}
