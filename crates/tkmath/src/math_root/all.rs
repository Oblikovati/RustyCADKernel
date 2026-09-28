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

//! Null intervals and isolated roots preserving MathRoot_All.hxx.
use super::multiple::MultipleResult;
use super::utils::{find_multiple_roots, is_valid_bounds};
use crate::math::{Function, Vector};
use crate::math_utils::Status;
/// Native null-interval record; `NullInterval::default()`.
#[derive(Clone, Copy, Default)]
pub struct NullInterval {
    /// Start parameter; `interval.a`.
    pub a: f64,
    /// End parameter; `interval.b`.
    pub b: f64,
    /// Native state identifier (zero in this solver); `interval.state`.
    pub state: i32,
}
/// Isolated roots and null intervals; `AllRootsResult::default()`.
#[derive(Default)]
pub struct AllRootsResult {
    /// Completion status; `result.status`.
    pub status: Status,
    /// Isolated root parameters; `result.roots`.
    pub roots: Vec<f64>,
    /// State identifiers corresponding to roots; `result.root_states`.
    pub root_states: Vec<i32>,
    /// Null intervals; `result.null_intervals`.
    pub null_intervals: Vec<NullInterval>,
}
impl AllRootsResult {
    /// Reports native success; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
    /// Counts isolated roots; `result.nb_roots()`.
    pub fn nb_roots(&self) -> usize {
        self.roots.len()
    }
    /// Counts null intervals; `result.nb_intervals()`.
    pub fn nb_intervals(&self) -> usize {
        self.null_intervals.len()
    }
    fn failed(status: Status) -> Self {
        Self {
            status,
            ..Default::default()
        }
    }
}
impl From<&AllRootsResult> for bool {
    fn from(result: &AllRootsResult) -> bool {
        result.is_done()
    }
}
fn valid_tolerances(t: [f64; 3]) -> bool {
    t.iter().all(|x| x.is_finite()) && t[0] > 0.0 && t[1] >= 0.0 && t[2] >= 0.0
}
fn valid_samples(samples: &Vector<'_>, t: [f64; 3]) -> bool {
    if samples.length() < 2 || !valid_tolerances(t) {
        return false;
    }
    (0..samples.length() as usize).all(|i| {
        samples.at(i).unwrap().is_finite()
            && (i == 0 || samples.at(i).unwrap() > samples.at(i - 1).unwrap())
    })
}
struct Scan<'a, F> {
    function: &'a mut F,
    samples: &'a Vector<'a>,
    tolerances: [f64; 3],
    starts: Vec<f64>,
    ends: Vec<f64>,
    null_start: bool,
    null_end: bool,
    result: AllRootsResult,
}
impl<F: Function> Scan<'_, F> {
    fn value_into(&mut self, x: f64, value: &mut f64) -> Result<(), Status> {
        if !self.function.value(x, value) {
            return Err(Status::CallbackError);
        }
        if !value.is_finite() {
            return Err(Status::NumericalError);
        }
        Ok(())
    }
    fn roots(
        &mut self,
        lower: f64,
        upper: f64,
        count: usize,
        offset: f64,
    ) -> Result<MultipleResult, Status> {
        let [eps_x, eps_f, _] = self.tolerances;
        let result: MultipleResult =
            find_multiple_roots(self.function, lower, upper, count, eps_x, eps_f, offset);
        if result.is_done() {
            Ok(result)
        } else {
            Err(result.status)
        }
    }
    fn boundary(&mut self, lower: f64, upper: f64, value: f64, start: bool) -> Result<f64, Status> {
        let offset: f64 = if value > 0.0 {
            self.tolerances[2]
        } else {
            -self.tolerances[2]
        };
        let first: MultipleResult = self.roots(lower, upper, 10, offset)?;
        let mut boundary: f64 = if start {
            first.roots.last().copied().unwrap_or(upper)
        } else {
            first.roots.first().copied().unwrap_or(lower)
        };
        let second: MultipleResult = self.roots(lower, upper, 10, -offset)?;
        let candidate: Option<f64> = if start {
            second.roots.last().copied()
        } else {
            second.roots.first().copied()
        };
        if let Some(candidate) = candidate
            && ((start && candidate > boundary) || (!start && candidate < boundary))
        {
            boundary = candidate;
        }
        Ok(boundary)
    }
    fn scan(&mut self) -> Result<(), Status> {
        let mut previous: f64 = 0.0;
        self.value_into(*self.samples.at(0).unwrap(), &mut previous)?;
        let mut previous_null: bool = previous.abs() <= self.tolerances[2];
        let mut in_interval: bool = false;
        let mut start: f64 = 0.0;
        let mut saved: f64 = previous;
        let mut value: f64 = 0.0;
        for i in 1..self.samples.length() as usize {
            self.value_into(*self.samples.at(i).unwrap(), &mut value)?;
            let current_null: bool = value.abs() <= self.tolerances[2];
            if !current_null {
                saved = value;
            }
            self.transition(
                i,
                previous_null,
                current_null,
                value,
                saved,
                &mut in_interval,
                &mut start,
            )?;
            previous_null = current_null;
        }
        if in_interval {
            self.starts.push(start);
            self.ends
                .push(*self.samples.at(self.samples.length() as usize - 1).unwrap());
            self.null_end = true;
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn transition(
        &mut self,
        i: usize,
        previous: bool,
        current: bool,
        value: f64,
        saved: f64,
        in_interval: &mut bool,
        start: &mut f64,
    ) -> Result<(), Status> {
        if *in_interval && !current {
            *in_interval = false;
            self.starts.push(*start);
            let end: f64 = self.boundary(
                *self.samples.at(i - 1).unwrap(),
                *self.samples.at(i).unwrap(),
                value,
                false,
            )?;
            self.ends.push(end);
        } else if !*in_interval && previous && current {
            *in_interval = true;
            if i == 1 {
                *start = *self.samples.at(0).unwrap();
                self.null_start = true;
            } else {
                *start = self.boundary(
                    *self.samples.at(i - 2).unwrap(),
                    *self.samples.at(i - 1).unwrap(),
                    saved,
                    true,
                )?;
            }
        }
        Ok(())
    }
    fn append_roots(&mut self, start: f64, end: f64, count: usize) -> Result<(), Status> {
        let result: MultipleResult = self.roots(start, end, count, 0.0)?;
        for root in result.roots {
            self.result.roots.push(root);
            self.result.root_states.push(0);
        }
        Ok(())
    }
    fn segment(&mut self, start: f64, end: f64, first: f64, last: f64) -> Result<(), Status> {
        let count: usize =
            (((end - start) / (last - first)).abs() * self.samples.length() as f64) as usize;
        self.append_roots(start, end, count.max(3))
    }
    fn isolated_roots(&mut self) -> Result<(), Status> {
        let first: f64 = *self.samples.at(0).unwrap();
        let last: f64 = *self.samples.at(self.samples.length() as usize - 1).unwrap();
        if self.starts.is_empty() {
            return self.append_roots(first, last, self.samples.length() as usize);
        }
        if !self.null_start {
            self.segment(first, self.starts[0], first, last)?;
        }
        for i in 1..self.starts.len() {
            self.segment(self.ends[i - 1], self.starts[i], first, last)?;
        }
        if !self.null_end {
            self.segment(*self.ends.last().unwrap(), last, first, last)?;
        }
        Ok(())
    }
    fn run(&mut self) -> Result<(), Status> {
        self.scan()?;
        for i in 0..self.starts.len() {
            self.result.null_intervals.push(NullInterval {
                a: self.starts[i],
                b: self.ends[i],
                state: 0,
            });
        }
        self.isolated_roots()?;
        self.result.status = Status::Ok;
        Ok(())
    }
}
/// Searches prescribed samples for null intervals and roots; `find_all_roots_at_samples(&mut function,&samples,[1e-10;3])`.
pub fn find_all_roots_at_samples(
    function: &mut impl Function,
    samples: &Vector<'_>,
    tolerances: [f64; 3],
) -> AllRootsResult {
    if !valid_samples(samples, tolerances) {
        return AllRootsResult::failed(Status::InvalidInput);
    }
    let mut scan = Scan {
        function,
        samples,
        tolerances,
        starts: Vec::new(),
        ends: Vec::new(),
        null_start: false,
        null_end: false,
        result: AllRootsResult::default(),
    };
    if let Err(status) = scan.run() {
        scan.result.status = status;
    }
    scan.result
}
/// Builds the native uniform sample sequence; `find_all_roots_with_intervals(&mut function,0.0,1.0,20,[1e-10;3])`.
pub fn find_all_roots_with_intervals(
    function: &mut impl Function,
    lower: f64,
    upper: f64,
    count: usize,
    tolerances: [f64; 3],
) -> AllRootsResult {
    if !is_valid_bounds(lower, upper) || count < 2 || !valid_tolerances(tolerances) {
        return AllRootsResult::failed(Status::InvalidInput);
    }
    let mut samples: Vector<'static> = Vector::with_size(count);
    for i in 0..count {
        let ratio: f64 = i as f64 / (count - 1) as f64;
        *samples.change_at(i).unwrap() = (1.0 - ratio) * lower + ratio * upper;
    }
    *samples.change_at(count - 1).unwrap() = upper;
    find_all_roots_at_samples(function, &samples, tolerances)
}
