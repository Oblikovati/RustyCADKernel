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

use super::{DitoBuilder, EXTREMAL_POINTS, UNSET};
use crate::gp::Xyz;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

impl DitoBuilder<'_, '_> {
    /// `ProcessDiTetrahedron`: every extremal-point triangle (optimal), otherwise the base
    /// triangle and the up to six faces of the two tetrahedra over it.
    pub(super) fn process_ditetrahedron(&mut self) -> Result<(), Failure> {
        if self.optimal {
            return self.process_all_triangles();
        }
        let [t0, t1, t2, _, _] = self.triangle;
        if [t0, t1, t2].contains(&UNSET) {
            // Only NaN coordinates leave a base index unset; native C++ then reads
            // myLExtremalPoints[INT_MAX] (undefined behavior).
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!(
                    "Bnd_OBB::ReBuild: base triangle {:?} has an unset index",
                    [t0, t1, t2]
                ),
                "",
            ));
        }
        self.process_triangle([t0, t1, t2], true)?;
        for apex in [self.triangle[3], self.triangle[4]] {
            if apex < EXTREMAL_POINTS {
                self.process_triangle([t0, t1, apex], false)?;
                self.process_triangle([t1, t2, apex], false)?;
                self.process_triangle([t0, t2, apex], false)?;
            }
        }
        Ok(())
    }
    fn process_all_triangles(&mut self) -> Result<(), Failure> {
        for i in 0..EXTREMAL_POINTS - 2 {
            for j in i + 1..EXTREMAL_POINTS - 1 {
                for k in j + 1..EXTREMAL_POINTS {
                    self.process_triangle([i, j, k], false)?;
                }
            }
        }
        Ok(())
    }
    /// `FillToTriangle3`: the extremal point farthest from the line of the first two.
    pub(super) fn fill_to_triangle3(&mut self) {
        let origin = self.extremal[self.triangle[0]];
        let axis = self.extremal[self.triangle[1]] - origin;
        let mut max_square_distance = -1.0;
        for index in 0..EXTREMAL_POINTS {
            if index == self.triangle[0] || index == self.triangle[1] {
                continue;
            }
            let square_distance = axis.cross_square_magnitude(self.extremal[index] - origin);
            if square_distance > max_square_distance {
                self.triangle[2] = index;
                max_square_distance = square_distance;
            }
        }
    }
    /// `FillToTriangle5`: the farthest extremal points strictly below and above the
    /// triangle plane, kept only beyond `Precision::Confusion()`.
    fn fill_to_triangle5(&mut self, normal: Xyz, barycenter: Xyz) {
        let (mut below, mut above) = (0.0, 0.0);
        let (mut below_index, mut above_index) = (None, None);
        for index in 0..EXTREMAL_POINTS {
            if self.triangle[..3].contains(&index) {
                continue;
            }
            let parameter = normal.dot(self.extremal[index] - barycenter);
            if parameter < below {
                below_index = Some(index);
                below = parameter;
            } else if parameter > above {
                above_index = Some(index);
                above = parameter;
            }
        }
        if let Some(index) = below_index.filter(|_| below < -precision::confusion()) {
            self.triangle[3] = index;
        }
        if let Some(index) = above_index.filter(|_| above > precision::confusion()) {
            self.triangle[4] = index;
        }
    }
    /// `ProcessTriangle`: tries the frames made of the triangle normal and one edge, keeping
    /// the one of least half surface area over all triangles processed so far.
    fn process_triangle(
        &mut self,
        [a, b, c]: [usize; 3],
        build_apexes: bool,
    ) -> Result<(), Failure> {
        let (heads, tails) = ([b, c, a], [a, b, c]);
        let edges: [Xyz; 3] =
            std::array::from_fn(|i| self.extremal[heads[i]] - self.extremal[tails[i]]);
        let normal = edges[0].crossed(edges[1]);
        let square_modulus = normal.square_modulus();
        if square_modulus < precision::square_confusion() {
            return Ok(());
        }
        let normal = normal / square_modulus.sqrt();
        let x_axes: [Option<Xyz>; 3] = edges.map(|edge| unit_cross(edge, normal));
        if build_apexes {
            self.fill_to_triangle5(normal, self.extremal[a]);
        }
        let mut parameters = [0.0; 6];
        (parameters[4], parameters[5]) = self.find_min_max(normal)?;
        let mut best = None;
        for (index, x_axis) in x_axes.iter().enumerate() {
            let Some(x_axis) = *x_axis else { continue };
            (parameters[0], parameters[1]) = self.find_min_max(x_axis)?;
            (parameters[2], parameters[3]) = self.edge_interval(heads[index], tails[index])?;
            let area = quality(&parameters);
            if area < self.quality {
                self.quality = area;
                best = Some((index, x_axis));
            }
        }
        if let Some((index, x_axis)) = best {
            self.axes = [x_axis, edges[index].normalized()?, normal];
        }
        Ok(())
    }
    /// `ComputeParams`: the interval on the axis joining two extremal points, cached
    /// symmetrically once computed (an interval with min > max is recomputed).
    fn edge_interval(&mut self, head: usize, tail: usize) -> Result<(f64, f64), Failure> {
        let cached = self.intervals[head][tail];
        // Only `min > max` (the void initial state) triggers a computation; NaN is cached.
        let uncomputed = cached.minimum > cached.maximum;
        if !uncomputed {
            return Ok((cached.minimum, cached.maximum));
        }
        let axis = (self.extremal[head] - self.extremal[tail]).normalized()?;
        let (minimum, maximum) = self.find_min_max(axis)?;
        let interval = super::AxisInterval { minimum, maximum };
        self.intervals[head][tail] = interval;
        self.intervals[tail][head] = interval;
        Ok((minimum, maximum))
    }
}

/// `edge × normal` normalized, or `None` when its square norm is below `SquareConfusion`.
fn unit_cross(edge: Xyz, normal: Xyz) -> Option<Xyz> {
    let cross = edge.crossed(normal);
    let square_modulus = cross.square_modulus();
    if square_modulus < precision::square_confusion() {
        return None;
    }
    Some(cross / square_modulus.sqrt())
}

/// `ComputeQuality`: half the surface area `dX·dY + dY·dZ + dX·dZ` of the candidate box.
fn quality(parameters: &[f64; 6]) -> f64 {
    let dx = parameters[1] - parameters[0];
    let dy = parameters[3] - parameters[2];
    let dz = parameters[5] - parameters[4];
    dx * dy + dy * dz + dx * dz
}
