// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Direct translation of IntAna_IntQuadQuad_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkgeombase::int_ana::{IntQuadQuad, QuadQuadGeo, Quadric, ResultType};
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::gp::{
    Axis2d, Axis3, Circle2, Cone3, Cylinder3, Direction2, Direction3, Line3, Plane3, Point2,
    Point3, Sphere3, StandardDirection3,
};
use std::f64::consts::PI;

/// The IntAna_IntQuadQuad_Test fixture (SetUp).
#[allow(dead_code)]
struct Fixture {
    my_sphere1: Sphere3,
    my_cylinder1: Cylinder3,
    my_cone1: Cone3,
    my_plane1: Plane3,
}

fn set_up() -> Fixture {
    let an_origin = Point3::new(0.0, 0.0, 0.0);
    let a_z_dir = Direction3::from_standard(StandardDirection3::Z);
    let an_x_dir = Direction3::from_standard(StandardDirection3::X);
    let an_axis = Axis3::new(an_origin, a_z_dir, an_x_dir).unwrap();
    let a_plane_origin = Point3::new(0.0, 0.0, 2.0);
    Fixture {
        // Sphere at origin with radius 5.
        my_sphere1: Sphere3::new(an_axis, 5.0).unwrap(),
        // Cylinder with axis along Z, radius 3.
        my_cylinder1: Cylinder3::new(an_axis, 3.0).unwrap(),
        // Cone with semi-angle 30 degrees.
        my_cone1: Cone3::new(an_axis, PI / 6.0, 2.0).unwrap(),
        // Plane at Z = 2.
        my_plane1: Plane3::new(a_plane_origin, a_z_dir).unwrap(),
    }
}

fn frame(origin: Point3, main: StandardDirection3, x: StandardDirection3) -> Axis3 {
    Axis3::new(origin, Direction3::from_standard(main), Direction3::from_standard(x)).unwrap()
}

#[test]
fn IntAna_IntQuadQuad_Test__CylinderVsSphereIntersection() {
    let f = set_up();
    let a_sphere_quad = Quadric::from_sphere(f.my_sphere1);
    let an_intersector = IntQuadQuad::cylinder(f.my_cylinder1, &a_sphere_quad, 1e-7).unwrap();
    assert!(an_intersector.is_done());
    assert!(!an_intersector.identical_elements().unwrap());
    assert!(an_intersector.nb_curve().unwrap() > 0);
    for i in 1..=an_intersector.nb_curve().unwrap() {
        assert!(an_intersector.curve(i).is_ok());
    }
}

#[test]
fn IntAna_IntQuadQuad_Test__NextCurveMethodCorrectness() {
    let f = set_up();
    let a_sphere_quad = Quadric::from_sphere(f.my_sphere1);
    let an_intersector = IntQuadQuad::cylinder(f.my_cylinder1, &a_sphere_quad, 1e-7).unwrap();
    assert!(an_intersector.is_done());
    for i in 1..=an_intersector.nb_curve().unwrap() {
        let a_has_next = an_intersector.has_next_curve(i).unwrap();
        if a_has_next {
            let (a_next_idx, _an_opposite) = an_intersector.next_curve(i).unwrap();
            assert!(a_next_idx > 0);
            assert!(a_next_idx <= an_intersector.nb_curve().unwrap());
            assert_ne!(a_next_idx, i);
        }
    }
}

#[test]
fn IntAna_IntQuadQuad_Test__NextCurveBoundaryConditions() {
    let f = set_up();
    let a_sphere_quad = Quadric::from_sphere(f.my_sphere1);
    let an_intersector = IntQuadQuad::cylinder(f.my_cylinder1, &a_sphere_quad, 1e-7).unwrap();
    assert!(an_intersector.is_done());
    assert_eq!(
        an_intersector.has_next_curve(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let past = an_intersector.nb_curve().unwrap() + 1;
    assert_eq!(
        an_intersector.has_next_curve(past).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    for i in 1..=an_intersector.nb_curve().unwrap() {
        if !an_intersector.has_next_curve(i).unwrap() {
            assert_eq!(
                an_intersector.next_curve(i).unwrap_err().exception_type(),
                "Standard_DomainError"
            );
        }
    }
}

#[test]
fn IntAna_IntQuadQuad_Test__ConnectedCurvesScenario() {
    let _f = set_up();
    let a_center1 = Point3::new(0.0, 0.0, 0.0);
    let a_center2 = Point3::new(3.0, 0.0, 0.0);
    let an_axis1 = frame(a_center1, StandardDirection3::Z, StandardDirection3::X);
    let an_axis2 = frame(a_center2, StandardDirection3::Z, StandardDirection3::X);
    let _a_sphere1 = Sphere3::new(an_axis1, 2.0).unwrap();
    let a_sphere2 = Sphere3::new(an_axis2, 2.0).unwrap();
    let a_sphere2_quad = Quadric::from_sphere(a_sphere2);
    let mut an_intersector = IntQuadQuad::new();
    an_intersector
        .perform_cylinder(Cylinder3::new(an_axis1, 2.0).unwrap(), &a_sphere2_quad, 1e-7)
        .unwrap();
    assert!(an_intersector.is_done());
    assert!(!an_intersector.identical_elements().unwrap());
    for i in 1..=an_intersector.nb_curve().unwrap() {
        if an_intersector.has_next_curve(i).unwrap() {
            let (a_next_idx, _an_opposite) = an_intersector.next_curve(i).unwrap();
            assert!(a_next_idx > 0);
            assert!(a_next_idx <= an_intersector.nb_curve().unwrap());
            if an_intersector.has_next_curve(a_next_idx).unwrap() {
                let (a_reverse_next, _a_reverse_opposite) =
                    an_intersector.next_curve(a_next_idx).unwrap();
                assert!(a_reverse_next > 0);
                assert!(a_reverse_next <= an_intersector.nb_curve().unwrap());
            }
        }
    }
}

#[test]
fn IntAna_IntQuadQuad_Test__IndexingConsistencyTest() {
    let f = set_up();
    let a_sphere_quad = Quadric::from_sphere(f.my_sphere1);
    let an_intersector = IntQuadQuad::cylinder(f.my_cylinder1, &a_sphere_quad, 1e-7).unwrap();
    assert!(an_intersector.is_done());
    for i in 1..=an_intersector.nb_curve().unwrap() {
        if an_intersector.has_next_curve(i).unwrap() {
            let (a_next_idx, _an_opposite) = an_intersector.next_curve(i).unwrap();
            assert!(a_next_idx > 0);
            assert!(a_next_idx <= an_intersector.nb_curve().unwrap());
        }
    }
}

#[test]
fn IntAna_IntQuadQuad_Test__CylinderCylinderNearTangent_CollapsesToSingleLine() {
    let _f = set_up();
    let an_axis1 = frame(Point3::new(0.0, 0.0, 0.0), StandardDirection3::Z, StandardDirection3::X);
    let an_axis2 = frame(
        Point3::new(2.0 - 1.0e-9, 0.0, 0.0),
        StandardDirection3::Z,
        StandardDirection3::X,
    );
    let a_cyl1 = Cylinder3::new(an_axis1, 1.0).unwrap();
    let a_cyl2 = Cylinder3::new(an_axis2, 1.0).unwrap();
    let an_inter = QuadQuadGeo::cylinders(a_cyl1, a_cyl2, 1.0e-4).unwrap();
    assert!(an_inter.is_done());
    assert_eq!(an_inter.type_inter().unwrap(), ResultType::Line);
    assert_eq!(an_inter.nb_solutions().unwrap(), 1);
}

#[test]
fn IntAna_IntQuadQuad_Test__CylinderCylinderNonTangent_HasTwoLines() {
    let _f = set_up();
    let an_axis1 = frame(Point3::new(0.0, 0.0, 0.0), StandardDirection3::Z, StandardDirection3::X);
    let an_axis2 = frame(Point3::new(1.8, 0.0, 0.0), StandardDirection3::Z, StandardDirection3::X);
    let a_cyl1 = Cylinder3::new(an_axis1, 1.0).unwrap();
    let a_cyl2 = Cylinder3::new(an_axis2, 1.0).unwrap();
    let an_inter = QuadQuadGeo::cylinders(a_cyl1, a_cyl2, 1.0e-7).unwrap();
    assert!(an_inter.is_done());
    assert_eq!(an_inter.type_inter().unwrap(), ResultType::Line);
    assert_eq!(an_inter.nb_solutions().unwrap(), 2);
}

#[test]
fn IntAna_IntQuadQuad_Test__CylinderCylinderSkewExternallyTangent_HasPoint() {
    let _f = set_up();
    let a_r1 = 3.0;
    let a_r2 = 2.0;
    let a_dist = a_r1 + a_r2;
    let an_axis1 = frame(Point3::new(0.0, 0.0, 0.0), StandardDirection3::Z, StandardDirection3::X);
    let an_axis2 = frame(Point3::new(0.0, a_dist, 0.0), StandardDirection3::X, StandardDirection3::Y);
    let a_cyl1 = Cylinder3::new(an_axis1, a_r1).unwrap();
    let a_cyl2 = Cylinder3::new(an_axis2, a_r2).unwrap();
    let an_inter = QuadQuadGeo::cylinders(a_cyl1, a_cyl2, 1.0e-7).unwrap();
    assert!(an_inter.is_done());
    assert_eq!(an_inter.type_inter().unwrap(), ResultType::Point);
    assert_eq!(an_inter.nb_solutions().unwrap(), 1);
    let a_p = an_inter.point(1).unwrap();
    assert!((Line3::from_axis(a_cyl1.axis()).distance(a_p) - a_r1).abs() <= 1.0e-7);
    assert!((Line3::from_axis(a_cyl2.axis()).distance(a_p) - a_r2).abs() <= 1.0e-7);
}

#[test]
fn IntAna2d_AnaIntersectionTest__ModDataBug_23939_TangentCircleParameters() {
    let an_x_axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    let a_circle1 = Circle2::from_x_axis(an_x_axis, 10.0, true).unwrap();
    let a_circle2 = Circle2::from_x_axis(
        Axis2d::new(Point2::new(7.0, 0.0), Direction2::new(1.0, 0.0).unwrap()),
        3.0,
        true,
    )
    .unwrap();
    let an_intersection = AnaIntersection::circles(a_circle1, a_circle2).unwrap();
    assert!(an_intersection.is_done());
    assert_eq!(an_intersection.nb_points().unwrap(), 1);
    let point = an_intersection.point(1).unwrap();
    assert!((point.param_on_first() - 0.0).abs() <= 1.0e-15);
    assert!((point.param_on_second().unwrap() - 0.0).abs() <= 1.0e-15);
    assert!((point.value().x() - 10.0).abs() <= 1.0e-12);
    assert!((point.value().y() - 0.0).abs() <= 1.0e-12);

    let a_reversed_intersection = AnaIntersection::circles(a_circle2, a_circle1).unwrap();
    assert!(a_reversed_intersection.is_done());
    assert_eq!(a_reversed_intersection.nb_points().unwrap(), 1);
    let a_reversed_point = a_reversed_intersection.point(1).unwrap();
    assert!((a_reversed_point.param_on_first() - 0.0).abs() <= 1.0e-15);
    assert!((a_reversed_point.param_on_second().unwrap() - 0.0).abs() <= 1.0e-15);
}

#[test]
fn IntAna2d_AnaIntersectionTest__ModDataBug_24375_NearlyCoincidentCircleParameters() {
    let a_circle1 = Circle2::from_x_axis(
        Axis2d::new(Point2::new(10.0, 0.0), Direction2::new(1.0, 0.0).unwrap()),
        100.0,
        true,
    )
    .unwrap();
    let a_circle2 = Circle2::from_x_axis(
        Axis2d::new(Point2::new(10.00000000000001, 0.0), Direction2::new(1.0, 0.0).unwrap()),
        100.0,
        true,
    )
    .unwrap();
    let an_intersection = AnaIntersection::circles(a_circle1, a_circle2).unwrap();
    assert!(an_intersection.is_done());
    assert_eq!(an_intersection.nb_points().unwrap(), 2);
    let an_expected_parameters: [f64; 2] = [PI / 2.0, 3.0 * PI / 2.0];
    for an_index in 1..=2 {
        let point = an_intersection.point(an_index).unwrap();
        let expected = an_expected_parameters[(an_index - 1) as usize];
        assert!((point.param_on_first() - expected).abs() <= 1.0e-7);
        assert!((point.param_on_second().unwrap() - expected).abs() <= 1.0e-7);
    }
}
