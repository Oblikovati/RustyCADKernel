//! Direct translation of AppCont_ContMatrices_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR, EXPECT_THROW};
use rustycad_tkgeombase::app_cont::cont_matrices as AppCont_ContMatrices;
use rustycad_tkmath::math::{self, Matrix, Vector};

const THE_FORMULA_TOL: f64 = 1e-14;
const THE_SYMMETRY_TOL: f64 = 1e-15;

// Helper: compute binomial coefficient C(n,k).
fn testBinomCoeff(theN: i32, mut theK: i32) -> f64 {
    if theK < 0 || theK > theN {
        return 0.0;
    }
    if theK > theN - theK {
        theK = theN - theK;
    }
    let mut aResult: f64 = 1.0;
    for i in 0..theK {
        aResult = aResult * f64::from(theN - i) / f64::from(i + 1);
    }
    aResult
}

// Helper: compute Bernstein mass matrix entry M(i,j) for degree n (0-based).
fn testBernsteinMassEntry(theN: i32, theI: i32, theJ: i32) -> f64 {
    testBinomCoeff(theN, theI) * testBinomCoeff(theN, theJ)
        / (f64::from(2 * theN + 1) * testBinomCoeff(2 * theN, theI + theJ))
}

fn square(theSize: i32) -> Matrix<'static> {
    Matrix::with_bounds(1, theSize, 1, theSize).unwrap()
}

// Verify MMatrix entries match the closed-form Bernstein mass matrix formula.
#[test]
fn AppCont_ContMatricesTest__MMatrix_MatchesFormula() {
    for aClasse in 2..=20 {
        let mut aMat = square(aClasse);
        AppCont_ContMatrices::m_matrix(aClasse, &mut aMat).unwrap();

        let n = aClasse - 1;
        for i in 1..=aClasse {
            for j in 1..=aClasse {
                let anExpected = testBernsteinMassEntry(n, i - 1, j - 1);
                EXPECT_NEAR(aMat[(i, j)], anExpected, THE_FORMULA_TOL);
            }
        }
    }
}

// Verify MMatrix is symmetric.
#[test]
fn AppCont_ContMatricesTest__MMatrix_Symmetric() {
    for aClasse in 2..=20 {
        let mut aMat = square(aClasse);
        AppCont_ContMatrices::m_matrix(aClasse, &mut aMat).unwrap();

        for i in 1..=aClasse {
            for j in i + 1..=aClasse {
                EXPECT_NEAR(aMat[(i, j)], aMat[(j, i)], THE_SYMMETRY_TOL);
            }
        }
    }
}

// Verify InvMMatrix * MMatrix = Identity for low-to-moderate degrees.
#[test]
fn AppCont_ContMatricesTest__InvMMatrix_IsInverse() {
    for aClasse in 2..=12 {
        let mut aMat = square(aClasse);
        let mut anInvMat = square(aClasse);
        AppCont_ContMatrices::m_matrix(aClasse, &mut aMat).unwrap();
        AppCont_ContMatrices::inv_m_matrix(aClasse, &mut anInvMat).unwrap();

        let aProduct = &anInvMat * &aMat;
        for i in 1..=aClasse {
            for j in 1..=aClasse {
                let anExpected = if i == j { 1.0 } else { 0.0 };
                EXPECT_NEAR(aProduct[(i, j)], anExpected, 1e-8);
            }
        }
    }
}

// Verify InvMMatrix is symmetric (inverse of symmetric matrix is symmetric).
#[test]
fn AppCont_ContMatricesTest__InvMMatrix_Symmetric() {
    for aClasse in 2..=24 {
        let mut anInvMat = square(aClasse);
        AppCont_ContMatrices::inv_m_matrix(aClasse, &mut anInvMat).unwrap();

        for i in 1..=aClasse {
            for j in i + 1..=aClasse {
                EXPECT_DOUBLE_EQ(anInvMat[(i, j)], anInvMat[(j, i)]);
            }
        }
    }
}

// Verify strict legacy class limits for matrix providers.
#[test]
fn AppCont_ContMatricesTest__LegacyRangeLimits() {
    {
        let mut aMat24 = square(24);
        let mut aMat25 = square(25);
        assert!(AppCont_ContMatrices::m_matrix(24, &mut aMat24).is_ok());
        EXPECT_THROW(
            AppCont_ContMatrices::m_matrix(25, &mut aMat25),
            "Standard_DimensionError",
        );
    }

    {
        let mut anInv24 = square(24);
        let mut anInv25 = square(25);
        assert!(AppCont_ContMatrices::inv_m_matrix(24, &mut anInv24).is_ok());
        EXPECT_THROW(
            AppCont_ContMatrices::inv_m_matrix(25, &mut anInv25),
            "Standard_DimensionError",
        );
    }

    {
        let mut anIBP26 = square(24);
        let mut anIBP27 = square(25);
        assert!(AppCont_ContMatrices::ibp_matrix(26, &mut anIBP26).is_ok());
        EXPECT_THROW(
            AppCont_ContMatrices::ibp_matrix(27, &mut anIBP27),
            "Standard_DimensionError",
        );
    }

    {
        let mut anIBT26 = square(22);
        let mut anIBT27 = square(23);
        assert!(AppCont_ContMatrices::ibt_matrix(26, &mut anIBT26).is_ok());
        EXPECT_THROW(
            AppCont_ContMatrices::ibt_matrix(27, &mut anIBT27),
            "Standard_DimensionError",
        );
    }

    {
        let mut aVB26x24 = Matrix::with_bounds(1, 26, 1, 24).unwrap();
        let mut aVB27x24 = Matrix::with_bounds(1, 27, 1, 24).unwrap();
        let mut aVB26x25 = Matrix::with_bounds(1, 26, 1, 25).unwrap();
        assert!(AppCont_ContMatrices::v_bernstein(26, 24, &mut aVB26x24).is_ok());
        EXPECT_THROW(
            AppCont_ContMatrices::v_bernstein(27, 24, &mut aVB27x24),
            "Standard_DimensionError",
        );
        EXPECT_THROW(
            AppCont_ContMatrices::v_bernstein(26, 25, &mut aVB26x25),
            "Standard_DimensionError",
        );
    }
}

// Verify IBPMatrix is symmetric for degrees where legacy data retains symmetry.
#[test]
fn AppCont_ContMatricesTest__IBPMatrix_Symmetric() {
    for aClasse in 3..=22 {
        let aDim = aClasse - 2;
        let mut anIBP = square(aDim);
        AppCont_ContMatrices::ibp_matrix(aClasse, &mut anIBP).unwrap();

        for i in 1..=aDim {
            for j in i + 1..=aDim {
                EXPECT_DOUBLE_EQ(anIBP[(i, j)], anIBP[(j, i)]);
            }
        }
    }
}

// Verify IBTMatrix is symmetric and has correct dimensions for all cached classes.
#[test]
fn AppCont_ContMatricesTest__IBTMatrix_Symmetric() {
    for aClasse in 5..=26 {
        let aDim = aClasse - 4;
        let mut anIBT = square(aDim);
        AppCont_ContMatrices::ibt_matrix(aClasse, &mut anIBT).unwrap();

        for i in 1..=aDim {
            for j in i + 1..=aDim {
                EXPECT_DOUBLE_EQ(anIBT[(i, j)], anIBT[(j, i)]);
            }
        }
    }
}

// Verify VBernstein values match direct Bernstein evaluation at Gauss points.
#[test]
fn AppCont_ContMatricesTest__VBernstein_MatchesBernsteinEval() {
    for aClasse in 2..=20 {
        for aNbPts in [4, 8, 12, 20, 24] {
            let mut aVB = Matrix::with_bounds(1, aClasse, 1, aNbPts).unwrap();
            AppCont_ContMatrices::v_bernstein(aClasse, aNbPts, &mut aVB).unwrap();

            // Get Gauss-Legendre points on [-1, 1] and remap to [0, 1].
            let mut aGaussP = Vector::with_bounds(1, aNbPts).unwrap();
            math::gauss_points(aNbPts, &mut aGaussP);

            let mut aParams = Vector::with_bounds(1, aNbPts).unwrap();
            let aHalf = (aNbPts + 1) / 2;
            for k in 1..=aNbPts {
                let aT = 0.5 * (1.0 + aGaussP[k]);
                if k <= aHalf {
                    aParams[aNbPts - k + 1] = aT;
                } else {
                    aParams[k - aHalf] = aT;
                }
            }

            let n = aClasse - 1;
            for i in 1..=aClasse {
                let aI = i - 1;
                let aBinom = testBinomCoeff(n, aI);
                for j in 1..=aNbPts {
                    let aT: f64 = aParams[j];
                    let anExpected =
                        aBinom * aT.powf(f64::from(aI)) * (1.0 - aT).powf(f64::from(n - aI));
                    EXPECT_NEAR(aVB[(i, j)], anExpected, THE_FORMULA_TOL);
                }
            }
        }
    }
}

// Verify partition of unity: sum of Bernstein basis values at each point should be 1.
#[test]
fn AppCont_ContMatricesTest__VBernstein_PartitionOfUnity() {
    for aClasse in [3, 5, 10, 20] {
        let aNbPts = 12;
        let mut aVB = Matrix::with_bounds(1, aClasse, 1, aNbPts).unwrap();
        AppCont_ContMatrices::v_bernstein(aClasse, aNbPts, &mut aVB).unwrap();

        for j in 1..=aNbPts {
            let mut aSum = 0.0;
            for i in 1..=aClasse {
                aSum += aVB[(i, j)];
            }
            EXPECT_NEAR(aSum, 1.0, 1e-13);
        }
    }
}
