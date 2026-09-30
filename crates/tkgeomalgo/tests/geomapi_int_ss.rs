//! Direct translation of GeomAPI_IntSS_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! The surface data of the two OCC26675 cases (identical statements) is in
//! `geomapi_int_ss/occ26675_data.rs`; `ASSERT_NO_THROW` checks that `perform` is `Ok`.
#![allow(non_snake_case, clippy::excessive_precision, clippy::approx_constant)]
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface,
    GeomLine, GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomToroidalSurface,
};
use rustycad_tkgeomalgo::geom_api::{IntSS, ProjectPointOnSurf};
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Point3};
use std::f64::consts::PI;

include!("geomapi_int_ss/occ26675_data.rs");

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

/// The two OCC26675 surfaces: the B-spline surface `aS1` and the extrusion `aS2`.
fn occ26675_surfaces() -> (Handle<dyn GeomSurface>, Handle<dyn GeomSurface>) {
    let aCircCenter = Point3::new(
        -112.93397037070100000000,
        177.52792379072199000000,
        10.63374076104853900000,
    );
    let aCircN = dir(
        0.00000000000000000000,
        1.00000000000000000000,
        0.00000000000000000000,
    );
    let aCircX = dir(
        0.00000000000000000000,
        -0.00000000000000000000,
        1.00000000000000000000,
    );
    let anExtrDir = dir(
        -500.00000000000000000000,
        -866.02540378443609000000,
        0.00000000000000000000,
    );
    let aCircAxes = Axis2::with_x_direction(aCircCenter, aCircN, aCircX).unwrap();
    let aCurv: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(aCircAxes, 50.00000000000000000000).unwrap()).cast();
    let aS2: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfLinearExtrusion::new(&aCurv, anExtrDir).unwrap()).cast();

    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 7, 1, 81).unwrap();
    let mut aWeights: Array2<'static, f64> = Array2::with_bounds(1, 7, 1, 81).unwrap();
    let mut aUKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    let mut aVKnots: Array1<'static, f64> = Array1::with_bounds(1, 12).unwrap();
    let mut aUMult: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    let mut aVMult: Array1<'static, i32> = Array1::with_bounds(1, 12).unwrap();
    let (aUDegree, aVDegree) = (6, 8);
    aUKnots[1] = 0.00000000000000000000;
    aUKnots[2] = 1.00000000000000000000;
    for (i, knot) in OCC26675_V_KNOTS.iter().enumerate() {
        aVKnots[i as i32 + 1] = *knot;
    }
    aUMult[1] = 7;
    aUMult[2] = 7;
    for (i, mult) in OCC26675_V_MULTS.iter().enumerate() {
        aVMult[i as i32 + 1] = *mult;
    }
    for &(i, j, w, x, y, z) in OCC26675_WEIGHTED_POLES.iter() {
        aWeights[(i, j)] = w;
        aPoles[(i, j)] = Point3::new(x, y, z);
    }
    let aS1: Handle<dyn GeomSurface> = Handle::new(
        GeomBSplineSurface::with_weights(
            &aPoles,
            &aWeights,
            (&aUKnots, &aUMult),
            (&aVKnots, &aVMult),
            (aUDegree, aVDegree),
            (false, false),
        )
        .unwrap(),
    )
    .cast();
    (aS1, aS2)
}

/// OCC26675: BSplineSurface intersect SurfaceOfLinearExtrusion: exactly one curve.
#[test]
fn GeomAPI_IntSSTest__OCC26675_IntersectionCurveCount() {
    let (aS1, aS2) = occ26675_surfaces();
    let anIntersection = IntSS::with_surfaces(&aS1, &aS2, precision::confusion()).unwrap();
    assert!(anIntersection.is_done(), "Intersection failed");
    let aNbLines = anIntersection.nb_lines();
    assert_eq!(
        aNbLines, 1,
        "Expected exactly 1 intersection curve, got {aNbLines}"
    );
}

/// OCC26675: the first curve has a non-degenerate parameter range.
#[test]
fn GeomAPI_IntSSTest__OCC26675_IntersectionCurveRangeNonDegenerate() {
    let (aS1, aS2) = occ26675_surfaces();
    let anIntersection = IntSS::with_surfaces(&aS1, &aS2, precision::confusion()).unwrap();
    assert!(anIntersection.is_done(), "Intersection failed");
    assert!(
        anIntersection.nb_lines() >= 1,
        "No intersection curves found"
    );
    // Verify the first curve has a non-degenerate parameter range.
    let aCurve = anIntersection.line(1).unwrap();
    assert!(!aCurve.is_null(), "First intersection curve is null");
    let aU1 = aCurve.first_parameter();
    let aU2 = aCurve.last_parameter();
    let aSpan = aU2 - aU1;
    assert!(
        aSpan > 1.0e-20,
        "Curve range is degenerate: U2 - U1 = {aSpan}"
    );
}

/// OCC23972: exception thrown when intersecting two cones.
#[test]
fn GeomAPI_IntSSTest__OCC23972_TwoConesIntersection() {
    // Specific cone parameters that previously triggered an exception in math_FunctionRoots.
    let aCone1: Handle<dyn GeomSurface> = Handle::new(
        GeomConicalSurface::new(
            Axis3::new(
                Point3::new(123.694345356663, 789.9, 68.15),
                dir(-1.0, 3.48029791472957e-016, -8.41302743359754e-017),
                dir(-3.48029791472957e-016, -1.0, -3.17572289932207e-016),
            )
            .unwrap(),
            0.780868809443031,
            3.28206830417112,
        )
        .unwrap(),
    )
    .cast();
    let aCone2: Handle<dyn GeomSurface> = Handle::new(
        GeomConicalSurface::new(
            Axis3::new(
                Point3::new(123.694345356663, 784.9, 68.15),
                dir(-1.0, -2.5209507537117e-016, -1.49772808948866e-016),
                dir(1.49772808948866e-016, 3.17572289932207e-016, -1.0),
            )
            .unwrap(),
            0.780868809443031,
            3.28206830417112,
        )
        .unwrap(),
    )
    .cast();
    // Trim to a finite V range so that GeomAPI_IntSS does not search an infinite domain.
    let aS1: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(&aCone1, (0.0, 2.0 * PI), (0.0, 10.0), (true, true))
            .unwrap(),
    )
    .cast();
    let aS2: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(&aCone2, (0.0, 2.0 * PI), (0.0, 10.0), (true, true))
            .unwrap(),
    )
    .cast();
    let mut anIntersection = IntSS::new();
    assert!(anIntersection.perform(&aS1, &aS2, 1.0e-7).is_ok());
    assert!(
        anIntersection.is_done(),
        "Surface-surface intersection not done"
    );
    assert!(
        anIntersection.nb_lines() >= 2,
        "Expected at least 2 intersection curves"
    );
    // Verify each intersection curve is non-null and has a non-degenerate parameter range.
    for anIdx in 1..=anIntersection.nb_lines() {
        let aCurve = anIntersection.line(anIdx).unwrap();
        assert!(!aCurve.is_null(), "Curve {anIdx} is null");
        let mut aU1 = aCurve.first_parameter();
        let mut aU2 = aCurve.last_parameter();
        // Hyperbolic curves may be unbounded: clamp to a finite window.
        if aU1 < -20.0 {
            aU1 = -20.0;
        }
        if aU2 > 20.0 {
            aU2 = 20.0;
        }
        assert!(
            aU2 - aU1 > 1.0e-20,
            "Degenerate parameter range for curve {anIdx}"
        );
    }
}

/// Migrated from tests/lowalgos/intss/bug25782_2: two parallel cylinders give two lines.
#[test]
fn GeomAPI_IntSSTest__OCC25782_2_ParallelCylindersProduceTwoLines() {
    let anAxisDirection = dir(12.0, 35.0, 47.0);
    let aCylinder1: Handle<dyn GeomSurface> = Handle::new(
        GeomCylindricalSurface::new(
            Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), anAxisDirection).unwrap(),
            5.0,
        )
        .unwrap(),
    )
    .cast();
    let aCylinder2: Handle<dyn GeomSurface> = Handle::new(
        GeomCylindricalSurface::new(
            Axis3::from_normal(Point3::new(3.0, 2.0, 8.0), anAxisDirection).unwrap(),
            4.0,
        )
        .unwrap(),
    )
    .cast();
    let anIntersection = IntSS::with_surfaces(&aCylinder1, &aCylinder2, 1.0e-7).unwrap();
    assert!(anIntersection.is_done());
    assert_eq!(anIntersection.nb_lines(), 2);
    let aParameters = [0.0, 50.0, 100.0];
    for anIndex in 1..=anIntersection.nb_lines() {
        let aCurve = anIntersection.line(anIndex).unwrap();
        let aLine: Handle<GeomLine> = aCurve.cast();
        assert!(!aLine.is_null(), "Intersection {anIndex} is not a line");
        for aParameter in aParameters {
            let aPoint = aLine.value(aParameter).unwrap();
            let aProjection1 =
                ProjectPointOnSurf::with_tolerance(aPoint, &aCylinder1, 1.0e-7, ExtAlgo::Grad)
                    .unwrap();
            let aProjection2 =
                ProjectPointOnSurf::with_tolerance(aPoint, &aCylinder2, 1.0e-7, ExtAlgo::Grad)
                    .unwrap();
            assert!(aProjection1.is_done());
            assert!(aProjection2.is_done());
            assert!(aProjection1.nb_points().unwrap() > 0);
            assert!(aProjection2.nb_points().unwrap() > 0);
            assert!(aProjection1.lower_distance().unwrap() <= 1.0e-7);
            assert!(aProjection2.lower_distance().unwrap() <= 1.0e-7);
        }
    }
}

/// Migrated from tests/lowalgos/intss/bug33244: two cones do not give duplicate curves.
#[test]
fn GeomAPI_IntSSTest__OCC33244_TwoConesDoNotDuplicateCurves() {
    let aCone1: Handle<dyn GeomSurface> = Handle::new(
        GeomConicalSurface::new(
            Axis3::from_normal(
                Point3::new(-1.11630646267172, -4.54487349779333, 13.2493435203532),
                dir(
                    -1.05794851588922e-07,
                    -1.39278337794573e-08,
                    0.999999999999994,
                ),
            )
            .unwrap(),
            0.785398163360967,
            0.560000000061149,
        )
        .unwrap(),
    )
    .cast();
    let aCone2: Handle<dyn GeomSurface> = Handle::new(
        GeomConicalSurface::new(
            Axis3::from_normal(
                Point3::new(-2.08647872350287e-07, -5.78732475509323, 13.2493436211),
                dir(
                    -1.05794850062242e-07,
                    -1.39278350756825e-08,
                    0.999999999999994,
                ),
            )
            .unwrap(),
            0.785398163396248,
            0.785398163396248,
        )
        .unwrap(),
    )
    .cast();
    let mut anIntersection = IntSS::new();
    assert!(
        anIntersection
            .perform(&aCone1, &aCone2, precision::confusion())
            .is_ok()
    );
    assert!(anIntersection.is_done());
    assert_eq!(anIntersection.nb_lines(), 2);
    for anIndex in 1..=anIntersection.nb_lines() {
        let aCurve = anIntersection.line(anIndex).unwrap();
        assert!(!aCurve.is_null());
        assert!(aCurve.last_parameter() - aCurve.first_parameter() > 0.0);
    }
}

/// Migrated from tests/lowalgos/intss/bug24418_2: the first curve of a torus and a cone
/// lies on both trimmed surfaces.
#[test]
fn GeomAPI_IntSSTest__OCC24418_2_AnalyticalSurfaceIntersection() {
    let aPi = 3.1415926535897932384626433832795;
    let aTorus: Handle<dyn GeomSurface> = Handle::new(
        GeomToroidalSurface::new(
            Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap(),
            25.0,
            24.0,
        )
        .unwrap(),
    )
    .cast();
    let aCone: Handle<dyn GeomSurface> = Handle::new(
        GeomConicalSurface::new(
            Axis3::from_normal(Point3::new(20.0, 20.0, 10.0), dir(0.0, 0.0, 1.0)).unwrap(),
            2.0_f64.atan2(40.0),
            10.0,
        )
        .unwrap(),
    )
    .cast();
    let aTorusTrimmed: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::with_one_direction(&aTorus, 0.0, 0.5 * aPi, true, true)
            .unwrap(),
    )
    .cast();
    let aConeTrimmed: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::with_one_direction(
            &aCone,
            0.0,
            40.049968789001575,
            false,
            true,
        )
        .unwrap(),
    )
    .cast();
    let mut anIntersection = IntSS::new();
    assert!(
        anIntersection
            .perform(&aTorusTrimmed, &aConeTrimmed, precision::confusion())
            .is_ok()
    );
    assert!(anIntersection.is_done());
    assert!(anIntersection.nb_lines() > 0);
    let aCurve = anIntersection.line(1).unwrap();
    assert!(!aCurve.is_null());
    let aFirstParameter = aCurve.first_parameter();
    let aLastParameter = aCurve.last_parameter();
    assert!(aLastParameter > aFirstParameter);
    for anIndex in 0..10 {
        let aParameter =
            aFirstParameter + (aLastParameter - aFirstParameter) * f64::from(anIndex) / 9.0;
        let aPoint = aCurve.d0(aParameter).unwrap();
        let aTorusProjection =
            ProjectPointOnSurf::with_tolerance(aPoint, &aTorusTrimmed, 1.0e-7, ExtAlgo::Grad)
                .unwrap();
        assert!(
            aTorusProjection.is_done(),
            "Torus projection failed at sample {anIndex}"
        );
        assert!(aTorusProjection.nb_points().unwrap() > 0);
        assert!(
            aTorusProjection.lower_distance().unwrap() <= 2.0e-7,
            "Intersection curve is off the torus at sample {anIndex}"
        );
        let aConeProjection =
            ProjectPointOnSurf::with_tolerance(aPoint, &aConeTrimmed, 1.0e-7, ExtAlgo::Grad)
                .unwrap();
        assert!(
            aConeProjection.is_done(),
            "Cone projection failed at sample {anIndex}"
        );
        assert!(aConeProjection.nb_points().unwrap() > 0);
        assert!(
            aConeProjection.lower_distance().unwrap() <= 2.0e-7,
            "Intersection curve is off the cone at sample {anIndex}"
        );
    }
}
