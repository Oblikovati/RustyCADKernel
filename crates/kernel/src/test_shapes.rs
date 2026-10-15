// SPDX-License-Identifier: GPL-2.0-only
//! Real primitive and profile solids for the property and validation tests.
use crate::properties::MeasuredPoint;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Circle3, Direction3, Ellipse3, Point3, Transform3, Vector3,
};
use rustycad_tkmath::toploc::Location;
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakePrism,
    BRepPrimAPIMakeSphere, BRepPrimAPIMakeTorus,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};

/// `dx x dy x dz` box with a corner at the origin.
pub(crate) fn box_solid(dx: f64, dy: f64, dz: f64) -> Shape {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(dx, dy, dz).unwrap();
    maker.shape().unwrap().clone()
}

/// Cylinder of radius `r` and height `h` on the Z axis.
pub(crate) fn cylinder(r: f64, h: f64) -> Shape {
    BRepPrimAPIMakeCylinder::new(r, h).shape().unwrap().clone()
}

/// Sphere of radius `r` at the origin.
pub(crate) fn sphere(r: f64) -> Shape {
    BRepPrimAPIMakeSphere::new(r)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

/// Cone (or frustum) with base radius `r1`, top radius `r2`, height `h`.
pub(crate) fn cone(r1: f64, r2: f64, h: f64) -> Shape {
    BRepPrimAPIMakeCone::new(r1, r2, h)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

/// Torus of major radius `major` and minor radius `minor` around Z.
pub(crate) fn torus(major: f64, minor: f64) -> Shape {
    BRepPrimAPIMakeTorus::new(major, minor)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

/// The annulus `inner <= |p| <= outer` in z = 0 extruded by `h` along Z.
pub(crate) fn annulus_prism(outer: f64, inner: f64, h: f64) -> Shape {
    let mut maker = BRepBuilderAPIMakeFace::from_wire(&circle_wire(outer), true).unwrap();
    maker
        .add(&topods::wire(&circle_wire(inner).reversed()).unwrap())
        .unwrap();
    let face = maker.face().unwrap();
    prism(&face, h)
}

/// An elliptic cylinder: free-form lateral surface, elliptic planar caps.
pub(crate) fn elliptic_prism(major: f64, minor: f64, h: f64) -> Shape {
    let ellipse = Ellipse3::new(z_frame(), major, minor).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_elips(ellipse)
        .unwrap()
        .edge()
        .unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap();
    let face = BRepBuilderAPIMakeFace::from_wire(&wire, true)
        .unwrap()
        .face()
        .unwrap();
    prism(&face, h)
}

fn prism(face: &Face, h: f64) -> Shape {
    let mut maker =
        BRepPrimAPIMakePrism::new(face, Vector3::new(0.0, 0.0, h), false, true).unwrap();
    maker.shape().unwrap().clone()
}

fn circle_wire(radius: f64) -> rustycad_tkbrep::topods::Wire {
    let circle = Circle3::new(z_frame(), radius).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(circle)
        .unwrap()
        .edge()
        .unwrap();
    BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap()
}

fn z_frame() -> Axis2 {
    Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

/// `shape` rotated by `angle` about the axis through `(1, 2, 3)` along `(1, 1, 1)`,
/// then translated by `(10, -20, 30)`, as a located shape.
pub(crate) fn moved(shape: &Shape, angle: f64) -> Shape {
    let axis = Axis1::new(
        Point3::new(1.0, 2.0, 3.0),
        Direction3::new(1.0, 1.0, 1.0).unwrap(),
    );
    let mut rotation = Transform3::default();
    rotation.set_rotation(axis, angle).unwrap();
    let mut translation = Transform3::default();
    translation.set_translation(Vector3::new(10.0, -20.0, 30.0));
    let location = Location::new(translation.multiplied(rotation));
    shape.moved(&location, true).unwrap()
}

/// Every face occurrence of `shape`.
pub(crate) fn faces(shape: &Shape) -> Vec<Face> {
    let mut found = Vec::new();
    let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    while explorer.more() {
        found.push(topods::face(explorer.current()).unwrap());
        explorer.next().unwrap();
    }
    found
}

/// Asserts `|actual - expected| <= bound` and that the bound is a tight closed-form one.
pub(crate) fn assert_within(actual: f64, bound: f64, expected: f64, what: &str) {
    let error = (actual - expected).abs();
    assert!(
        error <= bound,
        "{what}: |{actual} - {expected}| = {error} > bound {bound}"
    );
    assert!(
        bound <= 1e-11 * (1.0 + expected.abs()),
        "{what}: loose bound {bound}"
    );
}

/// Asserts every coordinate of `point` lies within its bound of `expected`.
pub(crate) fn assert_point_within(point: &MeasuredPoint, expected: [f64; 3], what: &str) {
    for (actual, wanted) in point.point.iter().zip(expected) {
        assert_within(*actual, point.error_bound, wanted, what);
    }
}
