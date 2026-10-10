//! Camera animation's retained state and live camera identity.
use super::common::{Reader, dir, end, flag, int, pnt, real};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
use rustycad_tkv3d::{
    ais::*,
    v3d::{V3dView, V3dViewer},
};
struct CameraAnimationProbe {
    _viewer: Handle<V3dViewer>,
    view: Handle<V3dView>,
    live: Handle<Graphic3dCamera>,
    start: Handle<Graphic3dCamera>,
    end: Handle<Graphic3dCamera>,
    animation: AisAnimationCamera,
}
/// Apply one independent native lifecycle input; `camera_case(&[0.,2.,0.,1.])`.
pub fn camera_case(args: &[f64]) {
    let mut reader = Reader::new(args);
    let flags = reader.integer();
    let duration = reader.real();
    let start_pts = reader.real();
    let probe = camera_probe(flags, duration, start_pts);
    probe.animation.start(false).unwrap();
    if flags & 8 != 0 {
        probe.animation.pause().unwrap();
    }
    flag(probe.animation.update(reader.real()).unwrap());
    camera_probe_state(&probe);
    end();
}
fn camera_probe(flags: i32, duration: f64, start_pts: f64) -> CameraAnimationProbe {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    view.set_immediate_update(flags & 32 != 0);
    let live = view.camera();
    configure_probe_camera(&live);
    let (start, end) = probe_endpoints(&live, flags);
    let animation = probe_atom(&view, &start, &end, flags);
    animation.set_start_pts(start_pts);
    animation.set_own_duration(duration);
    CameraAnimationProbe {
        _viewer: viewer,
        view,
        live,
        start,
        end,
        animation,
    }
}
fn configure_probe_camera(camera: &Graphic3dCamera) {
    camera.set_eye(&Point3::new(0., 0., 20.)).unwrap();
    camera.set_center(&Point3::default()).unwrap();
    camera.set_up(&Direction3::new(0., 1., 0.).unwrap());
    camera.set_scale(20.);
    camera.set_z_range(1., 100.).unwrap();
}
fn probe_endpoints(
    live: &Graphic3dCamera,
    flags: i32,
) -> (Handle<Graphic3dCamera>, Handle<Graphic3dCamera>) {
    let start = Handle::new(Graphic3dCamera::default());
    start.copy(live);
    let end = Handle::new(Graphic3dCamera::default());
    end.copy(live);
    end.set_eye(&Point3::new(8., 4., 20.)).unwrap();
    end.set_center(&Point3::new(2., -1., 0.)).unwrap();
    end.set_up(&Direction3::new(1., 1., 0.).unwrap());
    end.set_scale(if flags & 16 != 0 { 60. } else { 40. });
    if flags & 64 != 0 {
        end.set_projection_type(Graphic3dCameraProjection::Perspective);
    }
    (start, end)
}
fn probe_atom(
    view: &Handle<V3dView>,
    start: &Handle<Graphic3dCamera>,
    end: &Handle<Graphic3dCamera>,
    flags: i32,
) -> AisAnimationCamera {
    let target = if flags & 1 != 0 {
        Handle::default()
    } else {
        view.clone()
    };
    let animation = AisAnimationCamera::new("camera", &target);
    animation.set_camera_start(&if flags & 2 != 0 {
        Handle::default()
    } else {
        start.clone()
    });
    animation.set_camera_end(&if flags & 4 != 0 {
        Handle::default()
    } else {
        end.clone()
    });
    animation
}
fn camera_probe_state(probe: &CameraAnimationProbe) {
    flag(probe.animation.is_stopped());
    flag(probe.view.camera() == probe.live);
    flag(probe.animation.camera_start() == probe.start);
    flag(probe.animation.camera_end() == probe.end);
    flag(probe.view.set_immediate_update(false));
    pnt(probe.live.eye());
    pnt(probe.live.center());
    dir(probe.live.up());
    real(probe.live.scale());
    int(probe.live.projection_type() as i32);
}
