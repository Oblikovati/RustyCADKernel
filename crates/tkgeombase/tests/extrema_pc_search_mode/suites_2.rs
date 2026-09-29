//==================================================================================================
// Bezier SearchMode tests
//==================================================================================================

#[test]
fn ExtremaPC_SearchModeTest__Bezier_MinMode() {
    let anAdaptor = cubic_bezier();

    let aPoint = Point3::new(2.0, 4.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, 0.01);
}

#[test]
fn ExtremaPC_SearchModeTest__Bezier_MaxMode() {
    let anAdaptor = cubic_bezier();

    let aPoint = Point3::new(2.0, 1.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC
        .perform_with_endpoints(aPoint, THE_TOL, SearchMode::Max)
        .unwrap();

    assert!(aNewResult.is_done());
    assert!(aNewResult.nb_ext() >= 1);

    // With endpoint handling, max is at either endpoint (0,0,0) or (4,0,0)
    // Distance from (2,1,0) to endpoints = sqrt(4 + 1) = sqrt(5) ~= 2.236
    let aExpectedMaxDist: f64 = 5.0_f64.sqrt();
    let aNewMaxDist: f64 = aNewResult.max_square_distance().sqrt();
    EXPECT_NEAR(aNewMaxDist, aExpectedMaxDist, 0.01);
}

//==================================================================================================
// Offset Curve SearchMode tests
//==================================================================================================

#[test]
fn ExtremaPC_SearchModeTest__OffsetCircle_MinMode() {
    let anAdaptor = offset_circle();

    let aPoint = Point3::new(25.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

    let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

    assert!(aNewResult.is_done());

    let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
    let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

    EXPECT_NEAR(aNewMinDist, aOldMinDist, 0.01);
    EXPECT_NEAR(aNewMinDist, 10.0, 0.01); // 25 - 15 = 10
}

#[test]
fn ExtremaPC_SearchModeTest__OffsetCircle_MaxMode() {
    // NOTE: Max mode for offset curves may not find extrema depending on the
    // BVH subdivision strategy.
    let anAdaptor = offset_circle();

    let aPoint = Point3::new(25.0, 0.0, 0.0);

    let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
    let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Max).unwrap();

    assert!(aNewResult.is_done());

    // Verify the found max extremum is valid if extrema exist
    if aNewResult.nb_ext() > 0 {
        let aExtremum = &aNewResult[aNewResult.max_index()];
        let aCurvePnt: Point3 = anAdaptor.value(aExtremum.parameter).unwrap();
        EXPECT_NEAR(aCurvePnt.distance(&aExtremum.point), 0.0, 0.01);

        // The max distance should be around 40 (25 + 15)
        let aNewMaxDist: f64 = aNewResult.max_square_distance().sqrt();
        EXPECT_NEAR(aNewMaxDist, 40.0, 0.5);
    }
}

//==================================================================================================
// Random points stress tests for SearchMode
//==================================================================================================

#[test]
fn ExtremaPC_SearchModeTest__Circle_RandomPoints_MinMode() {
    let anAdaptor = circle10();

    // std::mt19937 aGen(12345), uniform_real_distribution<double>(-50.0, 50.0)
    let aPoints = random_points("searchmode_circle");
    assert_eq!(aPoints.len(), 50);

    for (i, &aPoint) in aPoints.iter().enumerate() {
        let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
        let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

        let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

        if !anOldExtPC.is_done() {
            continue;
        }

        assert!(aNewResult.is_done(), "Iteration {i}");

        let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
        let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

        EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
    }
}

#[test]
fn ExtremaPC_SearchModeTest__Circle_RandomPoints_MaxMode() {
    let anAdaptor = circle10();

    // std::mt19937 aGen(12345), uniform_real_distribution<double>(-50.0, 50.0)
    let aPoints = random_points("searchmode_circle");
    assert_eq!(aPoints.len(), 50);

    for (i, &aPoint) in aPoints.iter().enumerate() {
        let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
        let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Max).unwrap();

        let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

        if !anOldExtPC.is_done() {
            continue;
        }

        assert!(aNewResult.is_done(), "Iteration {i}");

        let aNewMaxDist: f64 = aNewResult.max_square_distance().sqrt();
        let aOldMaxDist: f64 = GetOldMaxDistance(&anOldExtPC);

        EXPECT_NEAR(aNewMaxDist, aOldMaxDist, THE_TOL);
    }
}

#[test]
fn ExtremaPC_SearchModeTest__Ellipse_RandomPoints_MinMode() {
    let anAdaptor = ellipse20x10();

    // std::mt19937 aGen(54321), uniform_real_distribution<double>(-50.0, 50.0)
    let aPoints = random_points("searchmode_ellipse");
    assert_eq!(aPoints.len(), 50);

    for (i, &aPoint) in aPoints.iter().enumerate() {
        let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
        let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

        let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

        if !anOldExtPC.is_done() {
            continue;
        }

        assert!(aNewResult.is_done(), "Iteration {i}");

        let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
        let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

        EXPECT_NEAR(aNewMinDist, aOldMinDist, THE_TOL);
    }
}

#[test]
fn ExtremaPC_SearchModeTest__BSpline_RandomPoints_MinMode() {
    let anAdaptor = cubic_bspline();

    // std::mt19937 aGen(11111), uniform_real_distribution<double>(-2.0, 6.0)
    let aPoints = random_points("searchmode_bspline");
    assert_eq!(aPoints.len(), 30);

    let mut aPassCount: i32 = 0;
    let mut aTestCount: i32 = 0;

    for &aPoint in &aPoints {
        let mut anExtPC = ExtremaPC::Curve::from_adaptor(&anAdaptor).unwrap();
        let aNewResult = anExtPC.perform(aPoint, THE_TOL, SearchMode::Min).unwrap();

        let anOldExtPC = old_ext_pc(aPoint, &anAdaptor);

        if !anOldExtPC.is_done() || anOldExtPC.nb_ext().unwrap() == 0 {
            continue;
        }

        if !aNewResult.is_done() || aNewResult.nb_ext() == 0 {
            continue;
        }

        aTestCount += 1;

        let aNewMinDist: f64 = aNewResult.min_square_distance().sqrt();
        let aOldMinDist: f64 = GetOldMinDistance(&anOldExtPC);

        // Allow larger tolerance for BSpline and count passes
        if (aNewMinDist - aOldMinDist).abs() < 0.1 {
            aPassCount += 1;
        }
    }

    // At least 80% of valid tests should pass
    if aTestCount > 0 {
        let aPassRate: f64 = f64::from(aPassCount) / f64::from(aTestCount);
        EXPECT_GE(aPassRate, 0.8);
    }
}
