// Rust translation of FoundationClasses/TKMath/PLib/PLib_JacobiPolynomial.cxx (MaxError, ReduceDegree, AverageError) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Truncation error estimates of a Jacobi expansion.
use super::JacobiPolynomial;
use crate::math::Vector;
use crate::plib::access::buffer_value;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

// Coefficient sums at or below this magnitude count as zero in the ReduceDegree fallback.
const NEGLIGIBLE: f64 = 1.0e-9;

impl JacobiPolynomial {
    fn max_values(&self) -> Result<Array1<'static, f64>, Failure> {
        let mut tab_max: Array1<'static, f64> = Array1::with_bounds(0, self.degree + 1)
            .map_err(|error| Failure::out_of_range(&error.to_string()))?;
        self.max_value(&mut tab_max)?;
        Ok(tab_max)
    }

    /// Bounds the error of dropping coefficients NewDegree+1..WorkDegree
    /// (`PLib_JacobiPolynomial::MaxError`).
    ///
    /// Coefficients are stored degree by degree, Dimension values each;
    /// `jacobi.max_error(1, &coefficients, 6)?`.
    pub fn max_error(
        &self,
        dimension: i32,
        jac_coeff: &[f64],
        new_degree: i32,
    ) -> Result<f64, Failure> {
        let tab_max: Array1<'static, f64> = self.max_values()?;
        let first: i32 = 2 * (self.niv_constr + 1);
        let cut: i32 = first.max(new_degree + 1);
        let mut max_err_dim: Vector<'static> = Vector::filled(1, dimension, 0.0)?;
        for d in 1..=dimension {
            for c in cut..=self.work_degree {
                let value: f64 = coefficient(jac_coeff, c, dimension, d)?;
                max_err_dim[d] += value.abs() * tab_max[c - first];
            }
        }
        max_err_dim.norm()
    }

    /// Finds the lowest degree <= MaxDegree whose truncation error stays within Tol
    /// (`PLib_JacobiPolynomial::ReduceDegree`).
    ///
    /// Returns `(new_degree, max_error)`; when no truncation qualifies, the last
    /// coefficient row above 1e-9 decides; `jacobi.reduce_degree(1, 8, 1e-6, &c)?`.
    pub fn reduce_degree(
        &self,
        dimension: i32,
        max_degree: i32,
        tol: f64,
        jac_coeff: &[f64],
    ) -> Result<(i32, f64), Failure> {
        let last_fixed: i32 = 2 * (self.niv_constr + 1) - 1;
        let tab_max: Array1<'static, f64> = self.max_values()?;
        let mut max_err_dim: Vector<'static> = Vector::filled(1, dimension, 0.0)?;
        let (mut new_degree, mut max_error) = (last_fixed, 0.0);
        for i in (last_fixed + 1..=self.work_degree).rev() {
            for d in 1..=dimension {
                let value: f64 = coefficient(jac_coeff, i, dimension, d)?;
                max_err_dim[d] += value.abs() * tab_max[i - last_fixed - 1];
            }
            let error: f64 = max_err_dim.norm()?;
            if error > tol && i <= max_degree {
                new_degree = i;
                break;
            }
            max_error = error;
        }
        if new_degree == last_fixed {
            new_degree = last_significant_row(jac_coeff, dimension, last_fixed)?;
        }
        Ok((new_degree, max_error))
    }

    /// Returns sqrt(sum of squared coefficients beyond NewDegree / 2)
    /// (`PLib_JacobiPolynomial::AverageError`).
    ///
    /// As natively, the sum stops at the Q degree, not at WorkDegree;
    /// `jacobi.average_error(1, &coefficients, 6)?`.
    pub fn average_error(
        &self,
        dimension: i32,
        jac_coeff: &[f64],
        new_degree: i32,
    ) -> Result<f64, Failure> {
        let cut: i32 = (2 * (self.niv_constr + 1) + 1).max(new_degree + 1);
        let mut average_err: f64 = 0.0;
        for d in 1..=dimension {
            for i in cut..=self.degree {
                let value: f64 = coefficient(jac_coeff, i, dimension, d)?;
                average_err += value * value;
            }
        }
        Ok((average_err / 2.0).sqrt())
    }
}

fn coefficient(
    jac_coeff: &[f64],
    degree: i32,
    dimension: i32,
    component: i32,
) -> Result<f64, Failure> {
    let index: i64 = i64::from(degree) * i64::from(dimension) + i64::from(component) - 1;
    buffer_value("PLib_JacobiPolynomial", jac_coeff, index)
}

fn last_significant_row(
    jac_coeff: &[f64],
    dimension: i32,
    last_fixed: i32,
) -> Result<i32, Failure> {
    for i in (1..=last_fixed).rev() {
        let mut magnitude: f64 = 0.0;
        for d in 1..=dimension {
            magnitude += coefficient(jac_coeff, i, dimension, d)?.abs();
        }
        if magnitude > NEGLIGIBLE {
            return Ok(i);
        }
    }
    Ok(0)
}
