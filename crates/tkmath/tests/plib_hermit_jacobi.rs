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

//! Direct translation of PLib_HermitJacobi_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, unused_mut, unused_assignments)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::plib::HermitJacobi;

fn reals(lower: i32, upper: i32) -> Array1<'static, f64> {
    Array1::with_bounds(lower, upper).unwrap()
}

// Test constructor and basic properties
#[test]
fn PLibHermitJacobiTest__ConstructorAndBasicProperties() {
    // Test with different constraint orders
    let aHermC0 = HermitJacobi::new(10, Shape::C0).unwrap();
    let aHermC1 = HermitJacobi::new(15, Shape::C1).unwrap();
    let aHermC2 = HermitJacobi::new(20, Shape::C2).unwrap();

    // Test WorkDegree property
    assert_eq!(aHermC0.work_degree(), 10);
    assert_eq!(aHermC1.work_degree(), 15);
    assert_eq!(aHermC2.work_degree(), 20);

    // Test NivConstr property
    assert_eq!(aHermC0.niv_constr(), 0);
    assert_eq!(aHermC1.niv_constr(), 1);
    assert_eq!(aHermC2.niv_constr(), 2);
}

// Test basis function evaluation D0
#[test]
fn PLibHermitJacobiTest__BasisFunctionD0() {
    let aHerm = HermitJacobi::new(6, Shape::C0).unwrap();

    let mut aBasisValue = reals(0, aHerm.work_degree());

    // Test at various parameter values
    let aTestParams: Vec<f64> = vec![-1.0, -0.5, 0.0, 0.5, 1.0];

    for aU in aTestParams {
        assert!(
            aHerm.d0(aU, &mut aBasisValue).is_ok(),
            "D0 evaluation failed at U={aU}"
        );

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
fn PLibHermitJacobiTest__BasisFunctionDerivatives() {
    let aHerm = HermitJacobi::new(8, Shape::C1).unwrap();

    let mut aBasisValue = reals(0, aHerm.work_degree());
    let mut aBasisD1 = reals(0, aHerm.work_degree());
    let mut aBasisD2 = reals(0, aHerm.work_degree());
    let mut aBasisD3 = reals(0, aHerm.work_degree());

    let aU: f64 = 0.5; // Test at middle point

    // Test D1
    assert!(
        aHerm.d1(aU, &mut aBasisValue, &mut aBasisD1).is_ok(),
        "D1 evaluation failed"
    );

    // Test D2
    assert!(
        aHerm
            .d2(aU, &mut aBasisValue, &mut aBasisD1, &mut aBasisD2)
            .is_ok(),
        "D2 evaluation failed"
    );

    // Test D3
    assert!(
        aHerm
            .d3(
                aU,
                &mut aBasisValue,
                &mut aBasisD1,
                &mut aBasisD2,
                &mut aBasisD3
            )
            .is_ok(),
        "D3 evaluation failed"
    );

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
fn PLibHermitJacobiTest__CoefficientConversion() {
    let aWorkDegree: i32 = 6; // Use smaller degree that works well with ToCoefficients
    let aHerm = HermitJacobi::new(aWorkDegree, Shape::C0).unwrap();

    let aDimension: i32 = 1;
    let aDegree: i32 = aHerm.work_degree() - 2 * (aHerm.niv_constr() + 1); // Use computational degree

    // Create test HermitJacobi coefficients with proper size
    // ToCoefficients expects arrays sized based on the degree and dimension
    let aHermJacSize: i32 = (aDegree + 1) * aDimension;
    let aCoeffSize: i32 = (aDegree + 1) * aDimension;

    // Use 0-based arrays to match the ToCoefficients indexing expectations
    let mut aHermJacCoeff = reals(0, aHermJacSize - 1);
    for i in aHermJacCoeff.lower()..=aHermJacCoeff.upper() {
        aHermJacCoeff[i] = (f64::from(i) * 0.3).sin(); // Some test values
    }

    let mut aCoefficients = reals(0, aCoeffSize - 1);

    assert!(
        aHerm
            .to_coefficients(aDimension, aDegree, &aHermJacCoeff, &mut aCoefficients)
            .is_ok(),
        "Coefficient conversion failed"
    );

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
fn PLibHermitJacobiTest__DegreeReduction() {
    let aHerm = HermitJacobi::new(10, Shape::C0).unwrap();

    let aDimension: i32 = 1;
    let aMaxDegree: i32 = 8;
    let aTol: f64 = 1e-6;

    // Create test coefficients - must be sized for full WorkDegree
    let aWorkDegree: i32 = aHerm.work_degree();
    let mut aCoeff = reals(1, (aWorkDegree + 1) * aDimension);
    for i in aCoeff.lower()..=aCoeff.upper() {
        aCoeff[i] = 1.0 / f64::from(i + 1); // Decreasing coefficients to allow reduction
    }

    let mut aNewDegree: i32 = -1;
    let mut aMaxError: f64 = -1.0;

    let aReduction = aHerm.reduce_degree(aDimension, aMaxDegree, aTol, aCoeff.as_slice());
    assert!(aReduction.is_ok(), "Degree reduction failed");
    (aNewDegree, aMaxError) = aReduction.unwrap();

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
fn PLibHermitJacobiTest__ErrorEstimation() {
    let aHerm = HermitJacobi::new(8, Shape::C1).unwrap();

    let aDimension: i32 = 1;

    // Create test coefficients
    let mut aCoeff = reals(1, 10 * aDimension);
    for i in aCoeff.lower()..=aCoeff.upper() {
        aCoeff[i] = 1.0 / f64::from(i + 1);
    }

    let aNewDegree: i32 = 6; // Reduced from original

    // Test MaxError
    let mut aMaxErr: f64 = -1.0;
    let aMaxResult = aHerm.max_error(aDimension, aCoeff.as_slice(), aNewDegree);
    assert!(aMaxResult.is_ok(), "MaxError calculation failed");
    aMaxErr = aMaxResult.unwrap();

    assert!(aMaxErr >= 0.0, "Max error should be non-negative");
    assert!(
        !precision::is_infinite(aMaxErr),
        "Max error should be finite"
    );

    // Test AverageError
    let mut aAvgErr: f64 = -1.0;
    let aAvgResult = aHerm.average_error(aDimension, aCoeff.as_slice(), aNewDegree);
    assert!(aAvgResult.is_ok(), "AverageError calculation failed");
    aAvgErr = aAvgResult.unwrap();

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

// Test extreme parameter values
#[test]
fn PLibHermitJacobiTest__ExtremeParameterValues() {
    let aHerm = HermitJacobi::new(10, Shape::C2).unwrap();

    let mut aBasisValue = reals(0, aHerm.work_degree());

    // Test with boundary values
    let aExtremeParams: Vec<f64> = vec![-0.99999, -1e-12, 1e-12, 0.99999];

    for aU in aExtremeParams {
        assert!(
            aHerm.d0(aU, &mut aBasisValue).is_ok(),
            "Extreme parameter U={aU} should not crash"
        );

        // Check that results are finite
        for i in aBasisValue.lower()..=aBasisValue.upper() {
            assert!(
                !precision::is_infinite(aBasisValue[i]),
                "Basis value should be finite at extreme parameter U={aU}"
            );
        }
    }
}

// Test consistency between different derivative orders
#[test]
fn PLibHermitJacobiTest__DerivativeConsistency() {
    let aHerm = HermitJacobi::new(6, Shape::C2).unwrap();

    let mut aBasisValue1 = reals(0, aHerm.work_degree());
    let mut aBasisD1_1 = reals(0, aHerm.work_degree());

    let mut aBasisValue2 = reals(0, aHerm.work_degree());
    let mut aBasisD1_2 = reals(0, aHerm.work_degree());
    let mut aBasisD2_2 = reals(0, aHerm.work_degree());

    let aU: f64 = 0.3;

    // Get values from D1 and D2 calls
    aHerm.d1(aU, &mut aBasisValue1, &mut aBasisD1_1).unwrap();
    aHerm
        .d2(aU, &mut aBasisValue2, &mut aBasisD1_2, &mut aBasisD2_2)
        .unwrap();

    // Values and first derivatives should be consistent
    for i in aBasisValue1.lower()..=aBasisValue1.upper() {
        assert!(
            (aBasisValue1[i] - aBasisValue2[i]).abs() <= precision::confusion(),
            "Function values should be consistent between D1 and D2 calls at index {i}"
        );
        assert!(
            (aBasisD1_1[i] - aBasisD1_2[i]).abs() <= precision::confusion(),
            "First derivatives should be consistent between D1 and D2 calls at index {i}"
        );
    }
}

// Performance test with maximum degree
#[test]
fn PLibHermitJacobiTest__PerformanceTest() {
    let aHermMax = HermitJacobi::new(30, Shape::C2).unwrap();

    let mut aBasisValue = reals(0, aHermMax.work_degree());
    let mut aBasisD1 = reals(0, aHermMax.work_degree());
    let mut aBasisD2 = reals(0, aHermMax.work_degree());
    let mut aBasisD3 = reals(0, aHermMax.work_degree());

    // Test that operations complete in reasonable time
    let aTestParams: Vec<f64> = vec![-0.8, -0.5, 0.0, 0.5, 0.8];

    for aU in aTestParams {
        let aRun = (|| {
            aHermMax.d0(aU, &mut aBasisValue)?;
            aHermMax.d1(aU, &mut aBasisValue, &mut aBasisD1)?;
            aHermMax.d2(aU, &mut aBasisValue, &mut aBasisD1, &mut aBasisD2)?;
            aHermMax.d3(
                aU,
                &mut aBasisValue,
                &mut aBasisD1,
                &mut aBasisD2,
                &mut aBasisD3,
            )
        })();
        assert!(
            aRun.is_ok(),
            "High degree operations should complete without crashing at U={aU}"
        );
    }
}
