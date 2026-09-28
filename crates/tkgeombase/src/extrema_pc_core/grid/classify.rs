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
// Rust translation of stationaritySign, addExtremum, classifyRoots and classifyJunctions
// of ExtremaPC2d_GridEvaluator.hxx and ExtremaPC_GridEvaluator.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The classification of the isolated roots, the C2 junctions and the periodic seam
//! from the stationarity sign on both sides of each candidate.
use super::{GridFailurePolicy, PcGridEvaluator};
use crate::extrema_pc_core::distance::PcDistanceFunction;
use crate::extrema_pc_core::result::{PcExtremum, SearchMode};
use crate::extrema_pc_core::space::{PcPoint, PcSpace};
use crate::extrema_pc_core::validity::{THE_DEFAULT_TOLERANCE, THE_PARAM_TOLERANCE};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::math_utils::Domain1D;

/// The inputs shared by the classification passes.
pub(super) struct Classification<'c, 'a, S: PcSpace> {
    function: &'c PcDistanceFunction<'a, S>,
    domain: &'c Domain1D,
    tol: f64,
    mode: SearchMode,
}

impl<S: PcSpace> PcGridEvaluator<S> {
    /// `classifyRoots` then `classifyJunctions`.
    pub(super) fn classify(
        &mut self,
        function: &PcDistanceFunction<'_, S>,
        domain: &Domain1D,
        is_closed: bool,
        (tol, mode): (f64, SearchMode),
    ) -> Result<(), Failure> {
        let pass = Classification {
            function,
            domain,
            tol,
            mode,
        };
        self.classify_roots(&pass)?;
        self.classify_junctions(&pass)?;
        if is_closed {
            self.classify_seam(&pass)?;
        }
        Ok(())
    }
    // `classifyRoots`: smooth stationary roots within their isolation cells.
    fn classify_roots(&mut self, pass: &Classification<'_, '_, S>) -> Result<(), Failure> {
        for index in 0..self.roots.len() {
            let root = self.roots[index];
            let is_junction: bool = self
                .continuity_bounds
                .iter()
                .any(|junction| (root.parameter - junction).abs() <= THE_PARAM_TOLERANCE);
            let is_at_first: bool = root.parameter <= pass.domain.min + THE_PARAM_TOLERANCE;
            let is_at_last: bool = root.parameter >= pass.domain.max - THE_PARAM_TOLERANCE;
            if is_junction || is_at_first || is_at_last {
                continue;
            }
            let (mut left, mut right) = (root.left_bound, root.right_bound);
            if index > 0 {
                left = std_max(
                    left,
                    (self.roots[index - 1].parameter + root.parameter) * 0.5,
                );
            }
            if index + 1 < self.roots.len() {
                right = std_min(
                    right,
                    (root.parameter + self.roots[index + 1].parameter) * 0.5,
                );
            }
            let left_sign: i32 = stationarity_sign(pass, (left + root.parameter) * 0.5)?;
            let right_sign: i32 = stationarity_sign(pass, (root.parameter + right) * 0.5)?;
            self.add_extremum(pass, root.parameter, (left_sign, right_sign))?;
        }
        Ok(())
    }
    // `classifyJunctions`: the C2 continuity bounds from one-sided values.
    fn classify_junctions(&mut self, pass: &Classification<'_, '_, S>) -> Result<(), Failure> {
        for index in 0..self.continuity_bounds.len() {
            let junction: f64 = self.continuity_bounds[index];
            let (left, right) = self.junction_cell(pass.domain, junction);
            let left_sign: i32 = stationarity_sign(pass, (left + junction) * 0.5)?;
            let right_sign: i32 = stationarity_sign(pass, (junction + right) * 0.5)?;
            self.add_extremum(pass, junction, (left_sign, right_sign))?;
        }
        Ok(())
    }
    // The partition cell of a junction, narrowed to the nearest roots on each side.
    fn junction_cell(&self, domain: &Domain1D, junction: f64) -> (f64, f64) {
        let (mut left, mut right) = (domain.min, domain.max);
        let partition: &[f64] = &self.partition;
        if let Some(index) = (1..partition.len()).find(|&index| partition[index] >= junction) {
            left = partition[index - 1];
            right = partition[index];
            if (right - junction).abs() <= THE_PARAM_TOLERANCE && index + 1 < partition.len() {
                right = partition[index + 1];
            }
        }
        for root in &self.roots {
            let root: f64 = root.parameter;
            if root > left + THE_PARAM_TOLERANCE && root < junction - THE_PARAM_TOLERANCE {
                left = root;
            } else if root > junction + THE_PARAM_TOLERANCE && root < right - THE_PARAM_TOLERANCE {
                right = root;
                break;
            }
        }
        (left, right)
    }
    // The periodic seam of a closed domain, classified at its first parameter.
    fn classify_seam(&mut self, pass: &Classification<'_, '_, S>) -> Result<(), Failure> {
        let domain: &Domain1D = pass.domain;
        let mut left: f64 = self.params[self.params.len() - 2];
        let mut right: f64 = self.params[1];
        for root in &self.roots {
            let root: f64 = root.parameter;
            if root > left + THE_PARAM_TOLERANCE && root < domain.max - THE_PARAM_TOLERANCE {
                left = root;
            }
            if root > domain.min + THE_PARAM_TOLERANCE && root < right - THE_PARAM_TOLERANCE {
                right = root;
            }
        }
        let left_sign: i32 = stationarity_sign(pass, (left + domain.max) * 0.5)?;
        let right_sign: i32 = stationarity_sign(pass, (domain.min + right) * 0.5)?;
        self.add_extremum(pass, domain.min, (left_sign, right_sign))
    }
    // `addExtremum`: a sign change from - to + is a minimum, from + to - a maximum; one
    // matching the search mode and not duplicated is appended.
    fn add_extremum(
        &mut self,
        pass: &Classification<'_, '_, S>,
        parameter: f64,
        (left_sign, right_sign): (i32, i32),
    ) -> Result<(), Failure> {
        let is_minimum: bool = left_sign < 0 && right_sign > 0;
        let is_maximum: bool = left_sign > 0 && right_sign < 0;
        if (!is_minimum && !is_maximum)
            || (pass.mode == SearchMode::Min && !is_minimum)
            || (pass.mode == SearchMode::Max && !is_maximum)
        {
            return Ok(());
        }
        let duplicate: bool = self
            .result
            .extrema
            .iter()
            .any(|extremum| (extremum.parameter - parameter).abs() <= THE_PARAM_TOLERANCE);
        if duplicate {
            return Ok(());
        }
        let curve_point: S::Point = S::value(pass.function.curve(), parameter)?;
        let square_distance: f64 = pass.function.point().square_distance_to(&curve_point);
        let extremum = PcExtremum::new(
            parameter,
            curve_point,
            square_distance,
            is_minimum,
            is_maximum,
        );
        self.result.extrema.push(extremum);
        Ok(())
    }
}

/// `stationaritySign`: the sign of F at a probe, 0 when |F| is within
/// min(tol, THE_DEFAULT_TOLERANCE). The 3D evaluator also answers 0 when the evaluation
/// raises; the 2D one lets the failure reach its classification `catch`.
fn stationarity_sign<S: PcSpace>(
    pass: &Classification<'_, '_, S>,
    parameter: f64,
) -> Result<i32, Failure> {
    let sign_tolerance: f64 = std_min(pass.tol, THE_DEFAULT_TOLERANCE);
    let value: f64 = match pass.function.value(parameter) {
        Ok(value) => value,
        Err(_) if S::FAILURES == GridFailurePolicy::CatchSigns => return Ok(0),
        Err(failure) => return Err(failure),
    };
    if value.abs() <= sign_tolerance {
        return Ok(0);
    }
    Ok(if value < 0.0 { -1 } else { 1 })
}
