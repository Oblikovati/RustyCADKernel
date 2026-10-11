use super::{AisSelectionScheme, AisViewInputBuffer, AisViewSelectionTool};
use crate::v3d::V3dTypeOfOrientation;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkservice::aspect::AspectScrollDelta;

const SELECTION_TOOLS: [AisViewSelectionTool; 4] = [
    AisViewSelectionTool::Picking,
    AisViewSelectionTool::RubberBand,
    AisViewSelectionTool::Polygon,
    AisViewSelectionTool::ZoomWindow,
];
const SELECTION_SCHEMES: [AisSelectionScheme; 7] = [
    AisSelectionScheme::Unknown,
    AisSelectionScheme::Replace,
    AisSelectionScheme::Add,
    AisSelectionScheme::Remove,
    AisSelectionScheme::Xor,
    AisSelectionScheme::Clear,
    AisSelectionScheme::ReplaceExtra,
];

fn request_flag(index: i32, side: i32, bit: i32) -> bool {
    let pattern = index * if side == 0 { 73 } else { 151 } + side * 37;
    (pattern >> bit) & 1 != 0
}

pub(super) fn seed_queue(b: &mut AisViewInputBuffer, index: i32, side: i32) {
    let origin = index * 100 + side * 1_000_000;
    b.is_new_gesture = request_flag(index, side, 0);
    b.orientation.to_fit_all = request_flag(index, side, 1);
    b.orientation.to_set_view_orient = request_flag(index, side, 2);
    b.orientation.view_orient = V3dTypeOfOrientation::ALL[((index + side * 7) % 26) as usize];
    b.move_to.to_hilight = request_flag(index, side, 3);
    b.move_to.point = Vec2::new(origin + 1, origin + 2);
    b.panning.to_start = request_flag(index, side, 5);
    b.panning.to_pan = request_flag(index, side, 6);
    b.panning.point_start = Vec2::new(origin + 3, origin + 4);
    b.panning.delta = Vec2::new(origin + 5, origin + 6);
    seed_drag_queue(b, if side == 0 { index % 32 } else { index / 32 }, origin);
    seed_rotation_queues(b, index, side, origin);
    seed_selection_zoom(b, index, side, origin);
}

fn seed_drag_queue(b: &mut AisViewInputBuffer, mask: i32, origin: i32) {
    b.dragging.to_start = mask & 1 != 0;
    b.dragging.to_confirm = mask & 2 != 0;
    b.dragging.to_move = mask & 4 != 0;
    b.dragging.to_stop = mask & 8 != 0;
    b.dragging.to_abort = mask & 16 != 0;
    b.dragging.point_start = Vec2::new(origin + 9, origin + 10);
    b.dragging.point_to = Vec2::new(origin + 11, origin + 12);
}

fn seed_rotation_queues(b: &mut AisViewInputBuffer, index: i32, side: i32, origin: i32) {
    let origin = f64::from(origin);
    b.orbit_rotation.to_start = request_flag(index, side, 7);
    b.orbit_rotation.to_rotate = request_flag(index, side, 8);
    b.orbit_rotation.point_start = Vec2::new(origin + 13.25, origin + 14.5);
    b.orbit_rotation.point_to = Vec2::new(origin + 15.75, origin + 16.125);
    b.view_rotation.to_start = request_flag(index, side, 9);
    b.view_rotation.to_rotate = request_flag(index, side, 10);
    b.view_rotation.point_start = Vec2::new(origin + 17.25, origin + 18.5);
    b.view_rotation.point_to = Vec2::new(origin + 19.75, origin + 20.125);
    b.z_rotate.to_rotate = request_flag(index, side, 11);
    b.z_rotate.point = Vec2::new(origin as i32 + 21, origin as i32 + 22);
    b.z_rotate.angle = origin + 23.5;
}

fn seed_selection_zoom(b: &mut AisViewInputBuffer, index: i32, side: i32, origin: i32) {
    b.selection.tool = SELECTION_TOOLS[((index + side) % 4) as usize];
    b.selection.scheme = SELECTION_SCHEMES[((index + side * 3) % 7) as usize];
    b.selection.to_apply_tool = request_flag(index, side, 4);
    for point in 0..2 + side {
        b.selection
            .points
            .append(Vec2::new(origin + 30 + point * 2, origin + 31 + point * 2));
    }
    seed_zoom_actions(b, side, origin);
}

fn seed_zoom_actions(b: &mut AisViewInputBuffer, side: i32, origin: i32) {
    for zoom in 0..2 - side {
        b.zoom_actions.append(AspectScrollDelta::new(
            Vec2::new(origin + 40 + zoom * 2, origin + 41 + zoom * 2),
            f64::from(origin) + 42.25 + f64::from(zoom),
            (256 * (zoom + side + 1)) as u32,
        ));
    }
}

pub(super) fn queue_snapshot(b: &AisViewInputBuffer) -> Vec<f64> {
    let mut values = vec![
        f64::from(b.is_new_gesture),
        f64::from(b.orientation.to_fit_all),
        f64::from(b.orientation.to_set_view_orient),
        f64::from(b.orientation.view_orient as i32),
        f64::from(b.move_to.to_hilight),
    ];
    append_pixel(&mut values, b.move_to.point);
    append_selection(&mut values, b);
    append_pan(&mut values, b);
    append_drag(&mut values, b);
    append_rotation(&mut values, b);
    append_zoom(&mut values, b);
    values
}

fn append_pan(values: &mut Vec<f64>, b: &AisViewInputBuffer) {
    values.push(f64::from(b.panning.to_start));
    append_pixel(values, b.panning.point_start);
    values.push(f64::from(b.panning.to_pan));
    append_pixel(values, b.panning.delta);
}

fn append_pixel(values: &mut Vec<f64>, point: Vec2<i32>) {
    values.extend([f64::from(point.x), f64::from(point.y)]);
}

fn append_selection(values: &mut Vec<f64>, b: &AisViewInputBuffer) {
    values.extend([
        f64::from(b.selection.tool as i32),
        f64::from(b.selection.scheme as i32),
        f64::from(b.selection.to_apply_tool),
        b.selection.points.len() as f64,
    ]);
    for point in 1..=3 {
        append_pixel(
            values,
            b.selection.points.value(point).copied().unwrap_or_default(),
        );
    }
}

fn append_drag(values: &mut Vec<f64>, b: &AisViewInputBuffer) {
    let drag = &b.dragging;
    values.extend([
        f64::from(drag.to_start),
        f64::from(drag.to_confirm),
        f64::from(drag.to_move),
        f64::from(drag.to_stop),
        f64::from(drag.to_abort),
    ]);
    append_pixel(values, drag.point_start);
    append_pixel(values, drag.point_to);
}

fn append_rotation(values: &mut Vec<f64>, b: &AisViewInputBuffer) {
    append_orbit_rotation(values, b);
    append_view_rotation(values, b);
    values.push(f64::from(b.z_rotate.to_rotate));
    append_pixel(values, b.z_rotate.point);
    values.push(b.z_rotate.angle);
}

fn append_orbit_rotation(values: &mut Vec<f64>, b: &AisViewInputBuffer) {
    values.extend([
        f64::from(b.orbit_rotation.to_start),
        b.orbit_rotation.point_start.x,
        b.orbit_rotation.point_start.y,
        f64::from(b.orbit_rotation.to_rotate),
        b.orbit_rotation.point_to.x,
        b.orbit_rotation.point_to.y,
    ]);
}

fn append_view_rotation(values: &mut Vec<f64>, b: &AisViewInputBuffer) {
    values.extend([
        f64::from(b.view_rotation.to_start),
        b.view_rotation.point_start.x,
        b.view_rotation.point_start.y,
        f64::from(b.view_rotation.to_rotate),
        b.view_rotation.point_to.x,
        b.view_rotation.point_to.y,
    ]);
}

fn append_zoom(values: &mut Vec<f64>, b: &AisViewInputBuffer) {
    values.push(b.zoom_actions.len() as f64);
    for index in 1..=2 {
        let action = b.zoom_actions.value(index);
        if let Ok(action) = action {
            append_pixel(values, action.point);
            values.extend([action.delta, f64::from(action.flags)]);
            continue;
        }
        values.extend([0.0; 4]);
    }
}
