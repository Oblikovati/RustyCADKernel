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
// Rust translation of FoundationClasses/TKMath/PLib/PLib_HermitJacobi.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hermite-Jacobi basis: Hermite polynomials for the end constraints plus W(t) times Jacobi.
use super::access::{array_cell, array_value};
use super::hermite::hermite_coefficients;
use super::jacobi_polynomial::JacobiPolynomial;
use crate::geom_abs::Shape;
use crate::math::Matrix;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use std::sync::OnceLock;

mod basis;

// W(t) = (1 - t^2)^(NivConstr+1) in canonical coefficients.
const W_COEFFICIENTS: [&[f64]; 3] = [
    &[1.0, 0.0, -1.0],
    &[1.0, 0.0, -2.0, 0.0, 1.0],
    &[1.0, 0.0, -3.0, 0.0, 3.0, 0.0, -1.0],
];

/// Row-major Hermite bases on [-1, 1] for NivConstr 0..2, built once like the native statics.
static HERMITE_MATRICES: OnceLock<Result<[Vec<f64>; 3], Failure>> = OnceLock::new();

fn hermite_matrix(niv_constr: i32) -> Result<&'static [f64], Failure> {
    let matrices =
        HERMITE_MATRICES.get_or_init(|| Ok([hermite_rows(0)?, hermite_rows(1)?, hermite_rows(2)?]));
    match matrices {
        Ok(tables) => Ok(&tables[niv_constr as usize]),
        Err(error) => Err(error.clone()),
    }
}

fn hermite_rows(order: i32) -> Result<Vec<f64>, Failure> {
    let size: i32 = 2 * order + 2;
    let mut matrix: Matrix<'static> = Matrix::filled(1, size, 1, size, 0.0)
        .map_err(|error| Failure::out_of_range(&error.to_string()))?;
    if !hermite_coefficients(-1.0, 1.0, order, order, &mut matrix)? {
        return Err(Failure::new("PLib::HermiteCoefficients failed on [-1, 1]"));
    }
    let mut rows: Vec<f64> = Vec::with_capacity((size * size) as usize);
    for row in 1..=size {
        for col in 1..=size {
            rows.push(matrix[(row, col)]);
        }
    }
    Ok(rows)
}

/// Hermite-Jacobi basis for constrained approximation (`PLib_HermitJacobi`);
/// `HermitJacobi::new(10, Shape::C1)?.niv_constr() == 1`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HermitJacobi {
    jacobi: JacobiPolynomial,
}

impl HermitJacobi {
    /// Builds the Jacobi part with the same WorkDegree and constraint rules;
    /// `HermitJacobi::new(6, Shape::C0)?`.
    pub fn new(work_degree: i32, constraint_order: Shape) -> Result<Self, Failure> {
        Ok(Self {
            jacobi: JacobiPolynomial::new(work_degree, constraint_order)?,
        })
    }

    /// Returns WorkDegree; `basis.work_degree()`.
    pub fn work_degree(&self) -> i32 {
        self.jacobi.work_degree()
    }

    /// Returns NivConstr; `basis.niv_constr()`.
    pub fn niv_constr(&self) -> i32 {
        self.jacobi.niv_constr()
    }

    /// Delegates to the Jacobi estimate (`PLib_HermitJacobi::MaxError`);
    /// `basis.max_error(1, &coefficients, 6)?`.
    pub fn max_error(
        &self,
        dimension: i32,
        herm_jac_coeff: &[f64],
        new_degree: i32,
    ) -> Result<f64, Failure> {
        self.jacobi.max_error(dimension, herm_jac_coeff, new_degree)
    }

    /// Delegates to the Jacobi reduction, returning `(new_degree, max_error)`
    /// (`PLib_HermitJacobi::ReduceDegree`); `basis.reduce_degree(1, 8, 1e-6, &c)?`.
    pub fn reduce_degree(
        &self,
        dimension: i32,
        max_degree: i32,
        tol: f64,
        herm_jac_coeff: &[f64],
    ) -> Result<(i32, f64), Failure> {
        self.jacobi
            .reduce_degree(dimension, max_degree, tol, herm_jac_coeff)
    }

    /// Delegates to the Jacobi estimate (`PLib_HermitJacobi::AverageError`);
    /// `basis.average_error(1, &coefficients, 6)?`.
    pub fn average_error(
        &self,
        dimension: i32,
        herm_jac_coeff: &[f64],
        new_degree: i32,
    ) -> Result<f64, Failure> {
        self.jacobi
            .average_error(dimension, herm_jac_coeff, new_degree)
    }

    /// Converts P(t) = H(t) + W(t) Q(t) to canonical coefficients
    /// (`PLib_HermitJacobi::ToCoefficients`).
    ///
    /// The Hermite rows are expanded first; above the Hermite degree the combined
    /// array goes through the Jacobi conversion, as natively;
    /// `basis.to_coefficients(1, 4, &hermite_jacobi, &mut canonical)?`.
    pub fn to_coefficients(
        &self,
        dimension: i32,
        degree: i32,
        herm_jac_coeff: &Array1<'_, f64>,
        coefficients: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let degree_h: i32 = 2 * self.niv_constr() + 1;
        let upper: i32 = (degree + 1) * dimension - 1;
        let mut aux: Array1<'static, f64> = Array1::with_bounds(0, upper)
            .map_err(|error| Failure::out_of_range(&error.to_string()))?;
        self.expand_hermite(dimension, herm_jac_coeff, &mut aux)?;
        for k in (degree_h + 1) * dimension..=upper {
            *array_cell(&mut aux, k)? = array_value(herm_jac_coeff, herm_jac_coeff.lower() + k)?;
        }
        if degree > degree_h {
            return self
                .jacobi
                .to_coefficients(dimension, degree, &aux, coefficients);
        }
        let lower: i32 = coefficients.lower();
        for k in 0..=upper {
            *array_cell(coefficients, lower + k)? = array_value(&aux, k)?;
        }
        Ok(())
    }

    // Accumulates value/derivative coefficients times the Hermite basis rows at both ends.
    fn expand_hermite(
        &self,
        dimension: i32,
        herm_jac_coeff: &Array1<'_, f64>,
        aux: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let niv: i32 = self.niv_constr();
        let size: usize = 2 * niv as usize + 2;
        let hermite: &[f64] = hermite_matrix(niv)?;
        let start: i32 = herm_jac_coeff.lower();
        for k in 0..size {
            for i in 0..=niv {
                let h1: f64 = hermite[i as usize * size + k];
                let h2: f64 = hermite[(i + niv + 1) as usize * size + k];
                let (i1, i2) = (start + i * dimension, start + (i + niv + 1) * dimension);
                for idim in 0..dimension {
                    let term: f64 = array_value(herm_jac_coeff, i1 + idim)? * h1
                        + array_value(herm_jac_coeff, i2 + idim)? * h2;
                    *array_cell(aux, idim + k as i32 * dimension)? += term;
                }
            }
        }
        Ok(())
    }
}
