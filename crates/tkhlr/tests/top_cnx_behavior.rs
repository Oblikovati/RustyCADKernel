//! Behaviour of the TopBas and TopCnx ports: the interference record and the
//! edge-face transition, with the native quirks the port keeps
//! (docs/tkhlr-progress.md). tools/verify_tkhlr_numeric.py compares the same members
//! with OCCT.
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkhlr::top_bas::TestInterference;
use rustycad_tkhlr::top_cnx::EdgeFaceTransition;
use rustycad_tkmath::gp::Direction3;

#[test]
fn interference_record_setters_and_change_accessors() {
    let mut record = TestInterference::new();
    assert_eq!(record, TestInterference::default());
    record.set_intersection(1.5);
    record.set_boundary(4);
    record.set_orientation(Orientation::Reversed);
    record.set_transition(Orientation::Internal);
    record.set_boundary_transition(Orientation::External);
    *record.change_intersection() *= 2.0;
    *record.change_boundary() -= 1;
    let expected = TestInterference::with_values(
        3.0,
        3,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    );
    assert_eq!(record, expected);
    assert_eq!(record.orientation(), Orientation::Reversed);
    assert_eq!(record.boundary_transition(), Orientation::External);
}

#[test]
fn a_new_transition_is_forward_before_any_reset() {
    // TopTrans_CurveTransition starts with Init false and FORWARD transitions, so the
    // states read OUT then IN before the first Reset.
    let transition = EdgeFaceTransition::new();
    assert_eq!(transition.transition(), Orientation::Forward);
    assert_eq!(transition.boundary_transition(), Orientation::External);
}

#[test]
fn a_reset_without_interference_is_internal() {
    let mut transition = EdgeFaceTransition::new();
    transition.reset(Direction3::X, Direction3::Y, 0.5);
    assert_eq!(transition.transition(), Orientation::Internal);
}

#[test]
fn boundary_transition_counts_forward_and_reversed() {
    let mut transition = EdgeFaceTransition::new();
    transition.reset_line(Direction3::X);
    let add = |t: &mut EdgeFaceTransition, b_tr: Orientation| {
        let (o, tr) = (Orientation::Forward, Orientation::Forward);
        t.add_interference(1e-7, Direction3::Y, Direction3::Z, 0.0, o, tr, b_tr);
    };
    add(&mut transition, Orientation::Reversed);
    assert_eq!(transition.boundary_transition(), Orientation::Reversed);
    add(&mut transition, Orientation::Forward);
    add(&mut transition, Orientation::Internal);
    assert_eq!(transition.boundary_transition(), Orientation::External);
    add(&mut transition, Orientation::Forward);
    assert_eq!(transition.boundary_transition(), Orientation::Forward);
    transition.reset_line(Direction3::X);
    assert_eq!(transition.boundary_transition(), Orientation::External);
}
