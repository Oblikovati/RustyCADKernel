// Rust translation of FoundationClasses/TKMath/PLib/PLib_JacobiPolynomial.cxx (D0123, FORTRAN MPOJAC) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Normalized Jacobi basis values and derivatives by three-term recurrence.
use super::{JacobiPolynomial, table_value};
use crate::plib::access::require_length;
use crate::plib::jacobi_tables::{self, ChunkedTable};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

// Degree-30 recurrence tables; lower degrees use their prefixes (proved by the generator).
static TNORM: [&ChunkedTable; 3] = [
    &jacobi_tables::TNORM_C0,
    &jacobi_tables::TNORM_C1,
    &jacobi_tables::TNORM_C2,
];
static COF_A: [&ChunkedTable; 3] = [
    &jacobi_tables::COF_A_C0,
    &jacobi_tables::COF_A_C1,
    &jacobi_tables::COF_A_C2,
];
static COF_B: [&ChunkedTable; 3] = [
    &jacobi_tables::COF_B_C0,
    &jacobi_tables::COF_B_C1,
    &jacobi_tables::COF_B_C2,
];
static DENOM: [&ChunkedTable; 3] = [
    &jacobi_tables::DENOM_C0,
    &jacobi_tables::DENOM_C1,
    &jacobi_tables::DENOM_C2,
];

/// Mutable outputs for a basis evaluation; rows above the requested order stay untouched.
///
/// `BasisRows { rows: [&mut values, &mut first, &mut second, &mut third] }`.
pub struct BasisRows<'r> {
    /// Values, then first, second and third derivatives.
    pub rows: [&'r mut [f64]; 4],
}

impl JacobiPolynomial {
    /// Evaluates the Degree+1 basis functions at `u` (`PLib_JacobiPolynomial::D0`);
    /// `jacobi.d0(0.5, &mut values)?`.
    pub fn d0(&self, u: f64, basis_value: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let rows = BasisRows {
            rows: [basis_value.as_mut_slice(), &mut [], &mut [], &mut []],
        };
        self.d0123(0, u, rows)
    }

    /// Evaluates values and first derivatives (`PLib_JacobiPolynomial::D1`);
    /// `jacobi.d1(0.5, &mut values, &mut d1)?`.
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

    /// Evaluates values and two derivatives (`PLib_JacobiPolynomial::D2`);
    /// `jacobi.d2(0.5, &mut values, &mut d1, &mut d2)?`.
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

    /// Evaluates values and three derivatives (`PLib_JacobiPolynomial::D3`);
    /// `jacobi.d3(0.5, &mut values, &mut d1, &mut d2, &mut d3)?`.
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

    /// Evaluates the requested derivative rows (`PLib_JacobiPolynomial::D0123`);
    /// `jacobi.d0123(2, 0.5, BasisRows { rows: [&mut value, &mut d1, &mut d2, &mut []] })?`.
    /// Rows start at each native lower bound; negative orders preserve the native initialized D1 path.
    /// For `NDeriv > 3`, OCCT computes rows 0–2 and still scales row 3.
    pub fn d0123(
        &self,
        native_order: i32,
        u: f64,
        mut basis: BasisRows<'_>,
    ) -> Result<(), Failure> {
        if native_order < 0 {
            return self.evaluate_negative_order(&mut basis, native_order, u);
        }
        let (compute_order, normalize_order) = jacobi_orders(native_order);
        let count: usize = self.degree as usize + 1;
        self.validate_rows(normalize_order, count, &basis.rows)?;
        self.seed_rows(compute_order, u, &mut basis.rows);
        self.recur_rows(compute_order, u, &mut basis.rows)?;
        self.scale_rows(normalize_order, count, &mut basis.rows)?;
        Ok(())
    }

    fn evaluate_negative_order(
        &self,
        basis: &mut BasisRows<'_>,
        _native_order: i32,
        u: f64,
    ) -> Result<(), Failure> {
        let count: usize = self.degree as usize + 1;
        self.validate_rows(1, count, &basis.rows)?;
        self.seed_rows(0, u, &mut basis.rows);
        self.recur_negative_rows(u, &mut basis.rows)?;
        self.scale_rows(1, count, &mut basis.rows)?;
        Ok(())
    }

    fn validate_rows(
        &self,
        order: usize,
        count: usize,
        rows: &[&mut [f64]; 4],
    ) -> Result<(), Failure> {
        for (derivative, row) in rows.iter().enumerate().take(order + 1) {
            require_length(
                "PLib_JacobiPolynomial::D0123",
                ["BasisValue", "BasisD1", "BasisD2", "BasisD3"][derivative],
                row.len(),
                count,
            )?;
        }
        Ok(())
    }

    fn scale_rows(
        &self,
        order: usize,
        count: usize,
        rows: &mut [&mut [f64]; 4],
    ) -> Result<(), Failure> {
        for index in 0..count {
            let factor: f64 = table_value(TNORM[self.level()], index, "TNorm")?;
            for row in rows.iter_mut().take(order + 1) {
                row[index] *= factor;
            }
        }
        Ok(())
    }

    fn recur_negative_rows(&self, u: f64, rows: &mut [&mut [f64]; 4]) -> Result<(), Failure> {
        for i in 2..=self.degree as usize {
            let (a, b, denom) = self.recurrence_coefficients(i)?;
            rows[0][i] = recurrence_term(a, b, denom, u, rows[0][i - 1], rows[0][i - 2], 0, 0.0);
            rows[1][i] = recurrence_term(
                a,
                b,
                denom,
                u,
                rows[1][i - 1],
                rows[1][i - 2],
                1,
                rows[0][i - 1],
            );
        }
        Ok(())
    }

    // Degree 0 and 1 terms ("positionnements triviaux").
    fn seed_rows(&self, order: usize, u: f64, rows: &mut [&mut [f64]; 4]) {
        rows[0][0] = 1.0;
        for row in rows.iter_mut().take(order + 1).skip(1) {
            row[0] = 0.0;
        }
        if self.degree == 0 {
            return;
        }
        let aux: f64 = f64::from(2 * (self.niv_constr + 1) + 1);
        rows[0][1] = aux * u;
        if order >= 1 {
            rows[1][1] = aux;
        }
        for row in rows.iter_mut().take(order + 1).skip(2) {
            row[1] = 0.0;
        }
    }

    // J_i = (A_i u J_{i-1} + B_i J_{i-2}) D_i, differentiated term by term.
    fn recur_rows(&self, order: usize, u: f64, rows: &mut [&mut [f64]; 4]) -> Result<(), Failure> {
        for i in 2..=self.degree.max(1) as usize {
            let (a, b, denom) = self.recurrence_coefficients(i)?;
            // Derivative k adds k times derivative k-1 inside the A term; k = 0 has no such term.
            for k in (0..=order).rev() {
                let lower: f64 = if k == 0 { 0.0 } else { rows[k - 1][i - 1] };
                rows[k][i] =
                    recurrence_term(a, b, denom, u, rows[k][i - 1], rows[k][i - 2], k, lower);
            }
        }
        Ok(())
    }

    fn recurrence_coefficients(&self, index: usize) -> Result<(f64, f64, f64), Failure> {
        let level: usize = self.level();
        let a: f64 = table_value(COF_A[level], index, "CofA")?;
        let b: f64 = table_value(COF_B[level], index, "CofB")?;
        let denom: f64 = table_value(DENOM[level], index, "Denom")?;
        Ok((a, b, denom))
    }
}

fn jacobi_orders(native_order: i32) -> (usize, usize) {
    if native_order > 3 {
        return (2, 3);
    }
    let order: usize = native_order as usize;
    (order, order)
}

#[allow(clippy::too_many_arguments)] // One scalar per recurrence operand keeps the native order visible.
fn recurrence_term(
    a: f64,
    b: f64,
    denom: f64,
    u: f64,
    previous: f64,
    before: f64,
    k: usize,
    lower: f64,
) -> f64 {
    match k {
        0 => (a * u * previous + b * before) * denom,
        1 => (a * (u * previous + lower) + b * before) * denom,
        _ => (a * (u * previous + k as f64 * lower) + b * before) * denom,
    }
}
