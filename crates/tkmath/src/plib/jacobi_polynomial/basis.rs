// Rust translation of FoundationClasses/TKMath/PLib/PLib_JacobiPolynomial.cxx (D0123, FORTRAN MPOJAC) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Normalized Jacobi basis values and derivatives by three-term recurrence.
use super::{JacobiPolynomial, table_value};
use crate::plib::jacobi_tables::{self, ChunkedTable};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

use super::storage::{IndexedRows, OutputRows, SeparateRows};

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

/// Mutable outputs for a basis evaluation; D0–D3 calls leave rows above their order untouched.
///
/// `BasisRows { rows: [&mut values, &mut first, &mut second, &mut third] }`.
/// Separate slices cannot alias one native array across argument positions.
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
    pub fn d0123(&self, native_order: i32, u: f64, basis: BasisRows<'_>) -> Result<(), Failure> {
        let mut rows: SeparateRows<'_> = SeparateRows::new(basis.rows);
        self.evaluate_rows(native_order, u, &mut rows)
    }

    /// Evaluates rows over one caller buffer, preserving native write order when spans overlap;
    /// `jacobi.d0123_overlapping(2, 0.5, &mut storage, [0, 0, 2, 3])?`.
    pub fn d0123_overlapping(
        &self,
        native_order: i32,
        u: f64,
        values: &mut [f64],
        row_starts: [usize; 4],
    ) -> Result<(), Failure> {
        let (_, normalize_order) = jacobi_orders(native_order);
        let count: usize = self.degree as usize + 1;
        let mut rows: IndexedRows<'_> =
            IndexedRows::new(values, row_starts, count, normalize_order)?;
        self.evaluate_rows(native_order, u, &mut rows)
    }

    fn evaluate_rows<S: OutputRows>(
        &self,
        native_order: i32,
        u: f64,
        rows: &mut S,
    ) -> Result<(), Failure> {
        let (compute_order, normalize_order) = jacobi_orders(native_order);
        let count: usize = self.degree as usize + 1;
        validate_rows(rows, normalize_order, count)?;
        seed_rows(self, native_order, u, rows);
        recur_rows(self, compute_order, u, rows)?;
        scale_rows(self, normalize_order, count, rows)?;
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
    if native_order < 0 {
        return (1, 1);
    }
    if native_order > 3 {
        return (2, 3);
    }
    let order: usize = native_order as usize;
    (order, order)
}

fn validate_rows<S: OutputRows>(rows: &S, order: usize, count: usize) -> Result<(), Failure> {
    for row in 0..=order {
        if rows.len(row) < count {
            return Err(Failure::out_of_range(&format!(
                "PLib_JacobiPolynomial::D0123 row {row} has length {}, expected {count}",
                rows.len(row)
            )));
        }
    }
    Ok(())
}

fn seed_rows<S: OutputRows>(basis: &JacobiPolynomial, native_order: i32, u: f64, rows: &mut S) {
    rows.write(0, 0, 1.0);
    if basis.degree == 0 {
        seed_constant_derivatives(native_order, rows);
        return;
    }
    seed_linear_value(basis, u, rows);
    seed_derivatives(basis, native_order, rows);
}

fn seed_constant_derivatives<S: OutputRows>(native_order: i32, rows: &mut S) {
    if native_order >= 1 {
        rows.write(1, 0, 0.0);
    }
    if native_order >= 2 {
        rows.write(2, 0, 0.0);
    }
    if native_order == 3 {
        rows.write(3, 0, 0.0);
    }
}

fn seed_linear_value<S: OutputRows>(basis: &JacobiPolynomial, u: f64, rows: &mut S) {
    let aux: f64 = f64::from(2 * (basis.niv_constr + 1) + 1);
    rows.write(0, 1, aux * u);
}

fn seed_derivatives<S: OutputRows>(basis: &JacobiPolynomial, native_order: i32, rows: &mut S) {
    if native_order < 1 {
        return;
    }
    rows.write(1, 0, 0.0);
    let aux: f64 = f64::from(2 * (basis.niv_constr + 1) + 1);
    rows.write(1, 1, aux);
    if native_order >= 2 {
        rows.write(2, 0, 0.0);
        rows.write(2, 1, 0.0);
    }
    if native_order == 3 {
        rows.write(3, 0, 0.0);
        rows.write(3, 1, 0.0);
    }
}

fn recur_rows<S: OutputRows>(
    basis: &JacobiPolynomial,
    compute_order: usize,
    u: f64,
    rows: &mut S,
) -> Result<(), Failure> {
    if basis.degree <= 1 {
        return Ok(());
    }
    for index in 2..=basis.degree as usize {
        recur_index(basis, compute_order, u, index, rows)?;
    }
    Ok(())
}

fn recur_index<S: OutputRows>(
    basis: &JacobiPolynomial,
    compute_order: usize,
    u: f64,
    index: usize,
    rows: &mut S,
) -> Result<(), Failure> {
    let (a, b, denom) = basis.recurrence_coefficients(index)?;
    for row in 0..=compute_order {
        let lower: f64 = if row == 0 {
            0.0
        } else {
            rows.read(row - 1, index - 1)
        };
        let value: f64 = recurrence_term(
            a,
            b,
            denom,
            u,
            rows.read(row, index - 1),
            rows.read(row, index - 2),
            row,
            lower,
        );
        rows.write(row, index, value);
    }
    Ok(())
}

fn scale_rows<S: OutputRows>(
    basis: &JacobiPolynomial,
    order: usize,
    count: usize,
    rows: &mut S,
) -> Result<(), Failure> {
    for index in 0..count {
        let factor: f64 = table_value(TNORM[basis.level()], index, "TNorm")?;
        for row in 0..=order {
            rows.write(row, index, rows.read(row, index) * factor);
        }
    }
    Ok(())
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
