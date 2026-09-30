//! Direct translation of GC_MakeArcOfCircle_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::precision;
use rustycad_tkg3d::geom::GeomBoundedCurve;
use rustycad_tkgeombase::gc::{ErrorType, MakeArcOfCircle, Root};
use rustycad_tkmath::gp::{Point3, Vector3};

#[test]
fn GC_MakeArcOfCircleTest__FromPointsAndTangent_Done() {
    let aP1 = Point3::new(1.0, 0.0, 0.0);
    let aP2 = Point3::new(0.0, 1.0, 0.0);
    let aTangentAtP1 = Vector3::new(0.0, 1.0, 0.0);

    let aMaker = MakeArcOfCircle::from_tangent(aP1, aTangentAtP1, aP2).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    assert!(!aMaker.value().unwrap().is_null());
}

#[test]
fn GC_MakeArcOfCircleTest__FromThreePoints_Done() {
    let aMaker = MakeArcOfCircle::through_points(
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
    )
    .unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    assert!(!aMaker.value().unwrap().is_null());
}

#[test]
fn GC_MakeArcOfCircleTest__FromPointsAndTangent_ConfusionScale_Done() {
    let aScale: f64 = 2.0 * precision::confusion();
    let aP1 = Point3::new(aScale, 0.0, 0.0);
    let aP2 = Point3::new(0.0, aScale, 0.0);
    let aTangentAtP1 = Vector3::new(0.0, 1.0, 0.0);

    let aMaker = MakeArcOfCircle::from_tangent(aP1, aTangentAtP1, aP2).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    assert!(!aMaker.value().unwrap().is_null());
    let aStart: Point3 = aMaker.value().unwrap().start_point().unwrap();
    assert!(aStart.is_equal(&aP1, precision::confusion()));
}
