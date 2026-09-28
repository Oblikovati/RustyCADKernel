// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (Intervals) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Splitting of a parameter range at the knots where the continuity drops.
use super::knots::{first_u_knot_index, last_u_knot_index};
use super::locate::locate_parameter;
use super::views::one_based;
use crate::plib::access::{array_cell, array_value};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const INTERVALS: &str = "BSplCLib::Intervals";

/// Number of intervals of `[first, last]` on which the curve has `continuity`
/// (`BSplCLib::Intervals`); knots of multiplicity above Degree-Continuity split the
/// range, periodic curves repeating their knots over the periods crossed.
///
/// When `intervals` is given it is resized to `1..=N+1` and filled with the bounds,
/// its first and last value being `first` and `last`;
/// `bsplclib::intervals(&knots, &mults, 3, false, 2, 0.0, 1.0, 1e-9, Some(&mut out))?`.
pub fn intervals(
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    degree: i32,
    is_periodic: bool,
    continuity: i32,
    first: f64,
    last: f64,
    tolerance: f64,
    intervals: Option<&mut Array1<'_, f64>>,
) -> Result<i32, Failure> {
    let new_knots: Array1<'static, f64> =
        split_knots(knots, mults, degree, is_periodic, continuity)?;
    let count: i32 = new_knots.len() as i32;
    let range = PeriodicRange::new(knots, is_periodic, first, last)?;
    let (mut index1, mut index2): (i32, i32) = (0, 0);
    let mut dummy: f64 = 0.0;
    // The multiplicity-free LocateParameter overload.
    let no_mults: Array1<'static, i32> = Array1::default();
    locate_parameter(
        degree,
        &new_knots,
        &no_mults,
        range.cur_first,
        false,
        1,
        count,
        &mut index1,
        &mut dummy,
    )?;
    locate_parameter(
        degree,
        &new_knots,
        &no_mults,
        range.cur_last,
        false,
        1,
        count,
        &mut index2,
        &mut dummy,
    )?;
    // The range may begin on the next knot or end on the current one.
    if index1 < count && (array_value(&new_knots, index1 + 1)? - range.cur_first).abs() < tolerance
    {
        index1 += 1;
    }
    if count != 0 && (array_value(&new_knots, index2)? - range.cur_last).abs() < tolerance {
        index2 -= 1;
    }
    let nb_intervals: i32 =
        index2 - index1 + 1 + (range.last_period - range.first_period) * (count - 1);
    if let Some(output) = intervals {
        range.fill(&new_knots, [index1, index2], nb_intervals, output)?;
        *array_cell(output, 1)? = first;
        *array_cell(output, nb_intervals + 1)? = last;
    }
    Ok(nb_intervals)
}

// Keeps the first, last and low-continuity knots.
fn split_knots(
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    degree: i32,
    is_periodic: bool,
    continuity: i32,
) -> Result<Array1<'static, f64>, Failure> {
    let first_index: i32 = if is_periodic {
        1
    } else {
        first_u_knot_index(degree, mults)?
    };
    let last_index: i32 = if is_periodic {
        knots.len() as i32
    } else {
        last_u_knot_index(degree, mults)?
    };
    let mut kept: Vec<f64> = Vec::new();
    for index in first_index..=last_index {
        if array_value(mults, index)? > degree - continuity
            || index == first_index
            || index == last_index
        {
            kept.push(array_value(knots, index)?);
        }
    }
    let mut new_knots: Array1<'static, f64> = one_based(INTERVALS, kept.len() as i32)?;
    new_knots.as_mut_slice().copy_from_slice(&kept);
    Ok(new_knots)
}

// The range moved into the base period, with the numbers of periods removed.
struct PeriodicRange {
    cur_first: f64,
    cur_last: f64,
    period: f64,
    first_period: i32,
    last_period: i32,
}

impl PeriodicRange {
    fn new(
        knots: &Array1<'_, f64>,
        is_periodic: bool,
        first: f64,
        last: f64,
    ) -> Result<Self, Failure> {
        let mut range = Self {
            cur_first: first,
            cur_last: last,
            period: 0.0,
            first_period: 0,
            last_period: 0,
        };
        if !is_periodic {
            return Ok(range);
        }
        let lower: f64 = array_value(knots, knots.lower())?;
        let upper: f64 = array_value(knots, knots.upper())?;
        range.period = upper - lower;
        if !(range.period > 0.0
            && range.period.is_finite()
            && first.is_finite()
            && last.is_finite())
        {
            // The native loops would never end.
            return Err(Failure::out_of_range(&format!(
                "{INTERVALS}: period {} cannot bring [{first}, {last}] into [{lower}, {upper})",
                range.period
            )));
        }
        range.cur_first = wrap(
            range.cur_first,
            lower,
            upper,
            range.period,
            &mut range.first_period,
        );
        range.cur_last = wrap(
            range.cur_last,
            lower,
            upper,
            range.period,
            &mut range.last_period,
        );
        Ok(range)
    }

    fn fill(
        &self,
        new_knots: &Array1<'_, f64>,
        [index1, index2]: [i32; 2],
        nb_intervals: i32,
        output: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        output
            .resize_bounds(1, nb_intervals + 1, false)
            .map_err(|error| Failure::out_of_range(&format!("{INTERVALS}: {error}")))?;
        let length: i32 = new_knots.len() as i32;
        let mut index: i32 = 1;
        let mut put = |knot: i32, period: i32| -> Result<(), Failure> {
            *array_cell(output, index)? =
                array_value(new_knots, knot)? + f64::from(period) * self.period;
            index += 1;
            Ok(())
        };
        if self.last_period == self.first_period {
            for i in index1..=index2 {
                put(i, self.first_period)?;
            }
            return Ok(());
        }
        // From the beginning of the range to the end of the first period ...
        for i in index1..length {
            put(i, self.first_period)?;
        }
        // ... full periods ...
        for period in self.first_period + 1..self.last_period {
            for i in 1..length {
                put(i, period)?;
            }
        }
        // ... and from the beginning of the last period to the end of the range.
        for i in 1..=index2 {
            put(i, self.last_period)?;
        }
        Ok(())
    }
}

// The native while loops shifting by whole periods, counting them.
fn wrap(value: f64, lower: f64, upper: f64, period: f64, count: &mut i32) -> f64 {
    let mut result: f64 = value;
    while result < lower {
        result += period;
        *count -= 1;
    }
    while result >= upper {
        result -= period;
        *count += 1;
    }
    result
}
