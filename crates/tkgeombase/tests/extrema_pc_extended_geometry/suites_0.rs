//==================================================================================================
// Transformed geometry tests - Circle
//==================================================================================================

#[test]
fn ExtremaPC_ExtendedGeometryTest__Circle_Translated() {
    let anAdaptor = whole(
        GeomCircle::new(ax2(Point3::new(100.0, 200.0, 50.0), dir3(0.0, 0.0, 1.0)), 25.0).unwrap(),
    );

    let aPoint = Point3::new(150.0, 200.0, 50.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Circle_Translated", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Circle_Rotated_XY_Plane() {
    // Circle in XY plane, rotated 45 degrees
    let aNormal = dir3(1.0 / 2.0_f64.sqrt(), 1.0 / 2.0_f64.sqrt(), 0.0);
    let anAdaptor = whole(GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), aNormal), 15.0).unwrap());

    let aPoint = Point3::new(20.0, 20.0, 5.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Circle_Rotated_XY", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Circle_Rotated_XZ_Plane() {
    // Circle in XZ plane
    let anAdaptor = whole(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 1.0, 0.0)), 10.0).unwrap(),
    );

    let aPoint = Point3::new(15.0, 5.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Circle_Rotated_XZ", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Circle_Rotated_YZ_Plane() {
    // Circle in YZ plane
    let anAdaptor = whole(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0)), 10.0).unwrap(),
    );

    let aPoint = Point3::new(5.0, 15.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Circle_Rotated_YZ", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Circle_Arbitrary_Orientation() {
    // Circle with arbitrary orientation
    let aNormal = dir3(1.0, 1.0, 1.0);
    let anAdaptor =
        whole(GeomCircle::new(ax2(Point3::new(10.0, 20.0, 30.0), aNormal), 8.0).unwrap());

    let aPoint = Point3::new(15.0, 25.0, 35.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Circle_Arbitrary", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Circle_VerySmall() {
    let anAdaptor = whole(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 0.001).unwrap(),
    );

    let aPoint = Point3::new(0.002, 0.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Circle_VerySmall", 1.0e-9)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Circle_VeryLarge() {
    let anAdaptor = whole(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 1000.0).unwrap(),
    );

    let aPoint = Point3::new(1500.0, 0.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Circle_VeryLarge", THE_TOL)
    });
}

//==================================================================================================
// Transformed geometry tests - Ellipse
//==================================================================================================

#[test]
fn ExtremaPC_ExtendedGeometryTest__Ellipse_Translated() {
    let anAdaptor = whole(
        GeomEllipse::new(ax2(Point3::new(50.0, 100.0, 25.0), dir3(0.0, 0.0, 1.0)), 30.0, 15.0)
            .unwrap(),
    );

    let aPoint = Point3::new(90.0, 100.0, 25.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Ellipse_Translated", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Ellipse_Rotated() {
    let aNormal = dir3(0.0, 1.0, 0.0); // In XZ plane
    let anAdaptor =
        whole(GeomEllipse::new(ax2(Point3::new(0.0, 0.0, 0.0), aNormal), 20.0, 8.0).unwrap());

    let aPoint = Point3::new(25.0, 10.0, 5.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Ellipse_Rotated", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Ellipse_HighEccentricity() {
    // Very elongated ellipse (major = 50, minor = 2)
    let anAdaptor = whole(
        GeomEllipse::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 50.0, 2.0)
            .unwrap(),
    );

    let aPoint = Point3::new(40.0, 5.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Ellipse_HighEccentricity", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Ellipse_NearlyCircular() {
    // Nearly circular ellipse (major = 10.1, minor = 10.0)
    let anAdaptor = whole(
        GeomEllipse::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.1, 10.0)
            .unwrap(),
    );

    let aPoint = Point3::new(15.0, 8.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinMax(aNewResult, anOldExtPC, "Ellipse_NearlyCircular", THE_TOL)
    });
}

//==================================================================================================
// Transformed geometry tests - Line
//==================================================================================================

#[test]
fn ExtremaPC_ExtendedGeometryTest__Line_Diagonal() {
    let anAdaptor = ranged(
        GeomLine::from_point(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 1.0, 1.0)),
        -100.0,
        100.0,
    );

    let aPoint = Point3::new(10.0, 0.0, 0.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Line_Diagonal", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Line_Arbitrary() {
    let anAdaptor = ranged(
        GeomLine::from_point(Point3::new(10.0, 20.0, 30.0), dir3(2.0, 3.0, 1.0)),
        -50.0,
        50.0,
    );

    let aPoint = Point3::new(25.0, 35.0, 40.0);

    compare_point(&anAdaptor, aPoint, |aNewResult, anOldExtPC| {
        CompareMinDistance(aNewResult, anOldExtPC, "Line_Arbitrary", THE_TOL)
    });
}

#[test]
fn ExtremaPC_ExtendedGeometryTest__Line_PointOnLine() {
    let anAdaptor = ranged(
        GeomLine::from_point(Point3::new(0.0, 0.0, 0.0), dir3(1.0, 0.0, 0.0)),
        -100.0,
        100.0,
    );

    let aPoint = Point3::new(50.0, 0.0, 0.0); // Exactly on line

    compare_point(&anAdaptor, aPoint, |aNewResult, _anOldExtPC| {
        assert!(aNewResult.is_done());
        EXPECT_NEAR(aNewResult.min_square_distance().sqrt(), 0.0, THE_TOL);
    });
}
