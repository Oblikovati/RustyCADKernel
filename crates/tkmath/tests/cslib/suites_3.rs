#[test]
fn CSLibClass2dTest__LazyGridRemainsExactAwayFromBoundary() {
    const THE_POINT_COUNT: i32 = 64;
    let mut aPnts = Array1::with_bounds(1, THE_POINT_COUNT).unwrap();
    for anIdx in 0..THE_POINT_COUNT {
        let anAngle: f64 = 2.0 * PI * f64::from(anIdx) / f64::from(THE_POINT_COUNT);
        aPnts[anIdx + 1] = Point2::new(0.5 + 0.4 * anAngle.cos(), 0.5 + 0.4 * anAngle.sin());
    }

    let aClassifier = Class2d::from_array(&aPnts, 1.0e-8, 1.0e-8, 0.0, 0.0, 1.0, 1.0);

    // Cross the lazy-build threshold with exact interior queries.
    for _aQueryIdx in 0..64 {
        assert_eq!(aClassifier.si_dans(Point2::new(0.5, 0.5)), Class2dResult::Inside);
    }

    // Exercise cached cells, excluding an annulus around the polygon boundary.
    for aY in 0..16 {
        for anX in 0..16 {
            let aPoint = Point2::new(
                (f64::from(anX) + 0.5) / 16.0,
                (f64::from(aY) + 0.5) / 16.0,
            );
            let aDx: f64 = aPoint.x() - 0.5;
            let aDy: f64 = aPoint.y() - 0.5;
            let aRadius: f64 = (aDx * aDx + aDy * aDy).sqrt();
            if aRadius < 0.35 {
                assert_eq!(aClassifier.si_dans(aPoint), Class2dResult::Inside);
            } else if aRadius > 0.45 {
                assert_eq!(aClassifier.si_dans(aPoint), Class2dResult::Outside);
            }
        }
    }

    // Boundary candidates must stay on the exact/tolerance path after caching.
    assert_eq!(aClassifier.si_dans(aPnts[1]), Class2dResult::Uncertain);

    let aCopy = aClassifier.clone();
    let mut anAssigned = Class2d::default();
    anAssigned.clone_from(&aClassifier);
    assert_eq!(aCopy.si_dans(Point2::new(0.5, 0.5)), Class2dResult::Inside);
    assert_eq!(anAssigned.si_dans(Point2::new(0.99, 0.99)), Class2dResult::Outside);
    assert_eq!(aCopy.si_dans(aPnts[1]), Class2dResult::Uncertain);
}

#[test]
fn CSLibClass2dTest__LazyGridMatchesExactPathForConcaveDiagonalPolygon() {
    const THE_POINT_COUNT: i32 = 32;
    let mut aPnts = Array1::with_bounds(1, THE_POINT_COUNT).unwrap();
    for anIdx in 0..THE_POINT_COUNT {
        let anAngle: f64 = 2.0 * PI * f64::from(anIdx) / f64::from(THE_POINT_COUNT);
        let aRadius: f64 = if anIdx % 4 == 1 {
            0.22
        } else if anIdx % 2 == 0 {
            0.46
        } else {
            0.34
        };
        aPnts[anIdx + 1] =
            Point2::new(0.5 + aRadius * anAngle.cos(), 0.5 + aRadius * anAngle.sin());
    }

    let aCached = Class2d::from_array(&aPnts, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0);
    let anExact = Class2d::from_array(&aPnts, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0);
    for _aQueryIdx in 0..64usize {
        assert_eq!(aCached.si_dans(Point2::new(0.5, 0.5)), Class2dResult::Inside);
    }

    for aY in 0..47 {
        for anX in 0..47 {
            let aPoint = Point2::new(
                (f64::from(anX) + 0.37) / 47.0,
                (f64::from(aY) + 0.61) / 47.0,
            );
            assert_eq!(
                aCached.si_dans(aPoint),
                anExact.si_dans_on_mode(aPoint, 0.0),
                "sample ({anX}, {aY})"
            );
        }
    }
}

#[test]
fn CSLibClass2dTest__LazyGridConcurrentActivationQueriesAreThreadSafe() {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    // Release concurrent queries together at the lazy-grid activation threshold.
    const THE_POINT_COUNT: i32 = 32768;
    let mut aPnts = Array1::with_bounds(1, THE_POINT_COUNT).unwrap();
    for anIdx in 0..THE_POINT_COUNT {
        let anAngle: f64 = 2.0 * PI * f64::from(anIdx) / f64::from(THE_POINT_COUNT);
        aPnts[anIdx + 1] = Point2::new(0.5 + 0.4 * anAngle.cos(), 0.5 + 0.4 * anAngle.sin());
    }

    let aClassifier = Class2d::from_array(&aPnts, 1.0e-8, 1.0e-8, 0.0, 0.0, 1.0, 1.0);
    for _aQueryIdx in 0..63usize {
        assert_eq!(aClassifier.si_dans(Point2::new(0.5, 0.5)), Class2dResult::Inside);
    }

    let hasFailure = AtomicBool::new(false);
    let canStart = AtomicBool::new(false);
    let aReadyCount = AtomicUsize::new(0);
    const THE_THREAD_COUNT: usize = 8;
    std::thread::scope(|aScope| {
        let aQueryThreads: Vec<std::thread::ScopedJoinHandle<'_, ()>> = (0..THE_THREAD_COUNT)
            .map(|_aThreadIdx| {
                aScope.spawn(|| {
                    aReadyCount.fetch_add(1, Ordering::Release);
                    while !canStart.load(Ordering::Acquire) {
                        std::thread::yield_now();
                    }
                    for aQueryIdx in 0..8usize {
                        let isInside: bool = (aQueryIdx & 1) == 0;
                        let aPoint = if isInside {
                            Point2::new(0.5, 0.5)
                        } else {
                            Point2::new(0.99, 0.99)
                        };
                        let anExpected = if isInside {
                            Class2dResult::Inside
                        } else {
                            Class2dResult::Outside
                        };
                        if aClassifier.si_dans(aPoint) != anExpected {
                            hasFailure.store(true, Ordering::Relaxed);
                            return;
                        }
                    }
                })
            })
            .collect();
        while aReadyCount.load(Ordering::Acquire) != aQueryThreads.len() {
            std::thread::yield_now();
        }
        canStart.store(true, Ordering::Release);
        for aThread in aQueryThreads {
            aThread.join().unwrap();
        }
    });
    assert!(!hasFailure.load(Ordering::Relaxed));
    assert_eq!(aClassifier.si_dans(Point2::new(0.5, 0.5)), Class2dResult::Inside);
    assert_eq!(aClassifier.si_dans(Point2::new(0.99, 0.99)), Class2dResult::Outside);
}

#[test]
fn CSLibClass2dTest__SiDansHandlesNegativeNonFiniteAndExtremeInputs() {
    const THE_POINT_COUNT: i32 = 32;
    const THE_POINTS_PER_SIDE: i32 = THE_POINT_COUNT / 4;
    let mut aPnts = Array1::with_bounds(1, THE_POINT_COUNT).unwrap();
    for anIdx in 0..THE_POINTS_PER_SIDE {
        let aParameter: f64 = f64::from(anIdx) / f64::from(THE_POINTS_PER_SIDE);
        aPnts[1 + anIdx] = Point2::new(aParameter, 0.0);
        aPnts[1 + THE_POINTS_PER_SIDE + anIdx] = Point2::new(1.0, aParameter);
        aPnts[1 + 2 * THE_POINTS_PER_SIDE + anIdx] = Point2::new(1.0 - aParameter, 1.0);
        aPnts[1 + 3 * THE_POINTS_PER_SIDE + anIdx] = Point2::new(0.0, 1.0 - aParameter);
    }

    let aNegative = Class2d::from_array(&aPnts, -1.0, -f64::INFINITY, 0.0, 0.0, 1.0, 1.0);
    assert_eq!(aNegative.si_dans(Point2::new(0.5, 0.5)), Class2dResult::Inside);
    assert_eq!(aNegative.si_dans_on_mode(Point2::new(1.5, 0.5), -1.0), Class2dResult::Outside);

    let anExtremeTolerance =
        Class2d::from_array(&aPnts, f64::INFINITY, f64::MAX, 0.0, 0.0, 1.0, 1.0);
    for _aQueryIdx in 0..80usize {
        assert_eq!(
            anExtremeTolerance.si_dans(Point2::new(0.5, 0.5)),
            Class2dResult::Uncertain
        );
    }

    let aLimit: f64 = PRECISION_INFINITE;
    let mut anExtremePnts = Array1::with_bounds(1, 4).unwrap();
    anExtremePnts[1] = Point2::new(-0.5 * aLimit, -0.5 * aLimit);
    anExtremePnts[2] = Point2::new(0.5 * aLimit, -0.5 * aLimit);
    anExtremePnts[3] = Point2::new(0.5 * aLimit, 0.5 * aLimit);
    anExtremePnts[4] = Point2::new(-0.5 * aLimit, 0.5 * aLimit);
    let anExtremeCoordinates =
        Class2d::from_array(&anExtremePnts, 0.0, 0.0, -aLimit, -aLimit, aLimit, aLimit);
    assert_eq!(anExtremeCoordinates.si_dans(Point2::new(0.0, 0.0)), Class2dResult::Inside);
    assert_eq!(
        anExtremeCoordinates.si_dans(Point2::new(0.75 * aLimit, 0.0)),
        Class2dResult::Outside
    );

    let aLargeTolerance: f64 = 0.1 * aLimit;
    let aTolerantExtreme = Class2d::from_array(
        &anExtremePnts,
        aLargeTolerance,
        aLargeTolerance,
        -aLimit,
        -aLimit,
        aLimit,
        aLimit,
    );
    assert_eq!(
        aTolerantExtreme.si_dans(Point2::new(0.55 * aLimit, 0.0)),
        Class2dResult::Uncertain
    );
    assert_eq!(
        aTolerantExtreme.si_dans(Point2::new(0.65 * aLimit, 0.0)),
        Class2dResult::Outside
    );
    assert_eq!(
        anExtremeCoordinates.si_dans_on_mode(Point2::new(0.0, 0.55 * aLimit), aLargeTolerance),
        Class2dResult::Uncertain
    );
}

#[test]
fn CSLibClass2dTest__LazyGridPreservesToleranceAcrossCellBoundary() {
    const THE_POINTS_PER_SIDE: i32 = 8;
    const THE_POINT_COUNT: i32 = 4 * THE_POINTS_PER_SIDE;
    const THE_MIN: f64 = 0.25;
    const THE_MAX: f64 = 0.75;
    const THE_TOLERANCE: f64 = 1.0e-4;
    let mut aPnts = Array1::with_bounds(1, THE_POINT_COUNT).unwrap();
    for anIdx in 0..THE_POINTS_PER_SIDE {
        let aParameter: f64 = f64::from(anIdx) / f64::from(THE_POINTS_PER_SIDE);
        aPnts[1 + anIdx] = Point2::new(THE_MIN + (THE_MAX - THE_MIN) * aParameter, THE_MIN);
        aPnts[1 + THE_POINTS_PER_SIDE + anIdx] =
            Point2::new(THE_MAX, THE_MIN + (THE_MAX - THE_MIN) * aParameter);
        aPnts[1 + 2 * THE_POINTS_PER_SIDE + anIdx] =
            Point2::new(THE_MAX - (THE_MAX - THE_MIN) * aParameter, THE_MAX);
        aPnts[1 + 3 * THE_POINTS_PER_SIDE + anIdx] =
            Point2::new(THE_MIN, THE_MAX - (THE_MAX - THE_MIN) * aParameter);
    }

    let aClassifier =
        Class2d::from_array(&aPnts, THE_TOLERANCE, THE_TOLERANCE, 0.0, 0.0, 1.0, 1.0);
    for _aQueryIdx in 0..64usize {
        assert_eq!(aClassifier.si_dans(Point2::new(0.5, 0.5)), Class2dResult::Inside);
    }

    assert_eq!(
        aClassifier.si_dans(Point2::new(THE_MIN - 0.5 * THE_TOLERANCE, 0.5)),
        Class2dResult::Uncertain
    );
}

// Test SiDans_OnMode
#[test]
fn CSLibClass2dTest__SiDans_OnMode_PointInside() {
    let aPnts = unitSquare();

    let aClassifier = Class2d::from_array(&aPnts, 1e-10, 1e-10, 0.0, 0.0, 1.0, 1.0);

    let aPointInside = Point2::new(0.5, 0.5);
    assert_eq!(aClassifier.si_dans_on_mode(aPointInside, 0.01) as i32, 1);
}

#[test]
fn CSLibClass2dTest__SiDans_OnMode_NormalizesAndReplacesToleranceOnAnisotropicDomain() {
    let mut aPnts = Array1::with_bounds(1, 4).unwrap();
    aPnts[1] = Point2::new(20.0, -1.0);
    aPnts[2] = Point2::new(80.0, -1.0);
    aPnts[3] = Point2::new(80.0, 1.0);
    aPnts[4] = Point2::new(20.0, 1.0);

    // Constructor tolerances are deliberately larger than the explicit one.
    let aClassifier = Class2d::from_array(&aPnts, 10.0, 1.0, 0.0, -2.0, 100.0, 2.0);
    assert_eq!(aClassifier.si_dans_on_mode(Point2::new(50.0, 1.5), 0.01), Class2dResult::Outside);

    // The same original-space tolerance has different normalized U/V values.
    assert_eq!(aClassifier.si_dans_on_mode(Point2::new(19.75, 0.0), 0.5), Class2dResult::Uncertain);
    assert_eq!(aClassifier.si_dans_on_mode(Point2::new(19.0, 0.0), 0.5), Class2dResult::Outside);
    assert_eq!(aClassifier.si_dans_on_mode(Point2::new(50.0, 1.25), 0.5), Class2dResult::Uncertain);
    assert_eq!(aClassifier.si_dans_on_mode(Point2::new(50.0, 1.75), 0.5), Class2dResult::Outside);
}

// Test with degenerate polygon (less than 3 points effective)
#[test]
fn CSLibClass2dTest__DegeneratePolygon_InvalidBounds() {
    let aPnts = unitSquare();

    // Invalid bounds (umax <= umin)
    let aClassifier = Class2d::from_array(&aPnts, 1e-10, 1e-10, 1.0, 0.0, 0.0, 1.0);

    // Should return 0 for any point with invalid bounds
    let aPoint = Point2::new(0.5, 0.5);
    assert_eq!(aClassifier.si_dans(aPoint) as i32, 0);
}
