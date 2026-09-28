#[test]
fn MathRoot_TrigTest__InvalidFiniteOrderedInputPolicy() {
    assert_eq!(
        trigonometric_linear(1.0, 0.0, 1.0, 0.0).status,
        Status::InvalidInput
    );
    assert_eq!(
        trigonometric_linear(f64::NAN, 0.0, 0.0, TAU).status,
        Status::InvalidInput
    );
    assert_eq!(
        trigonometric_linear(1.0, 0.0, 0.0, f64::INFINITY).status,
        Status::InvalidInput
    );
}

#[test]
fn MathRoot_TrigTest__PeriodicRootCountRejectsUnrepresentableSteps() {
    let mut aCount: usize = 0;
    assert!(utils::compute_periodic_root_count(
        0.5,
        0.5 + 6.0 * PI,
        2.0 * PI,
        0.0,
        false,
        &mut aCount
    ));
    assert_eq!(aCount, 4);
    assert!(utils::compute_periodic_root_count(
        0.5,
        0.5 + 6.0 * PI,
        2.0 * PI,
        1.0e-12,
        true,
        &mut aCount
    ));
    assert_eq!(aCount, 3);
    assert!(utils::compute_periodic_root_count(
        0.5,
        0.5 + 6.0 * PI,
        2.0 * PI,
        0.0,
        true,
        &mut aCount
    ));
    assert_eq!(aCount, 3);
    assert!(utils::compute_periodic_root_count(
        0.5,
        0.5,
        2.0 * PI,
        0.0,
        true,
        &mut aCount
    ));
    assert_eq!(aCount, 0);

    let aFourthRoot: f64 = 0.5 + 3.0 * (2.0 * PI);
    assert!(utils::compute_periodic_root_count(
        0.5,
        aFourthRoot.next_up(),
        2.0 * PI,
        0.0,
        true,
        &mut aCount
    ));
    assert_eq!(aCount, 4);
    assert!(utils::compute_periodic_root_count(
        0.5,
        aFourthRoot.next_down(),
        2.0 * PI,
        0.0,
        false,
        &mut aCount
    ));
    assert_eq!(aCount, 3);

    let aLargeParameter: f64 = 1.0e17;
    assert!(
        !(utils::compute_periodic_root_count(
            aLargeParameter,
            aLargeParameter + 100.0,
            2.0 * PI,
            0.0,
            false,
            &mut aCount
        ))
    );
}
