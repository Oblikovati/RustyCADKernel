//! Graphic3d_Camera: state counters, setters, tiles and dump; the numbers are compared with
//! the native class by the sv_camera probes.
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkservice::graphic3d::*;

#[test]
fn setters_bump_the_state_counters() {
    let camera = Graphic3dCamera::default();
    let (projection, world_view) = (camera.projection_state(), camera.world_view_state());
    camera.set_fov_y(30.0);
    assert!(camera.projection_state() > projection);
    assert_eq!(camera.world_view_state(), world_view);
    camera.set_distance(10.0);
    assert!(camera.world_view_state() > world_view);
    camera.set_fov_y(30.0);
    assert!(camera.world_view_proj_state().is_valid());
}

#[test]
fn invalid_ranges_and_scales_raise() {
    let camera = Graphic3dCamera::default();
    assert_eq!(
        camera.set_z_range(5.0, 1.0).unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
    camera.set_projection_type(Graphic3dCameraProjection::Perspective);
    assert!(camera.set_z_range(-1.0, 10.0).is_err());
    assert_eq!(
        camera
            .set_axial_scale(Xyz::new(1.0, 0.0, 1.0))
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn perspective_scale_moves_the_eye() {
    let camera = Graphic3dCamera::default();
    camera.set_projection_type(Graphic3dCameraProjection::Perspective);
    camera.set_scale(100.0);
    assert!((camera.scale() - 100.0).abs() < 1e-9);
    assert_eq!(camera.center(), Point3::new(0.0, 0.0, 0.0));
}

#[test]
fn tiles_compare_by_lower_left_offset() {
    let mut tile = Graphic3dCameraTile {
        total_size: Vec2::new(100, 80),
        tile_size: Vec2::new(50, 40),
        offset: Vec2::new(10, 79),
        is_top_down: true,
    };
    assert_eq!(tile.offset_lower_left(), Vec2::new(10, 0));
    let lower = Graphic3dCameraTile {
        offset: Vec2::new(10, 0),
        is_top_down: false,
        ..tile
    };
    assert!(tile.is_same(&lower));
    tile.offset = Vec2::new(-5, 60);
    let cropped = tile.cropped();
    assert_eq!(
        (cropped.offset, cropped.tile_size),
        (Vec2::new(0, 60), Vec2::new(50, 20))
    );
}

#[test]
fn copies_share_mapping_and_orientation() {
    let source = Graphic3dCamera::default();
    source.set_projection_type(Graphic3dCameraProjection::Stereo);
    source
        .set_eye_and_center(&Point3::new(1.0, 2.0, 3.0), &Point3::new(0.0, 0.0, 0.0))
        .unwrap();
    let copy = Graphic3dCamera::from_camera(&source);
    assert_eq!(copy.projection_type(), Graphic3dCameraProjection::Stereo);
    assert_eq!(copy.eye(), Point3::new(1.0, 2.0, 3.0));
    assert_eq!(
        copy.projection_matrix().get_value(0, 0),
        source.projection_matrix().get_value(0, 0)
    );
}

#[test]
fn dump_lists_valid_matrices_only() {
    let camera = Graphic3dCamera::default();
    let mut stream = DumpStream::default();
    camera.dump_json(&mut stream, -1).unwrap();
    assert!(
        stream
            .text()
            .starts_with("\"className\": \"Graphic3d_Camera\"")
    );
    assert!(!stream.text().contains("Orientation\": {"));
    camera.orientation_matrix();
    let mut stream = DumpStream::default();
    camera.dump_json(&mut stream, -1).unwrap();
    assert!(stream.text().contains("\"Orientation\""));
    assert!(!stream.text().contains("\"MProjection\""));
}
