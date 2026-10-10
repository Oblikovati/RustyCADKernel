//! Selection projection preserves the camera handle and pixel/viewport conventions.
use rustycad_tkernel::{ncollection::Vec3, standard::Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkv3d::select_mgr::SelectMgrFrustumBuilder;

#[test]
fn default_builder_and_camera_state_follow_native_defaults() {
    let builder = SelectMgrFrustumBuilder::default();
    assert!(builder.camera().is_null());
    assert_eq!(builder.window_size(), (i32::MAX, i32::MAX));
    assert_eq!(
        builder.project_pnt_on_view_plane(10., 20., 0.5),
        Point3::default()
    );
    let camera = Handle::new(Graphic3dCamera::default());
    builder.set_camera(&camera);
    builder.set_window_size(800, 600);
    assert_eq!(builder.window_size(), (800, 600));
    assert_eq!(builder.camera().object_address(), camera.object_address());
}

#[test]
fn invalidating_viewport_restores_the_pixel_projection() {
    let builder = SelectMgrFrustumBuilder::new();
    builder.set_camera(&Handle::new(Graphic3dCamera::default()));
    builder.set_window_size(800, 600);
    let original = builder.project_pnt_on_view_plane(120., 240., 0.5);
    builder.set_viewport(0.1, 0.2, 0.9, 0.8);
    assert_ne!(builder.project_pnt_on_view_plane(120., 240., 0.5), original);
    builder.invalidate_viewport();
    assert_eq!(builder.project_pnt_on_view_plane(120., 240., 0.5), original);
}

#[test]
fn plane_distance_normalizes_and_rejects_tiny_normals() {
    let builder = SelectMgrFrustumBuilder::new();
    let point = Vec3::new(2., -3., 4.);
    assert_eq!(
        builder.signed_plane_pnt_dist(&Vec3::new(0., 0., -2.), &point),
        -4.
    );
    assert_eq!(
        builder.signed_plane_pnt_dist(&Vec3::new(0., 1e-8, 0.), &point),
        0.
    );
}
