//! Translation of Bnd_OBB_Test.cxx at pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::bnd::{ObbHalfSizes, OrientedBoundingBox};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::{brep_bnd_lib::BRepBndLib, brep_builder_api::BRepBuilderAPIMakeShape};

// GoogleTest EXPECT_DOUBLE_EQ uses a four-ULP comparison.
fn double_eq(a: f64, b: f64) {
    assert!(!a.is_nan() && !b.is_nan());
    assert!(ordered_bits(a).abs_diff(ordered_bits(b)) <= 4, "{a} != {b}");
}
fn ordered_bits(value: f64) -> u64 {
    let bits: u64 = value.to_bits();
    if bits >> 63 != 0 {
        (!bits).wrapping_add(1)
    } else {
        bits | (1 << 63)
    }
}
fn cube(center: Point3, half: f64) -> OrientedBoundingBox {
    let axes = [Direction3::X, Direction3::Y, Direction3::Z];
    OrientedBoundingBox::new(center, axes, half, half, half)
}

/// Upstream `Bnd_OBB_Test.OCC33009_ReBuildWithPoints`.
#[test]
fn Bnd_OBB_Test__OCC33009_ReBuildWithPoints() {
    let mut aBndBox = OrientedBoundingBox::default();
    let mut aPoints = Array1::with_bounds(1, 5).unwrap();
    aPoints[1] = Point3::new(1.0, 2.0, 3.0);
    aPoints[2] = Point3::new(3.0, 2.0, 1.0);
    aPoints[3] = Point3::new(2.0, 3.0, 1.0);
    aPoints[4] = Point3::new(1.0, 3.0, 2.0);
    aPoints[5] = Point3::new(2.0, 1.0, 3.0);
    // Should not throw exception when rebuilding with points
    assert!(aBndBox.rebuild(&aPoints, None, true).is_ok());
}

/// Upstream `Bnd_OBB_Test.OCC30704_AddBoundingBoxToVoidBox`.
///
/// `BRepBndLib::AddOBB(BRepPrimAPI_MakeBox(gp_Pnt(100,100,100),100,100,100), aOBB,
/// false, false, false)` rebuilds the void OBB from the box vertices without tolerances
/// (`CheckPoints`); this now uses those builders instead of the earlier plain-point surrogate.
#[test]
fn Bnd_OBB_Test__OCC30704_AddBoundingBoxToVoidBox() {
    let mut maker =
        BRepPrimAPIMakeBox::at_point(Point3::new(100.0, 100.0, 100.0), 100.0, 100.0, 100.0)
            .unwrap();
    let shape = maker.shape().unwrap();
    let mut aVoidBox = OrientedBoundingBox::default();
    let mut aOBB = OrientedBoundingBox::default();
    BRepBndLib::add_obb(shape, &mut aOBB, false, false, false).unwrap();
    aVoidBox.add_box(&aOBB).unwrap();

    // Check the center point of the bounding box
    let aCenter = aVoidBox.center();
    double_eq(aCenter.x(), 150.0);
    double_eq(aCenter.y(), 150.0);
    double_eq(aCenter.z(), 150.0);
}

/// Upstream `Bnd_OBB_Test.OCC30704_AddPointToVoidBox`.
#[test]
fn Bnd_OBB_Test__OCC30704_AddPointToVoidBox() {
    // A point
    let aP = Point3::new(100.0, 200.0, 300.0);

    // Add the point to a void bounding box
    let mut aVoidBox = OrientedBoundingBox::default();
    aVoidBox.add_point(aP).unwrap();

    // Check the center point of the bounding box
    let aCenter = aVoidBox.center();
    double_eq(aCenter.x(), 100.0);
    double_eq(aCenter.y(), 200.0);
    double_eq(aCenter.z(), 300.0);
}

/// Upstream `Bnd_OBB_Test.Contains_Point`.
#[test]
fn Bnd_OBB_Test__Contains_Point() {
    let anOBB = cube(Point3::new(0.0, 0.0, 0.0), 5.0);
    assert!(anOBB.contains(Point3::new(0.0, 0.0, 0.0)));
    assert!(anOBB.contains(Point3::new(4.0, 4.0, 4.0)));
    assert!(!anOBB.contains(Point3::new(10.0, 0.0, 0.0)));
}

/// Upstream `Bnd_OBB_Test.Intersects_OBB`.
#[test]
fn Bnd_OBB_Test__Intersects_OBB() {
    let anOBB1 = cube(Point3::new(0.0, 0.0, 0.0), 5.0);
    let anOBB2 = cube(Point3::new(8.0, 0.0, 0.0), 5.0);
    let anOBB3 = cube(Point3::new(20.0, 0.0, 0.0), 5.0);
    assert!(anOBB1.intersects(&anOBB2));
    assert!(!anOBB1.intersects(&anOBB3));
}

/// Upstream `Bnd_OBB_Test.GetHalfSizes_StructuredBindings`.
#[test]
fn Bnd_OBB_Test__GetHalfSizes_StructuredBindings() {
    let axes = [Direction3::X, Direction3::Y, Direction3::Z];
    let anOBB = OrientedBoundingBox::new(Point3::new(0.0, 0.0, 0.0), axes, 3.0, 5.0, 7.0);
    let ObbHalfSizes {
        x: aHX,
        y: aHY,
        z: aHZ,
    } = anOBB.half_sizes();
    double_eq(aHX, 3.0);
    double_eq(aHY, 5.0);
    double_eq(aHZ, 7.0);
}
