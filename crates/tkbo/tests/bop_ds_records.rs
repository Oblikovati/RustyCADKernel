//! Tests every scalar BOPDS record operation, including native ordering quirks.
use rustycad_tkbo::bop_ds::{IndexRange, Pair, Pave, Point};
use rustycad_tkmath::gp::{Point2, Point3};
#[test]
fn index_range_contains_inclusive_and_reversed_bounds() {
    let mut range = IndexRange::default();
    assert_eq!(range.indices(), (0, 0));
    range = IndexRange::new(1, 3);
    assert!(range.contains(1));
    assert!(range.contains(3));
    range.set_first(2);
    range.set_last(4);
    assert_eq!((range.first(), range.last()), (2, 4));
    range.set_indices(4, 2);
    assert!(!range.contains(3));
    let mut text = String::new();
    range.dump(&mut text).unwrap();
    assert_eq!(text, " range: 4 2");
}
#[test]
fn pave_keeps_vertex_and_edge_parameter() {
    let mut pave = Pave::default();
    assert_eq!(pave.contents(), (-1, 99.0));
    pave = Pave::new(2, 0.5);
    pave.set_index(3);
    pave.set_parameter(0.25);
    assert_eq!((pave.index(), pave.parameter()), (3, 0.25));
    assert!(pave.is_less(&Pave::new(0, 1.0)));
    assert!(pave.is_equal(&Pave::new(3, 0.25)));
    let mut text = String::new();
    pave.dump(&mut text).unwrap();
    assert_eq!(text, " { 3 0.250  }");
    assert_ne!(
        Pave::new(0, 0.0).hash_code(),
        Pave::new(0, -0.0).hash_code()
    );
}
#[test]
fn pair_equality_and_ordering_follow_different_native_rules() {
    let mut pair = Pair::default();
    assert_eq!(pair.indices(), (-1, -1));
    pair.set_indices(2, 1);
    assert_eq!(pair.indices(), (2, 1));
    let reversed = Pair::new(1, 2);
    assert!(pair.is_equal(&reversed));
    assert_eq!(pair, reversed);
    assert!(reversed.is_less(&pair));
    assert!(!pair.is_less(&reversed));
    assert_eq!(pair.hash_code(), reversed.hash_code());
}
#[test]
fn point_defaults_and_all_coordinates_match_native() {
    let mut point = Point::default();
    assert_eq!(point.index(), -1);
    assert_eq!(point.pnt(), &Point3::new(99.0, 99.0, 99.0));
    assert_eq!(point.pnt2d1(), &Point2::new(99.0, 99.0));
    assert_eq!(point.pnt2d2(), point.pnt2d1());
    point.set_pnt(Point3::new(1.0, 2.0, 3.0));
    point.set_pnt2d1(Point2::new(4.0, 5.0));
    point.set_pnt2d2(Point2::new(6.0, 7.0));
    point.set_index(8);
    assert_eq!(point.pnt(), &Point3::new(1.0, 2.0, 3.0));
    assert_eq!(point.pnt2d1(), &Point2::new(4.0, 5.0));
    assert_eq!(point.pnt2d2(), &Point2::new(6.0, 7.0));
    assert_eq!(point.index(), 8);
}
