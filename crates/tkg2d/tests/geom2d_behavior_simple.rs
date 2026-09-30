//! Geom2d_Point, Geom2d_Vector and Geom2d_Transformation members.
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dCartesianPoint, Geom2dDirection, Geom2dPoint, Geom2dTransformation, Geom2dVector,
    Geom2dVectorWithMagnitude,
};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, TransformForm};
use std::f64::consts::FRAC_PI_2;

#[test]
fn point_distances_through_the_point_interface() {
    let a: Handle<dyn Geom2dPoint> = Handle::new(Geom2dCartesianPoint::new(1.0, 1.0)).cast();
    let b = Handle::new(Geom2dCartesianPoint::from_pnt2d(Point2::new(4.0, 5.0)));
    assert_eq!((a.distance(&*b), a.square_distance(&*b)), (5.0, 25.0));
    assert_eq!((a.coord(), a.x(), a.y()), ((1.0, 1.0), 1.0, 1.0));
    let copy: Handle<dyn Geom2dPoint> = a.copy().unwrap().cast();
    assert_eq!(copy.pnt2d(), Point2::new(1.0, 1.0));
}

#[test]
fn vector_interface_members() {
    let v: Handle<dyn Geom2dVector> = Handle::new(Geom2dVectorWithMagnitude::new(3.0, 4.0)).cast();
    let d: Handle<dyn Geom2dVector> = Handle::new(Geom2dDirection::new(0.0, 2.0).unwrap()).cast();
    assert_eq!(
        (v.coord(), v.x(), v.y(), v.magnitude()),
        ((3.0, 4.0), 3.0, 4.0, 5.0)
    );
    assert_eq!(
        (v.dot(&*d), v.crossed(&*d), d.crossed(&*v)),
        (4.0, 3.0, -3.0)
    );
    EXPECT_NEAR(
        d.angle(&*Handle::new(Geom2dDirection::new(1.0, 0.0).unwrap()))
            .unwrap(),
        -FRAC_PI_2,
        1e-15,
    );
    let zero = Handle::new(Geom2dVectorWithMagnitude::new(0.0, 0.0));
    assert_eq!(
        v.angle(&*zero).unwrap_err().exception_type(),
        "gp_VectorWithNullMagnitude"
    );
    let reversed = v.reversed().unwrap();
    assert_eq!(reversed.coord(), (-3.0, -4.0));
    assert_eq!(v.coord(), (3.0, 4.0));
    v.reverse();
    assert_eq!(v.coord(), (-3.0, -4.0));
    assert_eq!((d.magnitude(), d.square_magnitude()), (1.0, 1.0));
    let error = Geom2dDirection::new(0.0, 0.0).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let unit = Geom2dDirection::from_dir2d(Direction2::new(1.0, 0.0).unwrap());
    assert_eq!(
        unit.set_coord(0.0, 0.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert!(unit.set_x(0.0).is_err() && unit.set_y(0.0).is_ok());
}

#[test]
fn transformation_members() {
    let t = Handle::new(Geom2dTransformation::default());
    assert_eq!(
        (t.form(), t.scale_factor(), t.is_negative()),
        (TransformForm::Identity, 1.0, false)
    );
    let from = Axis2d::new(Point2::new(1.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    let to = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(0.0, 1.0).unwrap());
    // Expected coordinates from the native oracle (`g2_trsf` kinds 4 and 5 transform (1.25, -3.5)).
    t.set_transformation_between(from, to);
    assert_eq!(t.transforms(1.25, -3.5), (-3.5, -2.25));
    t.set_transformation_to(from);
    assert_eq!(t.transforms(1.25, -3.5), (0.25, -3.5));
    t.set_scale(Point2::new(0.0, 0.0), 2.0);
    t.power(-1).unwrap();
    assert_eq!(t.scale_factor(), 0.5);
    let doubled = t.powered(-2).unwrap();
    assert_eq!(doubled.scale_factor(), 4.0);
    let o = Handle::new(Geom2dTransformation::default());
    o.set_translation_between(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    t.pre_multiply(&o);
    assert_eq!(t.transforms(2.0, 2.0), (2.0, 1.0));
    t.set_mirror_axis(from);
    assert!(t.is_negative());
    assert_eq!(t.value(1, 1).unwrap(), 1.0);
    assert_eq!(
        t.value(3, 1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    t.set_scale(Point2::new(0.0, 0.0), 0.0);
    assert!(t.inverted().is_err() && t.invert().is_err());
    assert!(t.power(-1).is_err());
}
