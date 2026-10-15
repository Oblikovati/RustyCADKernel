//! The revolved features on the box `[0, 10]^3`: the rectangle `x = 5`, `y` in `[2, 4]`,
//! two high, standing on (boss) or hanging from (groove) the top face, turned around the
//! vertical axis through `(5, 5)`. Its radii are 1 and 3, so a turn of `a` sweeps
//! `a / 2 (9 - 1) 2 = 8 a`.
mod support;

use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkfeat::brep_feat::{BRepFeatMakeRevol, FormFeature};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use std::f64::consts::PI;
use support::{assert_close, face_near, is_valid, make_box, polygon_face, volume};

const BOX: f64 = 1000.;

fn basis() -> Shape {
    make_box((0., 0., 0.), (10., 10., 10.))
}

fn top(sbase: &Shape) -> Face {
    face_near(sbase, Point3::new(5., 5., 10.))
}

fn axis() -> Axis1 {
    Axis1::new(
        Point3::new(5., 5., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
}

/// The profile between the heights `z0` and `z1`.
fn profile(z0: f64, z1: f64) -> Shape {
    polygon_face(&[(5., 2., z0), (5., 4., z0), (5., 4., z1), (5., 2., z1)]).into_shape()
}

fn feature(sbase: &Shape, mode: i32) -> BRepFeatMakeRevol {
    let (z0, z1) = if mode == 0 { (8., 10.) } else { (10., 12.) };
    BRepFeatMakeRevol::with_shapes(sbase, &profile(z0, z1), &top(sbase), axis(), mode, true)
        .unwrap()
}

fn result_volume(revol: &mut BRepFeatMakeRevol) -> f64 {
    assert!(
        revol.is_done(),
        "status {:?}",
        revol.form().current_status_error()
    );
    let s = revol.shape().unwrap().clone();
    assert!(is_valid(&s));
    volume(&s)
}

#[test]
fn boss_of_a_quarter_turn_adds_the_sector() {
    let sbase = basis();
    let mut revol = feature(&sbase, 1);
    revol.perform_angle(PI / 2.).unwrap();
    assert_close(result_volume(&mut revol), BOX + 8. * PI / 2., 1e-7);
}

#[test]
fn groove_of_a_half_turn_removes_the_sector() {
    let sbase = basis();
    let mut revol = feature(&sbase, 0);
    revol.perform_angle(PI).unwrap();
    assert_close(result_volume(&mut revol), BOX - 8. * PI, 1e-7);
}

#[test]
fn groove_through_all_is_a_full_ring() {
    let sbase = basis();
    let mut revol = feature(&sbase, 0);
    revol.perform_thru_all().unwrap();
    assert_close(result_volume(&mut revol), BOX - 16. * PI, 1e-7);
}

#[test]
fn feature_alone_is_the_sector() {
    let sbase = basis();
    let mut revol = feature(&sbase, 2);
    revol.perform_angle(PI / 3.).unwrap();
    // natively the fusion of the revolution ignores the just-feature flag
    assert_close(result_volume(&mut revol), BOX + 8. * PI / 3., 1e-7);
}

#[test]
fn groove_until_a_wall_stops_on_it() {
    let sbase = basis();
    let mut revol = feature(&sbase, 0);
    // the half plane x > 5 at y = 5
    let wall = polygon_face(&[(5., 5., -5.), (15., 5., -5.), (15., 5., 15.), (5., 5., 15.)]);
    revol.perform_until(&wall.into_shape()).unwrap();
    let removed = BOX - result_volume(&mut revol);
    // the profile at y < 5 turns counterclockwise and meets the wall after a quarter turn
    assert_close(removed, 8. * PI / 2., 1e-7);
}

#[test]
fn until_without_face_is_refused() {
    let sbase = basis();
    let mut revol = feature(&sbase, 0);
    let empty = rustycad_tkbrep::topods::Compound::default().into_shape();
    assert!(revol.perform_until(&empty).is_err());
}
