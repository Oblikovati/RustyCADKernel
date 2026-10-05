//! Return-value and state parity for CSLib Class2d assignment operators.
use rustycad_tkernel::ncollection::DynamicArray;
use rustycad_tkmath::cslib::{Class2d, Class2dResult};
use rustycad_tkmath::gp::Point2;

fn classifier() -> Class2d {
    let mut points = DynamicArray::default();
    for index in 0..32 {
        let angle = std::f64::consts::TAU * f64::from(index) / 32.0;
        points.append(Point2::new(
            0.5 + angle.cos() * 0.4,
            0.5 + angle.sin() * 0.4,
        ));
    }
    Class2d::from_dynamic_array(&points, 1e-10, 1e-10, 0.0, 0.0, 1.0, 1.0)
}

fn complete_grid(classifier: &Class2d) {
    for _ in 0..64 {
        assert_eq!(
            classifier.si_dans(Point2::new(0.5, 0.5)),
            Class2dResult::Inside
        );
    }
}

#[test]
fn copy_assignment_returns_target_and_preserves_source_and_completed_grid() {
    let source = classifier();
    complete_grid(&source);
    let mut target = Class2d::default();
    let target_address = &mut target as *mut Class2d;
    let returned = target.assign_returning(&source);
    assert!(std::ptr::eq(returned, target_address));
    assert_eq!(target.si_dans(Point2::new(0.5, 0.5)), Class2dResult::Inside);
    assert_eq!(
        source.si_dans(Point2::new(1.2, 0.5)),
        Class2dResult::Outside
    );
}

#[test]
fn move_assignment_returns_target_and_resets_source() {
    let mut source = classifier();
    complete_grid(&source);
    let mut target = Class2d::default();
    let target_address = &mut target as *mut Class2d;
    let returned = target.move_from_returning(&mut source);
    assert!(std::ptr::eq(returned, target_address));
    assert_eq!(target.si_dans(Point2::new(0.5, 0.5)), Class2dResult::Inside);
    assert_eq!(
        source.si_dans(Point2::new(0.5, 0.5)),
        Class2dResult::Uncertain
    );
}

#[test]
fn self_assignment_keeps_completed_grid_and_returns_receiver() {
    let mut target = classifier();
    complete_grid(&target);
    let address = &mut target as *mut Class2d;
    assert!(std::ptr::eq(target.assign_self(), address));
    assert_eq!(target.si_dans(Point2::new(0.5, 0.5)), Class2dResult::Inside);
    assert_eq!(
        target.si_dans(Point2::new(1.2, 0.5)),
        Class2dResult::Outside
    );
}

#[test]
fn empty_self_assignment_keeps_the_empty_classifier() {
    let mut target = Class2d::default();
    let address = &mut target as *mut Class2d;
    assert!(std::ptr::eq(target.assign_self(), address));
    assert_eq!(
        target.si_dans(Point2::new(0.5, 0.5)),
        Class2dResult::Uncertain
    );
}
