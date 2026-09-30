// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_2.cxx (MovePointAndTangent) and BSplCLib_CurveComputation.pxx (BSplCLib_MovePointAndTangent) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Modification of all poles so that a point and its tangent move by given deltas.
//!
//! Two cubic bump functions over the Schoenberg points are combined; their 2x2
//! value/derivative system is inverted like gp_Mat2d::Invert, whose zero-determinant
//! check is compiled out in the pinned release profile.
use super::extrapolate::{eval_extrapolated, eval_extrapolated_rational};
use super::flat_points;
use super::interpolate::build_schoenberg_points;
use super::locate::{LocateBounds, locate};
use super::point::CurvePoint;
use super::views::one_based;
use crate::plib::access::{array_cell, array_value, require_length};
use crate::plib::rational_derivatives;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const MOVE: &str = "BSplCLib::MovePointAndTangent";

/// Moves the point at `u` by `delta` and its derivative by `delta_derivative`
/// (`BSplCLib::MovePointAndTangent` on reals of `array_dimension` values).
///
/// `starting_condition`/`ending_condition` (-1..=Degree) keep that many end
/// derivatives. `error_status` is 0, 1 when `u` is too close to the ends, or 2 for
/// invalid conditions; `new_poles` is only written on success;
/// `bsplclib::move_point_and_tangent_flat(0.5, 2, &d, &dd, 1e-7, 3, -1, -1, &p, None, &flat, &mut np, &mut e)?`.
pub fn move_point_and_tangent_flat(
    u: f64,
    array_dimension: i32,
    delta: &[f64],
    delta_derivative: &[f64],
    tolerance: f64,
    degree: i32,
    starting_condition: i32,
    ending_condition: i32,
    poles: &[f64],
    weights: Option<&Array1<'_, f64>>,
    flat_knots: &Array1<'_, f64>,
    new_poles: &mut [f64],
    error_status: &mut i32,
) -> Result<(), Failure> {
    *error_status = 0;
    let num_poles: i32 = flat_knots.len() as i32 - (degree + 1);
    let conditions: i32 = starting_condition + ending_condition + 4;
    let valid: bool = (-1..=degree).contains(&starting_condition)
        && (-1..=degree).contains(&ending_condition)
        && conditions <= num_poles;
    if !valid {
        *error_status = 2;
        return Ok(());
    }
    if !room_to_move(
        u,
        tolerance,
        degree,
        [starting_condition, ending_condition],
        flat_knots,
    )? {
        *error_status = 1;
        return Ok(());
    }
    let bumps = Bumps::new(
        u,
        degree,
        [starting_condition, ending_condition],
        flat_knots,
        num_poles,
    )?;
    let results: [[f64; 2]; 2] = bumps.values(u, degree, weights, flat_knots)?;
    let inverse: [[f64; 2]; 2] = invert_2x2(results);
    let dim: usize = array_dimension.max(0) as usize;
    require_length(MOVE, "Delta", delta.len(), dim)?;
    require_length(MOVE, "DeltaDerivatives", delta_derivative.len(), dim)?;
    require_length(MOVE, "Poles", poles.len(), num_poles as usize * dim)?;
    require_length(MOVE, "NewPoles", new_poles.len(), num_poles as usize * dim)?;
    let a: Vec<f64> = (0..dim)
        .map(|ii| inverse[0][0] * delta[ii] + inverse[1][0] * delta_derivative[ii])
        .collect();
    let b: Vec<f64> = (0..dim)
        .map(|ii| inverse[0][1] * delta[ii] + inverse[1][1] * delta_derivative[ii])
        .collect();
    for ii in 0..num_poles {
        let first: f64 = array_value(&bumps.first, ii + 1)?;
        let second: f64 = array_value(&bumps.second, ii + 1)?;
        for jj in 0..dim {
            let index: usize = ii as usize * dim + jj;
            new_poles[index] = poles[index];
            new_poles[index] += first * a[jj];
            new_poles[index] += second * b[jj];
        }
    }
    Ok(())
}

// Checks there is enough room between U and the ends to move the poles.
fn room_to_move(
    u: f64,
    tolerance: f64,
    degree: i32,
    [starting, ending]: [i32; 2],
    flat_knots: &Array1<'_, f64>,
) -> Result<bool, Failure> {
    let start: f64 = array_value(flat_knots, flat_knots.lower() + degree)?;
    let end: f64 = array_value(flat_knots, flat_knots.upper() - degree)?;
    let start_ok: bool = if starting == -1 {
        start <= u
    } else {
        start + tolerance < u
    };
    let end_ok: bool = if ending == -1 {
        end >= u
    } else {
        end - tolerance > u
    };
    Ok(start_ok && end_ok)
}

// gp_Mat2d::SetValue + Invert without the release-disabled determinant check.
fn invert_2x2(m: [[f64; 2]; 2]) -> [[f64; 2]; 2] {
    let new00: f64 = m[1][1];
    let new01: f64 = -m[0][1];
    let new10: f64 = -m[1][0];
    let new11: f64 = m[0][0];
    let det: f64 = 1.0 / (new00 * new11 - new01 * new10);
    [[new00 * det, new01 * det], [new10 * det, new11 * det]]
}

// The two auxiliary functions, cubic in the Schoenberg abscissae around U.
struct Bumps {
    first: Array1<'static, f64>,
    second: Array1<'static, f64>,
}

impl Bumps {
    fn new(
        u: f64,
        degree: i32,
        [starting, ending]: [i32; 2],
        flat_knots: &Array1<'_, f64>,
        num_poles: i32,
    ) -> Result<Self, Failure> {
        let mut points: Array1<'static, f64> = one_based(MOVE, num_poles)?;
        build_schoenberg_points(degree, flat_knots, &mut points)?;
        let (start_num, end_num): (i32, i32) = (starting + 2, num_poles - ending - 1);
        let (mut index, mut new_parameter): (i32, f64) = (0, 0.0);
        let bounds = LocateBounds::new(start_num, end_num, 0.0, 1.0);
        locate(&points, u, false, bounds, &mut index, &mut new_parameter)?;
        let other: i32 = other_index(u, index, [start_num, end_num], flat_knots)?;
        let ends: [f64; 2] = end_values(&points, [start_num, end_num], num_poles)?;
        let shape = BumpShape {
            points: &points,
            start_num,
            end_num,
            ends,
        };
        Ok(Self {
            first: shape.build(num_poles, index)?,
            second: shape.build(num_poles, other)?,
        })
    }

    // Values and derivatives of both functions at U, rational when weighted.
    fn values(
        &self,
        u: f64,
        degree: i32,
        weights: Option<&Array1<'_, f64>>,
        flat_knots: &Array1<'_, f64>,
    ) -> Result<[[f64; 2]; 2], Failure> {
        let mode: [i32; 2] = [degree, degree];
        let mut results: [[f64; 2]; 2] = [[0.0; 2]; 2];
        for (row, function) in results.iter_mut().zip([&self.first, &self.second]) {
            let Some(weights) = weights else {
                eval_extrapolated(
                    u,
                    false,
                    1,
                    &mode,
                    degree,
                    flat_knots,
                    1,
                    function.as_slice(),
                    row,
                )?;
                continue;
            };
            // Evaluated in homogeneous form, then made rational in place.
            let mut weight_results: [f64; 2] = [0.0; 2];
            eval_extrapolated_rational(
                u,
                false,
                1,
                &mode,
                degree,
                flat_knots,
                1,
                function.as_slice(),
                weights.as_slice(),
                row,
                &mut weight_results,
            )?;
            let homogeneous: [f64; 2] = *row;
            rational_derivatives(1, 1, &homogeneous, &weight_results, row)?;
        }
        Ok(results)
    }
}

fn other_index(
    u: f64,
    index: i32,
    [start, end]: [i32; 2],
    flat_knots: &Array1<'_, f64>,
) -> Result<i32, Failure> {
    if index == start {
        return Ok(index + 1);
    }
    if index == end {
        return Ok(index - 1);
    }
    // The flat knots, not the Schoenberg points, decide the side, as natively.
    if u - array_value(flat_knots, index)? < array_value(flat_knots, index + 1)? - u {
        return Ok(index - 1);
    }
    Ok(index + 1)
}

// Abscissae beyond the moved range: mirrored ends or the neighbouring Schoenberg points.
fn end_values(
    points: &Array1<'_, f64>,
    [start, end]: [i32; 2],
    num_poles: i32,
) -> Result<[f64; 2], Failure> {
    let start_value: f64 = if start == 1 {
        let span: f64 = array_value(points, num_poles)? - array_value(points, 1)?;
        array_value(points, 1)? - span
    } else {
        array_value(points, start - 1)?
    };
    let end_value: f64 = if end == num_poles {
        let span: f64 = array_value(points, num_poles)? - array_value(points, 1)?;
        array_value(points, num_poles)? + span
    } else {
        array_value(points, end + 1)?
    };
    Ok([start_value, end_value])
}

struct BumpShape<'p, 'a> {
    points: &'p Array1<'a, f64>,
    start_num: i32,
    end_num: i32,
    ends: [f64; 2],
}

impl BumpShape<'_, '_> {
    // ((s - start)/(s_peak - start))^3 rising, ((end - s)/(end - s_peak))^3 falling.
    fn build(&self, num_poles: i32, peak: i32) -> Result<Array1<'static, f64>, Failure> {
        let mut function: Array1<'static, f64> = one_based(MOVE, num_poles)?;
        let [start_value, end_value] = self.ends;
        let peak_point: f64 = array_value(self.points, peak)?;
        let divide: f64 = 1.0e0 / (peak_point - start_value);
        for ii in self.start_num..=peak {
            let value: f64 = (array_value(self.points, ii)? - start_value) * divide;
            *array_cell(&mut function, ii)? = cube(value);
        }
        let divide: f64 = 1.0e0 / (end_value - peak_point);
        for ii in peak..=self.end_num {
            let value: f64 = (end_value - array_value(self.points, ii)?) * divide;
            *array_cell(&mut function, ii)? = cube(value);
        }
        Ok(function)
    }
}

// 1.0 multiplied three times by the value, as the native `type` loop does.
fn cube(value: f64) -> f64 {
    let mut result: f64 = 1.0e0;
    for _ in 0..3 {
        result *= value;
    }
    result
}

/// Moves a point and tangent of a 2D/3D curve (`BSplCLib::MovePointAndTangent` on
/// gp_Pnt/gp_Pnt2d); fails with Standard_ConstructionError when the pole arrays differ
/// in length; `bsplclib::move_point_and_tangent(0.5, d, dd, 1e-7, 3, -1, -1, &p, None, &flat, &mut np, &mut e)?`.
pub fn move_point_and_tangent<P: CurvePoint>(
    u: f64,
    delta: P::Vector,
    delta_derivative: P::Vector,
    tolerance: f64,
    degree: i32,
    starting_condition: i32,
    ending_condition: i32,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    flat_knots: &Array1<'_, f64>,
    new_poles: &mut Array1<'_, P>,
    error_status: &mut i32,
) -> Result<(), Failure> {
    if new_poles.len() != poles.len() {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            &format!(
                "{MOVE}: {} new poles for {} poles",
                new_poles.len(),
                poles.len()
            ),
            "",
        ));
    }
    let delta: Vec<f64> = (0..P::DIMENSION)
        .map(|k| P::vector_coordinate(&delta, k))
        .collect();
    let derivative: Vec<f64> = (0..P::DIMENSION)
        .map(|k| P::vector_coordinate(&delta_derivative, k))
        .collect();
    let mut flat_new: Vec<f64> = flat_points(new_poles);
    move_point_and_tangent_flat(
        u,
        P::DIMENSION as i32,
        &delta,
        &derivative,
        tolerance,
        degree,
        starting_condition,
        ending_condition,
        &flat_points(poles),
        weights,
        flat_knots,
        &mut flat_new,
        error_status,
    )?;
    super::store_flat_points(&flat_new, new_poles);
    Ok(())
}
