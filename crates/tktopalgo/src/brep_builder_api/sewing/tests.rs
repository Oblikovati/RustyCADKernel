//! Tests of the protected members that no public member reaches: `EvaluateAngulars`
//! (never called natively) on the free boundaries of two faces.
use super::BRepBuilderAPISewing;
use super::failures::array1;
use crate::brep_lib::{BRepLibMakeFace, BRepLibMakePolygon};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkmath::gp::Point3;
use std::f64::consts::FRAC_PI_2;

/// The planar face of a closed polygon of four points.
fn quad(points: [[f64; 3]; 4]) -> Shape {
    let p = |c: [f64; 3]| Point3::new(c[0], c[1], c[2]);
    let polygon = BRepLibMakePolygon::from_four_points(
        p(points[0]),
        p(points[1]),
        p(points[2]),
        p(points[3]),
        true,
    )
    .unwrap();
    BRepLibMakeFace::from_wire(&polygon.wire().unwrap(), true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

#[test]
fn evaluate_angulars_measures_the_angle_between_the_faces() {
    let floor = quad([[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]]);
    let wall = quad([[0., 0., 0.], [0., 0., 1.], [1., 0., 1.], [1., 0., 0.]]);
    let mut sewing = BRepBuilderAPISewing::default();
    sewing.add(&floor).unwrap();
    sewing.add(&wall).unwrap();
    sewing.find_free_boundaries().unwrap();
    // The two boundaries on the x axis: the first edge of each face.
    let mut sections: Sequence<Shape> = Sequence::default();
    sections.append(sewing.my_bound_faces.find_key(1).unwrap().clone());
    sections.append(sewing.my_bound_faces.find_key(5).unwrap().clone());
    let forward = array1(2, true).unwrap();
    let mut angles = array1(2, 0.0).unwrap();
    sewing
        .evaluate_angulars(&sections, &forward, &mut angles, 1)
        .unwrap();
    assert_eq!(angles[1], -1.0);
    assert!((angles[2] - FRAC_PI_2).abs() < 1e-12);
}
