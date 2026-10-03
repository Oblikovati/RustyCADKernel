//! Native `ShapeFix_Wire::FixTails` stage behavior and retained status.
mod support;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::{shape_extend::ShapeExtendStatus, shape_fix::ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};

const NATIVE: &str = include_str!("../../../tools/oracle/tkshhealing_fix_wire_tails.expected");

#[test]
fn no_match_tail_pairs_match_native_statuses() {
    assert_native("same_direction", observe_tails(false));
    assert_native("opposite_direction", observe_tails(true));
    assert_removal_native(observe_removable_tail());
}

fn observe_tails(reverse_second: bool) -> Vec<i32> {
    let first = edge(Point3::new(0., 0., 0.), Point3::new(2., 0., 0.));
    let mut second = edge(Point3::new(0., 0., 0.), Point3::new(2., 0., 0.));
    if reverse_second {
        second.reverse();
    }
    let mut maker = BRepBuilderAPIMakeWire::new();
    maker.add_edge(&first).unwrap();
    maker.add_edge(&second).unwrap();
    let wire = maker.wire().unwrap();
    let fixer =
        ShapeFixWire::from_wire_face(&wire, &support::surfaces::planar_face(), 1e-7).unwrap();
    fixer.set_max_tail_width(0.1);
    let fixed = fixer.fix_tails().unwrap();
    vec![
        i32::from(fixed),
        fixer.wire_data().unwrap().nb_edges(),
        i32::from(fixer.status_notches(ShapeExtendStatus::Ok)),
        i32::from(fixer.status_notches(ShapeExtendStatus::Done)),
        i32::from(fixer.status_fix_tails(ShapeExtendStatus::Ok)),
        i32::from(fixer.status_fix_tails(ShapeExtendStatus::Done)),
        i32::from(fixer.last_fix_status(ShapeExtendStatus::Ok)),
    ]
}

fn edge(first: Point3, last: Point3) -> rustycad_tkbrep::topods::Edge {
    BRepBuilderAPIMakeEdge::from_points(first, last)
        .unwrap()
        .edge()
        .unwrap()
}

fn observe_removable_tail() -> Vec<i32> {
    let vertices = [
        Point3::new(0., 0., 0.),
        Point3::new(2., 0., 0.),
        Point3::new(2., 1., 0.),
    ];
    let edges = [
        edge(vertices[0], vertices[1]),
        edge(vertices[1], vertices[2]),
        edge(vertices[2], vertices[1]),
        edge(vertices[1], vertices[0]),
    ];
    let mut maker = BRepBuilderAPIMakeWire::new();
    for edge in edges {
        maker.add_edge(&edge).unwrap();
    }
    let wire = maker.wire().unwrap();
    let fixer =
        ShapeFixWire::from_wire_face(&wire, &support::surfaces::planar_face(), 1e-7).unwrap();
    fixer.set_max_tail_width(0.1);
    let fixed = fixer.fix_tails().unwrap();
    vec![
        i32::from(fixed),
        fixer.wire_data().unwrap().nb_edges(),
        i32::from(fixer.status_notches(ShapeExtendStatus::Done)),
        i32::from(fixer.last_fix_status(ShapeExtendStatus::Done)),
    ]
}

fn assert_native(name: &str, actual: Vec<i32>) {
    assert_native_fields(
        name,
        actual,
        &[
            "fixed",
            "nb_edges",
            "notches_ok",
            "notches_done",
            "tails_ok",
            "tails_done",
            "last_ok",
        ],
    );
}

fn assert_removal_native(actual: Vec<i32>) {
    assert_native_fields(
        "removable_tail",
        actual,
        &["fixed", "nb_edges", "notches_done", "last_done"],
    );
}

fn assert_native_fields(name: &str, actual: Vec<i32>, fields: &[&str]) {
    let row = NATIVE
        .lines()
        .find(|row| row.starts_with(name))
        .unwrap_or_else(|| panic!("missing native row {name}"));
    let expected = parse_native_fields(row, fields);
    assert_eq!(actual, expected, "native FixTails row {name}");
}

fn parse_native_fields(row: &str, fields: &[&str]) -> Vec<i32> {
    let mut values = row.split_whitespace().skip(1);
    let mut selected = Vec::new();
    while let (Some(key), Some(value)) = (values.next(), values.next()) {
        if fields.contains(&key) {
            selected.push(value.parse().unwrap());
        }
    }
    selected
}
