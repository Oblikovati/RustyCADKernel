// Created by: Eugeny MALTCHIKOV
// Copyright (c) 2017 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_OBB.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::OrientedBoundingBox;
use super::super::accumulate::out_of_range;
use super::DitoBuilder;
use crate::gp::{Direction3, Point3, Xyz};
use rustycad_tkernel::standard::Failure;

/// `SetMinMax`: widens `[low, high]` with strict comparisons, both tested.
fn set_min_max(interval: &mut [f64], parameter: f64) {
    if parameter < interval[0] {
        interval[0] = parameter;
    }
    if parameter > interval[1] {
        interval[1] = parameter;
    }
}

impl DitoBuilder<'_, '_> {
    /// Runs the ditetrahedron search, then `BuildBox`; `DitoBuilder::new(..)?.build_box(&mut obb)?`.
    pub(in crate::bnd::obb) fn build_box(
        mut self,
        target: &mut OrientedBoundingBox,
    ) -> Result<(), Failure> {
        self.process_ditetrahedron()?;
        target.set_void();
        // The native "in fact, use Precision::SquareConfusion()" threshold is the literal 1e-14.
        let is_obb = self.axes[0].square_modulus()
            * self.axes[1].square_modulus()
            * self.axes[2].square_modulus()
            > 1.0e-14;
        let frame = if is_obb {
            [
                Direction3::from_xyz(self.axes[0])?,
                Direction3::from_xyz(self.axes[1])?,
                Direction3::from_xyz(self.axes[2])?,
            ]
        } else {
            [Direction3::X, Direction3::Y, Direction3::Z]
        };
        let parameters = self.frame_intervals(&frame)?;
        let half = |axis: usize| 0.5 * (parameters[2 * axis + 1] - parameters[2 * axis]);
        let sum = |axis: usize| parameters[2 * axis + 1] + parameters[2 * axis];
        let [x, y, z] = frame.map(Xyz::from);
        let center = 0.5 * (sum(0) * x + sum(1) * y + sum(2) * z);
        target.set_center(Point3::from(center));
        target.set_x_component(frame[0], half(0));
        target.set_y_component(frame[1], half(1));
        target.set_z_component(frame[2], half(2));
        target.set_aa_box(!is_obb);
        Ok(())
    }
    /// Min/max of every point on each frame axis, widened by the point tolerances when given.
    fn frame_intervals(&self, frame: &[Direction3; 3]) -> Result<[f64; 6], Failure> {
        let axes = frame.map(Xyz::from);
        let first = self.points.first().map_err(out_of_range)?.xyz();
        let mut parameters: [f64; 6] = std::array::from_fn(|i| first.dot(axes[i / 2]));
        if let Some(tolerances) = self.tolerances {
            let tolerance = *tolerances.first().map_err(out_of_range)?;
            for axis in 0..3 {
                parameters[2 * axis] -= tolerance;
                parameters[2 * axis + 1] += tolerance;
            }
        }
        for index in self.points.lower() + 1..=self.points.upper() {
            let point = self.points.value(index).map_err(out_of_range)?.xyz();
            let projections = axes.map(|axis| point.dot(axis));
            match self.tolerances {
                None => {
                    for (axis, projection) in projections.into_iter().enumerate() {
                        set_min_max(&mut parameters[2 * axis..2 * axis + 2], projection);
                    }
                }
                Some(tolerances) => {
                    let tolerance = *tolerances.value(index).map_err(out_of_range)?;
                    widen_with_tolerance(&mut parameters, projections, tolerance);
                }
            }
        }
        Ok(parameters)
    }
}

/// `std::min(p, d - tol)` / `std::max(p, d + tol)`: the stored bound wins ties and NaN.
fn widen_with_tolerance(parameters: &mut [f64; 6], projections: [f64; 3], tolerance: f64) {
    for (axis, projection) in projections.into_iter().enumerate() {
        let (low, high) = (projection - tolerance, projection + tolerance);
        if low < parameters[2 * axis] {
            parameters[2 * axis] = low;
        }
        if parameters[2 * axis + 1] < high {
            parameters[2 * axis + 1] = high;
        }
    }
}
