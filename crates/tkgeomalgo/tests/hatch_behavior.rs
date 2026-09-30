//! Behavior of the Hatch port (no upstream suite; the bits are compared with OCCT by
//! the numeric probe).
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkgeomalgo::hatch::{Hatcher, Line, LineForm, Parameter};
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

fn square_hatcher(oriented: bool) -> Hatcher {
    let mut hatcher = Hatcher::new(1.0e-7, oriented);
    hatcher.add_x_line(0.5);
    hatcher.add_y_line(0.25);
    let corners = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    for k in 0..4 {
        let (a, b) = (corners[k], corners[(k + 1) % 4]);
        let (p1, p2) = (Point2::new(a.0, a.1), Point2::new(b.0, b.1));
        hatcher.trim_segment(p1, p2, k as i32 + 1).unwrap();
    }
    hatcher
}

#[test]
fn a_square_trims_each_line_to_one_interval() {
    let hatcher = square_hatcher(true);
    assert_eq!((hatcher.nb_lines(), hatcher.nb_intervals()), (2, 2));
    assert_eq!(
        (hatcher.start(1, 1).unwrap(), hatcher.end(1, 1).unwrap()),
        (0.0, 1.0)
    );
    assert_eq!(hatcher.start_index(1, 1).unwrap(), (1, 0.5));
    assert_eq!(hatcher.end_index(1, 1).unwrap(), (3, 0.5));
    assert_eq!(hatcher.coordinate(1).unwrap(), 0.5);
    assert_eq!(hatcher.coordinate(2).unwrap(), 0.25);
    assert!(hatcher.is_x_line(1).unwrap() && hatcher.is_y_line(2).unwrap());
}

#[test]
fn an_untrimmed_line_is_infinite_only_in_oriented_mode() {
    let mut oriented = Hatcher::new(1.0e-7, true);
    let direction = Direction2::new(0.6, 0.8).unwrap();
    oriented.add_line_at_distance(direction, 2.0);
    assert_eq!(oriented.nb_intervals_on(1).unwrap(), 1);
    assert_eq!(oriented.start(1, 1).unwrap(), real_first());
    assert_eq!(oriented.end(1, 1).unwrap(), real_last());
    assert_eq!(oriented.line_form(1).unwrap(), LineForm::AnyLine);
    let error = oriented.coordinate(1).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    let mut unoriented = Hatcher::new(1.0e-7, false);
    unoriented.add_line(Line2::default(), LineForm::AnyLine);
    assert_eq!(unoriented.nb_intervals_on(1).unwrap(), 0);
    assert!(unoriented.start(1, 1).is_err());
    // StartIndex accepts J = 1 on an empty line even in unoriented mode, as native does.
    assert_eq!(unoriented.start_index(1, 1).unwrap(), (0, 0.0));
}

#[test]
fn bad_line_indices_raise_out_of_range() {
    let hatcher = square_hatcher(false);
    for index in [0, 3, -1] {
        assert_eq!(
            hatcher.line(index).unwrap_err().exception_type(),
            "Standard_OutOfRange"
        );
    }
    assert!(hatcher.start(1, 2).is_err());
}

#[test]
fn close_intersections_cancel_each_other() {
    // akm OCC109: an intersection closer than the tolerance removes the existing one.
    let mut line = Line::new(Line2::default(), LineForm::AnyLine);
    line.add_intersection(1.0, true, 1, 0.0, 1.0e-3);
    line.add_intersection(1.0005, false, 2, 0.0, 1.0e-3);
    let mut hatcher = Hatcher::new(1.0e-3, false);
    hatcher.add_line(Line2::default(), LineForm::AnyLine);
    let first = Point2::new(1.0, -1.0);
    hatcher
        .trim_segment(first, Point2::new(1.0, 1.0), 1)
        .unwrap();
    hatcher
        .trim_segment(Point2::new(1.0005, 1.0), Point2::new(1.0005, -1.0), 2)
        .unwrap();
    assert_eq!(hatcher.nb_intervals_on(1).unwrap(), 0);
    assert_ne!(Parameter::new(1.0, true, 0, 0.0), Parameter::default());
}
