//==================================================================================================
// Line SearchMode tests
//==================================================================================================

#[test]
fn ExtremaPC_SearchModeTest__Line_MinMode() {
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomLine.cast(), -100.0, 100.0).unwrap();

    let aPoint = Point3::new(25.0, 10.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());
    assert_eq!(aNewResult.nb_ext(), 1); // Min mode should return only 1 result

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
    EXPECT_NEAR(aNewMinDist, 10.0, THE_TOL); // Distance to line is 10
}

#[test]
fn ExtremaPC_SearchModeTest__Line_MaxMode() {
    // For bounded lines, Max mode with endpoints includes endpoint maxima.
    // The maximum distance is at the farthest endpoint, not the perpendicular projection.
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomLine.cast(), -50.0, 50.0).unwrap();

    let aPoint = Point3::new(0.0, 10.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC
        .perform_with_endpoints(aPoint, THE_TOL, SearchMode::Max)
        .unwrap();

    assert!(aNewResult.is_done());
    // With endpoints included, max mode finds the farthest endpoint
    EXPECT_GE(aNewResult.nb_ext(), 1);

    // Maximum distance is to endpoint at (-50, 0, 0) or (50, 0, 0)
    // Distance = sqrt(50^2 + 10^2) = sqrt(2600) ~= 50.99
    let aExpectedMaxDist: f64 = 2600.0_f64.sqrt();
    let aNewMaxDist: f64 = aNewResult.max_square_distance().sqrt();
    EXPECT_NEAR(aNewMaxDist, aExpectedMaxDist, THE_TOL);
}

#[test]
fn ExtremaPC_SearchModeTest__Line_MinMaxMode() {
    let aGeomLine = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));
    let anAdaptor = GeomAdaptorCurve::with_range(&aGeomLine.cast(), -50.0, 50.0).unwrap();

    let aPoint = Point3::new(10.0, 5.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::MinMax).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());
    // MinMax mode should find all extrema (min + both endpoints for bounded line)

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
    EXPECT_NEAR(aNewMinDist, 5.0, THE_TOL);
}

//==================================================================================================
// Circle SearchMode tests
//==================================================================================================

#[test]
fn ExtremaPC_SearchModeTest__Circle_MinMode_PointOutside() {
    let anAdaptor = circle10();

    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());
    assert_eq!(aNewResult.nb_ext(), 1);

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
    EXPECT_NEAR(aNewMinDist, 10.0, THE_TOL); // 20 - 10 = 10
}

#[test]
fn ExtremaPC_SearchModeTest__Circle_MaxMode_PointOutside() {
    let anAdaptor = circle10();

    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Max).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());
    assert_eq!(aNewResult.nb_ext(), 1);

    let aNewMaxDist: f64 = aNewResult.max_square_distance().sqrt();
    let aOldMaxDist: f64 = GetOldMaxDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMaxDist, aOldMaxDist, THE_TOL);
    EXPECT_NEAR(aNewMaxDist, 30.0, THE_TOL); // 20 + 10 = 30
}

#[test]
fn ExtremaPC_SearchModeTest__Circle_MinMode_PointInside() {
    let anAdaptor = circle10();

    let aPoint = Point3::new(3.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    assert!(aNewResult.is_done());
    assert_eq!(aNewResult.nb_ext(), 1);

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    EXPECT_NEAR(aNewMinDist, 7.0, THE_TOL); // 10 - 3 = 7
}

#[test]
fn ExtremaPC_SearchModeTest__Circle_MaxMode_PointInside() {
    let anAdaptor = circle10();

    let aPoint = Point3::new(3.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Max).unwrap();

    assert!(aNewResult.is_done());
    assert_eq!(aNewResult.nb_ext(), 1);

    let aNewMaxDist: f64 = aNewResult.max_square_distance().sqrt();
    EXPECT_NEAR(aNewMaxDist, 13.0, THE_TOL); // 10 + 3 = 13
}

#[test]
fn ExtremaPC_SearchModeTest__Circle_MinMaxMode() {
    let anAdaptor = circle10();

    let aPoint = Point3::new(15.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::MinMax).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());
    assert_eq!(aNewResult.nb_ext(), 2); // Circle has 2 extrema: min and max

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aNewMaxDist: f64 = aNewResult.max_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);
    let aOldMaxDist: f64 = GetOldMaxDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
    EXPECT_NEAR(aNewMaxDist, aOldMaxDist, THE_TOL);
    EXPECT_NEAR(aNewMinDist, 5.0, THE_TOL);
    EXPECT_NEAR(aNewMaxDist, 25.0, THE_TOL);
}

#[test]
fn ExtremaPC_SearchModeTest__Circle_PartialArc_MinMode() {
    let anAdaptor = quarter_circle10();

    let aPoint = Point3::new(15.0, 15.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
}

#[test]
fn ExtremaPC_SearchModeTest__Circle_PartialArc_MaxMode() {
    // NOTE: Max mode for partial arcs may not find extrema if the max is at an
    // endpoint (which is not a true extremum in the mathematical sense).
    // This test verifies the implementation completes successfully.
    let anAdaptor = quarter_circle10();

    let aPoint = Point3::new(15.0, 15.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Max).unwrap();

    assert!(aNewResult.is_done());

    // If extrema were found, verify they are valid
    if aNewResult.nb_ext() > 0 {
        let aMaxIdx: usize = aNewResult.max_index();
        check_extremum_on_curve(aNewResult, aMaxIdx, &anAdaptor, (0.0, PI / 2.0), THE_TOL);
    }
}
