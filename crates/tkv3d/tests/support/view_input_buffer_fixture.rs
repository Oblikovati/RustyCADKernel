//! Native fixture coordinates are independent from the production reset implementation.
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkservice::aspect::{
    ASPECT_VKEY_FLAGS_CTRL, ASPECT_VKEY_FLAGS_SHIFT, AspectScrollDelta,
};
use rustycad_tkv3d::{ais::*, v3d::V3dTypeOfOrientation};
/// Read one committed native record; `native_values("reset")`.
pub fn native_values(case: &str) -> Vec<f64> {
    let line = include_str!("../fixtures/view-input-buffer-native.txt")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(case))
        .unwrap_or_else(|| {
            panic!("received fixture case {case:?}; expected a committed native row")
        });
    line.split_whitespace()
        .skip(1)
        .map(|number| {
            number.parse().unwrap_or_else(|_| {
                panic!("received native fixture value {number:?}; expected f64")
            })
        })
        .collect()
}
/// Set all requests and nonzero persistent coordinates; `seeded().reset()`.
pub fn seeded() -> AisViewInputBuffer {
    let mut events = AisViewInputBuffer::new();
    seed_requests(&mut events);
    seed_motion_positions(&mut events);
    seed_sequences(&mut events);
    events.orientation.view_orient = V3dTypeOfOrientation::Zneg;
    events.selection.tool = AisViewSelectionTool::Polygon;
    events.selection.scheme = AisSelectionScheme::Xor;
    events
}
fn seed_requests(b: &mut AisViewInputBuffer) {
    b.is_new_gesture = true;
    b.orientation.to_fit_all = true;
    b.orientation.to_set_view_orient = true;
    b.move_to.to_hilight = true;
    b.selection.to_apply_tool = true;
    b.panning.to_start = true;
    b.panning.to_pan = true;
    b.dragging.to_start = true;
    b.dragging.to_confirm = true;
    b.dragging.to_move = true;
    b.dragging.to_stop = true;
    b.dragging.to_abort = true;
    b.orbit_rotation.to_start = true;
    b.orbit_rotation.to_rotate = true;
    b.view_rotation.to_start = true;
    b.view_rotation.to_rotate = true;
    b.z_rotate.to_rotate = true;
}
fn seed_motion_positions(b: &mut AisViewInputBuffer) {
    b.move_to.point = Vec2::new(17, 19);
    b.panning.point_start = Vec2::new(23, -29);
    b.panning.delta = Vec2::new(-3, 7);
    b.dragging.point_start = Vec2::new(31, 37);
    b.dragging.point_to = Vec2::new(41, 43);
    b.orbit_rotation.point_start = Vec2::new(0.25, -0.5);
    b.orbit_rotation.point_to = Vec2::new(1.5, 2.25);
    b.view_rotation.point_start = Vec2::new(-0.75, 3.5);
    b.view_rotation.point_to = Vec2::new(4.25, -5.5);
    b.z_rotate.point = Vec2::new(47, 53);
    b.z_rotate.angle = 0.75;
}
fn seed_sequences(b: &mut AisViewInputBuffer) {
    b.selection.points.append(Vec2::new(27, -14));
    b.selection.points.append(Vec2::new(61, 67));
    b.zoom_actions.append(AspectScrollDelta::new(
        Vec2::new(71, 73),
        0.5,
        ASPECT_VKEY_FLAGS_SHIFT,
    ));
    b.zoom_actions.append(AspectScrollDelta::new(
        Vec2::new(79, 83),
        -1.25,
        ASPECT_VKEY_FLAGS_CTRL,
    ));
}
/// Serialize only public event state in native column order; `snapshot(&seeded())`.
pub fn snapshot(b: &AisViewInputBuffer) -> Vec<f64> {
    let mut values: Vec<f64> = request_flags(b)
        .map(|flag| f64::from(u8::from(flag)))
        .to_vec();
    values.extend(
        [
            b.orientation.view_orient as i32,
            b.selection.tool as i32,
            b.selection.scheme as i32,
        ]
        .map(f64::from),
    );
    append_motion_positions(&mut values, b);
    values.push(b.z_rotate.angle);
    append_sequences(&mut values, b);
    values
}
fn request_flags(b: &AisViewInputBuffer) -> [bool; 17] {
    [
        b.is_new_gesture,
        b.orientation.to_fit_all,
        b.orientation.to_set_view_orient,
        b.move_to.to_hilight,
        b.selection.to_apply_tool,
        b.panning.to_start,
        b.panning.to_pan,
        b.dragging.to_start,
        b.dragging.to_confirm,
        b.dragging.to_move,
        b.dragging.to_stop,
        b.dragging.to_abort,
        b.orbit_rotation.to_start,
        b.orbit_rotation.to_rotate,
        b.view_rotation.to_start,
        b.view_rotation.to_rotate,
        b.z_rotate.to_rotate,
    ]
}
fn pair_values<T: Copy + Into<f64>>(point: Vec2<T>) -> [f64; 2] {
    [point.x.into(), point.y.into()]
}
fn append_motion_positions(values: &mut Vec<f64>, b: &AisViewInputBuffer) {
    values.extend(pair_values(b.move_to.point));
    values.extend(pair_values(b.panning.point_start));
    values.extend(pair_values(b.panning.delta));
    values.extend(pair_values(b.dragging.point_start));
    values.extend(pair_values(b.dragging.point_to));
    values.extend(pair_values(b.orbit_rotation.point_start));
    values.extend(pair_values(b.orbit_rotation.point_to));
    values.extend(pair_values(b.view_rotation.point_start));
    values.extend(pair_values(b.view_rotation.point_to));
    values.extend(pair_values(b.z_rotate.point));
}
fn append_sequences(values: &mut Vec<f64>, b: &AisViewInputBuffer) {
    values.push(b.selection.points.len() as f64);
    for point in b.selection.points.iter() {
        values.extend(pair_values(*point));
    }
    values.push(b.zoom_actions.len() as f64);
    for zoom in b.zoom_actions.iter() {
        values.extend(pair_values(zoom.point));
        values.extend([zoom.delta, f64::from(zoom.flags)]);
    }
}
