// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Hessian eigenvalue shift of MathOpt_Newton.hxx, `MathOpt::Utils::RegularizeHessian`.
use super::is_finite_matrix;
use crate::math::{Matrix, Vector};
use crate::math_utils::EigenResult;

/// Shifts the diagonal so the smallest Jacobi eigenvalue reaches `minimum_eigenvalue`;
/// false when Jacobi fails or the shifted matrix is not finite;
/// `MathOpt::Utils::RegularizeHessian`.
///
/// ```
/// use rustycad_tkmath::{math::Matrix, math_opt::utils::regularize_hessian};
/// let mut h = Matrix::filled(0, 1, 0, 1, 0.0).unwrap();
/// *h.change_at(0, 0).unwrap() = -1.0;
/// assert!(regularize_hessian(&mut h, 1.0));
/// assert_eq!(*h.at(0, 0).unwrap(), 1.0);
/// ```
pub fn regularize_hessian(hessian: &mut Matrix<'_>, minimum_eigenvalue: f64) -> bool {
    let eigen: EigenResult = crate::math_lin::jacobi(hessian, false);
    if !eigen.is_done() {
        return false;
    }
    let lowest: f64 = lowest_eigenvalue(eigen.eigen_values.as_ref().unwrap());
    if lowest < minimum_eigenvalue {
        let shift: f64 = minimum_eigenvalue - lowest;
        for index in 0..hessian.row_number() as usize {
            *hessian.change_at(index, index).unwrap() += shift;
        }
    }
    is_finite_matrix(hessian)
}

/// Native `std::min` fold from the first offset: a NaN candidate never replaces the minimum.
fn lowest_eigenvalue(values: &Vector<'_>) -> f64 {
    let mut lowest: f64 = *values.at(0).unwrap();
    for candidate in &values.values()[1..] {
        if *candidate < lowest {
            lowest = *candidate;
        }
    }
    lowest
}
