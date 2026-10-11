use super::view_controller_transfer_fixture::{queue_snapshot, seed_queue};
use super::{AisViewController, AisViewSelectionTool};
use rustycad_tkernel::ncollection::{BaseAllocator, Sequence, Vec2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::aspect::AspectScrollDelta;

#[test]
fn all_drag_flag_pairs_and_three_frame_queue_lifecycles_match_native() {
    for fixture in transfer_native_chunks() {
        replay_transfer_chunk(fixture);
    }
}

fn replay_transfer_chunk(fixture: &str) {
    let mut controller = AisViewController::new();
    for line in fixture.lines() {
        let expected: Vec<f64> = line
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        replay_transfer_frame(&mut controller, &expected);
    }
}

fn replay_transfer_frame(c: &mut AisViewController, expected: &[f64]) {
    if expected[1] == 0.0 {
        *c = AisViewController::new();
        seed_queue(&mut c.ui_input, expected[0] as i32, 0);
        seed_queue(&mut c.render_input, expected[0] as i32, 1);
    }
    c.set_ask_next_frame(true);
    c.flush_buffers(&Handle::default(), &Handle::default());
    let mut actual = vec![expected[0], expected[1], f64::from(c.to_ask_next_frame())];
    actual.extend(queue_snapshot(&c.ui_input));
    actual.extend(queue_snapshot(&c.render_input));
    assert_eq!(
        actual, expected,
        "native case {} frame {}",
        expected[0], expected[1]
    );
}

fn transfer_native_chunks() -> [&'static str; 7] {
    [
        include_str!("../../tests/fixtures/view-controller-transfer-0-native.txt"),
        include_str!("../../tests/fixtures/view-controller-transfer-1-native.txt"),
        include_str!("../../tests/fixtures/view-controller-transfer-2-native.txt"),
        include_str!("../../tests/fixtures/view-controller-transfer-3-native.txt"),
        include_str!("../../tests/fixtures/view-controller-transfer-4-native.txt"),
        include_str!("../../tests/fixtures/view-controller-transfer-5-native.txt"),
        include_str!("../../tests/fixtures/view-controller-transfer-6-native.txt"),
    ]
}

#[test]
fn transfer_preserves_allocator_identity_and_independent_selection_storage() {
    let mut controller = AisViewController::new();
    let ui_arena = BaseAllocator::incremental(1024);
    let render_arena = BaseAllocator::incremental(1024);
    seed_allocator_queues(&mut controller, &ui_arena, &render_arena);
    controller.flush_buffers(&Handle::default(), &Handle::default());
    assert_queue_allocators(&controller, &ui_arena, &render_arena);
    assert_eq!(controller.ui_input.zoom_actions.len(), 0);
    assert_eq!(controller.render_input.zoom_actions.len(), 1);
    controller.ui_input.selection.points.clear(None);
    assert_eq!(
        controller.render_input.selection.points.first(),
        Some(&Vec2::new(3, 7))
    );
    controller.flush_buffers(&Handle::default(), &Handle::default());
    assert!(controller.render_input.zoom_actions.is_empty());
    assert!(controller.render_input.selection.points.is_empty());
}

fn seed_allocator_queues(c: &mut AisViewController, ui: &BaseAllocator, render: &BaseAllocator) {
    c.ui_input.zoom_actions = Sequence::with_allocator(ui.clone());
    c.ui_input.selection.points = Sequence::with_allocator(ui.clone());
    c.render_input.zoom_actions = Sequence::with_allocator(render.clone());
    c.render_input.selection.points = Sequence::with_allocator(render.clone());
    c.ui_input.selection.tool = AisViewSelectionTool::Polygon;
    c.ui_input.selection.points.append(Vec2::new(3, 7));
    c.ui_input
        .zoom_actions
        .append(AspectScrollDelta::with_delta(2.0, 0));
    c.render_input
        .zoom_actions
        .append(AspectScrollDelta::with_delta(9.0, 0));
    c.render_input.selection.points.append(Vec2::new(9, 11));
}

fn assert_queue_allocators(c: &AisViewController, ui: &BaseAllocator, render: &BaseAllocator) {
    assert_eq!(c.ui_input.zoom_actions.allocator(), ui);
    assert_eq!(c.ui_input.selection.points.allocator(), ui);
    assert_eq!(c.render_input.zoom_actions.allocator(), render);
    assert_eq!(c.render_input.selection.points.allocator(), render);
}
