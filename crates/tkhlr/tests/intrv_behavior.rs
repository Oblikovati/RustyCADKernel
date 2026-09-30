//! Behaviour of the Intrv port: the constructors, the bound edits, the relative
//! positions and the boolean operations of interval sets, with the native quirks the
//! port keeps (docs/tkhlr-progress.md). tools/verify_tkhlr_numeric.py compares the
//! same members with OCCT bit for bit.
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkhlr::intrv::{Interval, Intervals, Position};

fn fuzzy(start: f64, end: f64, tol: f32) -> Interval {
    Interval::with_tolerances(start, tol, end, tol)
}

fn bounds(set: &Intervals) -> Vec<(f64, f64)> {
    (1..=set.nb_intervals())
        .map(|i| {
            set.value(i)
                .map(|v| (v.start(), v.end()))
                .unwrap_or_default()
        })
        .collect()
}

#[test]
fn default_interval_is_the_whole_line_with_zero_tolerances() {
    let interval = Interval::new();
    assert_eq!(interval.bounds(), (real_first(), 0.0, real_last(), 0.0));
    assert_eq!(Interval::default(), interval);
}

#[test]
fn constructors_take_the_double_spacing_as_the_smallest_tolerance() {
    let interval = Interval::from_bounds(1.0, 2.0);
    assert_eq!(interval.tol_start(), f64::EPSILON as f32);
    let raised = Interval::with_tolerances(1.0, -1.0, 2.0, 0.5);
    assert_eq!(
        (raised.tol_start(), raised.tol_end()),
        (f64::EPSILON as f32, 0.5)
    );
}

#[test]
fn setters_keep_the_given_tolerances_unchecked() {
    let mut interval = Interval::new();
    interval.set_start(3.0, -0.5);
    interval.set_end(1.0, 0.0);
    assert_eq!(interval.bounds(), (3.0, -0.5, 1.0, 0.0));
}

#[test]
fn fuse_and_cut_move_the_fuzzy_bounds() {
    let mut interval = fuzzy(0.0, 10.0, 0.1);
    interval.fuse_at_start(-0.5, 0.2);
    // The union of [-0.1, 0.1] and [-0.7, -0.3] at the start: low -0.7, high -0.3.
    assert!((interval.start() + 0.5).abs() < 1e-7);
    interval.cut_at_end(9.0, 0.0);
    assert_eq!(interval.end(), 9.0);
    assert_eq!(interval.tol_end(), 0.0);
}

#[test]
fn infinite_bounds_ignore_fuse_and_cut() {
    let mut interval = Interval::new();
    interval.fuse_at_start(1.0, 0.5);
    interval.cut_at_start(1.0, 0.5);
    interval.fuse_at_end(1.0, 0.5);
    interval.cut_at_end(1.0, 0.5);
    assert_eq!(interval, Interval::new());
}

#[test]
fn position_orders_every_case_around_a_reference() {
    let other = fuzzy(0.0, 10.0, 0.5);
    let cases = [
        ((-3.0, -1.0), Position::Before),
        ((-3.0, 0.0), Position::JustBefore),
        ((-3.0, 5.0), Position::OverlappingAtStart),
        ((-3.0, 10.0), Position::JustEnclosingAtEnd),
        ((-3.0, 13.0), Position::Enclosing),
        ((0.0, 5.0), Position::JustOverlappingAtStart),
        ((0.0, 10.0), Position::Similar),
        ((0.0, 13.0), Position::JustEnclosingAtStart),
        ((3.0, 5.0), Position::Inside),
        ((3.0, 10.0), Position::JustOverlappingAtEnd),
        ((3.0, 13.0), Position::OverlappingAtEnd),
        ((10.0, 13.0), Position::JustAfter),
        ((12.0, 13.0), Position::After),
    ];
    for ((start, end), expected) in cases {
        assert_eq!(fuzzy(start, end, 0.1).position(&other), expected);
    }
}

#[test]
fn the_tests_agree_with_position_on_clear_cases() {
    let other = fuzzy(0.0, 10.0, 0.5);
    assert!(fuzzy(-3.0, -1.0, 0.1).is_before(&other));
    assert!(fuzzy(12.0, 13.0, 0.1).is_after(&other));
    assert!(fuzzy(3.0, 5.0, 0.1).is_inside(&other));
    assert!(fuzzy(-3.0, 13.0, 0.1).is_enclosing(&other));
    assert!(fuzzy(0.0, 13.0, 0.1).is_just_enclosing_at_start(&other));
    assert!(fuzzy(-3.0, 10.0, 0.1).is_just_enclosing_at_end(&other));
    assert!(fuzzy(-3.0, 0.0, 0.1).is_just_before(&other));
    assert!(fuzzy(10.0, 13.0, 0.1).is_just_after(&other));
    assert!(fuzzy(-3.0, 5.0, 0.1).is_overlapping_at_start(&other));
    assert!(fuzzy(3.0, 13.0, 0.1).is_overlapping_at_end(&other));
    assert!(fuzzy(0.0, 5.0, 0.1).is_just_overlapping_at_start(&other));
    assert!(fuzzy(3.0, 10.0, 0.1).is_just_overlapping_at_end(&other));
    assert!(fuzzy(0.2, 9.9, 0.1).is_similar(&other));
    assert!(fuzzy(1.0, 1.5, 0.5).is_probably_empty());
}

#[test]
fn subtract_splits_and_trims() {
    let mut set = Intervals::from_interval(fuzzy(0.0, 10.0, 0.0));
    set.subtract_interval(&fuzzy(2.0, 3.0, 0.0));
    assert_eq!(bounds(&set), vec![(0.0, 2.0), (3.0, 10.0)]);
    set.subtract(&Intervals::from_interval(fuzzy(9.0, 12.0, 0.0)));
    assert_eq!(bounds(&set), vec![(0.0, 2.0), (3.0, 9.0)]);
}

#[test]
fn unite_keeps_the_set_sorted_and_fuses_overlaps() {
    let mut set = Intervals::new();
    set.unite_interval(&fuzzy(4.0, 5.0, 0.0));
    set.unite_interval(&fuzzy(0.0, 1.0, 0.0));
    set.unite_interval(&fuzzy(0.5, 4.5, 0.0));
    assert_eq!(bounds(&set), vec![(0.0, 5.0)]);
}

#[test]
fn intersect_and_xunite_are_built_from_unite_and_subtract() {
    let mut set = Intervals::from_interval(fuzzy(0.0, 10.0, 0.0));
    set.intersect_interval(&fuzzy(5.0, 15.0, 0.0));
    assert_eq!(bounds(&set), vec![(5.0, 10.0)]);
    let mut exclusive = Intervals::from_interval(fuzzy(0.0, 10.0, 0.0));
    exclusive.xunite_interval(&fuzzy(5.0, 15.0, 0.0));
    assert_eq!(bounds(&exclusive), vec![(0.0, 5.0), (10.0, 15.0)]);
}

#[test]
fn value_outside_the_sequence_is_out_of_range() {
    let set = Intervals::from_interval(fuzzy(0.0, 1.0, 0.0));
    for index in [-1, 0, 2] {
        let error = set.value(index).expect_err("index outside 1..=1");
        assert_eq!(error.exception_type(), "Standard_OutOfRange");
    }
}
