// Tests for Binomial coefficient function - Basic values
#[test]
fn PLibTest__BinomialCoefficient_BasicValues() {
    // Test edge cases
    assert!(
        (plib::bin(0, 0).unwrap() - 1.0).abs() <= CONFUSION,
        "C(0,0) should be 1"
    );

    // Test row n=1: [1, 1]
    assert!((plib::bin(1, 0).unwrap() - 1.0).abs() <= CONFUSION);
    assert!((plib::bin(1, 1).unwrap() - 1.0).abs() <= CONFUSION);

    // Test row n=5: [1, 5, 10, 10, 5, 1]
    assert!((plib::bin(5, 0).unwrap() - 1.0).abs() <= CONFUSION);
    assert!((plib::bin(5, 1).unwrap() - 5.0).abs() <= CONFUSION);
    assert!((plib::bin(5, 2).unwrap() - 10.0).abs() <= CONFUSION);
    assert!((plib::bin(5, 3).unwrap() - 10.0).abs() <= CONFUSION);
    assert!((plib::bin(5, 4).unwrap() - 5.0).abs() <= CONFUSION);
    assert!((plib::bin(5, 5).unwrap() - 1.0).abs() <= CONFUSION);

    // Test row n=10: some specific values
    assert!((plib::bin(10, 0).unwrap() - 1.0).abs() <= CONFUSION);
    assert!((plib::bin(10, 1).unwrap() - 10.0).abs() <= CONFUSION);
    assert!((plib::bin(10, 2).unwrap() - 45.0).abs() <= CONFUSION);
    assert!((plib::bin(10, 3).unwrap() - 120.0).abs() <= CONFUSION);
    assert!((plib::bin(10, 4).unwrap() - 210.0).abs() <= CONFUSION);
    assert!((plib::bin(10, 5).unwrap() - 252.0).abs() <= CONFUSION);
}

// Tests for Binomial coefficient - Symmetry property
#[test]
fn PLibTest__BinomialCoefficient_Symmetry() {
    // Test symmetry property: C(n,k) = C(n,n-k)
    for n in 0..=20 {
        for k in 0..=n {
            assert!(
                (plib::bin(n, k).unwrap() - plib::bin(n, n - k).unwrap()).abs() <= CONFUSION,
                "Binomial coefficient symmetry failed for C({n},{k})"
            );
        }
    }
}

// Tests for Binomial coefficient - Pascal's triangle recurrence
#[test]
fn PLibTest__BinomialCoefficient_Recurrence() {
    // Test Pascal's triangle recurrence: C(n,k) = C(n-1,k-1) + C(n-1,k)
    for n in 2..=20 {
        for k in 1..n {
            let expected: f64 = plib::bin(n - 1, k - 1).unwrap() + plib::bin(n - 1, k).unwrap();
            let actual: f64 = plib::bin(n, k).unwrap();
            assert!(
                (actual - expected).abs() <= CONFUSION,
                "Pascal recurrence failed for C({n},{k})"
            );
        }
    }
}

// Tests for Binomial coefficient - Known large values
#[test]
fn PLibTest__BinomialCoefficient_LargeValues() {
    // Test some larger known values
    assert!(
        (plib::bin(20, 10).unwrap() - 184756.0).abs() <= CONFUSION,
        "C(20,10)"
    );
    assert!(
        (plib::bin(15, 7).unwrap() - 6435.0).abs() <= CONFUSION,
        "C(15,7)"
    );
    assert!(
        (plib::bin(25, 5).unwrap() - 53130.0).abs() <= CONFUSION,
        "C(25,5)"
    );
    assert!(
        (plib::bin(25, 12).unwrap() - 5200300.0).abs() <= CONFUSION,
        "C(25,12)"
    );

    // Test maximum supported degree (25)
    assert!((plib::bin(25, 0).unwrap() - 1.0).abs() <= CONFUSION);
    assert!((plib::bin(25, 1).unwrap() - 25.0).abs() <= CONFUSION);
    assert!((plib::bin(25, 25).unwrap() - 1.0).abs() <= CONFUSION);
}

// Tests for Binomial coefficient - Edge cases with first and last columns
#[test]
fn PLibTest__BinomialCoefficient_EdgeColumns() {
    // Test first column: C(n,0) = 1 for all n
    for n in 0..=25 {
        assert!(
            (plib::bin(n, 0).unwrap() - 1.0).abs() <= CONFUSION,
            "C({n},0) should be 1"
        );
    }

    // Test diagonal: C(n,n) = 1 for all n
    for n in 0..=25 {
        assert!(
            (plib::bin(n, n).unwrap() - 1.0).abs() <= CONFUSION,
            "C({n},{n}) should be 1"
        );
    }

    // Test second column: C(n,1) = n for all n >= 1
    for n in 1..=25 {
        assert!(
            (plib::bin(n, 1).unwrap() - f64::from(n)).abs() <= CONFUSION,
            "C({n},1) should be {n}"
        );
    }
}

// Tests for Binomial coefficient - Complete rows verification
#[test]
fn PLibTest__BinomialCoefficient_CompleteRows() {
    // Verify complete row n=6: [1, 6, 15, 20, 15, 6, 1]
    let row6: [f64; 7] = [1.0, 6.0, 15.0, 20.0, 15.0, 6.0, 1.0];
    for k in 0..=6 {
        assert!(
            (plib::bin(6, k).unwrap() - row6[k as usize]).abs() <= CONFUSION,
            "C(6,{k}) verification failed"
        );
    }

    // Verify complete row n=8: [1, 8, 28, 56, 70, 56, 28, 8, 1]
    let row8: [f64; 9] = [1.0, 8.0, 28.0, 56.0, 70.0, 56.0, 28.0, 8.0, 1.0];
    for k in 0..=8 {
        assert!(
            (plib::bin(8, k).unwrap() - row8[k as usize]).abs() <= CONFUSION,
            "C(8,{k}) verification failed"
        );
    }
}

// Tests for Binomial coefficient - Sum property
#[test]
fn PLibTest__BinomialCoefficient_SumProperty() {
    // Test sum property: Sum of C(n,k) for k=0 to n equals 2^n
    for n in 0..=20 {
        let mut sum: f64 = 0.0;
        let expected: f64 = 2.0_f64.powi(n);

        for k in 0..=n {
            sum += plib::bin(n, k).unwrap();
        }

        assert!(
            (sum - expected).abs() <= CONFUSION,
            "Sum property failed for n={n} (sum should be 2^{n}={expected})"
        );
    }
}
