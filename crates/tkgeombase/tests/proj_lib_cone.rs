//! Direct translation of ProjLib_Cone_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::precision;
use rustycad_tkgeombase::proj_lib::Cone;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Cone3, Direction3, Point3};
use std::f64::consts::PI;

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

// gp_Cone(gp_Ax2(0, Z), M_PI / 6.0, 5.0).
fn cone() -> Cone3 {
    let axis = Axis2::new(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap();
    Cone3::new(Axis3::from_axis2(axis), PI / 6.0, 5.0).unwrap()
}

fn circle(location: Point3, normal: Direction3, x: Direction3, radius: f64) -> Circle3 {
    Circle3::new(
        Axis2::with_x_direction(location, normal, x).unwrap(),
        radius,
    )
    .unwrap()
}

#[test]
fn ProjLib_ConeTest__ProjectCircle_ParallelAxes_DoneLine() {
    let aCone = cone();
    let aCircle = circle(
        Point3::new(0.0, 0.0, 10.0),
        dir(0.0, 0.0, 1.0),
        dir(1.0, 0.0, 0.0),
        2.0,
    );

    let aProjector = Cone::with_circle(&aCone, &aCircle).unwrap();
    assert!(aProjector.is_done());
    assert_eq!(aProjector.get_type(), CurveType::Line);
    assert!((aProjector.line().unwrap().direction().y() - 0.0).abs() <= precision::angular());
}

#[test]
fn ProjLib_ConeTest__ProjectCircle_NonParallelAxes_NotDone() {
    let aCone = cone();
    let aCircle = circle(
        Point3::new(0.0, 0.0, 10.0),
        dir(1.0, 0.0, 0.0),
        dir(0.0, 1.0, 0.0),
        2.0,
    );

    let aProjector = Cone::with_circle(&aCone, &aCircle).unwrap();
    assert!(!aProjector.is_done());
}

#[test]
fn ProjLib_ConeTest__ProjectCircle_OppositeNormal_DoneLineWithNegativeDirection() {
    let aCone = cone();
    let aCircle = circle(
        Point3::new(0.0, 0.0, 6.0),
        dir(0.0, 0.0, -1.0),
        dir(1.0, 0.0, 0.0),
        2.0,
    );

    let aProjector = Cone::with_circle(&aCone, &aCircle).unwrap();
    assert!(aProjector.is_done());
    assert_eq!(aProjector.get_type(), CurveType::Line);
    assert!(aProjector.line().unwrap().direction().x() < 0.0);
}
