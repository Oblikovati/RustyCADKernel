//! Behavior of the TopTrans port beyond the upstream suite: the curve transition, the
//! static state tables, the touch case and the native exceptions (the states are
//! compared with OCCT by the numeric probe).
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::toptrans::{CurveTransition, SurfaceTransition, get_after, get_before};
use rustycad_tkmath::gp::Direction3;

const TOLE: f64 = 1e-7;

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

#[test]
fn state_tables_follow_the_orientation() {
    let before: Vec<State> = Orientation::ALL.iter().map(|o| get_before(*o)).collect();
    let after: Vec<State> = Orientation::ALL.iter().map(|o| get_after(*o)).collect();
    assert_eq!(before, [State::Out, State::In, State::In, State::Out]);
    assert_eq!(after, [State::In, State::Out, State::In, State::Out]);
}

#[test]
fn curve_transition_is_unknown_until_the_first_comparison() {
    let mut transition = CurveTransition::new();
    // Never reset: `init` is false natively, so the stored FORWARD states answer.
    assert_eq!(transition.state_before(), State::Out);
    transition.reset(dir(1.0, 0.0, 0.0), dir(0.0, 1.0, 0.0), 0.5);
    assert_eq!(transition.state_before(), State::Unknown);
    assert_eq!(transition.state_after(), State::Unknown);
}

#[test]
fn curve_transition_keeps_the_boundary_nearest_on_each_side() {
    let mut transition = CurveTransition::new();
    transition.reset_line(dir(1.0, 0.0, 0.0));
    let (normal, forward, reversed) = (
        dir(0.0, 0.0, 1.0),
        Orientation::Forward,
        Orientation::Reversed,
    );
    transition.compare(TOLE, dir(0.0, 1.0, 0.0), normal, 0.0, forward, forward);
    // A boundary leaving closer to the tangent becomes the last one.
    transition.compare(TOLE, dir(1.0, 1.0, 0.0), normal, 0.0, reversed, forward);
    assert_eq!(transition.state_before(), State::Out);
    assert_eq!(transition.state_after(), State::Out);
}

#[test]
fn curve_transition_internal_takes_the_side_of_the_tangent() {
    let mut transition = CurveTransition::new();
    transition.reset_line(dir(1.0, 0.0, 0.0));
    let internal = Orientation::Internal;
    transition.compare(
        TOLE,
        dir(-1.0, 0.0, 0.0),
        dir(0.0, 0.0, 1.0),
        0.0,
        internal,
        Orientation::Forward,
    );
    assert_eq!(
        (transition.state_before(), transition.state_after()),
        (State::In, State::Out)
    );
}

#[test]
fn surface_transition_raises_for_a_tangent_normal() {
    let mut transition = SurfaceTransition::new();
    let error = transition
        .reset_line(dir(1.0, 0.0, 0.0), dir(1.0, 0.0, 0.0))
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert_eq!(transition.state_before(), State::Unknown);
    transition
        .reset_line(dir(1.0, 0.0, 0.0), dir(0.0, 0.0, 1.0))
        .unwrap();
    let error = transition
        .compare_plane(
            TOLE,
            dir(1.0, 0.0, 0.0),
            Orientation::Forward,
            Orientation::Forward,
        )
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn surface_transition_same_sided_tangent_faces_are_undefined_or_internal() {
    let (tgt, up) = (dir(1.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let mut same = SurfaceTransition::new();
    same.reset_line(tgt, up).unwrap();
    same.compare_plane(TOLE, up, Orientation::Forward, Orientation::Forward)
        .unwrap();
    same.compare_plane(TOLE, up, Orientation::Forward, Orientation::Forward)
        .unwrap();
    assert_eq!(same.state_after(), State::Unknown);
    let mut opposite = SurfaceTransition::new();
    opposite.reset_line(tgt, up).unwrap();
    opposite
        .compare_plane(TOLE, up, Orientation::Forward, Orientation::Forward)
        .unwrap();
    opposite
        .compare_plane(TOLE, up, Orientation::Reversed, Orientation::Forward)
        .unwrap();
    assert_eq!(
        (opposite.state_before(), opposite.state_after()),
        (State::In, State::In)
    );
}

#[test]
fn surface_transition_without_principal_tangent_is_undefined() {
    let (tgt, up) = (dir(1.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    let mut transition = SurfaceTransition::new();
    transition
        .reset(tgt, up, dir(0.0, 1.0, 0.0), dir(1.0, 0.0, 0.0), 2.0, 1.0)
        .unwrap();
    assert_eq!(transition.state_before(), State::Unknown);
    let diagonal = dir(1.0, 1.0, 0.0);
    transition
        .compare(
            TOLE,
            up,
            diagonal,
            diagonal,
            1.0,
            1.0,
            Orientation::Forward,
            Orientation::Forward,
        )
        .unwrap();
    assert_eq!(transition.state_before(), State::Unknown);
}

#[test]
fn defaults_are_the_empty_transitions() {
    assert_eq!(SurfaceTransition::default().state_after(), State::Unknown);
    // A never reset curve transition answers from its stored FORWARD transitions.
    assert_eq!(CurveTransition::default().state_after(), State::In);
}
