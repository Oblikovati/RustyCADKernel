//! Local native fixtures exercise real V3d_View invalidation through a retained CPU driver.
use crate::v3d::V3dViewSource;
use super::{AisNavigationMode, AisViewController};
use crate::v3d::{V3dView, V3dViewer};
use rustycad_tkernel::{ncollection::Vec2, standard::Handle};
use rustycad_tkservice::{
    aspect::{AspectNeutralWindow, AspectTouch, AspectWindowMembers},
    null_driver::NullGraphicDriver,
};
const TOUCH_MODES: [AisNavigationMode; 3] = [
    AisNavigationMode::Orbit,
    AisNavigationMode::FirstPersonFlight,
    AisNavigationMode::FirstPersonWalk,
];
const TOUCH_SCALES: [f32; 6] = [1.0, 0.0, -1.0, 2.0, f32::INFINITY, f32::NAN];
const TOUCH_MOVES: [f64; 8] = [0.0, 4.0, 6.0, 6.0001, -8.0, 0.25, 3.9999, 4.0001];
pub(super) fn synthesis_view() -> (Handle<V3dViewer>, Handle<dyn V3dViewSource>) {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let view = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(640, 480);
    window.set_virtual(true);
    view.set_window(&window.cast(), 0).unwrap();
    (viewer, view)
}
pub(super) fn synthesis_case(index: usize) -> AisViewController {
    let mut c = AisViewController::new().unwrap();
    c.navigation_mode = TOUCH_MODES[(index / 5) % 3];
    c.set_touch_tolerance_scale(TOUCH_SCALES[(index / 60) % 6]);
    c.set_mouse_acceleration(if index.is_multiple_of(2) { 2.0 } else { -0.5 });
    c.set_orbit_acceleration(if index.is_multiple_of(3) { 0.0 } else { 1.5 });
    c.set_allow_touch_z_rotation(!(index / 15).is_multiple_of(2));
    c.set_allow_rotation(false);
    c.set_allow_panning(false);
    c.set_allow_zooming(false);
    seed_touch_starts(&mut c, index);
    seed_synthesis_contacts(&mut c, index);
    if index >= 360 {
        configure_touch_boundary(&mut c, index);
    }
    c
}
fn seed_synthesis_contacts(c: &mut AisViewController, index: usize) {
    for touch in 0..index % 5 {
        c.input_base.touch_points.add(
            touch + 1,
            AspectTouch::new(touch_origin(touch), ((index / 10) >> touch) & 1 != 0),
        );
    }
}

fn seed_touch_starts(c: &mut AisViewController, index: usize) {
    c.touch_dispatch.start_rotation = Vec2::new(-11.25, 12.5);
    c.touch_dispatch.start_pan = Vec2::new(-13.25, 14.5);
    c.touch_dispatch.update_start_rotation = (index / 30).is_multiple_of(2);
    c.touch_dispatch.update_start_pan = (index / 20).is_multiple_of(2);
    c.render_input.orbit_rotation.point_start = Vec2::new(31.25, -32.5);
    c.render_input.view_rotation.point_start = Vec2::new(33.25, -34.5);
    c.render_input.z_rotate.point = Vec2::new(35, -36);
    c.render_input.z_rotate.angle = -0.75;
}
fn touch_origin(touch: usize) -> Vec2<f64> {
    Vec2::new(-3.25 + touch as f64 * 20.0, 7.5 + touch as f64 * 3.0)
}
pub(super) fn advance_touch_case(c: &mut AisViewController, index: usize, frame: usize) {
    if index >= 360 {
        advance_touch_boundary(c, index);
        return;
    }
    for touch in 0..c.input_base.touch_points.extent() {
        let dx = TOUCH_MOVES[(index / 5 + frame) % 8];
        let dy = TOUCH_MOVES[(index / 40 + frame) % 8];
        let direction = if touch == 1 && index.is_multiple_of(2) {
            -1.0
        } else {
            1.0
        };
        c.input_base
            .touch_points
            .change_from_index(touch + 1)
            .unwrap()
            .to = touch_origin(touch) + Vec2::new(dx * direction, dy);
    }
}
fn configure_touch_boundary(c: &mut AisViewController, index: usize) {
    c.input_base.touch_points.clear(false);
    c.set_touch_tolerance_scale(1.0);
    c.set_allow_touch_z_rotation(true);
    c.touch_dispatch.update_start_rotation = false;
    c.touch_dispatch.update_start_pan = false;
    c.input_base
        .touch_points
        .add(1, AspectTouch::at(0.0, 0.0, index >= 368));
    if index < 368 {
        c.input_base
            .touch_points
            .add(2, AspectTouch::at(20.0, 0.0, false));
    }
}
fn advance_touch_boundary(c: &mut AisViewController, index: usize) {
    let point = if index < 368 {
        let x: [f64; 8] = [-20.0, 0.0, 1e-9, 0.0, 26.0, 26.0001, 20.0, 20.0];
        let y: [f64; 8] = [0.0, 0.0, 0.0, 20.0, 0.0, 0.0, 0.7, 0.8];
        Vec2::new(x[index - 360], y[index - 360])
    } else {
        let small = f64::MIN_POSITIVE;
        Vec2::new([small * 0.5, small, small * 2.0, 0.0][index - 368], 0.0)
    };
    c.input_base
        .touch_points
        .change_from_index(if index < 368 { 2 } else { 1 })
        .unwrap()
        .to = point;
}

pub(super) fn synthesis_snapshot(c: &AisViewController, view: &V3dView) -> Vec<f64> {
    let mut v = vec![
        f64::from(view.is_invalidated()),
        c.touch_dispatch.last_touch_count as f64,
        f64::from(c.touch_dispatch.update_start_rotation),
        f64::from(c.touch_dispatch.update_start_pan),
        c.input_base.touch_points.extent() as f64,
    ];
    append_synthesis_contacts(&mut v, c);
    append_synthesis_motion(&mut v, c);
    append_synthesis_zoom(&mut v, c);
    v
}
fn append_synthesis_motion(v: &mut Vec<f64>, c: &AisViewController) {
    let b = &c.render_input;
    v.extend([
        f64::from(b.is_new_gesture),
        f64::from(b.dragging.to_move),
        f64::from(b.dragging.point_to.x),
        f64::from(b.dragging.point_to.y),
    ]);
    append_synthesis_rotations(v, c);
    v.extend([
        f64::from(b.z_rotate.to_rotate),
        f64::from(b.z_rotate.point.x),
        f64::from(b.z_rotate.point.y),
        b.z_rotate.angle,
    ]);
    append_synthesis_pan(v, c);
}

fn append_synthesis_pan(v: &mut Vec<f64>, c: &AisViewController) {
    let b = &c.render_input;
    v.extend([
        f64::from(b.panning.to_start),
        f64::from(b.panning.point_start.x),
        f64::from(b.panning.point_start.y),
        f64::from(b.panning.to_pan),
        f64::from(b.panning.delta.x),
        f64::from(b.panning.delta.y),
    ]);
}

fn append_synthesis_contacts(v: &mut Vec<f64>, c: &AisViewController) {
    for index in 1..=4 {
        if let Ok(t) = c.input_base.touch_points.find_from_index(index) {
            v.extend([
                t.from.x,
                t.from.y,
                t.to.x,
                t.to.y,
                f64::from(t.is_precise_device),
            ]);
        } else {
            v.extend([0.0; 5]);
        }
    }
}
fn append_synthesis_rotations(v: &mut Vec<f64>, c: &AisViewController) {
    let orbit = &c.render_input.orbit_rotation;
    let view = &c.render_input.view_rotation;
    v.extend([
        f64::from(orbit.to_start),
        orbit.point_start.x,
        orbit.point_start.y,
        f64::from(orbit.to_rotate),
        orbit.point_to.x,
        orbit.point_to.y,
    ]);
    v.extend([
        f64::from(view.to_start),
        view.point_start.x,
        view.point_start.y,
        f64::from(view.to_rotate),
        view.point_to.x,
        view.point_to.y,
    ]);
}
fn append_synthesis_zoom(v: &mut Vec<f64>, c: &AisViewController) {
    v.push(c.render_input.zoom_actions.len() as f64);
    for index in 1..=3 {
        if let Ok(a) = c.render_input.zoom_actions.value(index) {
            v.extend([
                f64::from(a.point.x),
                f64::from(a.point.y),
                a.delta,
                f64::from(a.flags),
            ]);
        } else {
            v.extend([0.0; 4]);
        }
    }
}
