// Rust translation of FoundationClasses/TKMath/PLib/PLib_HermitJacobi.cxx (D0123, FORTRAN MPOBAS) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hermite-Jacobi basis values: Hermite rows first, then W(t) times Jacobi rows.
use super::{HermitJacobi, W_COEFFICIENTS, hermite_matrix};
use crate::plib::access::require_length;
use crate::plib::horner::{eval_polynomial, no_derivative_eval_polynomial};
use crate::plib::jacobi_polynomial::BasisRows;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

impl HermitJacobi {
    /// Evaluates the WorkDegree+1 basis functions at `u` (`PLib_HermitJacobi::D0`);
    /// `basis.d0(0.5, &mut values)?`.
    pub fn d0(&self, u: f64, basis_value: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let rows = BasisRows {
            rows: [basis_value.as_mut_slice(), &mut [], &mut [], &mut []],
        };
        self.d0123(0, u, rows)
    }

    /// Evaluates values and first derivatives (`PLib_HermitJacobi::D1`);
    /// `basis.d1(0.5, &mut values, &mut d1)?`.
    pub fn d1(
        &self,
        u: f64,
        basis_value: &mut Array1<'_, f64>,
        basis_d1: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let rows = BasisRows {
            rows: [
                basis_value.as_mut_slice(),
                basis_d1.as_mut_slice(),
                &mut [],
                &mut [],
            ],
        };
        self.d0123(1, u, rows)
    }

    /// Evaluates values and two derivatives (`PLib_HermitJacobi::D2`);
    /// `basis.d2(0.5, &mut values, &mut d1, &mut d2)?`.
    pub fn d2(
        &self,
        u: f64,
        basis_value: &mut Array1<'_, f64>,
        basis_d1: &mut Array1<'_, f64>,
        basis_d2: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let rows = [
            basis_value.as_mut_slice(),
            basis_d1.as_mut_slice(),
            basis_d2.as_mut_slice(),
            &mut [],
        ];
        self.d0123(2, u, BasisRows { rows })
    }

    /// Evaluates values and three derivatives (`PLib_HermitJacobi::D3`);
    /// `basis.d3(0.5, &mut values, &mut d1, &mut d2, &mut d3)?`.
    pub fn d3(
        &self,
        u: f64,
        basis_value: &mut Array1<'_, f64>,
        basis_d1: &mut Array1<'_, f64>,
        basis_d2: &mut Array1<'_, f64>,
        basis_d3: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let rows = [
            basis_value.as_mut_slice(),
            basis_d1.as_mut_slice(),
            basis_d2.as_mut_slice(),
            basis_d3.as_mut_slice(),
        ];
        self.d0123(3, u, BasisRows { rows })
    }

    // Native protected D0123: rows start at each array's lower bound.
    fn d0123(&self, order: usize, u: f64, basis: BasisRows<'_>) -> Result<(), Failure> {
        let work_count: usize = self.work_degree() as usize + 1;
        for (derivative, row) in basis.rows.iter().enumerate().take(order + 1) {
            let name: &str = ["BasisValue", "BasisD1", "BasisD2", "BasisD3"][derivative];
            require_length("PLib_HermitJacobi::D0123", name, row.len(), work_count)?;
        }
        let hermite: Vec<[f64; 4]> = self.hermite_values(order, u)?;
        let jacobi_count: usize = work_count - hermite.len();
        let mut jacobi: [Vec<f64>; 4] = std::array::from_fn(|_| vec![0.0; jacobi_count]);
        let [j0, j1, j2, j3] = &mut jacobi;
        self.jacobi.d0123(
            order,
            u,
            BasisRows {
                rows: [j0, j1, j2, j3],
            },
        )?;
        let weight: [f64; 4] = self.weight_values(order, u)?;
        let [value, d1, d2, d3] = basis.rows;
        let mut outputs: [&mut [f64]; 4] = [value, d1, d2, d3];
        for (derivative, output) in outputs.iter_mut().enumerate().take(order + 1) {
            for (i, row) in hermite.iter().enumerate() {
                output[i] = row[derivative];
            }
            for (j, slot) in output[hermite.len()..work_count].iter_mut().enumerate() {
                *slot = leibniz_term(derivative, &weight, &jacobi, j);
            }
        }
        Ok(())
    }

    // Values and derivatives of each Hermite polynomial (rows of the Hermite matrix).
    fn hermite_values(&self, order: usize, u: f64) -> Result<Vec<[f64; 4]>, Failure> {
        let degree_h: i32 = 2 * self.niv_constr() + 1;
        let size: usize = degree_h as usize + 1;
        let matrix: &[f64] = hermite_matrix(self.niv_constr())?;
        let mut values: Vec<[f64; 4]> = vec![[0.0; 4]; size];
        for (i, row) in values.iter_mut().enumerate() {
            let coefficients: &[f64] = &matrix[i * size..(i + 1) * size];
            evaluate(u, order, degree_h, coefficients, &mut row[..=order])?;
        }
        Ok(values)
    }

    // W(t) and its derivatives, the Jacobi weight of degree DegreeH+1.
    fn weight_values(&self, order: usize, u: f64) -> Result<[f64; 4], Failure> {
        let niv: i32 = self.niv_constr();
        let mut values: [f64; 4] = [0.0; 4];
        evaluate(
            u,
            order,
            2 * niv + 2,
            W_COEFFICIENTS[niv as usize],
            &mut values[..=order],
        )?;
        Ok(values)
    }
}

// Order 0 uses the dedicated value-only Horner path, as the native code does.
fn evaluate(
    u: f64,
    order: usize,
    degree: i32,
    coefficients: &[f64],
    out: &mut [f64],
) -> Result<(), Failure> {
    if order == 0 {
        return no_derivative_eval_polynomial(u, degree, 1, degree, coefficients, out);
    }
    eval_polynomial(u, order as i32, degree, 1, coefficients, out)
}

// Leibniz rule for (W J)^(k) with the native grouping of each order.
fn leibniz_term(order: usize, w: &[f64; 4], jacobi: &[Vec<f64>; 4], j: usize) -> f64 {
    let [j0, j1, j2, j3] = [jacobi[0][j], jacobi[1][j], jacobi[2][j], jacobi[3][j]];
    match order {
        0 => w[0] * j0,
        1 => w[0] * j1 + w[1] * j0,
        2 => w[0] * j2 + 2.0 * w[1] * j1 + w[2] * j0,
        _ => w[0] * j3 + w[3] * j0 + 3.0 * (w[1] * j2 + w[2] * j1),
    }
}
