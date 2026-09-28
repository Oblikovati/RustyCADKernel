// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/PLib at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! PLib: polynomial evaluation, interpolation and basis conversion.
//!
//! Native `double&` buffer arguments become slices. Where OCCT would read or
//! write outside such a buffer, or where a negative degree, dimension or
//! derivative count makes its pointer arithmetic undefined, the port returns a
//! Standard_OutOfRange `Failure` instead. Native exceptions keep their type.
//! Overloads on gp_Pnt/gp_Pnt2d share one generic function over [`PolePoint`];
//! overloads on real arrays carry a `_scalar` or `_flat` suffix.
mod access;
mod binomial;
mod coefficients_poles;
mod constraint;
mod hermit_jacobi;
mod hermite;
mod horner;
mod interpolation;
mod jacobi_polynomial;
mod jacobi_tables;
mod length;
mod poles;
mod rational;
mod trimming;

pub use binomial::{MAX_BINOMIAL_DEGREE, bin};
pub use coefficients_poles::{
    coefficients_poles, coefficients_poles_flat, coefficients_poles_scalar,
};
pub use constraint::{constraint_order, jacobi_parameters, niv_constr};
pub use hermit_jacobi::HermitJacobi;
pub use hermite::{hermite_coefficients, hermite_interpolate};
pub use horner::{eval_poly_2var, eval_polynomial, no_derivative_eval_polynomial};
pub use interpolation::{eval_cubic_hermite, eval_lagrange};
pub use jacobi_polynomial::JacobiPolynomial;
pub use length::{eval_length, eval_length_with_tolerance};
pub use poles::{PolePoint, get_poles, get_rational_poles, set_poles, set_rational_poles};
pub use rational::{rational_derivative, rational_derivatives};
pub use trimming::{trimming, trimming_flat, trimming_scalar, u_trimming, v_trimming};

mod grid_poles;
pub use grid_poles::coefficients_poles_grid;

use rustycad_tkernel::ncollection::{Array1, Array2};

/// The absent weight array of non-rational calls (`PLib::NoWeights`);
/// `plib::no_weights().is_none()`.
pub const fn no_weights<'r, 'a>() -> Option<&'r mut Array1<'a, f64>> {
    None
}

/// The absent weight grid of non-rational calls (`PLib::NoWeights2`);
/// `plib::no_weights2().is_none()`.
pub const fn no_weights2<'r, 'a>() -> Option<&'r mut Array2<'a, f64>> {
    None
}
