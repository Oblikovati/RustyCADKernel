//! Real camera, ray, rectangle and polyline selecting volumes mirror the native corpus.
use crate::select_mgr::{SelectMgrSelectingVolumeManager, SelectMgrViewClipRange};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::bnd::Range1;
use rustycad_tkmath::gp::{Axis1, Direction3, Point2, Point3};
use rustycad_tkservice::graphic3d::{Graphic3dCamera, Graphic3dCameraProjection};

pub(super) fn quadric_volume(
    kind: i32,
    perspective: bool,
    clipping: bool,
    miss: bool,
) -> SelectMgrSelectingVolumeManager {
    let mut volume = SelectMgrSelectingVolumeManager::new();
    initialize_quadric_volume(&mut volume, kind, miss);
    volume.allow_overlap_detection(kind != 6 && kind != 8);
    if kind > 2 {
        configure_quadric_camera(&volume, perspective);
    }
    volume.build_selecting_volume().unwrap();
    if clipping {
        let mut range = SelectMgrViewClipRange::default();
        range.add_clip_sub_range(Range1::new(8.5, 10.5).unwrap());
        volume.set_view_clip_ranges(&range);
    }
    volume
}
fn initialize_quadric_volume(volume: &mut SelectMgrSelectingVolumeManager, kind: i32, miss: bool) {
    let shift = if miss { 300. } else { 0. };
    if kind <= 2 {
        volume.init_axis_selecting_volume(quadric_axis(kind, miss));
        return;
    }
    initialize_quadric_screen_volume(volume, kind, shift);
}
fn initialize_quadric_screen_volume(
    volume: &mut SelectMgrSelectingVolumeManager,
    kind: i32,
    shift: f64,
) {
    match kind {
        3 | 4 => volume.init_point_selecting_volume(quadric_screen_point(kind, shift)),
        5 | 6 => initialize_quadric_box(volume, shift, false),
        7 => volume
            .init_polyline_selecting_volume(&quadric_polygon(shift))
            .unwrap(),
        8 => initialize_quadric_box(volume, shift, true),
        _ => unreachable!(),
    }
}
fn quadric_screen_point(kind: i32, shift: f64) -> Point2 {
    let x = if kind == 3 { 100. } else { 110. };
    Point2::new(x + shift, 100.)
}
fn initialize_quadric_box(volume: &mut SelectMgrSelectingVolumeManager, shift: f64, wide: bool) {
    let (first, last) = if wide { (20., 180.) } else { (85., 115.) };
    volume.init_box_selecting_volume(
        Point2::new(first + shift, first),
        Point2::new(last + shift, last),
    );
}
fn quadric_axis(kind: i32, miss: bool) -> Axis1 {
    let x = if miss {
        30.
    } else if kind == 1 {
        1.
    } else {
        0.
    };
    if kind == 2 {
        return Axis1::new(Point3::new(x, -10., 1.), Direction3::Y);
    }
    Axis1::new(Point3::new(x, 0., 10.), -Direction3::Z)
}
fn quadric_polygon(shift: f64) -> Array1<'static, Point2> {
    let mut polygon = Array1::with_bounds(1, 4).unwrap();
    polygon.as_mut_slice().copy_from_slice(&[
        Point2::new(85. + shift, 85.),
        Point2::new(115. + shift, 85.),
        Point2::new(115. + shift, 115.),
        Point2::new(85. + shift, 115.),
    ]);
    polygon
}
fn configure_quadric_camera(volume: &SelectMgrSelectingVolumeManager, perspective: bool) {
    let camera = quadric_camera(perspective);
    volume.set_camera(&camera).unwrap();
    volume.set_window_size(200, 200).unwrap();
    volume.set_pixel_tolerance(3).unwrap();
}
fn quadric_camera(perspective: bool) -> Handle<Graphic3dCamera> {
    let camera = Handle::new(Graphic3dCamera::default());
    camera
        .set_eye_and_center(&Point3::new(0., 0., 10.), &Point3::default())
        .unwrap();
    camera.set_up(&Direction3::Y);
    camera.set_scale(20.);
    camera.set_aspect(1.);
    camera.set_z_range(0.1, 30.).unwrap();
    camera.set_projection_type(if perspective {
        Graphic3dCameraProjection::Perspective
    } else {
        Graphic3dCameraProjection::Orthographic
    });
    camera
}
