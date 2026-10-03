//! Native endpoint orientation and state-retention rows for the bounding selector.
use rustycad_tkbrep::{brep::BRepTool, topods::Shape};
use rustycad_tkernel::{
    ncollection::{HArray1, UBTree},
    standard::Handle,
};
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};
use rustycad_tkshhealing::{
    shape_analysis::{ShapeAnalysis, ShapeAnalysisBoxBndTreeSelector as Selector},
    shape_extend::ShapeExtendStatus,
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};
fn wire(offset: f64) -> Shape {
    let edge = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(offset, 0., 0.),
        Point3::new(offset + 1., 0., 0.),
    )
    .unwrap()
    .edge()
    .unwrap();
    let mut maker = BRepBuilderAPIMakeWire::new();
    maker.add_edge(&edge).unwrap();
    maker.wire().unwrap().into_shape()
}
fn sequence() -> Handle<HArray1<Shape>> {
    let array = Handle::new(HArray1::<Shape>::with_bounds(1, 2).unwrap());
    array.change_array1().set_value(1, wire(0.)).unwrap();
    array.change_array1().set_value(2, wire(2.)).unwrap();
    array
}
fn expected(name: &str) -> Vec<i32> {
    include_str!("../../../tools/oracle/tkshhealing_box_selector.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|word| word.parse().unwrap())
        .collect()
}
fn row(selector: &Selector, accepted: bool) -> Vec<i32> {
    let mut row = vec![
        i32::from(accepted),
        selector.nb(),
        i32::from(selector.stop()),
    ];
    row.extend(
        [
            ShapeExtendStatus::Done1,
            ShapeExtendStatus::Done2,
            ShapeExtendStatus::Done3,
            ShapeExtendStatus::Done4,
            ShapeExtendStatus::Fail2,
            ShapeExtendStatus::Ok,
        ]
        .map(|status| i32::from(selector.last_check_status(status))),
    );
    row
}
fn check(selector: &mut Selector, index: i32, name: &str) {
    let accepted = selector.accept(index).unwrap();
    assert_eq!(row(selector, accepted), expected(name), "{name}");
}
fn orientation_case(shared: bool, mode: usize) -> Vec<i32> {
    let array = sequence();
    let (first, last) = ShapeAnalysis::find_bounds(array.array1().value(1).unwrap()).unwrap();
    let (far, _) = ShapeAnalysis::find_bounds(&wire(10.)).unwrap();
    let head = if mode < 2 {
        far.clone()
    } else if mode == 2 {
        last.clone()
    } else {
        first.clone()
    };
    let tail = if mode == 0 {
        first
    } else if mode == 1 {
        last
    } else {
        far
    };
    let mut selector = Selector::new(&array, shared);
    selector.define_pnt(BRepTool::pnt(&head).unwrap(), BRepTool::pnt(&tail).unwrap());
    selector.define_vertexes(head, tail);
    let accepted = selector.accept(1).unwrap();
    row(&selector, accepted)
}
#[test]
fn four_endpoint_orientations_match_both_native_modes() {
    for shared in [true, false] {
        for mode in 0..4 {
            let name = format!("{}{}", if shared { "shared" } else { "distance" }, mode + 1);
            assert_eq!(orientation_case(shared, mode), expected(&name));
        }
    }
}
#[test]
fn native_candidate_and_minimum_retention_matches() {
    let mut selector = Selector::new(&sequence(), false);
    selector.set_tolerance(10.);
    selector.define_pnt(Point3::new(2., 0., 0.), Point3::new(-0.5, 0., 0.));
    check(&mut selector, 1, "first");
    selector.set_stop();
    check(&mut selector, 2, "head_closer");
    selector.load_list(2);
    check(&mut selector, 2, "visited");
    let mut bounds = BoundingBox3::default();
    bounds.set_whole();
    selector.define_boxes(&bounds, &bounds);
    check(&mut selector, 1, "boxes_reset");
    selector.set_tolerance(10.);
    check(&mut selector, 1, "tolerance_reset");
}
#[test]
fn bounds_exclusions_and_invalid_index_precedence_are_preserved() {
    let array = sequence();
    let mut selector = Selector::new(&array, false);
    selector.set_nb(7);
    assert_eq!(selector.nb(), 7);
    selector.load_list(1);
    selector.load_list(3);
    assert!(selector.cont_wire(1));
    assert!(!selector.cont_wire(2));
    array
        .change_array1()
        .set_value(1, Shape::default())
        .unwrap();
    assert!(!selector.accept(1).unwrap());
    assert_eq!(
        selector.accept(3).unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    let bounds = BoundingBox3::new(Point3::default(), Point3::new(1., 1., 1.));
    assert!(selector.reject(&bounds));
    selector.define_boxes(&bounds, &BoundingBox3::default());
    assert!(!selector.reject(&bounds));
    let mut tree = UBTree::<i32, BoundingBox3>::default();
    tree.add(3, bounds);
    assert_eq!(
        tree.try_select(&mut selector).unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn rejected_shared_candidate_sets_failure_without_changing_selected_index() {
    let mut selector = Selector::new(&sequence(), true);
    let (first, last) = ShapeAnalysis::find_bounds(&wire(10.)).unwrap();
    selector.define_vertexes(first, last);
    selector.set_nb(9);
    check(&mut selector, 1, "shared_failure");
}

#[test]
fn selector_observes_caller_changes_to_the_shared_array() {
    let array = sequence();
    let mut selector = Selector::new(&array, false);
    array.change_array1().set_value(1, wire(10.)).unwrap();
    selector.define_pnt(Point3::new(30., 0., 0.), Point3::new(10., 0., 0.));
    assert!(selector.accept(1).unwrap());
    assert!(selector.last_check_status(ShapeExtendStatus::Done1));
    assert_eq!(selector.nb(), 1);
}
