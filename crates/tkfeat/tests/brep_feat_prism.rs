//! The prism features on the box `[0, 10]^3` with the profile `[2, 6] x [3, 5]` (area 8)
//! sketched on its top face, checked against closed forms: a boss of height `h` adds
//! `8 h` to the volume of the box, a pocket of depth `d` removes `8 d`, and every result
//! is a valid solid.
mod support;

use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkfeat::brep_feat::{BRepFeatMakePrism, FormFeature, StatusError};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use support::{assert_close, edge_near, face_near, is_valid, make_box, rect_face, volume};

const BOX: f64 = 1000.;
const PROFILE_AREA: f64 = 8.;

fn basis() -> Shape {
    make_box((0., 0., 0.), (10., 10., 10.))
}

fn top(sbase: &Shape) -> Face {
    face_near(sbase, Point3::new(5., 5., 10.))
}

fn bottom(sbase: &Shape) -> Face {
    face_near(sbase, Point3::new(5., 5., 0.))
}

fn profile() -> Shape {
    rect_face(2., 3., 6., 5., 10.).into_shape()
}

/// The prism feature of the profile on the box along `dz` (+1 up, -1 down).
fn feature(sbase: &Shape, mode: i32, dz: f64) -> BRepFeatMakePrism {
    let dir = Direction3::new(0., 0., dz).unwrap();
    BRepFeatMakePrism::with_shapes(sbase, &profile(), &top(sbase), dir, mode, true).unwrap()
}

/// The result of a done feature, checked valid, and its volume.
fn result_volume(prism: &mut BRepFeatMakePrism) -> f64 {
    assert!(
        prism.is_done(),
        "status {:?}",
        prism.form().current_status_error()
    );
    let s = prism.shape().unwrap().clone();
    assert!(is_valid(&s));
    volume(&s)
}

#[test]
fn boss_of_a_length_adds_the_base_area_times_the_height() {
    let sbase = basis();
    let mut prism = feature(&sbase, 1, 1.);
    prism.perform_length(4.).unwrap();
    assert_close(result_volume(&mut prism), BOX + PROFILE_AREA * 4., 1e-9);
}

#[test]
fn pocket_of_a_length_removes_the_base_area_times_the_depth() {
    let sbase = basis();
    let mut prism = feature(&sbase, 0, -1.);
    prism.perform_length(4.).unwrap();
    assert_close(result_volume(&mut prism), BOX - PROFILE_AREA * 4., 1e-9);
}

#[test]
fn feature_alone_is_the_prism() {
    let sbase = basis();
    let mut prism = feature(&sbase, 2, 1.);
    prism.perform_length(3.).unwrap();
    assert_close(result_volume(&mut prism), PROFILE_AREA * 3., 1e-9);
}

#[test]
fn pocket_through_all_crosses_the_box() {
    let sbase = basis();
    let mut prism = feature(&sbase, 0, -1.);
    prism.perform_thru_all().unwrap();
    assert_close(result_volume(&mut prism), BOX - PROFILE_AREA * 10., 1e-9);
}

#[test]
fn pocket_until_the_end_crosses_the_box() {
    let sbase = basis();
    let mut prism = feature(&sbase, 0, -1.);
    prism.perform_until_end().unwrap();
    assert_close(result_volume(&mut prism), BOX - PROFILE_AREA * 10., 1e-9);
}

#[test]
fn pocket_until_the_bottom_face_crosses_the_box() {
    let sbase = basis();
    let mut prism = feature(&sbase, 0, -1.);
    prism.perform_until(&bottom(&sbase)).unwrap();
    assert_close(result_volume(&mut prism), BOX - PROFILE_AREA * 10., 1e-9);
}

#[test]
fn boss_until_a_face_above_stops_on_it() {
    let sbase = basis();
    let mut prism = feature(&sbase, 1, 1.);
    let roof = rect_face(-5., -5., 15., 15., 15.).into_shape();
    prism.perform_until(&roof).unwrap();
    assert_close(result_volume(&mut prism), BOX + PROFILE_AREA * 5., 1e-9);
}

#[test]
fn pocket_of_a_height_towards_a_face() {
    let sbase = basis();
    let mut prism = feature(&sbase, 0, -1.);
    prism.perform_until_height(&bottom(&sbase), 3.).unwrap();
    assert_close(result_volume(&mut prism), BOX - PROFILE_AREA * 3., 1e-9);
}

#[test]
fn boss_between_two_faces_spans_them() {
    let sbase = basis();
    let mut prism = feature(&sbase, 1, 1.);
    let from = rect_face(-5., -5., 15., 15., 12.).into_shape();
    let until = rect_face(-5., -5., 15., 15., 16.).into_shape();
    prism.perform_from_until(&from, &until).unwrap();
    assert_close(result_volume(&mut prism), BOX + PROFILE_AREA * 4., 1e-9);
}

#[test]
fn add_refuses_a_face_outside_the_basis() {
    let sbase = basis();
    let pbase = profile();
    let dir = Direction3::new(0., 0., 1.).unwrap();
    let mut prism =
        BRepFeatMakePrism::with_shapes(&sbase, &pbase, &top(&sbase), dir, 1, true).unwrap();
    let other = rect_face(0., 0., 1., 1., 0.);
    let stranger = edge_near(&profile(), Point3::new(4., 3., 10.));
    assert!(prism.add(&stranger, &top(&sbase)).is_err());
    let e = edge_near(&pbase, Point3::new(4., 3., 10.));
    assert!(prism.add(&e, &other).is_err());
    assert!(prism.add(&e, &top(&sbase)).is_ok());
}

#[test]
fn history_maps_the_top_face_of_a_boss() {
    let sbase = basis();
    let mut prism = feature(&sbase, 1, 1.);
    prism.perform_length(4.).unwrap();
    assert!(prism.is_done());
    // natively only GlobalPerform sets the status: the Boolean path keeps NotInitialized
    assert_eq!(
        prism.form().current_status_error(),
        StatusError::NotInitialized
    );
    let lasts = prism.form().last_shape().unwrap();
    let firsts = prism.form().first_shape().unwrap();
    // the bottom of the boss lies in the top face of the box and disappears in the fuse
    assert_eq!((lasts.len(), firsts.len()), (1, 0));
}
