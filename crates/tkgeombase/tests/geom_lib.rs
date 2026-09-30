//! Direct translation of GeomLib_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_DOUBLE_EQ;
use rustycad_tkernel::ncollection::{Array1, Array2, LinearVector};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomBoundedCurve, GeomBoundedSurface,
    GeomCurve, GeomPlane, GeomSphericalSurface, GeomSurface,
};
use rustycad_tkgeombase::geom_lib as GeomLib;
use rustycad_tkmath::gp::{
    self, Axis2, Axis3, Direction2, Direction3, GeneralTransform2, Matrix2, Point2, Point3,
    Vector3, Xy,
};

// NCollection_Array2<T>(1, rows, 1, cols).
fn grid<T: Default + Clone>(rows: i32, cols: i32) -> Array2<'static, T> {
    Array2::with_bounds(1, rows, 1, cols).unwrap()
}

fn set<T>(array: &mut Array2<'static, T>, u: usize, v: usize, value: T) {
    *array.change_at(u, v).unwrap() = value;
}

fn knots01(mult: i32) -> (Array1<'static, f64>, Array1<'static, i32>) {
    let mut knots: Array1<'static, f64> = one_based(2).unwrap();
    (knots[1], knots[2]) = (0.0, 1.0);
    let mut mults: Array1<'static, i32> = one_based(2).unwrap();
    mults.init(mult);
    (knots, mults)
}

fn makeSurface(theRational: bool) -> Handle<GeomBSplineSurface> {
    let mut aPoles: Array2<'static, Point3> = grid(3, 3);
    let mut aWeights: Array2<'static, f64> = grid(3, 3);
    for aUIndex in 0..aPoles.row_size() {
        let aX: f64 = 0.5 * aUIndex as f64;
        for aVIndex in 0..aPoles.col_size() {
            let aY: f64 = 0.5 * aVIndex as f64;
            set(
                &mut aPoles,
                aUIndex,
                aVIndex,
                Point3::new(aX, aY, aX * aX + 0.2 * aX * aY),
            );
            let w: f64 = if theRational && aUIndex == 1 && aVIndex == 1 {
                1.75
            } else {
                1.0
            };
            set(&mut aWeights, aUIndex, aVIndex, w);
        }
    }
    let (aUKnots, aUMults) = knots01(3);
    let (aVKnots, aVMults) = knots01(3);
    let u = (&aUKnots, &aUMults);
    let v = (&aVKnots, &aVMults);
    if theRational {
        return Handle::new(
            GeomBSplineSurface::with_weights(&aPoles, &aWeights, u, v, (2, 2), (false, false))
                .unwrap(),
        );
    }
    Handle::new(GeomBSplineSurface::new(&aPoles, u, v, (2, 2), (false, false)).unwrap())
}

fn makeRationalBoundarySurface(theProjectivelyEqual: bool) -> Handle<GeomBSplineSurface> {
    let mut aPoles: Array2<'static, Point3> = grid(2, 3);
    let mut aWeights: Array2<'static, f64> = grid(2, 3);
    let aFirstWeights: [f64; 3] = [1.0, 2.0, 1.0];
    let aLastWeights: [f64; 3] = [2.0, if theProjectivelyEqual { 4.0 } else { 1.0 }, 2.0];
    for aVIndex in 0..aPoles.col_size() {
        let aFirstPole = Point3::new(1.0, 1.0 + aVIndex as f64, 0.2 * aVIndex as f64);
        set(&mut aPoles, 0, aVIndex, aFirstPole);
        set(&mut aWeights, 0, aVIndex, aFirstWeights[aVIndex]);
        set(&mut aWeights, 1, aVIndex, aLastWeights[aVIndex]);
        let last = if theProjectivelyEqual {
            aFirstPole
        } else {
            let xyz = aFirstPole.xyz() * (aFirstWeights[aVIndex] / aLastWeights[aVIndex]);
            Point3::new(xyz.x(), xyz.y(), xyz.z())
        };
        set(&mut aPoles, 1, aVIndex, last);
    }
    let (aUKnots, aUMults) = knots01(2);
    let (aVKnots, aVMults) = knots01(3);
    Handle::new(
        GeomBSplineSurface::with_weights(
            &aPoles,
            &aWeights,
            (&aUKnots, &aUMults),
            (&aVKnots, &aVMults),
            (1, 2),
            (false, false),
        )
        .unwrap(),
    )
}

fn checkExtendedSurface(theRational: bool, theInU: bool, theAfter: bool, trace: &str) {
    let aSource: Handle<GeomBSplineSurface> = makeSurface(theRational);
    let aBoundary: f64 = if theAfter { 1.0 } else { 0.0 };
    let anExtensionLength: f64 = 0.4;
    let aContinuityTolerance: f64 = 1.0e-10;
    // Pointwise length is approximate because normalized boundary tangents are represented by the
    // extended surface pole grid.
    let aRelativeLengthTolerance: f64 = 5.0e-3;
    let aLengthTolerance: f64 = anExtensionLength * aRelativeLengthTolerance;

    let mut aBoundaryPoints: Vec<Point3> = Vec::with_capacity(3);
    let mut aBoundaryDerivatives: Vec<Vector3> = Vec::with_capacity(3);
    for anIndex in 0..3 {
        let aParameter: f64 = 0.25 + 0.25 * anIndex as f64;
        let (u, v) = if theInU {
            (aBoundary, aParameter)
        } else {
            (aParameter, aBoundary)
        };
        let d1 = aSource.d1(u, v).unwrap();
        aBoundaryPoints.push(d1.point);
        aBoundaryDerivatives.push(if theInU { d1.d1u } else { d1.d1v });
    }

    let mut anExtended: Handle<dyn GeomBoundedSurface> = aSource.cast();
    GeomLib::extend_surf_by_length(&mut anExtended, anExtensionLength, 1, theInU, theAfter)
        .unwrap();
    let aResult: Handle<GeomBSplineSurface> = anExtended.cast();
    assert!(!aResult.is_null(), "{trace}");

    let (aUMin, aUMax, aVMin, aVMax) = aResult.bounds();
    let anEndParameter: f64 = if theInU {
        if theAfter { aUMax } else { aUMin }
    } else if theAfter {
        aVMax
    } else {
        aVMin
    };
    assert!(
        if theAfter {
            anEndParameter > aBoundary
        } else {
            anEndParameter < aBoundary
        },
        "{trace}"
    );
    for anIndex in 0..3 {
        let aParameter: f64 = 0.25 + 0.25 * anIndex as f64;
        let (u, v) = if theInU {
            (aBoundary, aParameter)
        } else {
            (aParameter, aBoundary)
        };
        let d1 = aResult.d1(u, v).unwrap();
        assert!(
            d1.point.distance(&aBoundaryPoints[anIndex]) <= aContinuityTolerance,
            "{trace}"
        );
        let aDerivative: Vector3 = if theInU { d1.d1u } else { d1.d1v };
        assert!(
            (aDerivative - aBoundaryDerivatives[anIndex]).magnitude() <= aContinuityTolerance,
            "{trace}"
        );

        let (u, v) = if theInU {
            (anEndParameter, aParameter)
        } else {
            (aParameter, anEndParameter)
        };
        let aPoint: Point3 = aResult.d0(u, v).unwrap();
        assert!(
            (aPoint.distance(&aBoundaryPoints[anIndex]) - anExtensionLength).abs()
                <= aLengthTolerance,
            "{trace}"
        );
    }

    if theRational {
        assert!(
            aResult.is_u_rational() || aResult.is_v_rational(),
            "{trace}"
        );
        let weights: Array2<'static, f64> = aResult.weights_array(|w| w.clone());
        for aWeight in weights.iter() {
            assert!(*aWeight > 0.0, "{trace}");
        }
    }
}

#[test]
fn GeomLibTest__ExtendSurfByLength_CoversAllDirectionsAndRationality() {
    for isRational in [false, true] {
        for isInU in [false, true] {
            for isAfter in [false, true] {
                let trace = format!(
                    "rational={}, inU={}, after={}",
                    isRational as i32, isInU as i32, isAfter as i32
                );
                checkExtendedSurface(isRational, isInU, isAfter, &trace);
            }
        }
    }
}

#[test]
fn GeomLibTest__CancelDenominatorDerivative_PreservesSurfaceGeometry() {
    let mut aSurface: Handle<GeomBSplineSurface> = makeSurface(true);
    let mut aReferencePoints: Vec<Point3> = Vec::with_capacity(25);
    for aUIndex in 0..5 {
        let aU: f64 = 0.25 * aUIndex as f64;
        for aVIndex in 0..5 {
            let aV: f64 = 0.25 * aVIndex as f64;
            aReferencePoints.push(aSurface.d0(aU, aV).unwrap());
        }
    }

    GeomLib::cancel_denominator_derivative(&mut aSurface, true, false).unwrap();

    for aUIndex in 0..5 {
        let aU: f64 = 0.25 * aUIndex as f64;
        for aVIndex in 0..5 {
            let aV: f64 = 0.25 * aVIndex as f64;
            let aPoint: Point3 = aSurface.d0(aU, aV).unwrap();
            assert!(aPoint.distance(&aReferencePoints[aUIndex * 5 + aVIndex]) <= 1.0e-9);
        }
    }
}

#[test]
fn GeomLibTest__CancelDenominatorDerivative_IdempotentCallPreservesParameterRange() {
    let mut aSurface: Handle<GeomBSplineSurface> = makeSurface(true);
    let mut aUKnots: Array1<'static, f64> = aSurface.u_knots();
    *aUKnots.as_mut_slice().first_mut().unwrap() = 2.0;
    *aUKnots.as_mut_slice().last_mut().unwrap() = 5.0;
    aSurface.set_u_knots(&aUKnots).unwrap();

    GeomLib::cancel_denominator_derivative(&mut aSurface, true, false).unwrap();
    EXPECT_DOUBLE_EQ(aSurface.u_knot(1).unwrap(), 2.0);
    EXPECT_DOUBLE_EQ(aSurface.u_knot(aSurface.nb_u_knots()).unwrap(), 5.0);

    GeomLib::cancel_denominator_derivative(&mut aSurface, true, false).unwrap();
    EXPECT_DOUBLE_EQ(aSurface.u_knot(1).unwrap(), 2.0);
    EXPECT_DOUBLE_EQ(aSurface.u_knot(aSurface.nb_u_knots()).unwrap(), 5.0);
}

fn segment_poles() -> Array1<'static, Point3> {
    let mut aPoles: Array1<'static, Point3> = one_based(2).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 0.0, 0.0);
    aPoles
}

#[test]
fn GeomLibTest__ExtendCurveToPoint_ReachesTargetAndPreservesInitialEnd() {
    let aPoles = segment_poles();
    let mut aCurve: Handle<dyn GeomBoundedCurve> =
        Handle::new(GeomBezierCurve::new(&aPoles).unwrap()).cast();

    let aTarget = Point3::new(2.0, 0.0, 0.0);
    GeomLib::extend_curve_to_point(&mut aCurve, aTarget, 1, true).unwrap();

    assert!(
        aCurve
            .value(aCurve.first_parameter())
            .unwrap()
            .distance(&aPoles[1])
            <= 1.0e-12
    );
    assert!(
        aCurve
            .value(aCurve.last_parameter())
            .unwrap()
            .distance(&aTarget)
            <= 1.0e-12
    );
}

#[test]
fn GeomLibTest__AdjustExtremity_ReachesEndpointsAndTangentDirections() {
    let aPoles = segment_poles();
    let mut aCurve: Handle<dyn GeomBoundedCurve> =
        Handle::new(GeomBezierCurve::new(&aPoles).unwrap()).cast();

    let (aFirst, aLast) = (Point3::new(0.0, 1.0, 0.0), Point3::new(1.0, 1.0, 0.0));
    let aTangent = Vector3::new(1.0, 0.0, 0.0);
    GeomLib::adjust_extremity(&mut aCurve, aFirst, aLast, aTangent, aTangent).unwrap();

    let (aPoint, aDerivative) = aCurve.d1(aCurve.first_parameter()).unwrap();
    assert!(aPoint.distance(&aFirst) <= 1.0e-12);
    assert!(aDerivative.is_parallel(aTangent, 1.0e-12).unwrap());
    let (aPoint, aDerivative) = aCurve.d1(aCurve.last_parameter()).unwrap();
    assert!(aPoint.distance(&aLast) <= 1.0e-12);
    assert!(aDerivative.is_parallel(aTangent, 1.0e-12).unwrap());
}

#[test]
fn GeomLibTest__ParameterArrays_SupportArbitraryLowerBounds() {
    let mut anInput: Array1<'static, f64> = Array1::with_bounds(-2, 4).unwrap();
    let anInputValues: [f64; 7] = [0.0, 0.1, 0.2, 0.4, 0.6, 0.8, 1.0];
    anInput.as_mut_slice().copy_from_slice(&anInputValues);

    let mut aReduced: Array1<'static, f64> = Array1::default();
    GeomLib::remove_points_from_array(4, &anInput, &mut aReduced).unwrap();
    let aReducedValues: [f64; 5] = [0.0, 0.1, 0.4, 0.8, 1.0];
    assert_eq!(aReduced.len(), aReducedValues.len());
    for (anIndex, aValue) in aReducedValues.iter().enumerate() {
        EXPECT_DOUBLE_EQ(*aReduced.at(anIndex).unwrap(), *aValue);
    }

    let mut aSparse: Array1<'static, f64> = Array1::with_bounds(-1, 1).unwrap();
    aSparse.as_mut_slice().copy_from_slice(&[0.0, 0.6, 1.0]);
    let mut aDense: Array1<'static, f64> = Array1::default();
    GeomLib::densify_array1_of_real(5, &aSparse, &mut aDense).unwrap();
    let aDenseValues: [f64; 5] = [0.0, 1.0 / 3.0, 0.6, 2.0 / 3.0, 1.0];
    assert_eq!(aDense.len(), aDenseValues.len());
    for (anIndex, aValue) in aDenseValues.iter().enumerate() {
        EXPECT_DOUBLE_EQ(*aDense.at(anIndex).unwrap(), *aValue);
    }
}

#[test]
fn GeomLibTest__DensifyArray_ConstantRangeInitializesEveryValue() {
    let mut aConstantParameters: Array1<'static, f64> = Array1::with_bounds(-1, 1).unwrap();
    aConstantParameters.init(4.25);

    let mut aDense: Array1<'static, f64> = Array1::default();
    GeomLib::densify_array1_of_real(7, &aConstantParameters, &mut aDense).unwrap();
    assert_eq!(aDense.len(), 7);
    for aParameter in aDense.iter() {
        EXPECT_DOUBLE_EQ(*aParameter, 4.25);
    }
}

#[test]
fn GeomLibTest__FuseIntervals_SupportsArbitraryLowerBounds() {
    let mut aFirst: Array1<'static, f64> = Array1::with_bounds(-1, 1).unwrap();
    let mut aSecond: Array1<'static, f64> = Array1::with_bounds(3, 5).unwrap();
    aFirst.as_mut_slice().copy_from_slice(&[0.0, 0.5, 1.0]);
    aSecond
        .as_mut_slice()
        .copy_from_slice(&[0.25, 0.5000000001, 0.75]);

    let mut aResult: LinearVector<f64> = LinearVector::default();
    aResult.append(-1.0); // Verify that reusable output storage is cleared before filling.
    GeomLib::fuse_intervals(&aFirst, &aSecond, &mut aResult, 1.0e-8, false);
    let anExpected: [f64; 5] = [0.0, 0.25, 0.50000000005, 0.75, 1.0];
    assert_eq!(aResult.len(), anExpected.len());
    for (anIndex, aValue) in anExpected.iter().enumerate() {
        EXPECT_DOUBLE_EQ(*aResult.value(anIndex).unwrap(), *aValue);
    }
}

#[test]
fn GeomLibTest__FuseIntervals_AdjustsMatchesToFirstInput() {
    let mut aFirst: Array1<'static, f64> = Array1::with_size(2);
    let mut aSecond: Array1<'static, f64> = Array1::with_size(2);
    aFirst.as_mut_slice().copy_from_slice(&[0.0, 1.0]);
    aSecond
        .as_mut_slice()
        .copy_from_slice(&[1.0e-10, 1.0 + 1.0e-10]);

    let mut aResult: LinearVector<f64> = LinearVector::default();
    GeomLib::fuse_intervals(&aFirst, &aSecond, &mut aResult, 1.0e-8, true);
    assert_eq!(aResult.len(), 2);
    EXPECT_DOUBLE_EQ(*aResult.value(0).unwrap(), *aFirst.at(0).unwrap());
    EXPECT_DOUBLE_EQ(*aResult.value(1).unwrap(), *aFirst.at(1).unwrap());
}

#[test]
fn GeomLibTest__GTransformLine_UsesExactVectorialDirection() {
    let aLine: Handle<Geom2dLine> = Handle::new(Geom2dLine::from_point(
        Point2::new(1.0, 2.0),
        Direction2::new(1.0, 0.0).unwrap(),
    ));
    let aMatrix = Matrix2::from_columns(Xy::new(2.0, 0.5), Xy::new(1.0, 3.0));
    let aTransform = GeneralTransform2::new(aMatrix, Xy::new(4.0, -1.0));

    let aResult: Handle<Geom2dLine> = GeomLib::gtransform(&aLine.cast(), &aTransform)
        .unwrap()
        .cast();
    assert!(!aResult.is_null());
    assert!(aResult.location().distance(Point2::new(8.0, 5.5)) <= 1.0e-12);
    assert!(
        aResult
            .direction()
            .is_parallel(Direction2::new(2.0, 0.5).unwrap(), 1.0e-12)
    );
}

#[test]
fn GeomLibTest__To3d_PreservesRationalBezierGeometry() {
    let mut aPoles: Array1<'static, Point2> = one_based(3).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(0.5, 1.0);
    aPoles[3] = Point2::new(1.0, 0.0);
    let mut aWeights: Array1<'static, f64> = one_based(3).unwrap();
    aWeights[1] = 1.0;
    aWeights[2] = 2.0;
    aWeights[3] = 1.0;
    let aCurve2d: Handle<Geom2dBezierCurve> =
        Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap());

    let aPosition = Axis2::with_x_direction(gp::origin(), gp::dz(), gp::dx()).unwrap();
    let aCurve3d: Handle<GeomBezierCurve> =
        GeomLib::to_3d(aPosition, &aCurve2d.cast()).unwrap().cast();
    assert!(!aCurve3d.is_null());
    assert!(aCurve3d.is_rational());
    for aParameter in [0.0, 0.3, 1.0] {
        let aPoint2d: Point2 = aCurve2d.value(aParameter).unwrap();
        let aPoint3d: Point3 = aCurve3d.value(aParameter).unwrap();
        assert!(aPoint3d.distance(&Point3::new(aPoint2d.x(), aPoint2d.y(), 0.0)) <= 1.0e-12);
    }
}

#[test]
fn GeomLibTest__IsIsoLine_AcceptsShortNonDegenerateLinearBezier() {
    let mut aPoles: Array1<'static, Point2> = one_based(2).unwrap();
    aPoles[1] = Point2::new(2.0, 3.0);
    aPoles[2] = Point2::new(2.0 + 1.0e-5, 3.0);
    let aCurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap()).cast();
    let anAdaptor: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(&aCurve).unwrap()).cast();

    let anIso = GeomLib::is_iso_line(&anAdaptor).unwrap();
    assert!(anIso.is_some());
    let anIso = anIso.unwrap();
    assert!(!anIso.is_u);
    assert!(anIso.is_forward);
    EXPECT_DOUBLE_EQ(anIso.param, 3.0);
}

#[test]
fn GeomLibTest__RationalBSplineClosedness_UsesCompleteHomogeneousPoles() {
    let aDifferentSurface: Handle<GeomBSplineSurface> = makeRationalBoundarySurface(false);
    assert!(
        aDifferentSurface
            .value(0.0, 0.25)
            .unwrap()
            .distance(&aDifferentSurface.value(1.0, 0.25).unwrap())
            > 0.1
    );
    assert!(!GeomLib::is_bspl_u_closed(&aDifferentSurface, 0.0, 1.0, 1.0e-10).unwrap());

    let anEquivalentSurface: Handle<GeomBSplineSurface> = makeRationalBoundarySurface(true);
    assert!(
        anEquivalentSurface
            .value(0.0, 0.25)
            .unwrap()
            .distance(&anEquivalentSurface.value(1.0, 0.25).unwrap())
            <= 1.0e-12
    );
    assert!(GeomLib::is_bspl_u_closed(&anEquivalentSurface, 0.0, 1.0, 1.0e-10).unwrap());
}

#[test]
fn GeomLibTest__RationalBezierClosedness_AccountsForWeights() {
    let mut aPoles: Array2<'static, Point3> = grid(2, 3);
    let mut aWeights: Array2<'static, f64> = grid(2, 3);
    for aUIndex in 0..aPoles.row_size() {
        for aVIndex in 0..aPoles.col_size() {
            set(
                &mut aPoles,
                aUIndex,
                aVIndex,
                Point3::new(1.0, 1.0 + aVIndex as f64, 0.0),
            );
            set(
                &mut aWeights,
                aUIndex,
                aVIndex,
                if aUIndex == 0 && aVIndex == 1 {
                    2.0
                } else {
                    1.0
                },
            );
        }
    }
    let aSurface: Handle<GeomBezierSurface> =
        Handle::new(GeomBezierSurface::with_weights(&aPoles, &aWeights).unwrap());

    assert!(
        aSurface
            .value(0.0, 0.25)
            .unwrap()
            .distance(&aSurface.value(1.0, 0.25).unwrap())
            > 0.1
    );
    assert!(!GeomLib::is_bz_u_closed(&aSurface, 0.0, 1.0, 1.0e-10).unwrap());
}

#[test]
fn GeomLibTest__NormEstim_HandlesRegularAndPoleParameters() {
    let aPosition = Axis3::new(gp::origin(), gp::dz(), gp::dx()).unwrap();
    let mut aNormal: Direction3 = gp::dz();

    let aPlane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(aPosition)).cast();
    assert_eq!(
        GeomLib::norm_estim(&aPlane, Point2::new(2.0, -3.0), 1.0e-9, &mut aNormal).unwrap(),
        0
    );
    assert!(aNormal.is_parallel(gp::dz(), 1.0e-12));

    let aSphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(aPosition, 2.0).unwrap()).cast();
    let aHalfPi: f64 = 0.5 * (-1.0f64).acos();
    assert_eq!(
        GeomLib::norm_estim(&aSphere, Point2::new(0.4, aHalfPi), 1.0e-9, &mut aNormal).unwrap(),
        1
    );
    assert!(aNormal.is_parallel(gp::dz(), 1.0e-7));
}
