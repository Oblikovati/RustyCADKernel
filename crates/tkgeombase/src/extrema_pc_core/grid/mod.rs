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
// Rust translation of ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_GridEvaluator.hxx and
// ExtremaPC/ExtremaPC_GridEvaluator.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The numerical point-curve extrema over a cached parameter partition
//! (`ExtremaPC2d_GridEvaluator`, `ExtremaPC_GridEvaluator`).
//!
//! Each adjacent parameter pair is solved independently with the derivative-aware
//! multiple-root solver, so non-uniform knot- and curvature-aware partitions are
//! effective. C2 continuity bounds are inserted into the partition and classified
//! separately from ordinary sampling boundaries.
mod classify;
mod params;
mod solve;

use super::distance::PcDistanceFunction;
use super::result::{PcResult, SearchMode, Status};
use super::space::{PcPoint, PcSpace};
use super::validity::is_valid_tolerance;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;
use rustycad_tkmath::math_utils::Domain1D;

/// Which failures of the root classification the evaluator turns into NumericalError.
///
/// The 2D evaluator catches every exception of its classification and clears the
/// result; the 3D one catches only the stationarity-sign evaluations (an unresolved
/// sign) and lets a failing `Value` escape its `Perform`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridFailurePolicy {
    /// ExtremaPC2d: any classification failure clears the result with NumericalError.
    CatchAll,
    /// ExtremaPC: a failed sign is unresolved (0); other failures propagate.
    CatchSigns,
}

/// A root and the non-uniform cell used to isolate it (`RootCandidate`).
#[derive(Clone, Copy, Debug)]
struct RootCandidate {
    parameter: f64,
    left_bound: f64,
    right_bound: f64,
}

/// The grid evaluator (`ExtremaPC2d_GridEvaluator`, `ExtremaPC_GridEvaluator`): the
/// cached partition and the reusable result and work arrays;
/// `let mut evaluator = GridEvaluator::default(); evaluator.set_params(&params);`.
pub struct PcGridEvaluator<S: PcSpace> {
    params: Vec<f64>,
    continuity_bounds: Vec<f64>,
    partition: Vec<f64>,
    roots: Vec<RootCandidate>,
    result: PcResult<S::Point>,
}

impl<S: PcSpace> Default for PcGridEvaluator<S> {
    fn default() -> Self {
        Self {
            params: Vec::new(),
            continuity_bounds: Vec::new(),
            partition: Vec::new(),
            roots: Vec::new(),
            result: PcResult::not_done(),
        }
    }
}

impl<S: PcSpace> PcGridEvaluator<S> {
    /// Stores the ordered parameters of the search partition (`SetParams`);
    /// `evaluator.set_params(&GridEvaluator::build_uniform_params(0.0, 1.0, 8)?)`.
    pub fn set_params(&mut self, params: &Vector<'_>) {
        self.params.clear();
        self.params.extend(params.array1().iter().copied());
    }
    /// The result, mutable for post-processing (`Result() const` returns a mutable
    /// reference natively); `evaluator.result().clear()`.
    pub fn result(&mut self) -> &mut PcResult<S::Point> {
        &mut self.result
    }
    /// The body of `Perform`: `Err` only for a failure the 3D policy lets escape.
    pub(crate) fn perform_core(
        &mut self,
        curve: &S::Curve,
        point: &S::Point,
        domain: &Domain1D,
        (tol, mode): (f64, SearchMode),
    ) -> Result<(), Failure> {
        self.result.clear();
        if !domain.is_valid() || !is_valid_tolerance(tol) || !point.is_finite_point() {
            self.result.status = Status::InvalidInput;
            return Ok(());
        }
        if domain.min == domain.max {
            self.result.status = Status::NoSolution;
            return Ok(());
        }
        if !self.validate_params(domain) {
            self.result.status = Status::InvalidInput;
            return Ok(());
        }
        if let Some(distance) = S::constant_square_distance(curve, point, domain) {
            self.result.status = Status::InfiniteSolutions;
            self.result.infinite_square_distance = distance;
            return Ok(());
        }
        self.search(curve, point, domain, (tol, mode))
    }
    // Isolates the roots cell by cell, then classifies them.
    fn search(
        &mut self,
        curve: &S::Curve,
        point: &S::Point,
        domain: &Domain1D,
        (tol, mode): (f64, SearchMode),
    ) -> Result<(), Failure> {
        self.continuity_bounds.clear();
        self.partition.clear();
        self.build_partition(curve, domain);
        if !self.isolate_roots(curve, point, tol) {
            self.result.status = Status::NumericalError;
            return Ok(());
        }
        let function = PcDistanceFunction::<S>::new(curve, *point);
        let is_closed: bool = Self::is_closed_domain(curve, domain);
        if let Err(failure) = self.classify(&function, domain, is_closed, (tol, mode)) {
            if S::FAILURES == GridFailurePolicy::CatchSigns {
                return Err(failure);
            }
            self.result.clear();
            self.result.status = Status::NumericalError;
            return Ok(());
        }
        self.result.status = if self.result.extrema.is_empty() {
            Status::NoSolution
        } else {
            Status::OK
        };
        Ok(())
    }
    /// `validateParams`: at least two finite, strictly increasing parameters whose ends
    /// match the domain within THE_PARAM_TOLERANCE.
    fn validate_params(&self, domain: &Domain1D) -> bool {
        use super::validity::THE_PARAM_TOLERANCE;
        let (Some(first), Some(last)) = (self.params.first(), self.params.last()) else {
            return false;
        };
        if self.params.len() < 2
            || (first - domain.min).abs() > THE_PARAM_TOLERANCE
            || (last - domain.max).abs() > THE_PARAM_TOLERANCE
        {
            return false;
        }
        self.params
            .windows(2)
            .all(|pair| pair[0].is_finite() && pair[1].is_finite() && pair[1] > pair[0])
    }
}
