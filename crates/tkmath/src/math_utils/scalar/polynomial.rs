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

use super::*;
use crate::math_utils::{eval_poly, eval_poly_deriv};
/// Constant-first polynomial functor; `Polynomial::new(&[-2.0,0.0,1.0])`.
#[derive(Clone)]
pub struct Polynomial {
    coefficients: Vec<f64>,
}
impl Polynomial {
    /// Copies coefficients in their native order; `Polynomial::new(&[1.0,2.0])`.
    pub fn new(coefficients: &[f64]) -> Self {
        Self {
            coefficients: coefficients.to_vec(),
        }
    }
    /// Ignores declared bounds when copying coefficient order; `Polynomial::from_vector(&coefficients)`.
    pub fn from_vector(coefficients: &Vector<'_>) -> Self {
        Self::new(coefficients.values())
    }
    /// Empty polynomials have native degree zero; `polynomial.degree()`.
    pub fn degree(&self) -> i32 {
        self.coefficients.len().saturating_sub(1) as i32
    }
    /// Missing coefficients read as zero; `polynomial.coefficient(10)`.
    pub fn coefficient(&self, index: usize) -> f64 {
        self.coefficients.get(index).copied().unwrap_or(0.0)
    }
}
impl Function for Polynomial {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        if !x.is_finite() || self.coefficients.is_empty() {
            return false;
        }
        *value = eval_poly(&self.coefficients, x);
        value.is_finite()
    }
}
impl FunctionWithDerivative for Polynomial {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        self.values(x, &mut 0.0, derivative)
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        if !x.is_finite() || self.coefficients.is_empty() {
            return false;
        }
        (*value, *derivative) = eval_poly_deriv(&self.coefficients, x);
        value.is_finite() && derivative.is_finite()
    }
}
/// Ratio of two coefficient sequences; `Rational::new(&[1.0,1.0],&[1.0,0.0,1.0])`.
#[derive(Clone)]
pub struct Rational {
    numerator: Vec<f64>,
    denominator: Vec<f64>,
}
impl Rational {
    /// Copies numerator and denominator; `Rational::new(&[1.0],&[2.0])`.
    pub fn new(numerator: &[f64], denominator: &[f64]) -> Self {
        Self {
            numerator: numerator.to_vec(),
            denominator: denominator.to_vec(),
        }
    }
    /// Copies positional vector coefficients; `Rational::from_vectors(&numerator,&denominator)`.
    pub fn from_vectors(numerator: &Vector<'_>, denominator: &Vector<'_>) -> Self {
        Self::new(numerator.values(), denominator.values())
    }
}
impl Function for Rational {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        let numerator: f64 = if self.numerator.is_empty() {
            0.0
        } else {
            eval_poly(&self.numerator, x)
        };
        let denominator: f64 = if self.denominator.is_empty() {
            0.0
        } else {
            eval_poly(&self.denominator, x)
        };
        if !x.is_finite()
            || !numerator.is_finite()
            || !denominator.is_finite()
            || denominator == 0.0
        {
            return false;
        }
        *value = numerator / denominator;
        value.is_finite()
    }
}
