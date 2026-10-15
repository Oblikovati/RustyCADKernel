//! The drafted prism features on the box `[0, 10]^3` with the face `[2, 6] x [3, 5]`
//! (area 8, normal +Z) sketched on its top face, checked against closed forms: without
//! draft a boss of height `h` adds `8 h`; with the draft `t = tan a` the section shrinks
//! by `t z` on each side, `(4 - 2 t z)(2 - 2 t z)` integrated over the height.
mod support;

use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkfeat::brep_feat::{BRepFeatMakeDPrism, FormFeature};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use support::{assert_close, face_near, is_valid, make_box, rect_face, volume};

const BOX: f64 = 1000.;

fn basis() -> Shape {
    make_box((0., 0., 0.), (10., 10., 10.))
}

fn top(sbase: &Shape) -> Face {
    face_near(sbase, Point3::new(5., 5., 10.))
}

fn bottom(sbase: &Shape) -> Face {
    face_near(sbase, Point3::new(5., 5., 0.))
}

fn feature(sbase: &Shape, mode: i32, angle: f64) -> BRepFeatMakeDPrism {
    let face = rect_face(2., 3., 6., 5., 10.);
    BRepFeatMakeDPrism::with_shapes(sbase, &face, &top(sbase), angle, mode, true).unwrap()
}

/// The volume of the drafted prism of the face over the height `h` with `t = tan a`.
fn frustum(h: f64, t: f64) -> f64 {
    8. * h - 6. * t * h * h + 4. / 3. * t * t * h.powi(3)
}

fn result_volume(dprism: &mut BRepFeatMakeDPrism) -> f64 {
    assert!(
        dprism.is_done(),
        "status {:?}",
        dprism.form().current_status_error()
    );
    let s = dprism.shape().unwrap().clone();
    assert!(is_valid(&s));
    volume(&s)
}

#[test]
fn boss_without_draft_adds_the_base_area_times_the_height() {
    let sbase = basis();
    let mut dprism = feature(&sbase, 1, 0.);
    dprism.perform_height(4.).unwrap();
    assert_close(result_volume(&mut dprism), BOX + 32., 1e-9);
}

#[test]
fn drafted_boss_adds_a_frustum() {
    let sbase = basis();
    let a = 5_f64.to_radians();
    let mut dprism = feature(&sbase, 1, a);
    dprism.perform_height(3.).unwrap();
    assert_close(result_volume(&mut dprism), BOX + frustum(3., a.tan()), 1e-7);
}

#[test]
fn pocket_of_a_negative_height_goes_down() {
    let sbase = basis();
    let mut dprism = feature(&sbase, 0, 0.);
    dprism.perform_height(-4.).unwrap();
    assert_close(result_volume(&mut dprism), BOX - 32., 1e-9);
}

#[test]
fn pocket_through_all_crosses_the_box() {
    let sbase = basis();
    let mut dprism = feature(&sbase, 0, 0.);
    dprism.perform_thru_all().unwrap();
    // the Gauss integration of the long drafted prism loses ~1e-9 relative
    assert_close(result_volume(&mut dprism), BOX - 80., 1e-8);
}

#[test]
fn boss_until_a_face_above_stops_on_it() {
    let sbase = basis();
    let mut dprism = feature(&sbase, 1, 0.);
    let roof = rect_face(-5., -5., 15., 15., 15.).into_shape();
    dprism.perform_until(&roof).unwrap();
    assert_close(result_volume(&mut dprism), BOX + 40., 1e-9);
}

#[test]
fn pocket_of_a_height_towards_the_bottom() {
    let sbase = basis();
    let mut dprism = feature(&sbase, 0, 0.);
    dprism.perform_until_height(&bottom(&sbase), 3.).unwrap();
    assert_close(result_volume(&mut dprism), BOX - 24., 1e-9);
}

#[test]
fn boss_edges_of_a_boss_are_its_top_and_vertical_edges() {
    let sbase = basis();
    let mut dprism = feature(&sbase, 1, 0.);
    dprism.perform_height(4.).unwrap();
    dprism.boss_edges(2).unwrap();
    assert_eq!(dprism.top_edges().len(), 4);
    assert_eq!(dprism.lat_edges().len(), 4);
    dprism.boss_edges(0).unwrap();
    assert_eq!(dprism.top_edges().len(), 4);
}
