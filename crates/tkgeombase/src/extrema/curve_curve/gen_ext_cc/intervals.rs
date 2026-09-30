// Rust translation of the interval part of Extrema_GGenExtCC::Perform and of
// Extrema_GGenExtCC_ChangeIntervals at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The search intervals of Extrema_GGenExtCC: the C2 intervals of both curves (C1
//! when their product exceeds 100), the longer curve split for balance, and a
//! closed curve of one interval split in three.
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::failures::out_of_range;
use crate::gcpnts::AbscissaPoint;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkmath::geom_abs::Shape;

/// The continuity and the interval bounds of both curves.
pub(super) struct IntervalLayout {
    pub continuity: Shape,
    pub intervals1: Vec<f64>,
    pub intervals2: Vec<f64>,
}

const MULT: f64 = 20.;

/// The intervals of the curve pair.
pub(super) fn interval_layout<T: ExtremaCurveTool>(
    c1: &T::Curve,
    c2: &T::Curve,
) -> Result<IntervalLayout, Failure> {
    let mut continuity: Shape = Shape::C2;
    let mut nb_inter: [i32; 2] = [
        T::nb_intervals(c1, continuity)?,
        T::nb_intervals(c2, continuity)?,
    ];
    if nb_inter[0] * nb_inter[1] > 100 {
        continuity = Shape::C1;
        nb_inter = [
            T::nb_intervals(c1, continuity)?,
            T::nb_intervals(c2, continuity)?,
        ];
    }
    let balance: Option<(usize, i32)> = balanced_split::<T>(c1, c2, nb_inter)?;
    let mut intervals = [
        bounds::<T>(c1, nb_inter[0], continuity)?,
        bounds::<T>(c2, nb_inter[1], continuity)?,
    ];
    if let Some((indmax, nb_int_opt)) = balance {
        intervals[indmax] = change_intervals(&intervals[indmax], nb_int_opt)?;
    }
    for (index, curve) in [c1, c2].into_iter().enumerate() {
        if T::is_closed(curve)? && intervals[index].len() == 2 {
            intervals[index] = change_intervals(&intervals[index], 3)?;
        }
    }
    let [intervals1, intervals2] = intervals;
    Ok(IntervalLayout {
        continuity,
        intervals1,
        intervals2,
    })
}

// `Intervals(C, T, S)` into an array of `NbIntervals + 1` values.
fn bounds<T: ExtremaCurveTool>(
    curve: &T::Curve,
    count: i32,
    continuity: Shape,
) -> Result<Vec<f64>, Failure> {
    let mut array: Array1<'static, f64> =
        Array1::with_bounds(1, count + 1).map_err(|error| error.into_failure("Intervals"))?;
    T::intervals(curve, &mut array, continuity)?;
    Ok((1..=count + 1).map(|i| array[i]).collect())
}

// The curve whose intervals are more than 20 times longer (in length per interval)
// than the other's, with its balanced interval count, when that count is kept.
fn balanced_split<T: ExtremaCurveTool>(
    c1: &T::Curve,
    c2: &T::Curve,
    nb_inter: [i32; 2],
) -> Result<Option<(usize, i32)>, Failure> {
    let finite = |x: f64| !precision::is_infinite(x);
    let bounded = finite(T::first_parameter(c1)?)
        && finite(T::last_parameter(c1)?)
        && finite(T::first_parameter(c2)?)
        && finite(T::last_parameter(c2)?);
    if !bounded {
        return Ok(None);
    }
    let lengths: [f64; 2] = [AbscissaPoint::length(c1)?, AbscissaPoint::length(c2)?];
    let (n0, n1) = (f64::from(nb_inter[0]), f64::from(nb_inter[1]));
    let (indmax, indmin) = if lengths[0] / n0 > MULT * lengths[1] / n1 {
        (0, 1)
    } else if lengths[1] / n1 > MULT * lengths[0] / n0 {
        (1, 0)
    } else {
        return Ok(None);
    };
    let ratio: f64 = lengths[indmax] * f64::from(nb_inter[indmin]) / lengths[indmin] / (MULT / 4.);
    let mut nb_int_opt: i32 = real_to_int(ratio) + 1;
    if nb_int_opt > 100 || nb_int_opt < nb_inter[indmax] {
        return Ok(None);
    }
    if nb_int_opt * nb_inter[indmin] > 100 {
        nb_int_opt = 100 / nb_inter[indmin];
        if nb_int_opt < nb_inter[indmax] {
            return Ok(None);
        }
    }
    Ok(Some((indmax, nb_int_opt)))
}

/// `Extrema_GGenExtCC_ChangeIntervals(theInts, theNbInts)`: `theNbInts` intervals,
/// evenly split from one interval, or by halving the longest interval (the first of
/// equals) until the count is reached. Standard_OutOfRange when fewer intervals are
/// asked than there are.
pub(super) fn change_intervals(ints: &[f64], nb_ints: i32) -> Result<Vec<f64>, Failure> {
    let size: usize = (nb_ints + 1) as usize;
    let mut new_ints: Vec<f64> = vec![0.0; size];
    let (first, last) = (ints[0], ints[ints.len() - 1]);
    if ints.len() == 2 {
        new_ints[0] = first;
        new_ints[size - 1] = last;
        let dt: f64 = (last - first) / f64::from(nb_ints);
        let mut t: f64 = first + dt;
        for value in new_ints.iter_mut().take(size - 1).skip(1) {
            *value = t;
            t += dt;
        }
        return Ok(new_ints);
    }
    if ints.len() > size {
        let message = "Extrema_GGenExtCC_ChangeIntervals";
        return Err(out_of_range(message, ints.len() as i32, size as i32));
    }
    new_ints[..ints.len()].copy_from_slice(ints);
    let nb_added: usize = (nb_ints - (ints.len() as i32 - 1)).max(0) as usize;
    for nb_last in ints.len()..ints.len() + nb_added {
        let max_ind: usize = longest_interval(&new_ints[..nb_last]);
        let t: f64 = (new_ints[max_ind + 1] + new_ints[max_ind]) / 2.;
        new_ints.copy_within(max_ind + 1..nb_last, max_ind + 2);
        new_ints[max_ind + 1] = t;
    }
    Ok(new_ints)
}

// The 0-based start of the longest interval, the first of equals (from -1).
fn longest_interval(bounds: &[f64]) -> usize {
    let (mut max_length, mut max_ind): (f64, usize) = (-1., 0);
    for i in 0..bounds.len() - 1 {
        let length: f64 = bounds[i + 1] - bounds[i];
        if length > max_length {
            max_length = length;
            max_ind = i;
        }
    }
    max_ind
}
