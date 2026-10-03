//! Face-only mode reset retains the native shared wire configuration.
use rustycad_tkshhealing::shape_fix::ShapeFixFace;
fn modes(tool: &ShapeFixFace) -> Vec<i32> {
    vec![
        tool.fix_wire_mode(),
        tool.fix_orientation_mode(),
        tool.fix_add_natural_bound_mode(),
        tool.fix_missing_seam_mode(),
        tool.fix_small_area_wire_mode(),
        tool.remove_small_area_face_mode(),
        tool.fix_intersecting_wires_mode(),
        tool.fix_loop_wires_mode(),
        tool.fix_split_face_mode(),
        tool.auto_correct_precision_mode(),
        tool.fix_periodic_degenerated_mode(),
        i32::from(tool.fix_wire_tool().topology_mode()),
    ]
}
fn assert_modes(name: &str, tool: &ShapeFixFace) {
    let expected = include_str!("../../../tools/oracle/tkshhealing_face_modes.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|token| token.parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(modes(tool), expected, "{name}");
}
fn change_modes(tool: &ShapeFixFace) {
    tool.set_fix_wire_mode(3);
    tool.set_fix_orientation_mode(3);
    tool.set_fix_add_natural_bound_mode(3);
    tool.set_fix_missing_seam_mode(3);
    tool.set_fix_small_area_wire_mode(3);
    tool.set_remove_small_area_face_mode(3);
    tool.set_fix_intersecting_wires_mode(3);
    tool.set_fix_loop_wires_mode(3);
    tool.set_fix_split_face_mode(3);
    tool.set_auto_correct_precision_mode(3);
    tool.set_fix_periodic_degenerated_mode(3);
    tool.fix_wire_tool().set_topology_mode(true);
}
#[test]
fn native_mode_defaults_mutation_and_reset() {
    let tool = ShapeFixFace::default();
    assert_modes("default", &tool);
    change_modes(&tool);
    assert_modes("changed", &tool);
    tool.clear_modes();
    assert_modes("cleared", &tool);
}
