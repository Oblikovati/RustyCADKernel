//! Behavior tests of BRepCheck_Shell on cubes and bands built with BRep_Builder
//! (brep_check_support); the probes compare the same checks with OCCT.
mod brep_check_support;

use brep_check_support::{Planar, cube, solid_of};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Shape, Shell};
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Vector3;
use rustycad_tktopalgo::brep_check::{BRepCheckResult, BRepCheckShell, BRepCheckStatus};

use BRepCheckStatus::*;

fn in_solid(shell: &Shell) -> Vec<BRepCheckStatus> {
    let solid = solid_of(shell);
    let checker = BRepCheckShell::new(shell).unwrap();
    checker.in_context(&solid).unwrap();
    checker
        .status_on_shape_of(&solid)
        .unwrap()
        .iter()
        .copied()
        .collect()
}

#[test]
fn closed_cube_shell_is_valid_in_its_solid() {
    let cube = cube(&[0, 1, 2, 3, 4, 5]);
    let shell = cube.shell(&cube.faces);
    let checker = BRepCheckShell::new(&shell).unwrap();
    assert_eq!(checker.status().unwrap().first(), Some(&NoError));
    assert_eq!(checker.closed(false).unwrap(), NoError);
    assert_eq!(checker.orientation(false).unwrap(), NoError);
    assert!(!checker.is_unorientable().unwrap());
    assert_eq!(in_solid(&shell), [NoError]);
}

#[test]
fn open_and_disconnected_shells() {
    let open = cube(&[0, 1, 2, 3, 4]);
    let checker = BRepCheckShell::new(&open.shell(&open.faces)).unwrap();
    assert_eq!(checker.closed(true).unwrap(), NotClosed);
    assert_eq!(in_solid(&open.shell(&open.faces)), [NotClosed]);
    let apart = cube(&[0, 1]);
    let checker = BRepCheckShell::new(&apart.shell(&apart.faces)).unwrap();
    assert_eq!(checker.status().unwrap().first(), Some(&NotConnected));
    assert_eq!(checker.closed(false).unwrap(), NotConnected);
    let b = BRepBuilder::default();
    let mut empty = Shell::default();
    b.make_shell(&mut empty);
    assert_eq!(
        BRepCheckShell::new(&empty)
            .unwrap()
            .status()
            .unwrap()
            .first(),
        Some(&EmptyShell)
    );
}

#[test]
fn reversed_face_is_a_bad_orientation_of_an_orientable_shell() {
    let cube = cube(&[0, 1, 2, 3, 4, 5]);
    let mut faces = cube.faces.clone();
    faces[0] = rustycad_tkbrep::topods::Face::unchecked(faces[0].reversed());
    let shell = cube.shell(&faces);
    let checker = BRepCheckShell::new(&shell).unwrap();
    assert_eq!(
        checker.orientation(false).unwrap(),
        BadOrientationOfSubshape
    );
    assert!(checker.is_unorientable().unwrap());
    let updated = BRepCheckShell::new(&shell).unwrap();
    assert_eq!(updated.orientation(true).unwrap(), BadOrientationOfSubshape);
    let recorded: Vec<_> = updated.status().unwrap().iter().copied().collect();
    assert_eq!(recorded, [BadOrientationOfSubshape]);
}

/// A band of three quads, twisted or not (only the topology is read).
fn band(twisted: bool) -> Shell {
    let mut band = Planar::new(&[
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (2.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        (1.0, 1.0, 0.0),
        (2.0, 1.0, 0.0),
    ]);
    let z = Some(Vector3::new(0.0, 0.0, 1.0));
    band.face(&[&[0, 1, 4, 3]], z, &[], 0.0);
    band.face(&[&[1, 2, 5, 4]], z, &[], 0.0);
    let last: [usize; 4] = if twisted { [2, 3, 0, 5] } else { [2, 0, 3, 5] };
    band.face(&[&last], z, &[], 0.0);
    band.finish();
    band.shell(&band.faces)
}

#[test]
fn a_twisted_band_cannot_be_oriented() {
    let checker = BRepCheckShell::new(&band(false)).unwrap();
    assert_eq!(checker.closed(false).unwrap(), NotClosed);
    assert_eq!(checker.orientation(true).unwrap(), NoError);
    let checker = BRepCheckShell::new(&band(true)).unwrap();
    assert_eq!(
        checker.orientation(false).unwrap(),
        BadOrientationOfSubshape
    );
    let checker = BRepCheckShell::new(&band(true)).unwrap();
    assert_eq!(checker.orientation(true).unwrap(), UnorientableShape);
    checker.set_unorientable().unwrap();
    assert!(checker.is_unorientable().unwrap());
}

#[test]
fn connected_sets_split_at_an_edge_of_three_faces() {
    let mut fan = Planar::new(&[
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        (0.0, 0.0, 1.0),
        (0.0, -1.0, 0.0),
    ]);
    for lp in [[0, 1, 2], [0, 1, 3], [0, 1, 4]] {
        fan.face(&[&lp], None, &[], 0.0);
    }
    fan.finish();
    let shell = fan.shell(&fan.faces);
    let checker = BRepCheckShell::new(&shell).unwrap();
    let mut sets: List<Shape> = List::default();
    // The last free face ends the search before its set is appended, as natively.
    assert_eq!(checker.nb_connected_set(&mut sets).unwrap(), 2);
    assert_eq!(checker.closed(false).unwrap(), InvalidMultiConnexity);
    let faces = fan.faces.clone();
    let mut reversed = faces.clone();
    reversed[1] =
        rustycad_tkbrep::topods::Face::unchecked(faces[1].oriented(Orientation::Reversed));
    let checker = BRepCheckShell::new(&fan.shell(&reversed)).unwrap();
    assert_eq!(checker.closed(false).unwrap(), InvalidMultiConnexity);
}
