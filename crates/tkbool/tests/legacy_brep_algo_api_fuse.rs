//! Direct translation of src/ModelingAlgorithms/TKBool/GTests/BRepAlgoAPI_Fuse_Test.cxx (3d097a0).
#![allow(non_snake_case)]

use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICut, BRepAlgoAPIFuse};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeRevol,
    BRepPrimAPIMakeSphere, BRepPrimAPIMakeTorus,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use std::f64::consts::PI;

/// `EXPECT_NEAR(actual, expected, tolerance)`.
fn expect_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected} by more than {tolerance}"
    );
}

/// `gp_Dir(x, y, z)`.
fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

/// `gp_Ax2(P, D)`.
fn ax2(p: Point3, d: Direction3) -> Axis2 {
    Axis2::new(p, d).unwrap()
}

/// The checks closing every test: surface area within 0.1% and a valid shape.
fn CheckAreaAndValidity(a_result: &Shape, area: f64, tolerance: f64) {
    assert!(!a_result.is_null());
    let mut a_sprops = GProps::default();
    BRepGProp::surface_properties(a_result, &mut a_sprops, false, false).unwrap();
    expect_near(a_sprops.mass(), area, tolerance);
    let a_checker = BRepCheckAnalyzer::new(a_result, true, false, false).unwrap();
    assert!(a_checker.is_valid().unwrap());
}

/// The fuse of two shapes, required done.
fn Fused(a: &Shape, b: &Shape) -> Shape {
    let a_fuse = BRepAlgoAPIFuse::from_shapes(a, b).unwrap();
    assert!(a_fuse.is_done());
    a_fuse.shape().clone()
}

/// The planar face of a closed polygon through the points (`BRepBuilderAPI_MakeFace(W,
/// false)`).
fn PolygonFace(points: &[(f64, f64, f64)]) -> Shape {
    let mut a_wire = BRepBuilderAPIMakePolygon::new();
    for &(x, y, z) in points {
        a_wire.add_point(Point3::new(x, y, z)).unwrap();
    }
    BRepBuilderAPIMakeFace::from_wire(&a_wire.wire().unwrap(), false)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

/// `BRepPrimAPI_MakeRevol(F, A, 2 PI).Shape()`.
fn Revolved(face: &Shape, axis: Axis1) -> Shape {
    BRepPrimAPIMakeRevol::new(face, axis, 2.0 * PI, false)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

/// OCC822_1: fuses two pairs (inner/outer) of cylinders and cones, then cuts inner from
/// outer.
#[test]
fn cylinder_and_cone_fuse_then_cut() {
    let an_axis1 = ax2(Point3::new(0., 0., 0.), Direction3::Z);
    let a_cyl_in = BRepPrimAPIMakeCylinder::with_axes(an_axis1, 40., 110.)
        .shape()
        .unwrap()
        .clone();
    let a_cyl_out = BRepPrimAPIMakeCylinder::with_axes(an_axis1, 50., 100.)
        .shape()
        .unwrap()
        .clone();
    let an_axis2 = ax2(Point3::new(0., 0., 0.), dir(0., 0., -1.));
    let a_con_in = BRepPrimAPIMakeCone::with_axes(an_axis2, 40., 60., 110.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let a_con_out = BRepPrimAPIMakeCone::with_axes(an_axis2, 50., 70., 100.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let a_fuse_in = Fused(&a_cyl_in, &a_con_in);
    let a_fuse_out = Fused(&a_cyl_out, &a_con_out);
    let a_cut = BRepAlgoAPICut::from_shapes(&a_fuse_out, &a_fuse_in).unwrap();
    assert!(a_cut.is_done());
    CheckAreaAndValidity(a_cut.shape(), 133931.0, 133.931);
}

/// OCC822_2: fuse of box and sphere.
#[test]
fn box_and_sphere() {
    let a_box =
        BRepPrimAPIMakeBox::from_points(Point3::new(0., 0., 0.), Point3::new(100., 100., 100.))
            .unwrap()
            .shape()
            .unwrap()
            .clone();
    let a_sphere = BRepPrimAPIMakeSphere::at_center(Point3::new(100., 50., 50.), 25.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    CheckAreaAndValidity(&Fused(&a_box, &a_sphere), 61963.5, 61.9635);
}

/// OCC823: fuse of two nearly-parallel cylinders.
#[test]
fn two_cylinders() {
    let an_axis1 = ax2(Point3::new(40., 50., 0.), dir(100., 0., 0.));
    let a_cyl1 = BRepPrimAPIMakeCylinder::with_axes(an_axis1, 20., 100.)
        .shape()
        .unwrap()
        .clone();
    let a_size = 0.001;
    let an_axis2 = ax2(Point3::new(100., 50., a_size), dir(0., a_size, 80.));
    let a_cyl2 = BRepPrimAPIMakeCylinder::with_axes(an_axis2, 20., 80.)
        .shape()
        .unwrap()
        .clone();
    CheckAreaAndValidity(&Fused(&a_cyl2, &a_cyl1), 23189.5, 23.1895);
}

/// OCC824: fuse of cylinder and sphere.
#[test]
fn cylinder_and_sphere() {
    let a_center = Point3::new(100., 0., 0.);
    let an_axis = ax2(a_center, dir(-1., 0., 0.));
    let a_cyl = BRepPrimAPIMakeCylinder::with_axes(an_axis, 20., 100.)
        .shape()
        .unwrap()
        .clone();
    let a_sphere = BRepPrimAPIMakeSphere::at_center(a_center, 20.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    CheckAreaAndValidity(&Fused(&a_cyl, &a_sphere), 16336.3, 16.3363);
}

/// OCC826: fuse of revolved face and sphere; the intersection line passes near the pole
/// of the sphere.
#[test]
fn revolved_face_and_sphere() {
    let (a_x1, a_x2) = (181.82808, 202.39390);
    let (a_y1, a_y2) = (31.011970, 123.06856);
    let a_face = PolygonFace(&[
        (a_x1, a_y1, 0.),
        (a_x2, a_y1, 0.),
        (a_x2, a_y2, 0.),
        (a_x1, a_y2, 0.),
        (a_x1, a_y1, 0.),
    ]);
    let a_revol = Revolved(
        &a_face,
        Axis1::new(Point3::new(0., 0., 0.), dir(0., 30., 0.)),
    );
    let a_sphere =
        BRepPrimAPIMakeSphere::at_center(Point3::new(166.373, 77.0402, 96.0555), 23.218586)
            .unwrap()
            .shape()
            .unwrap()
            .clone();
    CheckAreaAndValidity(&Fused(&a_revol, &a_sphere), 272935.0, 272.935);
}

/// OCC827: fuse of revolved solid with two tori.
#[test]
fn revolved_solid_and_two_tori() {
    let a_face = PolygonFace(&[
        (10., 0., 0.),
        (20., 0., 0.),
        (20., 0., 50.),
        (10., 0., 50.),
        (10., 0., 0.),
    ]);
    let a_revol = Revolved(
        &a_face,
        Axis1::new(Point3::new(0., 0., 0.), dir(0., 0., 30.)),
    );
    let (a_maj_rad, a_min_rad) = (15.0, 5.0);
    let torus = |z: f64| {
        BRepPrimAPIMakeTorus::with_axes(
            ax2(Point3::new(0., 0., z), dir(0., 0., 30.)),
            a_maj_rad,
            a_min_rad,
        )
        .unwrap()
        .shape()
        .unwrap()
        .clone()
    };
    let (a_tor1, a_tor2) = (torus(50.), torus(10.));
    let a_fuse1 = Fused(&a_tor1, &a_revol);
    CheckAreaAndValidity(&Fused(&a_tor2, &a_fuse1), 11847.7, 11.8477);
}
