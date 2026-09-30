// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_CurveComputation.pxx (BSplCLib_MovePoint) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Local modification of poles so that the curve passes through a displaced point.
use super::basis::eval_bspline_basis;
use super::point::CurvePoint;
use crate::math::Matrix;
use crate::plib::access::{array_cell, array_value};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// Moves the point at `u` by `displ`, changing only poles `index1..index2`
/// (`BSplCLib::MovePoint`).
///
/// Returns `(FirstIndex, LastIndex)` of the modified poles; `(0, 0)` with unchanged
/// poles when the basis cannot be evaluated;
/// `bsplclib::move_point(0.5, v, 1, 4, 3, &poles, None, &flat, &mut new_poles)?`.
pub fn move_point<P: CurvePoint>(
    u: f64,
    displ: P::Vector,
    index1: i32,
    index2: i32,
    degree: i32,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    flat_knots: &Array1<'_, f64>,
    new_poles: &mut Array1<'_, P>,
) -> Result<(i32, i32), Failure> {
    let mut basis: Matrix<'static> = Matrix::with_bounds(1, 1, 1, degree + 1)
        .map_err(|error| Failure::out_of_range(&format!("BSplCLib::MovePoint: {error}")))?;
    let mut first_non_zero: i32 = 0;
    if eval_bspline_basis(
        0,
        degree + 1,
        flat_knots,
        u,
        &mut first_non_zero,
        &mut basis,
        false,
    )? != 0
    {
        copy_all(poles, new_poles)?;
        return Ok((0, 0));
    }
    let influence = Influence::new(&basis, first_non_zero, degree, index1, index2)?;
    let coef: f64 = influence.coefficient(&basis, weights, degree)?;
    for i in poles.lower()..=poles.upper() {
        let pole: P = array_value(poles, i)?;
        let moved: P = match influence.dval(i) {
            Some(dval) => translated(pole, displ, coef / (dval + 1.)),
            None => pole,
        };
        *array_cell(new_poles, i)? = moved;
    }
    Ok((influence.first, influence.last))
}

fn copy_all<P: CurvePoint>(
    poles: &Array1<'_, P>,
    new_poles: &mut Array1<'_, P>,
) -> Result<(), Failure> {
    for i in poles.lower()..=poles.upper() {
        *array_cell(new_poles, i)? = array_value(poles, i)?;
    }
    Ok(())
}

// Poles(i).Translated(scale * Displ): gp_Vec scaling, then coordinate addition.
fn translated<P: CurvePoint>(pole: P, displ: P::Vector, scale: f64) -> P {
    let mut result: P = pole;
    for k in 0..P::DIMENSION {
        let delta: f64 = P::vector_coordinate(&displ, k) * scale;
        result.set_pole_coordinate(k, pole.pole_coordinate(k) + delta);
    }
    result
}

// The modified range and the predominant pole(s) kk1..kk2 at U.
struct Influence {
    first_non_zero: i32,
    first: i32,
    last: i32,
    kk1: i32,
    kk2: i32,
}

impl Influence {
    fn new(
        basis: &Matrix<'_>,
        first_non_zero: i32,
        degree: i32,
        index1: i32,
        index2: i32,
    ) -> Result<Self, Failure> {
        let first: i32 = first_non_zero.max(index1);
        let last: i32 = (first_non_zero + degree).min(index2);
        let mut max_value: f64 = 0.0;
        let mut kk1: i32 = 0;
        for i in first - first_non_zero + 1..=last - first_non_zero + 1 {
            let value: f64 = basis_value(basis, i)?;
            if value > max_value {
                kk1 = i + first_non_zero - 1;
                max_value = value;
            }
        }
        // A symmetric maximum spreads the displacement over two poles.
        let mut kk2: i32 = kk1;
        if kk1 < last && (basis_value(basis, kk1 - first_non_zero + 2)? - max_value).abs() < 1.0e-10
        {
            kk2 = kk1 + 1;
        }
        Ok(Self {
            first_non_zero,
            first,
            last,
            kk1,
            kk2,
        })
    }

    // Distance to the predominant pole(s), or None outside FirstIndex..LastIndex.
    fn dval(&self, index: i32) -> Option<f64> {
        if index < self.first || index > self.last {
            return None;
        }
        if index < self.kk1 {
            return Some(f64::from(self.kk1 - index));
        }
        if index > self.kk2 {
            return Some(f64::from(index - self.kk2));
        }
        Some(0.0)
    }

    fn coefficient(
        &self,
        basis: &Matrix<'_>,
        weights: Option<&Array1<'_, f64>>,
        degree: i32,
    ) -> Result<f64, Failure> {
        let (mut d1, mut d2): (f64, f64) = (0.0, 0.0);
        for i in 1..=degree + 1 {
            let ii: i32 = i + self.first_non_zero - 1;
            let h_n: f64 = match weights {
                Some(weights) => {
                    let term: f64 = array_value(weights, ii)? * basis_value(basis, i)?;
                    d2 += term;
                    term
                }
                None => basis_value(basis, i)?,
            };
            if let Some(dval) = self.dval(ii) {
                d1 += 1. / (dval + 1.) * h_n;
            }
        }
        if weights.is_some() {
            return Ok(d2 / d1);
        }
        Ok(1. / d1)
    }
}

fn basis_value(basis: &Matrix<'_>, column: i32) -> Result<f64, Failure> {
    basis.value(1, column).copied()
}
