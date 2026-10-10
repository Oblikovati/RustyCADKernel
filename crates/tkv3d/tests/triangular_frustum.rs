//! Observable concrete-volume contracts, independently checked by the native corpus.
use rustycad_tkernel::{
    ncollection::{DynamicArray, Vec3, Vec4},
    standard::{Handle, dump::DumpStream},
};
use rustycad_tkmath::{
    bnd::Range1,
    gp::{GeneralTransform3, Point2, Point3, Transform3, Xy},
};
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkv3d::{select_basics::SelectBasicsPickResult, select_mgr::*};

fn built_triangle() -> SelectMgrTriangularFrustum {
    let volume = SelectMgrTriangularFrustum::new();
    let camera = Handle::new(Graphic3dCamera::default());
    camera
        .set_eye_and_center(&Point3::new(0., 0., 10.), &Point3::default())
        .unwrap();
    camera.set_z_range(0.1, 100.).unwrap();
    camera.set_scale(20.);
    volume.set_camera(&camera);
    volume.set_window_size(800, 600);
    volume.init(
        Point2::from(Xy::new(200., 100.)),
        Point2::from(Xy::new(600., 100.)),
        Point2::from(Xy::new(400., 500.)),
    );
    volume.build().unwrap();
    volume
}
#[test]
fn initialization_retains_geometry_until_build_and_rebuilds_from_new_pixels() {
    let volume = built_triangle();
    let original = volume.frustum_core.read().clone();
    volume.init(Point2::default(), Point2::default(), Point2::default());
    assert_eq!(volume.frustum_core.read().vertices(), original.vertices());
    volume.build().unwrap();
    assert_ne!(volume.frustum_core.read().vertices(), original.vertices());
    assert!(
        volume
            .frustum_core
            .read()
            .planes()
            .iter()
            .all(|normal| normal.square_modulus() == 0.)
    );
    assert!(!volume.is_scalable());
    assert_eq!(volume.get_selection_type(), SelectMgrSelectionType::Unknown);
}
#[test]
fn transformed_copy_ignores_scale_and_builder_and_preserves_source() {
    let volume = built_triangle();
    volume.set_pixel_tolerance(17);
    let source = volume.frustum_core.read().clone();
    let builder = Handle::new(SelectMgrFrustumBuilder::new());
    builder.set_window_size(13, 19);
    let mut transform = GeneralTransform3::default();
    transform.set_value(1, 4, 3.).unwrap();
    let first = volume
        .scale_and_transform(1, &transform, &builder)
        .unwrap()
        .cast::<SelectMgrTriangularFrustum>();
    let second = volume
        .scale_and_transform(42, &transform, &Handle::default())
        .unwrap()
        .cast::<SelectMgrTriangularFrustum>();
    assert_eq!(
        first.frustum_core.read().vertices(),
        second.frustum_core.read().vertices()
    );
    assert_eq!(
        first.frustum_core.read().vertices()[0].x(),
        source.vertices()[0].x() + 3.
    );
    assert_eq!(volume.frustum_core.read().vertices(), source.vertices());
    assert_eq!(first.window_size(), Some((i32::MAX, i32::MAX)));
    assert_eq!(*first.base.pixel_tolerance.read(), 2);
    assert_eq!(
        first.camera().object_address(),
        volume.camera().object_address()
    );
    assert_eq!(
        first.selection_triangle.read().points,
        volume.selection_triangle.read().points
    );
    assert!(!first.cast::<dyn SelectMgrBaseFrustum>().is_null());
}
#[test]
fn builder_copy_starts_empty_and_adopts_shared_configuration() {
    let volume = built_triangle();
    let builder = Handle::new(SelectMgrFrustumBuilder::new());
    let camera = Handle::new(Graphic3dCamera::default());
    builder.set_camera(&camera);
    builder.set_window_size(320, 240);
    let copy = volume
        .copy_with_builder(&builder)
        .unwrap()
        .cast::<SelectMgrTriangularFrustum>();
    assert_eq!(copy.frustum_core.read().vertices(), [Point3::default(); 6]);
    assert_eq!(copy.camera().object_address(), camera.object_address());
    assert_eq!(*copy.base.pixel_tolerance.read(), 2);
    builder.set_window_size(640, 480);
    assert_eq!(copy.window_size(), Some((640, 480)));
    copy.build().unwrap();
    assert_ne!(copy.frustum_core.read().vertices(), [Point3::default(); 6]);
    assert_eq!(volume.window_size(), Some((800, 600)));
}
#[test]
fn clear_releases_builder_but_retains_camera_and_reports_invalid_rebuild() {
    let volume = built_triangle();
    let camera = volume.camera();
    let core = volume.frustum_core.read().clone();
    volume.clear();
    assert!(volume.base.builder.read().is_null());
    assert_eq!(volume.window_size(), None);
    assert_eq!(volume.camera().object_address(), camera.object_address());
    assert_eq!(volume.frustum_core.read().vertices(), core.vertices());
    let failure = volume.build().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
    assert!(failure.message().contains("builder=null"));
    assert!(failure.message().contains("expected"));
}
#[test]
fn planes_append_to_existing_output_and_dump_depth_controls_inherited_geometry() {
    let volume = built_triangle();
    let mut planes = DynamicArray::default();
    planes.append(Vec4::new(1., 2., 3., 4.));
    volume.get_planes(&mut planes);
    volume.get_planes(&mut planes);
    assert_eq!(planes.len(), 11);
    assert_eq!(planes.value(0).unwrap().w, 4.);
    assert_eq!(planes.value(1).unwrap(), planes.value(6).unwrap());
    for depth in [-1, 0, 1, 2] {
        let mut stream = DumpStream::default();
        volume.dump_json(&mut stream, depth).unwrap();
        assert!(stream.text().contains("SelectMgr_TriangularFrustum"));
        assert_eq!(stream.text().contains("PixelTolerance"), depth != 0);
        assert_eq!(stream.text().contains("gp_Pnt"), !(0..=1).contains(&depth));
    }
}
#[test]
fn point_and_sphere_overloads_preserve_their_different_native_algorithms() {
    let volume = built_triangle();
    let clip = SelectMgrViewClipRange::default();
    let mut pick = SelectBasicsPickResult::default();
    assert!(!volume.overlaps_point(&Point3::default()).unwrap());
    assert!(
        volume
            .overlaps_point_pick(&Point3::default(), &clip, &mut pick)
            .unwrap()
    );
    let beyond = Point3::new(0., 0., 200.);
    let mut inside = true;
    assert!(
        !volume
            .overlaps_sphere(&beyond, 0.5, Some(&mut inside))
            .unwrap()
    );
    assert!(
        volume
            .overlaps_sphere_pick(&beyond, 0.5, &clip, &mut pick)
            .unwrap()
    );
    assert!(inside);
}
#[test]
fn boundary_queries_return_false_and_unknown_triangle_sensitivity_returns_true() {
    let volume = built_triangle();
    let clip = SelectMgrViewClipRange::default();
    let mut pick = SelectBasicsPickResult::default();
    let vertices = [
        Point3::new(-1., -1., 0.),
        Point3::new(1., -1., 0.),
        Point3::new(0., 1., 0.),
    ];
    assert!(
        volume
            .overlaps_polygon(&vertices, 0, &clip, &mut pick)
            .unwrap()
    );
    assert!(
        !volume
            .overlaps_polygon(&vertices, 1, &clip, &mut pick)
            .unwrap()
    );
    assert!(!volume.overlaps_polygon(&[], 1, &clip, &mut pick).unwrap());
    assert!(
        volume
            .overlaps_polygon(&vertices[..2], 0, &clip, &mut pick)
            .is_err()
    );
    assert!(
        !volume
            .overlaps_triangle(
                &vertices[0],
                &vertices[1],
                &vertices[2],
                1,
                &clip,
                &mut pick
            )
            .unwrap()
    );
    assert!(
        volume
            .overlaps_triangle(
                &vertices[0],
                &vertices[1],
                &vertices[2],
                7,
                &clip,
                &mut pick
            )
            .unwrap()
    );
    assert!(
        !volume
            .overlaps_polygon(&vertices, 7, &clip, &mut pick)
            .unwrap()
    );
}
#[test]
fn all_inside_outputs_and_pick_fields_are_untouched_and_clipping_is_ignored() {
    let volume = built_triangle();
    let mut clip = SelectMgrViewClipRange::default();
    *clip.change_unclip_range() = Range1::new(200., 201.).unwrap();
    let min = Vec3::new(-1., -1., -1.);
    let max = Vec3::new(1., 1., 1.);
    let transform = Transform3::default();
    let initial = SelectBasicsPickResult::new(123., 456., Point3::new(7., 8., 9.));
    let mut pick = initial;
    assert!(
        volume
            .overlaps_box_pick(&min, &max, &clip, &mut pick)
            .unwrap()
    );
    assert!(
        volume
            .overlaps_segment(
                &Point3::new(-1., 0., 0.),
                &Point3::new(1., 0., 0.),
                &clip,
                &mut pick
            )
            .unwrap()
    );
    for initial_flag in [false, true] {
        let mut inside = initial_flag;
        volume.overlaps_box(&min, &max, Some(&mut inside)).unwrap();
        volume
            .overlaps_cylinder(1., 1., 2., &transform, false, Some(&mut inside))
            .unwrap();
        volume
            .overlaps_circle(1., &transform, true, Some(&mut inside))
            .unwrap();
        assert_eq!(inside, initial_flag);
    }
    volume
        .overlaps_cylinder_pick(1., 1., 2., &transform, false, &clip, &mut pick)
        .unwrap();
    volume
        .overlaps_circle_pick(1., &transform, true, &clip, &mut pick)
        .unwrap();
    assert_eq!(pick, initial);
}
#[test]
fn dynamic_handles_preserve_ancestry_and_drop_releases_the_builder() {
    let volume = Handle::new(built_triangle());
    assert!(volume.is_instance_name("SelectMgr_TriangularFrustum"));
    assert!(volume.is_kind_name("SelectMgr_BaseFrustum"));
    assert!(volume.is_kind_name("SelectMgr_BaseIntersector"));
    let builder = volume.base.builder.read().clone();
    assert_eq!(builder.ref_count(), 2);
    drop(volume);
    assert_eq!(builder.ref_count(), 1);
    assert_eq!(builder.window_size(), (800, 600));
}
