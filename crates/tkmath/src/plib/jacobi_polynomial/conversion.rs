// Rust translation of FoundationClasses/TKMath/PLib/PLib_JacobiPolynomial.cxx (ToCoefficients) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Conversion of Jacobi coefficients to the canonical basis.
use super::{JacobiPolynomial, table_value};
use crate::plib::access::{array_cell, array_value};
use crate::plib::jacobi_tables::{self, ChunkedTable};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

static TRANS_MATRIX: [&ChunkedTable; 3] = [
    &jacobi_tables::TRANS_MATRIX_C0,
    &jacobi_tables::TRANS_MATRIX_C1,
    &jacobi_tables::TRANS_MATRIX_C2,
];
// Row length of the packed upper-triangular transfer matrices.
const MAX_M: i32 = 31;
// The odd-degree triangle follows the even one: MAX_M*(MAX_M+1)/2 entries.
const ODD_OFFSET: i32 = MAX_M * (MAX_M + 1) / 2;

/// Even and odd Jacobi coefficients feed disjoint canonical coefficients.
#[derive(Clone, Copy)]
struct Parity {
    first: i32,
    last_pair: i32,
    table_offset: i32,
}

impl JacobiPolynomial {
    /// Converts P(t) = R(t) + W(t) Q(t) to canonical coefficients
    /// (`PLib_JacobiPolynomial::ToCoefficients`).
    ///
    /// Both arrays are addressed from their lower bounds, Dimension values per degree;
    /// `jacobi.to_coefficients(1, 4, &jacobi_coefficients, &mut canonical)?`.
    pub fn to_coefficients(
        &self,
        dimension: i32,
        degree: i32,
        jac_coeff: &Array1<'_, f64>,
        coefficients: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let even = Parity {
            first: 0,
            last_pair: degree / 2,
            table_offset: 0,
        };
        self.convert_parity(dimension, even, jac_coeff, coefficients)?;
        if degree == 0 {
            return Ok(());
        }
        let odd = Parity {
            first: 1,
            last_pair: (degree - 1) / 2,
            table_offset: ODD_OFFSET,
        };
        self.convert_parity(dimension, odd, jac_coeff, coefficients)
    }

    // Canonical degree 2i+p sums transfer entries (i, j) times Jacobi degree 2j+p for j >= i.
    fn convert_parity(
        &self,
        dimension: i32,
        parity: Parity,
        jac_coeff: &Array1<'_, f64>,
        coefficients: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let table: &ChunkedTable = TRANS_MATRIX[self.level()];
        let double_dim: i32 = 2 * dimension;
        for i in 0..=parity.last_pair {
            let row: i32 = parity.table_offset + i * MAX_M - (i + 1) * i / 2;
            let target: i32 = coefficients.lower() + (2 * i + parity.first) * dimension;
            for idim in 1..=dimension {
                let mut source: i32 =
                    jac_coeff.lower() + (2 * i + parity.first) * dimension + idim - 1;
                let mut value: f64 = 0.0;
                for j in i..=parity.last_pair {
                    let entry: f64 = table_value(table, (row + j) as usize, "TransMatrix")?;
                    value += entry * array_value(jac_coeff, source)?;
                    source += double_dim;
                }
                *array_cell(coefficients, target + idim - 1)? = value;
            }
        }
        Ok(())
    }
}
