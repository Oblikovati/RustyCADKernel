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
// Rust translation of the root isolation loop of Perform and of appendRoot in
// ExtremaPC2d_GridEvaluator.hxx and ExtremaPC_GridEvaluator.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The root isolation of the grid evaluator: every partition cell is solved with
//! `MathRoot::FindAllRootsWithDerivative` on the curve trimmed to the cell.
use super::{PcGridEvaluator, RootCandidate};
use crate::extrema_pc_core::distance::{CapturingDistance, PcDistanceFunction};
use crate::extrema_pc_core::space::PcSpace;
use crate::extrema_pc_core::validity::{THE_DEFAULT_TOLERANCE, THE_PARAM_TOLERANCE};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::math_root::{MultipleConfig, MultipleResult, find_all_roots_with_derivative};

impl<S: PcSpace> PcGridEvaluator<S> {
    /// Solves each cell of the partition; false when a solve fails or stays all-null
    /// after the strict retry (the native NumericalError).
    pub(super) fn isolate_roots(&mut self, curve: &S::Curve, point: &S::Point, tol: f64) -> bool {
        let config = MultipleConfig {
            nb_samples: 2,
            x_tolerance: THE_PARAM_TOLERANCE,
            f_tolerance: tol,
            null_tolerance: tol,
            ..MultipleConfig::default()
        };
        self.roots.clear();
        for index in 0..self.partition.len().saturating_sub(1) {
            let (lower, upper) = (self.partition[index], self.partition[index + 1]);
            if upper - lower <= THE_PARAM_TOLERANCE {
                continue;
            }
            let Some(roots) = solve_cell::<S>(curve, point, (lower, upper), (&config, tol)) else {
                return false;
            };
            for root in roots.roots {
                append_root(
                    &mut self.roots,
                    RootCandidate {
                        parameter: root,
                        left_bound: lower,
                        right_bound: upper,
                    },
                );
            }
        }
        true
    }
}

// One cell: the curve trimmed to it (the whole curve when Trim fails or is null), the
// solve, and the strict retry of an all-null solve.
fn solve_cell<S: PcSpace>(
    curve: &S::Curve,
    point: &S::Point,
    (lower, upper): (f64, f64),
    (config, tol): (&MultipleConfig, f64),
) -> Option<MultipleResult> {
    // Some custom adaptors support evaluation but do not implement Trim().
    let local: Handle<S::Curve> =
        S::trim(curve, lower, upper, precision::p_confusion()).unwrap_or_default();
    let curve_for_interval: &S::Curve = local.get().unwrap_or(curve);
    let function = PcDistanceFunction::<S>::new(curve_for_interval, *point);
    let roots: MultipleResult = solve(&function, (lower, upper), config)?;
    if !roots.is_done() {
        return None;
    }
    if !roots.is_all_null {
        return Some(roots);
    }
    let retry = MultipleConfig {
        f_tolerance: std_min(tol, THE_DEFAULT_TOLERANCE),
        null_tolerance: 0.0,
        ..*config
    };
    let roots: MultipleResult = solve(&function, (lower, upper), &retry)?;
    (roots.is_done() && !roots.is_all_null).then_some(roots)
}

// `FindAllRootsWithDerivative` inside its try block: `None` for a raised exception.
fn solve<S: PcSpace>(
    function: &PcDistanceFunction<'_, S>,
    (lower, upper): (f64, f64),
    config: &MultipleConfig,
) -> Option<MultipleResult> {
    let mut functor = CapturingDistance::new(function);
    let roots: MultipleResult = find_all_roots_with_derivative(&mut functor, lower, upper, config);
    functor.capture.take().ok()?;
    Some(roots)
}

/// `appendRoot`: appends a root in order, merging the cells of an equivalent root; the
/// insertion path defends against unordered solver output, as natively.
fn append_root(roots: &mut Vec<RootCandidate>, root: RootCandidate) {
    if let Some(last) = roots.last_mut() {
        if (last.parameter - root.parameter).abs() <= THE_PARAM_TOLERANCE {
            merge_cells(last, &root);
            return;
        }
        if last.parameter < root.parameter {
            roots.push(root);
            return;
        }
    }
    for index in 0..roots.len() {
        let existing: &mut RootCandidate = &mut roots[index];
        if (existing.parameter - root.parameter).abs() <= THE_PARAM_TOLERANCE {
            merge_cells(existing, &root);
            return;
        }
        if root.parameter < existing.parameter {
            roots.insert(index, root);
            return;
        }
    }
    roots.push(root);
}

// `std::min` / `std::max` of the cell bounds.
fn merge_cells(existing: &mut RootCandidate, root: &RootCandidate) {
    existing.left_bound = std_min(existing.left_bound, root.left_bound);
    existing.right_bound =
        rustycad_tkg2d::geom2d::native::std_max(existing.right_bound, root.right_bound);
}
