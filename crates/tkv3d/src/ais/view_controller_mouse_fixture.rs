//! Frozen local-oracle scenarios; Rust tests never link or invoke OCCT.
use super::view_controller_keyboard_fixture::prepare;
use super::*;
use rustycad_tkernel::ncollection::Vec2;
pub(super) fn mouse_case(index: usize) -> AisViewController {
    let (mut c, _) = prepare(0.0, 0.0);
    if index >= 100 {
        seed_mouse_counter(&mut c, index);
        return c;
    }
    let scales = [1.0, 0.0, -1.0, 2.0, f32::INFINITY, f32::NAN];
    c.set_touch_tolerance_scale(scales[(index / 10) % 6]);
    c.change_mouse_gesture_map().clear(true);
    c.change_mouse_gesture_map().bind(
        rustycad_tkservice::aspect::ASPECT_VKEY_MOUSE_LEFT_BUTTON,
        gestures()[index % 10],
    );
    configure_mouse_policies(&mut c, index);
    c
}
pub(super) fn gestures() -> [AisMouseGesture; 10] {
    use AisMouseGesture::*;
    [
        None,
        SelectRectangle,
        SelectLasso,
        Zoom,
        ZoomVertical,
        ZoomWindow,
        Pan,
        RotateOrbit,
        RotateView,
        Drag,
    ]
}
pub(super) fn mouse_action(c: &mut AisViewController, row: &[f64]) -> bool {
    let p = Vec2::new(row[3] as i32, row[4] as i32);
    let b = row[5] as u32;
    let m = row[6] as u32;
    let e = row[7] != 0.0;
    match row[2] as i32 {
        0 => c.update_mouse_buttons(p, b, m, e).unwrap(),
        1 => c.update_mouse_position(p, b, m, e).unwrap(),
        op => panic!("mouse operation {op}; expected 0 or 1"),
    }
}
pub(super) fn mouse_snapshot(c: &AisViewController, result: bool) -> Vec<f64> {
    let mut v = vec![result as i32 as f64];
    append_mouse_state(c, &mut v);
    append_mouse_requests(c, &mut v);
    append_mouse_selection(c, &mut v);
    append_mouse_motion(c, &mut v);
    v
}
fn append_mouse_state(c: &AisViewController, v: &mut Vec<f64>) {
    let m = &c.mouse_dispatch;
    v.extend([
        c.click_state.active_gesture as i32 as f64,
        c.click_state.single_button as f64,
        c.click_state.counter as f64,
        c.click_state.timer.is_started() as i32 as f64,
        c.input_base.mouse_pressed as f64,
        c.input_base.mouse_modifiers as f64,
        m.press_point.x as f64,
        m.press_point.y as f64,
        m.progress_point.x as f64,
        m.progress_point.y as f64,
        c.input_base.mouse_position_last.x as f64,
        c.input_base.mouse_position_last.y as f64,
    ]);
    append_mouse_flags(c, v);
}
fn append_mouse_flags(c: &AisViewController, v: &mut Vec<f64>) {
    let m = &c.mouse_dispatch;
    v.extend([
        m.idle_rotation as i32 as f64,
        m.stop_drag_on_unclick as i32 as f64,
        c.touch_dispatch.update_start_rotation as i32 as f64,
        c.touch_dispatch.update_start_pan as i32 as f64,
        c.ui_input.is_new_gesture as i32 as f64,
    ]);
}
fn append_mouse_requests(c: &AisViewController, v: &mut Vec<f64>) {
    let d = &c.ui_input.dragging;
    v.extend([
        d.to_start as i32 as f64,
        d.to_confirm as i32 as f64,
        d.to_move as i32 as f64,
        d.to_stop as i32 as f64,
        d.to_abort as i32 as f64,
        d.point_start.x as f64,
        d.point_start.y as f64,
        d.point_to.x as f64,
        d.point_to.y as f64,
        c.ui_input.move_to.to_hilight as i32 as f64,
        c.ui_input.move_to.point.x as f64,
        c.ui_input.move_to.point.y as f64,
    ]);
}
fn append_mouse_selection(c: &AisViewController, v: &mut Vec<f64>) {
    let s = &c.ui_input.selection;
    v.extend([
        s.tool as i32 as f64,
        s.scheme as i32 as f64,
        s.to_apply_tool as i32 as f64,
        s.points.len() as f64,
    ]);
    for index in 1..=3 {
        let p = if index <= s.points.len() {
            *s.points.value(index).unwrap()
        } else {
            Vec2::new(0, 0)
        };
        v.extend([p.x as f64, p.y as f64]);
    }
}
fn append_mouse_motion(c: &AisViewController, v: &mut Vec<f64>) {
    let p = &c.ui_input.panning;
    v.extend([
        p.to_start as i32 as f64,
        p.point_start.x as f64,
        p.point_start.y as f64,
        p.to_pan as i32 as f64,
        p.delta.x as f64,
        p.delta.y as f64,
    ]);
    append_mouse_rotations(c, v);
    append_mouse_zoom(c, v);
}
fn append_mouse_rotations(c: &AisViewController, v: &mut Vec<f64>) {
    let o = &c.ui_input.orbit_rotation;
    v.extend([
        o.to_start as i32 as f64,
        o.point_start.x,
        o.point_start.y,
        o.to_rotate as i32 as f64,
        o.point_to.x,
        o.point_to.y,
    ]);
    let r = &c.ui_input.view_rotation;
    v.extend([
        r.to_start as i32 as f64,
        r.point_start.x,
        r.point_start.y,
        r.to_rotate as i32 as f64,
        r.point_to.x,
        r.point_to.y,
    ]);
}
fn append_mouse_zoom(c: &AisViewController, v: &mut Vec<f64>) {
    v.push(c.ui_input.zoom_actions.len() as f64);
    if let Some(z) = c.ui_input.zoom_actions.last() {
        v.extend([z.delta, z.point.x as f64, z.point.y as f64, z.flags as f64]);
    } else {
        v.extend([0.0; 4]);
    }
}

fn seed_mouse_counter(c: &mut AisViewController, index: usize) {
    if index >= 120 {
        c.click_state.counter = 1;
        c.click_state
            .timer
            .reset_to(if index.is_multiple_of(2) { 0.1 } else { 1.0 });
        c.click_state.timer.start();
    }
    if index == 124 {
        c.set_mouse_double_click_interval(-1.0);
    }
    if index == 125 {
        c.set_mouse_double_click_interval(f64::NAN);
    }
}

fn configure_mouse_policies(c: &mut AisViewController, index: usize) {
    c.set_allow_rotation(index < 60 || !index.is_multiple_of(4));
    c.set_allow_panning(index < 60 || index % 4 != 1);
    c.set_allow_zooming(index < 60 || index % 4 != 2);
    c.set_allow_dragging(index < 60 || index % 4 != 3);
    if index >= 80 {
        c.set_navigation_mode(AisNavigationMode::FirstPersonWalk);
    }
    c.set_orbit_acceleration(if index.is_multiple_of(2) { 1.5 } else { -0.5 });
    c.set_mouse_acceleration(2.0);
}
