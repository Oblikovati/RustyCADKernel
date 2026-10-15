//! The pipe feature, the gluer and the splitting of shapes on the box `[0, 10]^3`: a
//! square pipe along a straight spine is a box (`2 x 2 x length`), a small box glued on
//! the top adds its volume, and an edge across the top face splits it in two halves.
mod support;

use rustycad_tkbrep::topods::{Edge, Shape, Wire};
use rustycad_tkfeat::brep_feat::{BRepFeatGluer, BRepFeatMakePipe, BRepFeatSplitShape};
use rustycad_tkfeat::loc_ope::Operation;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use support::{area, assert_close, count, face_near, is_valid, make_box, rect_face, volume};

fn basis() -> Shape {
    make_box((0., 0., 0.), (10., 10., 10.))
}

fn segment(a: (f64, f64, f64), b: (f64, f64, f64)) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(a.0, a.1, a.2), Point3::new(b.0, b.1, b.2))
        .unwrap()
        .edge()
        .unwrap()
}

fn spine(z1: f64) -> Wire {
    BRepBuilderAPIMakeWire::from_edge(&segment((3., 3., 10.), (3., 3., z1)))
        .unwrap()
        .wire()
        .unwrap()
}

fn pipe(mode: i32, z1: f64) -> BRepFeatMakePipe {
    let sbase = basis();
    let top = face_near(&sbase, Point3::new(5., 5., 10.));
    let profile = rect_face(2., 2., 4., 4., 10.).into_shape();
    BRepFeatMakePipe::with_shapes(&sbase, &profile, &top, &spine(z1), mode, true).unwrap()
}

#[test]
fn pipe_boss_along_a_line_adds_a_box() {
    let mut p = pipe(1, 14.);
    p.perform().unwrap();
    assert!(p.is_done());
    let res = p.shape().unwrap().clone();
    assert!(is_valid(&res));
    assert_close(volume(&res), 1016., 1e-9);
}

#[test]
fn pipe_pocket_along_a_line_removes_a_box() {
    let mut p = pipe(0, 6.);
    p.perform().unwrap();
    let res = p.shape().unwrap().clone();
    assert!(is_valid(&res));
    assert_close(volume(&res), 984., 1e-9);
}

#[test]
fn pipe_refuses_a_null_limit() {
    let mut p = pipe(1, 14.);
    assert!(p.perform_until(&Shape::default()).is_err());
    assert!(p.perform_from_until(&Shape::default(), &basis()).is_err());
}

#[test]
fn gluer_fuses_a_small_box_on_the_top() {
    let cube = basis();
    let small = make_box((2., 2., 10.), (2., 2., 2.));
    let top = face_near(&cube, Point3::new(5., 5., 10.));
    let bottom = face_near(&small, Point3::new(3., 3., 10.));
    let mut gluer = BRepFeatGluer::with_shapes(&small, &cube);
    gluer.bind_faces(&bottom, &top).unwrap();
    assert_eq!(gluer.ope_type(), Operation::Fuse);
    assert!(gluer.basis_shape().is_same(&cube));
    assert!(gluer.glued_shape().is_same(&small));
    let res = gluer.shape().unwrap().clone();
    assert!(gluer.is_done());
    assert!(is_valid(&res));
    assert_close(volume(&res), 1008., 1e-9);
    // the top face of the cube is replaced by its piece around the small box
    assert_eq!(gluer.modified(&top).unwrap().len(), 1);
    assert!(!gluer.is_deleted(&top).unwrap());
    assert!(gluer.is_deleted(&bottom).unwrap());
}

#[test]
fn split_shape_cuts_the_top_face_in_halves() {
    let cube = basis();
    let top = face_near(&cube, Point3::new(5., 5., 10.));
    let mut split = BRepFeatSplitShape::with_shape(&cube);
    split
        .add_edge(&segment((5., 0., 10.), (5., 10., 10.)), &top)
        .unwrap();
    let res = split.shape().unwrap().clone();
    assert_eq!(count(&res, ShapeEnum::Face), 7);
    assert!(is_valid(&res));
    let halves = split.modified(&top).unwrap().clone();
    assert_eq!(halves.len(), 2);
    for h in halves.iter() {
        assert_close(area(h), 50., 1e-9);
    }
    assert_eq!(split.direct_left().unwrap().len(), 1);
    let left = split.left().unwrap().len();
    let right = split.right().unwrap().len();
    assert_eq!(left + right, 7);
}

#[test]
fn split_shape_without_shape_refuses_edges() {
    let mut split = BRepFeatSplitShape::new();
    let top = rect_face(0., 0., 1., 1., 0.);
    assert!(
        split
            .add_edge(&segment((0., 0., 0.), (1., 1., 0.)), &top)
            .is_err()
    );
    split.init(&basis());
    split.set_check_interior(true).unwrap();
}
