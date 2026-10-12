//! Camera animation fixtures are shared by capture, retained-stream and timing tests.
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::v3d::V3dViewSource;
pub const MODES: [&str; 8] = [
    "animcam-start",
    "animcam-mid",
    "animcam-end",
    "animcam-instant",
    "animcam-nested",
    "animcam-paused",
    "animcam-live-end",
    "animcam-detached",
];
/// Retain endpoints for a two-second camera move; `animation(&scene.view)`.
pub fn animation(view: &Handle<dyn V3dViewSource>) -> Handle<AisAnimationCamera> {
    let start = Handle::new(Graphic3dCamera::default());
    start.copy(&view.camera());
    let end = Handle::new(Graphic3dCamera::default());
    end.copy(&start);
    end.set_eye(&Point3::new(4., 0., 20.)).unwrap();
    end.set_center(&Point3::new(4., 0., 0.)).unwrap();
    end.set_scale(40.);
    let animation = Handle::new(AisAnimationCamera::new("camera", view));
    animation.set_camera_start(&start);
    animation.set_camera_end(&end);
    animation.set_own_duration(2.);
    animation
}
/// Apply one capture mode without submitting a frame; `apply(&view, "animcam-mid")`.
pub fn apply(view: &Handle<dyn V3dViewSource>, mode: &str) {
    let animation = animation(view);
    configure(&animation, mode);
    animation.start(false).unwrap();
    if mode == "animcam-paused" {
        animation.pause().unwrap();
    }
    if mode == "animcam-nested" {
        return nested(&animation);
    }
    let pts = match mode {
        "animcam-start" | "animcam-instant" => 0.,
        "animcam-end" => 2.,
        _ => 1.,
    };
    animation.update(pts).unwrap();
}
fn configure(animation: &AisAnimationCamera, mode: &str) {
    if mode == "animcam-instant" {
        animation.set_own_duration(0.);
    }
    if mode == "animcam-live-end" {
        animation.camera_end().set_scale(60.);
    }
    if mode == "animcam-detached" {
        animation.set_view(&Handle::default());
    }
}
fn nested(animation: &Handle<AisAnimationCamera>) {
    let parent = AisAnimation::new("parent");
    parent.set_start_pts(0.5);
    animation.set_start_pts(0.5);
    parent.add(&animation.cast()).unwrap();
    parent.start(false).unwrap();
    parent.update(2.).unwrap();
}
