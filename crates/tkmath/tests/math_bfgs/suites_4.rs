#[test]
fn MathBFGSTest__MultipleOptimizations() {
    let mut aFunc = QuadraticFunction2D;
    let mut anOptimizer = Bfgs::with_options(2, 1e-8, 200, 1e-12).unwrap();

    // Perform multiple optimizations with the same optimizer instance
    for (first, second) in [(5.0, 7.0), (-3.0, -4.0), (0.5, 1.5), (10.0, -5.0)] {
        let mut aStartPoint = Vector::with_bounds(1, 2).unwrap();
        aStartPoint[1] = first;
        aStartPoint[2] = second;

        anOptimizer.perform(&mut aFunc, &aStartPoint).unwrap();

        assert!(anOptimizer.is_done());
        assert_near(anOptimizer.location().unwrap()[1], 1.0, 1.0e-6);
        assert_near(anOptimizer.location().unwrap()[2], 2.0, 1.0e-6);
    }
}

// Simple 1D square function: f(x) = x^2 (minimum at x=0, value = 0)

#[test]
fn MathBFGSTest__OCC30492_StartingPointAtMinimum() {
    // Bug OCC30492: BFGS and FRPR fail if starting point is exactly the minimum
    // Test that both BFGS and FRPR work correctly when the starting point is at the minimum
    let mut aFunc = SquareFunction1D;
    let mut aStartPnt = Vector::with_bounds(1, 1).unwrap();
    aStartPnt[1] = 0.0; // Start exactly at the minimum

    // Test FRPR
    let mut aFRPR = Frpr::with_options(&aFunc, 1e-7, 200, 1e-12).unwrap();
    aFRPR.perform(&mut aFunc, &aStartPnt).unwrap();
    assert!(aFRPR.is_done());

    // Test BFGS
    let mut aBFGS = Bfgs::with_options(1, 1e-7, 200, 1e-12).unwrap();
    aBFGS.perform(&mut aFunc, &aStartPnt).unwrap();
    assert!(aBFGS.is_done());

    // Verify that both found the minimum (or stayed at it)
    if aFRPR.is_done() {
        assert_near(aFRPR.location().unwrap()[1], 0.0, 1e-7);
        assert_near(aFRPR.minimum().unwrap(), 0.0, 1e-7);
    }

    if aBFGS.is_done() {
        assert_near(aBFGS.location().unwrap()[1], 0.0, 1e-7);
        assert_near(aBFGS.minimum().unwrap(), 0.0, 1e-7);
    }
}
