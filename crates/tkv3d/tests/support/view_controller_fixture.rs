//! Native controller snapshots are numeric and independent of OCCT at test time.
use rustycad_tkv3d::ais::*;
pub fn assert_snapshot(actual: &[f64], name: &str) {
    let row = include_str!("../fixtures/view-controller-events-native.txt")
        .lines()
        .find(|row| row.split_whitespace().next() == Some(name))
        .expect(name);
    let expected: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(actual.len(), expected.len(), "snapshot {name} shape");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(*actual, expected, "snapshot {name}, field {index}");
    }
}
pub fn parameters(c: &AisViewController) -> Vec<f64> {
    let mut values = policy_flags(c);
    values.extend(policy_numbers(c));
    values
}
fn policy_flags(c: &AisViewController) -> Vec<f64> {
    [
        c.to_pause_objects_animation() as u8 as f64,
        c.is_continuous_redraw() as u8 as f64,
        c.to_show_pan_anchor_point() as u8 as f64,
        c.to_show_rotate_center() as u8 as f64,
        c.to_lock_orbit_z_up() as u8 as f64,
        c.to_allow_touch_z_rotation() as u8 as f64,
        c.to_allow_rotation() as u8 as f64,
        c.to_allow_panning() as u8 as f64,
        c.to_allow_zooming() as u8 as f64,
    ]
    .into_iter()
    .chain(other_policy_flags(c))
    .collect()
}
fn other_policy_flags(c: &AisViewController) -> Vec<f64> {
    vec![
        c.to_allow_z_focus() as u8 as f64,
        c.to_allow_highlight() as u8 as f64,
        c.to_allow_dragging() as u8 as f64,
        c.to_stick_to_ray_on_zoom() as u8 as f64,
        c.to_stick_to_ray_on_rotation() as u8 as f64,
        c.to_invert_pitch() as u8 as f64,
        c.to_display_xr_aux_devices() as u8 as f64,
        c.to_display_xr_hands() as u8 as f64,
    ]
}
fn policy_numbers(c: &AisViewController) -> Vec<f64> {
    vec![
        c.mouse_acceleration() as f64,
        c.orbit_acceleration() as f64,
        c.walk_speed_absolute() as f64,
        c.walk_speed_relative() as f64,
        c.thrust_speed() as f64,
        c.touch_tolerance_scale() as f64,
        c.mouse_double_click_interval(),
        c.min_zoom_distance(),
        c.rotation_mode() as i32 as f64,
    ]
}
pub fn event(c: &AisViewController, role: AisViewInputBufferType, result: i32) -> Vec<f64> {
    let b = c.input_buffer(role);
    let mut values = selection_values(b, result);
    values.push(b.zoom_actions.len() as f64);
    values.extend(
        b.zoom_actions
            .iter()
            .flat_map(|z| [z.point.x as f64, z.point.y as f64, z.delta, z.flags as f64]),
    );
    values.extend(rotation(c, b));
    values
}
fn selection_values(b: &AisViewInputBuffer, result: i32) -> Vec<f64> {
    let mut values = vec![
        result as f64,
        b.selection.tool as i32 as f64,
        b.selection.scheme as i32 as f64,
        b.selection.to_apply_tool as u8 as f64,
        b.selection.points.len() as f64,
    ];
    values.extend(
        b.selection
            .points
            .iter()
            .flat_map(|p| [p.x as f64, p.y as f64]),
    );
    values
}
fn rotation(c: &AisViewController, b: &AisViewInputBuffer) -> [f64; 10] {
    [
        b.z_rotate.to_rotate as u8 as f64,
        b.z_rotate.angle,
        b.orbit_rotation.to_start as u8 as f64,
        b.orbit_rotation.to_rotate as u8 as f64,
        b.view_rotation.to_start as u8 as f64,
        b.view_rotation.to_rotate as u8 as f64,
        c.navigation_mode() as i32 as f64,
        c.has_previous_move_to() as u8 as f64,
        c.previous_move_to().x as f64,
        c.previous_move_to().y as f64,
    ]
}
pub fn assert_event(c: &AisViewController, name: &str, result: i32) {
    assert_snapshot(&event(c, AisViewInputBufferType::Ui, result), name);
}
