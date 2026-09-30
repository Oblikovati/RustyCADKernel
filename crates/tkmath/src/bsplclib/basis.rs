// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_2.cxx (EvalBsplineBasis) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Values and derivatives of the non-vanishing B-spline basis functions.
use super::locate::locate_parameter_flat_knots;
use super::native::real_small;
use crate::math::Matrix;
use crate::plib::access::{array_value, non_negative};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const BASIS: &str = "BSplCLib::EvalBsplineBasis";

/// Evaluates the `order` non-zero basis functions at `parameter` and their derivatives
/// up to `derivative_request` (`BSplCLib::EvalBsplineBasis`).
///
/// Row r+1, column c of `bspline_basis` receives the r-th derivative of basis function
/// `first_non_zero_bspline_index + c - 1`. Returns 0, 1 when the matrix is not
/// `1..>request, 1..>=order`, or 2 on a null knot span. Quirk kept: the span is located
/// with periodic wrapping but the unwrapped `parameter` is used in the recurrence;
/// `bsplclib::eval_bspline_basis(1, 4, &flat, 0.5, &mut first, &mut basis, false)?`.
pub fn eval_bspline_basis(
    derivative_request: i32,
    order: i32,
    flat_knots: &Array1<'_, f64>,
    parameter: f64,
    first_non_zero_bspline_index: &mut i32,
    bspline_basis: &mut Matrix<'_>,
    is_periodic: bool,
) -> Result<i32, Failure> {
    *first_non_zero_bspline_index = 0;
    non_negative(BASIS, "DerivativeRequest", derivative_request)?;
    let local_request: i32 = derivative_request.min(order - 1);
    if bspline_basis.lower_col() != 1
        || bspline_basis.upper_col() < order
        || bspline_basis.lower_row() != 1
        || bspline_basis.upper_row() <= local_request
    {
        return Ok(1);
    }
    let num_poles: i32 = flat_knots.upper() - flat_knots.lower() + 1 - order;
    let (mut ii, mut new_param): (i32, f64) = (0, 0.0);
    locate_parameter_flat_knots(
        order - 1,
        flat_knots,
        parameter,
        is_periodic,
        order,
        num_poles + 1,
        &mut ii,
        &mut new_param,
    )?;
    *first_non_zero_bspline_index = ii - order + 1;
    let recurrence = BasisRecurrence {
        order,
        local_request,
        columns: bspline_basis.col_number() as usize,
        knots: flat_knots,
        span: ii,
        parameter,
    };
    recurrence.run(bspline_basis.cells_mut())
}

struct BasisRecurrence<'r, 'a> {
    order: i32,
    local_request: i32,
    columns: usize,
    knots: &'r Array1<'a, f64>,
    span: i32,
    parameter: f64,
}

impl BasisRecurrence<'_, '_> {
    fn run(&self, basis: &mut [f64]) -> Result<i32, Failure> {
        basis[0] = 1.0;
        let split: i32 = self.order - self.local_request;
        for qq in 2..=split {
            if !self.raise_values(basis, qq)? {
                return Ok(2);
            }
        }
        for qq in split + 1..=self.order {
            self.shift_rows(basis, qq);
            if !self.raise_with_derivatives(basis, qq)? {
                return Ok(2);
            }
        }
        Ok(0)
    }

    // Knot span (Knots[ii+pp] - Knots[ii-qq+pp+1]) and its left knot, or None when null.
    fn scale(&self, qq: i32, pp: i32) -> Result<Option<(f64, f64)>, Failure> {
        let upper: f64 = array_value(self.knots, self.span + pp)?;
        let left: f64 = array_value(self.knots, self.span - qq + pp + 1)?;
        let scale: f64 = upper - left;
        if scale.abs() < real_small() {
            return Ok(None);
        }
        Ok(Some((scale, left)))
    }

    // Degree-raising of the values only.
    fn raise_values(&self, basis: &mut [f64], qq: i32) -> Result<bool, Failure> {
        let q: usize = qq as usize - 1;
        basis[q] = 0.0;
        for pp in 1..qq {
            let Some((scale, left)) = self.scale(qq, pp)? else {
                return Ok(false);
            };
            let factor: f64 = (self.parameter - left) / scale;
            let p: usize = pp as usize - 1;
            let saved: f64 = factor * basis[p];
            basis[p] *= 1.0 - factor;
            basis[p] += basis[q];
            basis[q] = saved;
        }
        Ok(true)
    }

    // Copies the value row into the derivative row and clears the new column.
    fn shift_rows(&self, basis: &mut [f64], qq: i32) {
        let (order, cols): (i32, usize) = (self.order, self.columns);
        let row: usize = (order - qq + 1) as usize * cols;
        for pp in 1..qq as usize {
            basis[row + pp - 1] = basis[pp - 1];
        }
        basis[qq as usize - 1] = 0.0;
        for ss in order - self.local_request + 1..=qq {
            basis[(order - ss + 1) as usize * cols + qq as usize - 1] = 0.0;
        }
    }

    fn raise_with_derivatives(&self, basis: &mut [f64], qq: i32) -> Result<bool, Failure> {
        let q: usize = qq as usize - 1;
        for pp in 1..qq {
            let Some((scale, left)) = self.scale(qq, pp)? else {
                return Ok(false);
            };
            let inverse: f64 = 1.0 / scale;
            let factor: f64 = (self.parameter - left) * inverse;
            let p: usize = pp as usize - 1;
            let saved: f64 = factor * basis[p];
            basis[p] *= 1.0 - factor;
            basis[p] += basis[q];
            basis[q] = saved;
            let local_inverse: f64 = f64::from(qq - 1) * inverse;
            for ss in self.order - self.local_request + 1..=qq {
                let row: usize = (self.order - ss + 1) as usize * self.columns;
                let saved: f64 = local_inverse * basis[row + p];
                basis[row + p] *= -local_inverse;
                basis[row + p] += basis[row + q];
                basis[row + q] = saved;
            }
        }
        Ok(true)
    }
}
