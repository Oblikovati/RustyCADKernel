// Direct legacy declarations; see parent for OCCT license and provenance.
#[test]
fn MathPSOTest__RepeatedPerformCalls() {
    let mut aFunc = QuadraticFunction;

    let mut aLowerBorder = Vector::with_bounds(1, 2).unwrap();
    aLowerBorder[1] = -2.0;
    aLowerBorder[2] = -1.0;

    let mut aUpperBorder = Vector::with_bounds(1, 2).unwrap();
    aUpperBorder[1] = 4.0;
    aUpperBorder[2] = 5.0;

    let mut aSteps = Vector::with_bounds(1, 2).unwrap();
    aSteps[1] = 0.1;
    aSteps[2] = 0.1;

    let mut aSolver =
        Pso::with_options(&mut aFunc, &aLowerBorder, &aUpperBorder, &aSteps, 15, 20).unwrap();

    let mut aValue1: f64 = 0.0;
    let mut aValue2: f64 = 0.0;
    let mut aSolution1 = Vector::with_bounds(1, 2).unwrap();
    let mut aSolution2 = Vector::with_bounds(1, 2).unwrap();

    aSolver
        .perform(&aSteps, &mut aValue1, &mut aSolution1)
        .unwrap();

    assert!(aSolution1[1] >= -2.0 && aSolution1[1] <= 4.0);
    assert!(aSolution1[2] >= -1.0 && aSolution1[2] <= 5.0);

    aSolver
        .perform(&aSteps, &mut aValue2, &mut aSolution2)
        .unwrap();

    assert!(aSolution2[1] >= -2.0 && aSolution2[1] <= 4.0);
    assert!(aSolution2[2] >= -1.0 && aSolution2[2] <= 5.0);
}
