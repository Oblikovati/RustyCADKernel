// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

/// Output rows of one basis evaluation; rows above the requested order stay untouched.
pub(in crate::plib) struct BasisRows<'r> {
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

    /// Common part of D0..D3 (native protected `D0123`); rows start at each array's
    /// lower bound and must hold Degree+1 values up to the requested order.
    pub(in crate::plib) fn d0123(
        &self,
        order: usize,
        u: f64,
        mut basis: BasisRows<'_>,
    ) -> Result<(), Failure> {
        let count: usize = self.degree as usize + 1;
        for (derivative, row) in basis.rows.iter().enumerate().take(order + 1) {
            require_length(
                "PLib_JacobiPolynomial::D0123",
                ["BasisValue", "BasisD1", "BasisD2", "BasisD3"][derivative],
                row.len(),
                count,
            )?;
        }
        self.seed_rows(order, u, &mut basis.rows);
        self.recur_rows(order, u, &mut basis.rows)?;
        let norm: &ChunkedTable = TNORM[self.level()];
        for i in 0..count {
            let factor: f64 = table_value(norm, i, "TNorm")?;
            for row in basis.rows.iter_mut().take(order + 1) {
                row[i] *= factor;
            }
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
        let level: usize = self.level();
        for i in 2..=self.degree.max(1) as usize {
            let a: f64 = table_value(COF_A[level], i, "CofA")?;
            let b: f64 = table_value(COF_B[level], i, "CofB")?;
            let denom: f64 = table_value(DENOM[level], i, "Denom")?;
            // Derivative k adds k times derivative k-1 inside the A term; k = 0 has no such term.
            for k in (0..=order).rev() {
                let lower: f64 = if k == 0 { 0.0 } else { rows[k - 1][i - 1] };
                rows[k][i] =
                    recurrence_term(a, b, denom, u, rows[k][i - 1], rows[k][i - 2], k, lower);
            }
        }
        Ok(())
    }
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
