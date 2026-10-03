//! Native `ShapeFix_Wire::Perform` behavior on unloaded and free wires.
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::{shape_extend::ShapeExtendStatus, shape_fix::ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};

const NATIVE: &str = include_str!("../../../tools/oracle/tkshhealing_fix_wire_perform.expected");

#[test]
fn unloaded_and_free_wire_perform_match_native_statuses() {
    assert_native_row("unloaded", observe_unloaded());
    assert_native_row("ordered", observe_free_wire(false));
    assert_native_row("reversed_order", observe_free_wire(true));
}

fn observe_unloaded() -> Vec<i32> {
    let fixer = ShapeFixWire::new();
    let fixed = fixer.perform().unwrap();
    vec![
        i32::from(fixed),
        0,
        i32::from(fixer.last_fix_status(ShapeExtendStatus::Ok)),
    ]
}

fn observe_free_wire(reverse: bool) -> Vec<i32> {
    let first = edge(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.));
    let second = edge(Point3::new(1., 0., 0.), Point3::new(2., 0., 0.));
    let mut maker = BRepBuilderAPIMakeWire::new();
    for item in if reverse {
        [&second, &first]
    } else {
        [&first, &second]
    } {
        maker.add_edge(item).unwrap();
    }
    let wire = maker.wire().unwrap();
    let fixer = ShapeFixWire::new();
    fixer.load(&wire).unwrap();
    let fixed = fixer.perform().unwrap();
    observe_wire_result(&fixer, fixed)
}

fn edge(first: Point3, second: Point3) -> rustycad_tkbrep::topods::Edge {
    BRepBuilderAPIMakeEdge::from_points(first, second)
        .unwrap()
        .edge()
        .unwrap()
}

fn observe_wire_result(fixer: &ShapeFixWire, fixed: bool) -> Vec<i32> {
    use ShapeExtendStatus::{Done, Ok};
    vec![
        i32::from(fixed),
        fixer.wire_data().unwrap().nb_edges(),
        i32::from(fixer.status_reorder(Ok)),
        i32::from(fixer.status_reorder(Done)),
        i32::from(fixer.status_connected(Ok)),
        i32::from(fixer.status_connected(Done)),
        i32::from(fixer.status_edge_curves(Ok)),
        i32::from(fixer.status_edge_curves(Done)),
        i32::from(fixer.status_degenerated(Ok)),
        i32::from(fixer.status_degenerated(Done)),
        i32::from(fixer.status_self_intersection(Ok)),
        i32::from(fixer.status_self_intersection(Done)),
        i32::from(fixer.status_lacking(Ok)),
        i32::from(fixer.status_lacking(Done)),
        i32::from(fixer.status_notches(Ok)),
        i32::from(fixer.status_notches(Done)),
        i32::from(fixer.status_fix_tails(Ok)),
        i32::from(fixer.status_fix_tails(Done)),
        i32::from(fixer.last_fix_status(Ok)),
    ]
}

fn assert_native_row(name: &str, actual: Vec<i32>) {
    let native = NATIVE
        .lines()
        .find(|line| line.starts_with(name))
        .unwrap_or_else(|| panic!("missing native row {name}"));
    let expected = native
        .split_whitespace()
        .skip(2)
        .map(|field| field.split_once(' ').map_or(field, |(_, value)| value))
        .filter_map(|field| field.parse::<i32>().ok())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "native Perform row {name}");
}
