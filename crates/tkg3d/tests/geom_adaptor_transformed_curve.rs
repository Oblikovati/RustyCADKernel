//! Direct translation of GeomAdaptor_TransformedCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! The upstream file includes GeomGridEval_Curve.hxx but no declaration uses it.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{CreateUniformParams, EXPECT_NEAR};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorTransformedCurve;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3, Transform3, TransformForm, Vector3};
use std::f64::consts::PI;

const THE_TOLERANCE: f64 = 1e-10;

// Helper function to create a translation transformation.
fn CreateTranslation(theDx: f64, theDy: f64, theDz: f64) -> Transform3 {
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(theDx, theDy, theDz));
    aTrsf
}

// Helper function to create a rotation transformation around Z axis.
fn CreateRotationZ(theAngleRad: f64) -> Transform3 {
    let mut aTrsf = Transform3::default();
    let anAxis = Axis1::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    aTrsf.set_rotation(anAxis, theAngleRad).unwrap();
    aTrsf
}

fn line(p: Point3, d: Direction3) -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::from_point(p, d)).cast()
}

fn circle_z(radius: f64) -> Handle<dyn GeomCurve> {
    let frame = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    Handle::new(GeomCircle::new(frame, radius).unwrap()).cast()
}

fn x_line() -> Handle<dyn GeomCurve> {
    line(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    )
}

//==================================================================================================
// Tests for GeomAdaptor_TransformedCurve construction
//==================================================================================================

#[test]
fn GeomAdaptor_TransformedCurveTest__DefaultConstructor() {
    let aCurve = GeomAdaptorTransformedCurve::default();
    assert!(aCurve.is_3d_curve());
    assert!(!aCurve.is_curve_on_surface());
    assert_eq!(aCurve.trsf().form(), TransformForm::Identity);
}

#[test]
fn GeomAdaptor_TransformedCurveTest__ConstructWithCurveAndTrsf() {
    let aLine = x_line();
    let aTrsf = CreateTranslation(0.0, 0.0, 5.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aLine, aTrsf).unwrap();
    assert!(aCurve.is_3d_curve());
    assert_eq!(aCurve.get_type().unwrap(), CurveType::Line);
    assert_eq!(aCurve.trsf().form(), TransformForm::Translation);
}

#[test]
fn GeomAdaptor_TransformedCurveTest__ConstructWithBounds() {
    let aLine = x_line();
    let aTrsf = CreateTranslation(0.0, 0.0, 5.0);

    let aCurve = GeomAdaptorTransformedCurve::with_range(&aLine, 0.0, 10.0, aTrsf).unwrap();
    EXPECT_NEAR(aCurve.first_parameter().unwrap(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aCurve.last_parameter().unwrap(), 10.0, THE_TOLERANCE);
}

//==================================================================================================
// Tests for evaluation with identity transform
//==================================================================================================

#[test]
fn GeomAdaptor_TransformedCurveTest__Line_IdentityTransform() {
    let aLine = line(
        Point3::new(1.0, 2.0, 3.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let aTrsf = Transform3::default(); // identity

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aLine, aTrsf).unwrap();

    // With identity transform, results should match direct Geom_Line evaluation
    let aT: f64 = 5.0;
    let aPnt: Point3 = aCurve.value(aT).unwrap();
    let aExpected: Point3 = aLine.value(aT).unwrap();
    EXPECT_NEAR(aPnt.distance(&aExpected), 0.0, THE_TOLERANCE);
}

#[test]
fn GeomAdaptor_TransformedCurveTest__Circle_IdentityTransform() {
    let aCircle = circle_z(2.0);
    let aTrsf = Transform3::default(); // identity

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aCircle, aTrsf).unwrap();
    assert_eq!(aCurve.get_type().unwrap(), CurveType::Circle);

    let aParams: [f64; 5] = [0.0, PI / 2.0, PI, 3.0 * PI / 2.0, 2.0 * PI];

    for aParam in aParams {
        let aPnt: Point3 = aCurve.value(aParam).unwrap();
        let aExpected: Point3 = aCircle.value(aParam).unwrap();
        EXPECT_NEAR(aPnt.distance(&aExpected), 0.0, THE_TOLERANCE);
    }
}

//==================================================================================================
// Tests for evaluation with non-identity transform
//==================================================================================================

#[test]
fn GeomAdaptor_TransformedCurveTest__Line_Translation() {
    let aLine = x_line();
    let aTrsf = CreateTranslation(0.0, 0.0, 5.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aLine, aTrsf).unwrap();

    // Line along X at Z=0, translated to Z=5
    let aPnt: Point3 = aCurve.value(3.0).unwrap();
    EXPECT_NEAR(aPnt.x(), 3.0, THE_TOLERANCE);
    EXPECT_NEAR(aPnt.y(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aPnt.z(), 5.0, THE_TOLERANCE);
}

#[test]
fn GeomAdaptor_TransformedCurveTest__Line_Rotation() {
    let aLine = x_line();
    let aTrsf = CreateRotationZ(PI / 2.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aLine, aTrsf).unwrap();

    // Line along X rotated 90 degrees -> line along Y
    let aPnt: Point3 = aCurve.value(3.0).unwrap();
    EXPECT_NEAR(aPnt.x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aPnt.y(), 3.0, THE_TOLERANCE);
    EXPECT_NEAR(aPnt.z(), 0.0, THE_TOLERANCE);
}

#[test]
fn GeomAdaptor_TransformedCurveTest__Circle_Translation() {
    let aCircle = circle_z(2.0);
    let aTrsf = CreateTranslation(10.0, 20.0, 30.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aCircle, aTrsf).unwrap();

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 17);

    for i in 1..=17 {
        let aPnt: Point3 = aCurve.value(aParams[i]).unwrap();
        let aLocal: Point3 = aCircle.value(aParams[i]).unwrap();
        let aExpected = Point3::new(aLocal.x() + 10.0, aLocal.y() + 20.0, aLocal.z() + 30.0);
        EXPECT_NEAR(aPnt.distance(&aExpected), 0.0, THE_TOLERANCE);
    }
}

//==================================================================================================
// Tests for derivative evaluation with transform
//==================================================================================================

#[test]
fn GeomAdaptor_TransformedCurveTest__D1_Translation() {
    let aCircle = circle_z(2.0);
    let aTrsf = CreateTranslation(10.0, 20.0, 30.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aCircle, aTrsf).unwrap();

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    for i in 1..=9 {
        let (aPnt, aD1) = aCurve.d1(aParams[i]).unwrap();

        // Point should be translated
        let aLocalPnt: Point3 = aCircle.value(aParams[i]).unwrap();
        let aExpectedPnt = Point3::new(
            aLocalPnt.x() + 10.0,
            aLocalPnt.y() + 20.0,
            aLocalPnt.z() + 30.0,
        );
        EXPECT_NEAR(aPnt.distance(&aExpectedPnt), 0.0, THE_TOLERANCE);

        // D1 vector: translation doesn't change vectors, so should match local D1
        let (_aLocalP, aLocalD1) = aCircle.d1(aParams[i]).unwrap();
        EXPECT_NEAR((aD1 - aLocalD1).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomAdaptor_TransformedCurveTest__D1_Rotation() {
    let aLine = x_line();
    let aTrsf = CreateRotationZ(PI / 2.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aLine, aTrsf).unwrap();

    let (_aPnt, aD1) = aCurve.d1(3.0).unwrap();

    // D1 for line along X is (1,0,0), rotated 90 degrees becomes (0,1,0)
    EXPECT_NEAR(aD1.x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aD1.y(), 1.0, THE_TOLERANCE);
    EXPECT_NEAR(aD1.z(), 0.0, THE_TOLERANCE);
}

#[test]
fn GeomAdaptor_TransformedCurveTest__D2_Translation() {
    let aCircle = circle_z(2.0);
    let aTrsf = CreateTranslation(10.0, 20.0, 30.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aCircle, aTrsf).unwrap();

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    for i in 1..=9 {
        let (_aPnt, aD1, aD2) = aCurve.d2(aParams[i]).unwrap();

        let (_aLocalP, aLocalD1, aLocalD2) = aCircle.d2(aParams[i]).unwrap();

        EXPECT_NEAR((aD1 - aLocalD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aD2 - aLocalD2).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomAdaptor_TransformedCurveTest__D3_Rotation() {
    let aCircle = circle_z(2.0);
    let aTrsf = CreateRotationZ(PI / 4.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aCircle, aTrsf).unwrap();

    let aParams = CreateUniformParams(0.0, 2.0 * PI, 9);

    for i in 1..=9 {
        let (aPnt, aD1, aD2, aD3) = aCurve.d3(aParams[i]).unwrap();

        // Get local values and transform manually
        let (aLocalP, aLocalD1, aLocalD2, aLocalD3) = aCircle.d3(aParams[i]).unwrap();
        let aLocalP = aLocalP.transformed(aTrsf);
        let aLocalD1 = aLocalD1.transformed(aTrsf);
        let aLocalD2 = aLocalD2.transformed(aTrsf);
        let aLocalD3 = aLocalD3.transformed(aTrsf);

        EXPECT_NEAR(aPnt.distance(&aLocalP), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aD1 - aLocalD1).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aD2 - aLocalD2).magnitude(), 0.0, THE_TOLERANCE);
        EXPECT_NEAR((aD3 - aLocalD3).magnitude(), 0.0, THE_TOLERANCE);
    }
}

#[test]
fn GeomAdaptor_TransformedCurveTest__DN_Rotation() {
    let aCircle = circle_z(2.0);
    let aTrsf = CreateRotationZ(PI / 3.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aCircle, aTrsf).unwrap();

    for aOrder in 1..=3 {
        let aResult: Vector3 = aCurve.dn(1.0, aOrder).unwrap();
        let aExpected: Vector3 = aCircle.dn(1.0, aOrder).unwrap().transformed(aTrsf);
        EXPECT_NEAR((aResult - aExpected).magnitude(), 0.0, THE_TOLERANCE);
    }
}

//==================================================================================================
// Tests for geometry extraction with transform
//==================================================================================================

#[test]
fn GeomAdaptor_TransformedCurveTest__LineExtraction_Transform() {
    let aLine = x_line();
    let aTrsf = CreateTranslation(0.0, 0.0, 5.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aLine, aTrsf).unwrap();

    let aExtracted = aCurve.line().unwrap();

    // Line should be translated: origin at (0,0,5), direction unchanged
    EXPECT_NEAR(aExtracted.location().x(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aExtracted.location().y(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aExtracted.location().z(), 5.0, THE_TOLERANCE);
    EXPECT_NEAR(aExtracted.direction().x(), 1.0, THE_TOLERANCE);
}

#[test]
fn GeomAdaptor_TransformedCurveTest__CircleExtraction_Transform() {
    let aCircle = circle_z(2.0);
    let aTrsf = CreateTranslation(10.0, 0.0, 0.0);

    let aCurve = GeomAdaptorTransformedCurve::with_curve(&aCircle, aTrsf).unwrap();

    let aExtracted = aCurve.circle().unwrap();

    // Circle center should be translated to (10,0,0)
    EXPECT_NEAR(aExtracted.location().x(), 10.0, THE_TOLERANCE);
    EXPECT_NEAR(aExtracted.location().y(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aExtracted.location().z(), 0.0, THE_TOLERANCE);
    EXPECT_NEAR(aExtracted.radius(), 2.0, THE_TOLERANCE);
}

//==================================================================================================
// Tests for ShallowCopy
//==================================================================================================

#[test]
fn GeomAdaptor_TransformedCurveTest__ShallowCopy() {
    let aLine = x_line();
    let aTrsf = CreateTranslation(0.0, 0.0, 5.0);

    let aCurve = GeomAdaptorTransformedCurve::with_range(&aLine, 0.0, 10.0, aTrsf).unwrap();

    let aCopy: Handle<dyn Adaptor3dCurve> = aCurve.shallow_copy().unwrap();
    assert!(!aCopy.is_null());

    // Copy should evaluate the same
    let aPnt1: Point3 = aCurve.value(3.0).unwrap();
    let aPnt2: Point3 = aCopy.value(3.0).unwrap();
    EXPECT_NEAR(aPnt1.distance(&aPnt2), 0.0, THE_TOLERANCE);
}
