//! Translation of all declarations in ShapeAnalysis_CanonicalRecognition_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871, preserving builders, inputs and assertions.
#![allow(non_snake_case)]
mod canonical_support;
use canonical_support::*;
use rustycad_tkgeomalgo::geom_api::IntSS;

#[test]
fn CanonicalRecognitionApproxTest__PolylineToPlaneRecognition_A1() {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for point in [
        Point3::default(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ] {
        polygon.add_point(point).unwrap();
    }
    polygon.close().unwrap();
    assert!(polygon.is_done());
    let mut recognizer = canonical_recognizer(&polygon.wire().unwrap());
    let mut plane = Plane3::default();
    assert!(recognizer.is_plane(1e-3, &mut plane).unwrap());
    canonical_near(plane.axis().direction().z().abs(), 1.0, 1e-3);
    canonical_near(plane.location().z(), 0.0, 1e-3);
}

#[test]
fn CanonicalRecognitionApproxTest__CylinderRecognition_A2() {
    let axis = Axis2::new(Point3::default(), Direction3::new(1.0, 1.0, 1.0).unwrap()).unwrap();
    let surface =
        Handle::new(GeomCylindricalSurface::new(Axis3::from_axis2(axis), 1.0).unwrap()).cast();
    let shape = canonical_nurbs(&canonical_face(&surface, (0.0, 2.0 * PI, 0.0, 1.0)));
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Cylinder3::default();
    assert!(recognizer.is_cylinder(1e-3, &mut result).unwrap());
    canonical_near(result.radius(), 1.0, 1e-3);
}

#[test]
fn CanonicalRecognitionApproxTest__ConicalSurfaceRecognition_A3() {
    let surface = canonical_cone(2.0);
    let shape = canonical_nurbs(&canonical_face(&surface, (0.0, 2.0 * PI, 0.0, 3.0)));
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Cone3::default();
    assert!(recognizer.is_cone(1e-3, &mut result).unwrap());
    canonical_near(result.semi_angle(), PI / 6.0, 1e-3);
}

#[test]
fn CanonicalRecognitionApproxTest__SphericalSurfaceRecognition_A4() {
    let surface = canonical_sphere();
    let shape = canonical_nurbs(&canonical_face(&surface, (0.0, 2.0 * PI, 0.0, FRAC_PI_2)));
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Sphere3::default();
    assert!(recognizer.is_sphere(1e-3, &mut result).unwrap());
    canonical_near(result.radius(), 1.0, 1e-3);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__TrimmedPlaneRecognition_B1() {
    let surface = canonical_plane();
    let shape = canonical_nurbs(&canonical_face(&surface, (-1.0, 1.0, -1.0, 1.0)));
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Plane3::default();
    assert!(recognizer.is_plane(1e-7, &mut result).unwrap());
    canonical_near(result.axis().direction().z().abs(), 1.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__TrimmedCylinderRecognition_B2() {
    let surface = canonical_cylinder();
    let shape = canonical_nurbs(&canonical_face(&surface, (0.0, 2.0 * PI, -1.0, 1.0)));
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Cylinder3::default();
    assert!(recognizer.is_cylinder(1e-7, &mut result).unwrap());
    canonical_near(result.radius(), 1.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__TrimmedConeRecognition_B3() {
    let surface = canonical_cone(0.0);
    let shape = canonical_nurbs(&canonical_face(&surface, (0.0, 2.0 * PI, -1.0, 0.0)));
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Cone3::default();
    assert!(recognizer.is_cone(1e-7, &mut result).unwrap());
    canonical_near(result.semi_angle(), PI / 6.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseCurveTest__BezierToLineRecognition_A1() {
    let shape = canonical_edge(&canonical_bezier(0.0005));
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Line3::default();
    assert!(recognizer.is_line(1e-3, &mut result).unwrap());
    canonical_near(result.direction().x().abs(), 1.0, 1e-3);
}

#[test]
fn CanonicalRecognitionBaseCurveTest__BezierToCircleRecognition_A2() {
    let shape = canonical_edge(&canonical_bezier(0.0005));
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Circle3::default();
    assert!(recognizer.is_circle(1e-3, &mut result).unwrap());
    assert!(result.radius() > 0.0);
}

#[test]
fn CanonicalRecognitionBaseCurveTest__EllipseToEllipseRecognition_A3() {
    let shape = canonical_edge(&canonical_bspline_conic(true));
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Ellipse3::default();
    assert!(recognizer.is_ellipse(1e-7, &mut result).unwrap());
    canonical_near(result.major_radius(), 1.0, 1e-7);
    canonical_near(result.minor_radius(), 0.5, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseCurveTest__MultiSegmentWireToLineRecognition_A4() {
    let shape = canonical_wire(&canonical_bezier(0.0000005), [0.0, 0.3, 0.7, 1.0]);
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Line3::default();
    assert!(recognizer.is_line(1e-3, &mut result).unwrap());
}

#[test]
fn CanonicalRecognitionBaseCurveTest__MultiSegmentCircleWireRecognition_A5() {
    let shape = canonical_wire(&canonical_bspline_conic(false), [0.0, 1.0, 2.5, 6.0]);
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Circle3::default();
    assert!(recognizer.is_circle(1e-7, &mut result).unwrap());
    canonical_near(result.radius(), 1.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseCurveTest__MultiSegmentEllipseWireRecognition_A6() {
    let shape = canonical_wire(&canonical_bspline_conic(true), [0.0, 1.0, 2.5, 6.0]);
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Ellipse3::default();
    assert!(recognizer.is_ellipse(1e-7, &mut result).unwrap());
    canonical_near(result.major_radius(), 1.0, 1e-7);
    canonical_near(result.minor_radius(), 0.5, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__ConvertedSphereRecognition_B4() {
    let surface = geom_convert::surface_to_bspline_surface(&canonical_sphere())
        .unwrap()
        .cast();
    let maker = BRepBuilderAPIMakeFace::from_surface(&surface, precision::confusion()).unwrap();
    assert!(maker.is_done());
    let shape = maker.face().unwrap();
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Sphere3::default();
    assert!(recognizer.is_sphere(1e-7, &mut result).unwrap());
    canonical_near(result.radius(), 1.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__SewnPlanarSurfaceRecognition_B5() {
    let surface = canonical_plane();
    let shape = canonical_sew(
        &surface,
        [
            (-1.0, 0.0, -1.0, 0.0),
            (-1.0, 0.0, 0.0, 1.0),
            (0.0, 1.0, 0.0, 1.0),
            (0.0, 1.0, -1.0, 0.0),
        ],
    );
    let shape = canonical_nurbs(&shape);
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Plane3::default();
    assert!(recognizer.is_plane(1e-7, &mut result).unwrap());
    canonical_near(result.axis().direction().z().abs(), 1.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__SewnCylindricalSurfaceRecognition_B6() {
    let surface = canonical_cylinder();
    let shape = canonical_sew(
        &surface,
        [
            (0.0, 3.0, -1.0, 0.0),
            (0.0, 3.0, 0.0, 1.0),
            (3.0, 6.0, 0.0, 1.0),
            (3.0, 6.0, -1.0, 0.0),
        ],
    );
    let shape = canonical_nurbs(&shape);
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Cylinder3::default();
    assert!(recognizer.is_cylinder(1e-7, &mut result).unwrap());
    canonical_near(result.radius(), 1.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__SewnConicalSurfaceRecognition_B7() {
    let surface = canonical_cone(0.0);
    let shape = canonical_sew(
        &surface,
        [
            (0.0, 3.0, 0.0, 1.0),
            (0.0, 3.0, 1.0, 2.0),
            (3.0, 6.0, 1.0, 2.0),
            (3.0, 6.0, 0.0, 1.0),
        ],
    );
    let shape = canonical_nurbs(&shape);
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Cone3::default();
    assert!(recognizer.is_cone(1e-7, &mut result).unwrap());
    canonical_near(result.semi_angle(), PI / 6.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__SewnSphericalSurfaceRecognition_B8() {
    let surface = geom_convert::surface_to_bspline_surface(&canonical_sphere())
        .unwrap()
        .cast();
    let shape = canonical_sew(
        &surface,
        [
            (0.0, 3.0, -1.5, 0.0),
            (0.0, 3.0, 0.0, 1.5),
            (3.0, 6.0, 0.0, 1.5),
            (3.0, 6.0, -1.5, 0.0),
        ],
    );
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Sphere3::default();
    assert!(recognizer.is_sphere(1e-7, &mut result).unwrap());
    canonical_near(result.radius(), 1.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__ComplexCylindricalRecognitionWithSection_B9() {
    let surface = canonical_cylinder();
    let shape = canonical_sew(
        &surface,
        [
            (0.0, 1.0, -1.0, 1.0),
            (1.0, 2.0, -1.0, 1.0),
            (2.0, 3.0, -1.0, 1.0),
            (3.0, 4.0, -1.0, 1.0),
        ],
    );
    let shape = canonical_nurbs(&shape);
    // The upstream GTest uses sewn cylindrical faces; its TCL source also used a section.
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Cylinder3::default();
    assert!(recognizer.is_cylinder(1e-7, &mut result).unwrap());
    canonical_near(result.radius(), 1.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__ExtrudedCylindricalSurfaceRecognition_B10() {
    let plane = Plane3::new(Point3::default(), Direction3::new(1.0, 0.0, 1.0).unwrap()).unwrap();
    let surface = Handle::new(GeomPlane::from_pln(plane)).cast();
    let intersection =
        IntSS::with_surfaces(&canonical_cylinder(), &surface, precision::confusion()).unwrap();
    assert!(intersection.is_done());
    assert!(intersection.nb_lines() > 0);
    let curve = intersection.line(1).unwrap();
    assert!(!curve.is_null());
    let extrusion = GeomSurfaceOfLinearExtrusion::new(&curve, dz()).unwrap();
    let (umin, umax, _, _) = extrusion.bounds();
    let surface = Handle::new(extrusion).cast();
    let shape = canonical_nurbs(&canonical_face(&surface, (umin, umax, -1.0, 1.0)));
    let mut recognizer = canonical_recognizer(&shape);
    let mut result = Cylinder3::default();
    assert!(recognizer.is_cylinder(1e-7, &mut result).unwrap());
    canonical_near(result.radius(), 1.0, 1e-7);
}

#[test]
fn CanonicalRecognitionBaseSurfaceTest__PlaneDetectionWithGapValidation_Bug33170() {
    let shape = canonical_face(&canonical_plane(), (-1.0, 1.0, -1.0, 1.0));
    let mut recognizer = canonical_recognizer(&shape);
    let mut plane = Plane3::default();
    assert!(recognizer.is_plane(0.006, &mut plane).unwrap());
    let first_gap = recognizer.get_gap();
    assert!(first_gap < 0.006);
    assert!(first_gap >= 0.0);
    recognizer.clear_status();
    let mut second_plane = Plane3::default();
    assert!(recognizer.is_plane(1.0, &mut second_plane).unwrap());
    let second_gap = recognizer.get_gap();
    assert!(second_gap < 1.0);
    assert!(second_gap >= 0.0);
    assert_eq!(first_gap, second_gap);
}
