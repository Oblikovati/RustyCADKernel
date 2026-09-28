// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Direct translation of PLib_JacobiPolynomial_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, unused_mut, unused_assignments)]
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::plib::JacobiPolynomial;

fn reals(lower: i32, upper: i32) -> Array1<'static, f64> {
    Array1::with_bounds(lower, upper).unwrap()
}

// Test constructor and basic properties
#[test]
fn PLibJacobiPolynomialTest__ConstructorAndBasicProperties() {
    // Test with different constraint orders
    let aJacC0 = JacobiPolynomial::new(10, Shape::C0).unwrap();
    let aJacC1 = JacobiPolynomial::new(15, Shape::C1).unwrap();
    let aJacC2 = JacobiPolynomial::new(20, Shape::C2).unwrap();

    // Test WorkDegree property
    assert_eq!(aJacC0.work_degree(), 10);
    assert_eq!(aJacC1.work_degree(), 15);
    assert_eq!(aJacC2.work_degree(), 20);

    // Test NivConstr property
    assert_eq!(aJacC0.niv_constr(), 0);
    assert_eq!(aJacC1.niv_constr(), 1);
    assert_eq!(aJacC2.niv_constr(), 2);
}

// Test constructor with edge cases
#[test]
fn PLibJacobiPolynomialTest__ConstructorEdgeCases() {
    // Test minimum valid WorkDegree for each constraint order
    // For C0: WorkDegree >= 2 to get myDegree >= 0
    let aJacMinC0 = JacobiPolynomial::new(2, Shape::C0).unwrap();
    assert_eq!(aJacMinC0.work_degree(), 2);

    // For C1: WorkDegree >= 4 to get myDegree >= 0
    let aJacMinC1 = JacobiPolynomial::new(4, Shape::C1).unwrap();
    assert_eq!(aJacMinC1.work_degree(), 4);

    // For C2: WorkDegree >= 6 to get myDegree >= 0
    let aJacMinC2 = JacobiPolynomial::new(6, Shape::C2).unwrap();
    assert_eq!(aJacMinC2.work_degree(), 6);

    // Test reasonable maximum WorkDegree
    let aJacMax = JacobiPolynomial::new(30, Shape::C0).unwrap();
    assert_eq!(aJacMax.work_degree(), 30);

    // Test reasonable high degrees
    let aJacHigh = JacobiPolynomial::new(25, Shape::C0).unwrap();
    assert!(
        aJacHigh.work_degree() > 20,
        "High degree should be supported"
    );
}

// Test Gauss integration points
#[test]
fn PLibJacobiPolynomialTest__GaussIntegrationPoints() {
    let aJac = JacobiPolynomial::new(10, Shape::C0).unwrap();

    // Test various numbers of Gauss points (only valid values supported by OCCT)
    let aGaussNumbers: Vec<i32> = vec![8, 10, 15, 20, 25, 30, 40, 50, 61];

    for aNbGauss in aGaussNumbers {
        if aNbGauss > aJac.work_degree() {
            let mut aPoints = reals(0, aNbGauss / 2);

            assert!(
                aJac.points(aNbGauss, &mut aPoints).is_ok(),
                "Points calculation failed for {aNbGauss} Gauss points"
            );

            // Verify points are in valid range and ordered
            for i in aPoints.lower()..=aPoints.upper() {
                if i == 0 && aNbGauss % 2 == 0 {
                    // For even number of Gauss points, TabPoints(0) is set to UNDEFINED (-999)
                    assert_eq!(
                        aPoints[i], -999.0,
                        "TabPoints(0) should be UNDEFINED for even NbGaussPoints"
                    );
                } else if i == 0 && aNbGauss % 2 == 1 {
                    // For odd number of Gauss points, TabPoints(0) should be 0
                    assert_eq!(
                        aPoints[i], 0.0,
                        "TabPoints(0) should be 0 for odd NbGaussPoints"
                    );
                } else {
                    // Other points should be positive and <= 1
                    assert!(aPoints[i] > 0.0, "Gauss point should be positive");
                    assert!(aPoints[i] <= 1.0, "Gauss point should be <= 1");

                    if i > aPoints.lower() && i > 0 {
                        assert!(
                            aPoints[i] >= aPoints[i - 1],
                            "Gauss points should be ordered"
                        );
                    }
                }
            }
        }
    }
}

// Test Gauss integration weights
#[test]
fn PLibJacobiPolynomialTest__GaussIntegrationWeights() {
    let aJac = JacobiPolynomial::new(8, Shape::C1).unwrap();

    let aNbGauss: i32 = 15; // Must be > degree for valid computation
    let mut aWeights: Array2<f64> =
        Array2::with_bounds(0, aNbGauss / 2, 0, aJac.work_degree()).unwrap();

    aJac.weights(aNbGauss, &mut aWeights).unwrap();

    // Basic sanity checks on weights - the array is 2D with specific bounds
    assert_eq!(aWeights.lower_row(), 0, "Lower row should be 0");
    assert_eq!(aWeights.upper_row(), aNbGauss / 2, "Upper row mismatch");
    assert_eq!(aWeights.lower_col(), 0, "Lower col should be 0");
    assert_eq!(
        aWeights.upper_col(),
        aJac.work_degree(),
        "Upper col should match work degree"
    );

    for i in aWeights.lower_row()..=aWeights.upper_row() {
        for j in aWeights.lower_col()..=aWeights.upper_col() {
            assert!(
                !precision::is_infinite(aWeights[(i, j)]),
                "Weight should be finite at ({i},{j})"
            );
        }
    }
}

// Test MaxValue computation
#[test]
fn PLibJacobiPolynomialTest__MaxValue() {
    let aJac = JacobiPolynomial::new(10, Shape::C0).unwrap();

    let aTabSize: i32 = aJac.work_degree() - 2 * (aJac.niv_constr() + 1);
    if aTabSize > 0 {
        let mut aTabMax = reals(0, aTabSize);

        aJac.max_value(&mut aTabMax).unwrap();

        // Verify all max values are positive (they represent maximum absolute values)
        for i in aTabMax.lower()..=aTabMax.upper() {
            assert!(
                aTabMax[i] > 0.0,
                "Max value should be positive at index {i}"
            );
            assert!(
                !precision::is_infinite(aTabMax[i]),
                "Max value should be finite at index {i}"
            );
        }
    }
}

// Test basis function evaluation D0
#[test]
fn PLibJacobiPolynomialTest__BasisFunctionD0() {
    let aJac = JacobiPolynomial::new(6, Shape::C0).unwrap();

    // Calculate actual number of basis functions
    let aDegree: i32 = aJac.work_degree() - 2 * (aJac.niv_constr() + 1);

    let mut aBasisValue = reals(0, aDegree);

    // Test at various parameter values
    let aTestParams: Vec<f64> = vec![-1.0, -0.5, 0.0, 0.5, 1.0];

    for aU in aTestParams {
        aJac.d0(aU, &mut aBasisValue).unwrap();

        // Basic sanity checks
        for i in aBasisValue.lower()..=aBasisValue.upper() {
            assert!(
                !precision::is_infinite(aBasisValue[i]),
                "Basis value should be finite at index {i}, U={aU}"
            );
        }
    }
}

// Test basis function evaluation with derivatives
#[test]
fn PLibJacobiPolynomialTest__BasisFunctionDerivatives() {
    let aJac = JacobiPolynomial::new(8, Shape::C1).unwrap();

    // Calculate actual number of basis functions (same as MaxValue test pattern)
    let aDegree: i32 = aJac.work_degree() - 2 * (aJac.niv_constr() + 1);

    let mut aBasisValue = reals(0, aDegree);
    let mut aBasisD1 = reals(0, aDegree);
    let mut aBasisD2 = reals(0, aDegree);
    let mut aBasisD3 = reals(0, aDegree);

    let aU: f64 = 0.5; // Test at middle point

    // Test D1, D2, D3 evaluations
    aJac.d1(aU, &mut aBasisValue, &mut aBasisD1).unwrap();
    aJac.d2(aU, &mut aBasisValue, &mut aBasisD1, &mut aBasisD2)
        .unwrap();
    aJac.d3(
        aU,
        &mut aBasisValue,
        &mut aBasisD1,
        &mut aBasisD2,
        &mut aBasisD3,
    )
    .unwrap();

    // Verify all values are finite
    for i in aBasisValue.lower()..=aBasisValue.upper() {
        assert!(
            !precision::is_infinite(aBasisValue[i]),
            "Basis value should be finite at {i}"
        );
        assert!(
            !precision::is_infinite(aBasisD1[i]),
            "First derivative should be finite at {i}"
        );
        assert!(
            !precision::is_infinite(aBasisD2[i]),
            "Second derivative should be finite at {i}"
        );
        assert!(
            !precision::is_infinite(aBasisD3[i]),
            "Third derivative should be finite at {i}"
        );
    }
}

// Test coefficient conversion
#[test]
fn PLibJacobiPolynomialTest__CoefficientConversion() {
    let aWorkDegree: i32 = 6; // Use smaller degree that works well with ToCoefficients
    let aJac = JacobiPolynomial::new(aWorkDegree, Shape::C0).unwrap();

    let aDimension: i32 = 1;
    let aDegree: i32 = aJac.work_degree() - 2 * (aJac.niv_constr() + 1); // Use computational degree

    // Create test Jacobi coefficients with proper size
    // ToCoefficients expects arrays sized based on the degree and dimension
    // Analysis shows we need arrays that can handle the indexing patterns in the method
    let aJacSize: i32 = (aDegree + 1) * aDimension;
    let aCoeffSize: i32 = (aDegree + 1) * aDimension;

    // Use 0-based arrays to match the ToCoefficients indexing expectations
    let mut aJacCoeff = reals(0, aJacSize - 1);
    for i in aJacCoeff.lower()..=aJacCoeff.upper() {
        aJacCoeff[i] = (f64::from(i) * 0.1).sin(); // Some test values
    }

    let mut aCoefficients = reals(0, aCoeffSize - 1);

    aJac.to_coefficients(aDimension, aDegree, &aJacCoeff, &mut aCoefficients)
        .unwrap();

    // Verify output is finite
    for i in aCoefficients.lower()..=aCoefficients.upper() {
        assert!(
            !precision::is_infinite(aCoefficients[i]),
            "Converted coefficient should be finite at index {i}"
        );
    }
}

// Test degree reduction
#[test]
fn PLibJacobiPolynomialTest__DegreeReduction() {
    let aJac = JacobiPolynomial::new(10, Shape::C0).unwrap();

    let aDimension: i32 = 1;
    let aMaxDegree: i32 = 8;
    let aTol: f64 = 1e-6;

    // Create test coefficients - must be sized for full WorkDegree
    let aWorkDegree: i32 = aJac.work_degree();
    let mut aCoeff = reals(1, (aWorkDegree + 1) * aDimension);
    for i in aCoeff.lower()..=aCoeff.upper() {
        aCoeff[i] = 1.0 / f64::from(i + 1); // Decreasing coefficients to allow reduction
    }

    let mut aNewDegree: i32 = -1;
    let mut aMaxError: f64 = -1.0;

    (aNewDegree, aMaxError) = aJac
        .reduce_degree(aDimension, aMaxDegree, aTol, aCoeff.as_slice())
        .unwrap();

    // Verify results are reasonable
    assert!(
        aNewDegree <= aMaxDegree,
        "New degree should not exceed max degree"
    );
    assert!(aNewDegree >= 0, "New degree should be non-negative");
    assert!(aMaxError >= 0.0, "Max error should be non-negative");
    assert!(
        !precision::is_infinite(aMaxError),
        "Max error should be finite"
    );
}

// Test error estimation
#[test]
fn PLibJacobiPolynomialTest__ErrorEstimation() {
    let aJac = JacobiPolynomial::new(8, Shape::C1).unwrap();

    let aDimension: i32 = 1;

    // Create test coefficients
    let mut aCoeff = reals(1, 10 * aDimension);
    for i in aCoeff.lower()..=aCoeff.upper() {
        aCoeff[i] = 1.0 / f64::from(i + 1);
    }

    let aNewDegree: i32 = 6; // Reduced from original

    // Test MaxError
    let aMaxErr: f64 = aJac
        .max_error(aDimension, aCoeff.as_slice(), aNewDegree)
        .unwrap();

    assert!(aMaxErr >= 0.0, "Max error should be non-negative");
    assert!(
        !precision::is_infinite(aMaxErr),
        "Max error should be finite"
    );

    // Test AverageError
    let aAvgErr: f64 = aJac
        .average_error(aDimension, aCoeff.as_slice(), aNewDegree)
        .unwrap();

    assert!(aAvgErr >= 0.0, "Average error should be non-negative");
    assert!(
        !precision::is_infinite(aAvgErr),
        "Average error should be finite"
    );

    // Average error should typically be <= max error
    assert!(
        aAvgErr <= aMaxErr + precision::confusion(),
        "Average error should not exceed max error significantly"
    );
}

// Performance and stress tests
#[test]
fn PLibJacobiPolynomialTest__StressTests() {
    // Test with maximum degree
    let aJacMax = JacobiPolynomial::new(30, Shape::C2).unwrap();

    // Calculate actual number of basis functions
    let aDegree: i32 = aJacMax.work_degree() - 2 * (aJacMax.niv_constr() + 1);

    // Test that basic operations work with high degrees
    let mut aBasisValue = reals(0, aDegree);

    aJacMax.d0(0.0, &mut aBasisValue).unwrap();
    aJacMax.d0(0.5, &mut aBasisValue).unwrap();
    aJacMax.d0(1.0, &mut aBasisValue).unwrap();

    // Test with extreme parameter values
    let aExtremeParams: Vec<f64> = vec![-0.99999, -1e-10, 1e-10, 0.99999];

    for aU in aExtremeParams {
        aJacMax.d0(aU, &mut aBasisValue).unwrap();
        // Verify basis values are finite
        for i in aBasisValue.lower()..=aBasisValue.upper() {
            assert!(
                !precision::is_infinite(aBasisValue[i]),
                "Basis value should be finite"
            );
        }
    }
}
