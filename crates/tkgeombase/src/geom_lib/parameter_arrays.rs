// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx
// (RemovePointsFromArray, DensifyArray1OfReal, FuseIntervals)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameter-array members of GeomLib: thinning, densifying and fusing sorted
//! parameter sequences.
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::standard::Failure;

/// `GeomLib::RemovePointsFromArray(NumPoints, InParameters, OutParameters)`: keeps
/// the first and last parameters and, for each of `NumPoints - 2` uniform segments,
/// the input parameter before the first one that reaches the segment midpoint.
///
/// `in_parameters` must be increasing (unchecked, as natively); `out_parameters` is
/// resized with lower bound 1. An empty input, which the native code reads past, is
/// Standard_OutOfRange;
/// `geom_lib::remove_points_from_array(4, &input, &mut reduced)?`.
pub fn remove_points_from_array(
    num_points: i32,
    in_parameters: &Array1<'_, f64>,
    out_parameters: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let (first, last): (f64, f64) = bounds(in_parameters, "GeomLib::RemovePointsFromArray")?;
    let values: &[f64] = in_parameters.as_slice();
    let mut selected: Vec<f64> = vec![first];
    if num_points <= 2 {
        selected.push(last);
    } else {
        let requested_interior_count: i32 = (num_points - 2).max(0);
        let delta: f64 = (last - first) / f64::from(requested_interior_count + 1);
        let mut current_parameter: f64 = first + 0.5 * delta;
        let mut input_index: usize = 1;
        let mut segment: i32 = 0;
        while input_index + 1 < values.len() && segment < num_points {
            let previous_index: usize = input_index;
            while input_index + 1 < values.len() && values[input_index] < current_parameter {
                input_index += 1;
            }
            if input_index != previous_index {
                selected.push(values[input_index - 1]);
            }
            segment += 1;
            current_parameter += delta;
        }
        selected.push(last);
    }
    write_one_based(&selected, out_parameters)
}

/// `GeomLib::DensifyArray1OfReal(MinNumPoints, InParameters, OutParameters)`: at
/// least `MinNumPoints` parameters, the input ones plus uniform steps of
/// `(Last - First) / (added + 1)` merged in order, the last one set exactly.
///
/// An input that is not sorted, or already long enough, is copied; a constant input
/// fills every output value with it. `out_parameters` is resized with lower bound 1;
/// `geom_lib::densify_array1_of_real(5, &sparse, &mut dense)?`.
pub fn densify_array1_of_real(
    min_num_points: i32,
    in_parameters: &Array1<'_, f64>,
    out_parameters: &mut Array1<'_, f64>,
) -> Result<(), Failure> {
    let values: &[f64] = in_parameters.as_slice();
    let is_in_order: bool = values
        .windows(2)
        .all(|pair| pair[1].partial_cmp(&pair[0]) != Some(std::cmp::Ordering::Less));
    let num_points: i32 = if is_in_order {
        min_num_points.max(in_parameters.length())
    } else {
        in_parameters.length()
    };
    resize_one_based(out_parameters, num_points)?;
    if min_num_points <= in_parameters.length() || !is_in_order {
        out_parameters.as_mut_slice()[..values.len()].copy_from_slice(values);
        return Ok(());
    }
    let (first, last): (f64, f64) = bounds(in_parameters, "GeomLib::DensifyArray1OfReal")?;
    let num_parameters_to_add: i32 = min_num_points - in_parameters.length();
    let delta: f64 = (last - first) / f64::from(num_parameters_to_add + 1);
    if delta == 0.0 {
        out_parameters.init(first);
        return Ok(());
    }
    merge_uniform(values, first, delta, out_parameters.as_mut_slice());
    // Protect the last value from accumulated rounding error.
    if let Some(end) = out_parameters.as_mut_slice().last_mut() {
        *end = last;
    }
    Ok(())
}

// The uniform steps from `first` merged with the input parameters, in order.
fn merge_uniform(values: &[f64], first: f64, delta: f64, output: &mut [f64]) {
    let mut output_index: usize = 0;
    let mut current_parameter: f64 = first;
    output[output_index] = current_parameter;
    output_index += 1;
    current_parameter += delta;
    let mut input_index: usize = 1;
    while output_index < output.len() && input_index < values.len() {
        while current_parameter < values[input_index] && output_index < output.len() {
            output[output_index] = current_parameter;
            output_index += 1;
            current_parameter += delta;
        }
        if output_index < output.len() {
            output[output_index] = values[input_index];
            output_index += 1;
        }
        input_index += 1;
    }
}

/// `GeomLib::FuseIntervals(theInterval1, theInterval2, theFusion, theConfusion =
/// 1e-9, theAdjustToFirstInterval = false)`: the merge of two non-decreasing
/// arrays, a pair of values within `confusion` kept once (the first array's value
/// when `adjust_to_first_interval`, their midpoint otherwise).
///
/// `fusion` is cleared first (its capacity kept) and receives the second array's
/// remaining values before the first array's, as natively;
/// `geom_lib::fuse_intervals(&a, &b, &mut fused, 1e-8, false)`.
pub fn fuse_intervals(
    interval1: &Array1<'_, f64>,
    interval2: &Array1<'_, f64>,
    fusion: &mut LinearVector<f64>,
    confusion: f64,
    adjust_to_first_interval: bool,
) {
    fusion.clear(false);
    fusion.reserve(interval1.len() + interval2.len());
    let (first, second): (&[f64], &[f64]) = (interval1.as_slice(), interval2.as_slice());
    let (mut index1, mut index2): (usize, usize) = (0, 0);
    while index1 < first.len() && index2 < second.len() {
        let (value1, value2): (f64, f64) = (first[index1], second[index2]);
        if (value1 - value2).abs() <= confusion {
            fusion.append(if adjust_to_first_interval {
                value1
            } else {
                0.5 * (value1 + value2)
            });
            index1 += 1;
            index2 += 1;
        } else if value1 < value2 {
            fusion.append(value1);
            index1 += 1;
        } else {
            fusion.append(value2);
            index2 += 1;
        }
    }
    for value in &second[index2..] {
        fusion.append(*value);
    }
    for value in &first[index1..] {
        fusion.append(*value);
    }
}

// First() and Last() of a non-empty array.
fn bounds(parameters: &Array1<'_, f64>, member: &str) -> Result<(f64, f64), Failure> {
    let values: &[f64] = parameters.as_slice();
    match (values.first(), values.last()) {
        (Some(first), Some(last)) => Ok((*first, *last)),
        _ => Err(failure(
            "Standard_OutOfRange",
            &format!("{member}: empty InParameters; expected at least one value"),
        )),
    }
}

// OutParameters.Resize(1, count, false).
fn resize_one_based(target: &mut Array1<'_, f64>, count: i32) -> Result<(), Failure> {
    target
        .resize_bounds(1, count, false)
        .map_err(|error| array_failure(error, "OutParameters"))
}

fn write_one_based(values: &[f64], target: &mut Array1<'_, f64>) -> Result<(), Failure> {
    resize_one_based(target, values.len() as i32)?;
    target.as_mut_slice().copy_from_slice(values);
    Ok(())
}
