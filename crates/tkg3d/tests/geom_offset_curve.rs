//! Direct translation of Geom_OffsetCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_DOUBLE_EQ;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomGeometry, GeomOffsetCurve};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3, StandardDirection3};

// Geom_OffsetCurve_Test::SetUp: offset by 2 along Z of a circle of radius 5.
fn setup() -> Handle<GeomOffsetCurve> {
    let aCircle = Circle3::new(
        Axis2::with_standard(Point3::new(0.0, 0.0, 0.0), StandardDirection3::Z),
        5.0,
    )
    .unwrap();
    let aBasisCurve = Handle::new(GeomCircle::from_circ(aCircle));
    let anOffsetDir = Direction3::from_standard(StandardDirection3::Z);
    let anOffsetValue: f64 = 2.0;
    Handle::new(
        GeomOffsetCurve::new(&aBasisCurve.cast(), anOffsetValue, anOffsetDir, false).unwrap(),
    )
}

#[test]
fn Geom_OffsetCurve_Test__CopyConstructorBasicProperties() {
    let myOriginalCurve = setup();
    // Test copy constructor
    let aCopiedCurve = Handle::new(GeomOffsetCurve::copy_of(&myOriginalCurve).unwrap());

    // Verify basic properties are identical
    EXPECT_DOUBLE_EQ(myOriginalCurve.offset(), aCopiedCurve.offset());
    assert!(
        myOriginalCurve
            .direction()
            .is_equal(&aCopiedCurve.direction(), 1e-10)
    );
    assert_eq!(myOriginalCurve.is_periodic(), aCopiedCurve.is_periodic());
    assert_eq!(
        myOriginalCurve.is_closed().unwrap(),
        aCopiedCurve.is_closed().unwrap()
    );
}

#[test]
fn Geom_OffsetCurve_Test__CopyConstructorBasisCurve() {
    let myOriginalCurve = setup();
    let aCopiedCurve = Handle::new(GeomOffsetCurve::copy_of(&myOriginalCurve).unwrap());

    // Verify basis curves are equivalent but independent
    let anOrigBasis: Handle<dyn GeomCurve> = myOriginalCurve.basis_curve();
    let aCopyBasis: Handle<dyn GeomCurve> = aCopiedCurve.basis_curve();

    // They should be different objects
    assert_ne!(anOrigBasis.object_address(), aCopyBasis.object_address());

    // But functionally equivalent
    EXPECT_DOUBLE_EQ(anOrigBasis.first_parameter(), aCopyBasis.first_parameter());
    EXPECT_DOUBLE_EQ(anOrigBasis.last_parameter(), aCopyBasis.last_parameter());
}

#[test]
fn Geom_OffsetCurve_Test__CopyMethodUsesOptimizedConstructor() {
    let myOriginalCurve = setup();
    // Test that Copy() method uses the optimized copy constructor
    let aCopiedGeom: Handle<dyn GeomGeometry> = myOriginalCurve.copy().unwrap();
    let aCopiedCurve: Handle<GeomOffsetCurve> = aCopiedGeom.cast();

    assert!(!aCopiedCurve.is_null());

    // Verify the copy is functionally identical
    EXPECT_DOUBLE_EQ(myOriginalCurve.offset(), aCopiedCurve.offset());

    // Test evaluation at several points
    let anUFirst: f64 = myOriginalCurve.first_parameter();
    let anULast: f64 = myOriginalCurve.last_parameter();
    let aStep: f64 = (anULast - anUFirst) / 4.0;

    let mut u: f64 = anUFirst;
    while u <= anULast {
        let anOrigPnt: Point3 = myOriginalCurve.value(u).unwrap();
        let aCopyPnt: Point3 = aCopiedCurve.value(u).unwrap();
        assert!(anOrigPnt.is_equal(&aCopyPnt, 1e-10));
        u += aStep;
    }
}

#[test]
fn Geom_OffsetCurve_Test__CopyIndependence() {
    let myOriginalCurve = setup();
    let aCopiedCurve = Handle::new(GeomOffsetCurve::copy_of(&myOriginalCurve).unwrap());

    // Store original offset value
    let anOrigOffset: f64 = aCopiedCurve.offset();

    // Modify the original curve
    myOriginalCurve.set_offset_value(10.0);

    // Verify the copied curve is not affected
    EXPECT_DOUBLE_EQ(aCopiedCurve.offset(), anOrigOffset);
    assert_ne!(aCopiedCurve.offset(), myOriginalCurve.offset());
}
