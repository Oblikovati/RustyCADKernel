//==================================================================================================
// Ellipse SearchMode tests
//==================================================================================================

#[test]
fn ExtremaPC_SearchModeTest__Ellipse_MinMode_OnMajorAxis() {
    let anAdaptor = ellipse20x10();

    let aPoint = Point3::new(30.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());
    assert_eq!(aNewResult.nb_ext(), 1);

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
    EXPECT_NEAR(aNewMinDist, 10.0, THE_TOL); // 30 - 20 = 10
}

#[test]
fn ExtremaPC_SearchModeTest__Ellipse_MaxMode_OnMajorAxis() {
    let anAdaptor = ellipse20x10();

    let aPoint = Point3::new(30.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Max).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());
    assert_eq!(aNewResult.nb_ext(), 1);

    let aNewMaxDist: f64 = aNewResult.max_square_distance().sqrt();
    let aOldMaxDist: f64 = GetOldMaxDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMaxDist, aOldMaxDist, THE_TOL);
    EXPECT_NEAR(aNewMaxDist, 50.0, THE_TOL); // 30 + 20 = 50
}

#[test]
fn ExtremaPC_SearchModeTest__Ellipse_MinMode_General() {
    let anAdaptor = ellipse20x10();

    let aPoint = Point3::new(15.0, 12.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
}

#[test]
fn ExtremaPC_SearchModeTest__Ellipse_MinMaxMode() {
    let anAdaptor = ellipse20x10();

    let aPoint = Point3::new(15.0, 8.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::MinMax).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());
    // Ellipse should have 4 extrema for general point position

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aNewMaxDist: f64 = aNewResult.max_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);
    let aOldMaxDist: f64 = GetOldMaxDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
    EXPECT_NEAR(aNewMaxDist, aOldMaxDist, THE_TOL);
}

//==================================================================================================
// Parabola SearchMode tests
//==================================================================================================

#[test]
fn ExtremaPC_SearchModeTest__Parabola_MinMode_AtVertex() {
    let anAdaptor = parabola5();

    let aPoint = Point3::new(0.0, 5.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
}

#[test]
fn ExtremaPC_SearchModeTest__Parabola_MaxMode() {
    let anAdaptor = parabola5();

    let aPoint = Point3::new(0.0, 5.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    // Use PerformWithEndpoints to find maximum which includes endpoints
    let aNewResult = anExtPC
        .perform_with_endpoints(aPoint, THE_TOL, SearchMode::Max)
        .unwrap();

    assert!(aNewResult.is_done());
    EXPECT_GE(aNewResult.nb_ext(), 1);

    // With endpoints, Max mode finds the maximum at one of the endpoints.
    if aNewResult.nb_ext() > 0 {
        let aMaxIdx: usize = aNewResult.max_index();
        check_extremum_on_curve(aNewResult, aMaxIdx, &anAdaptor, (-20.0, 20.0), THE_TOL);
    }
}

#[test]
fn ExtremaPC_SearchModeTest__Parabola_MinMode_OffAxis() {
    let anAdaptor = parabola5();

    let aPoint = Point3::new(10.0, -5.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
}

//==================================================================================================
// Hyperbola SearchMode tests
//==================================================================================================

#[test]
fn ExtremaPC_SearchModeTest__Hyperbola_MinMode() {
    let anAdaptor = hyperbola10x5();

    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
}

#[test]
fn ExtremaPC_SearchModeTest__Hyperbola_MaxMode() {
    // NOTE: Max mode for bounded curves may not find extrema if the max is at
    // an endpoint (which is not a true extremum in the mathematical sense).
    let anAdaptor = hyperbola10x5();

    let aPoint = Point3::new(20.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Max).unwrap();

    assert!(aNewResult.is_done());

    // For hyperbola, verify the found point is valid if extrema exist
    if aNewResult.nb_ext() > 0 {
        let aMaxIdx: usize = aNewResult.max_index();
        check_extremum_on_curve(aNewResult, aMaxIdx, &anAdaptor, (-2.0, 2.0), THE_TOL);
    }
}

//==================================================================================================
// BSpline SearchMode tests
//==================================================================================================

#[test]
fn ExtremaPC_SearchModeTest__BSpline_MinMode() {
    let anAdaptor = cubic_bspline();

    let aPoint = Point3::new(2.0, 3.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    // Allow slightly larger tolerance for BSpline
    EXPECT_NEAR(aNewMinDist, aOldMinDist, 0.01);
}

#[test]
fn ExtremaPC_SearchModeTest__BSpline_MaxMode() {
    // NOTE: Max mode for BSpline may not find extrema if the max is at an endpoint.
    let anAdaptor = cubic_bspline();

    let aPoint = Point3::new(2.0, 1.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Max).unwrap();

    assert!(aNewResult.is_done());

    // Verify the found point is valid if extrema exist
    if aNewResult.nb_ext() > 0 {
        let aMaxIdx: usize = aNewResult.max_index();
        check_extremum_on_curve(aNewResult, aMaxIdx, &anAdaptor, (0.0, 1.0), 0.01);
    }
}

#[test]
fn ExtremaPC_SearchModeTest__BSpline_MinMaxMode() {
    let anAdaptor = cubic_bspline();

    let aPoint = Point3::new(2.0, 1.5, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::MinMax).unwrap();

    assert!(aNewResult.is_done());
    EXPECT_GE(aNewResult.nb_ext(), 1);

    // Verify all found points are valid
    for i in 0..aNewResult.nb_ext() {
        check_extremum_on_curve(aNewResult, i, &anAdaptor, (0.0, 1.0), 0.01);
    }

    // Compare minimum with old implementation
    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);
    if anOldExtPC.is_done() && anOldExtPC.nb_ext().unwrap() > 0 {
        let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
        let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);
        EXPECT_NEAR(aNewMinDist, aOldMinDist, 0.05); // Relaxed tolerance for BSpline
    }
}
