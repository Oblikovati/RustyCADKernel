//! Cylindrical holes of radius 1 in the box `[0, 10]^3` along the vertical axis through
//! `(5, 5)`, starting at height 12 and going down (the box spans the axis parameters 2 to
//! 12): a through hole removes `pi r^2 10`, a blind hole of length 6 stops at height 6
//! and removes `pi r^2 4`; every result is a valid solid.
mod support;

use rustycad_tkbrep::topods::Shape;
use rustycad_tkfeat::brep_feat::{BRepFeatMakeCylindricalHole, Status};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use std::f64::consts::PI;
use support::{assert_close, is_valid, make_box, volume};

const BOX: f64 = 1000.;

fn hole(x: f64) -> BRepFeatMakeCylindricalHole {
    let axis = Axis1::new(
        Point3::new(x, 5., 12.),
        Direction3::new(0., 0., -1.).unwrap(),
    );
    let mut h = BRepFeatMakeCylindricalHole::new();
    h.init(&make_box((0., 0., 0.), (10., 10., 10.)), axis);
    h
}

fn drilled(h: &mut BRepFeatMakeCylindricalHole) -> Shape {
    h.build().unwrap();
    assert_eq!(h.status(), Status::NoError);
    let s = h.shape().clone();
    assert!(is_valid(&s));
    s
}

#[test]
fn through_hole_removes_the_whole_cylinder() {
    let mut h = hole(5.);
    h.perform(1.).unwrap();
    assert_close(volume(&drilled(&mut h)), BOX - PI * 10., 1e-7);
}

#[test]
fn hole_through_next_crosses_the_box() {
    let mut h = hole(5.);
    h.perform_thru_next(1., true).unwrap();
    assert_close(volume(&drilled(&mut h)), BOX - PI * 10., 1e-7);
}

#[test]
fn hole_until_end_crosses_the_box() {
    let mut h = hole(5.);
    h.perform_until_end(1., true).unwrap();
    assert_close(volume(&drilled(&mut h)), BOX - PI * 10., 1e-7);
}

#[test]
fn hole_between_parameters_runs_to_the_next_exit() {
    let mut h = hole(5.);
    h.perform_between(1., 0., 7., true).unwrap();
    assert_close(volume(&drilled(&mut h)), BOX - PI * 10., 1e-7);
}

#[test]
fn blind_hole_stops_at_its_length() {
    let mut h = hole(5.);
    h.perform_blind(1., 6., true).unwrap();
    assert_close(volume(&drilled(&mut h)), BOX - PI * 4., 1e-7);
}

#[test]
fn blind_hole_longer_than_the_box_is_too_long() {
    let mut h = hole(5.);
    h.perform_blind(1., 13., true).unwrap();
    assert_eq!(h.status(), Status::HoleTooLong);
    assert!(h.perform_blind(1., 0., true).is_err());
}

#[test]
fn axis_missing_the_box_is_an_invalid_placement() {
    let mut h = hole(20.);
    h.perform(1.).unwrap();
    assert_eq!(h.status(), Status::InvalidPlacement);
    let mut h = hole(20.);
    h.perform_thru_next(1., false).unwrap();
    assert_eq!(h.status(), Status::InvalidPlacement);
}

#[test]
fn hole_without_axis_is_refused() {
    let mut h = BRepFeatMakeCylindricalHole::new();
    assert!(h.perform(1.).is_err());
    h.init_axis(Axis1::default());
    assert!(h.perform(1.).is_err());
}

/// Two plates `[0, 10]^2 x [0, 4]` and `[0, 10]^2 x [6, 10]` in a compound, drilled
/// from above.
fn plates_hole() -> BRepFeatMakeCylindricalHole {
    use rustycad_tkbrep::brep::BRepBuilder;
    use rustycad_tkbrep::topods::Compound;
    let b = BRepBuilder::default();
    let mut comp = Compound::default();
    b.make_compound(&mut comp);
    let mut comp = comp.into_shape();
    b.add(&mut comp, &make_box((0., 0., 0.), (10., 10., 4.)))
        .unwrap();
    b.add(&mut comp, &make_box((0., 0., 6.), (10., 10., 4.)))
        .unwrap();
    let axis = Axis1::new(
        Point3::new(5., 5., 12.),
        Direction3::new(0., 0., -1.).unwrap(),
    );
    let mut h = BRepFeatMakeCylindricalHole::new();
    h.init(&comp, axis);
    h
}

#[test]
fn hole_through_next_stops_after_the_first_plate() {
    let mut h = plates_hole();
    h.perform_thru_next(1., true).unwrap();
    assert_close(volume(&drilled(&mut h)), 800. - PI * 4., 1e-7);
}

#[test]
fn hole_until_end_through_plates_removes_nothing_as_natively() {
    let mut h = plates_hole();
    h.perform_until_end(1., true).unwrap();
    // the native kernel (checked with libTKFeat) also keeps both plates whole here
    assert_close(volume(&drilled(&mut h)), 800., 1e-12);
}
