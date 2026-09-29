// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_Batten.cxx
// (SlidingOfReference, Compute(D, A1, A2), Compute(D, A)) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reference sliding length of a batten: an estimate of the length of the
//! curve from the chord and the end angles.
use super::FairCurveBatten;
use rustycad_tkernel::precision;
use std::f64::consts::{FRAC_PI_2, PI};

impl FairCurveBatten {
    /// The reference length of the requested constraints;
    /// `batten.sliding_of_reference()`.
    pub fn sliding_of_reference(&self) -> f64 {
        self.sliding_of_reference_for(
            self.new.p1.distance(self.new.p2),
            self.new.angle1,
            self.new.angle2,
        )
    }
    /// The protected `SlidingOfReference(Dist, Angle1, Angle2)`. Quirk kept: an
    /// unconstrained end takes half of the opposite angle, and the test of that
    /// angle reads the requested (`New`) angle, not the argument.
    pub(super) fn sliding_of_reference_for(&self, dist: f64, angle1: f64, angle2: f64) -> f64 {
        let (order1, order2) = (self.new.constraint_order1, self.new.constraint_order2);
        if order1 == 0 && order2 == 0 {
            return dist;
        }
        let half = |requested: f64, angle: f64| {
            (if requested.abs() < PI {
                angle / 2.0
            } else {
                FRAC_PI_2
            })
            .abs()
        };
        let a1: f64 = if order1 == 0 {
            half(self.new.angle2, angle2)
        } else {
            angle1.abs()
        };
        let a2: f64 = if order2 == 0 {
            half(self.new.angle1, angle1)
        } else {
            angle2.abs()
        };
        if angle1 * angle2 >= 0.0 {
            return Self::length_of_pair(dist, a1, a2);
        }
        let sum: f64 = a1 + a2;
        if sum.abs() < precision::angular() {
            return Self::length_of_pair(dist, a1, FRAC_PI_2);
        }
        let ratio: f64 = a1 / sum;
        let mut angle_milieu: f64 = (1.0 - ratio) * (1.0 - ratio) * a1 + ratio * ratio * a2;
        if angle_milieu > FRAC_PI_2 {
            angle_milieu = FRAC_PI_2;
        }
        ratio * Self::length_of_pair(dist, a1, angle_milieu)
            + (1.0 - ratio) * Self::length_of_pair(dist, a2, angle_milieu)
    }
    /// The protected `Compute(Dist, Angle1, Angle2)`: 0.3 of the longer estimate
    /// plus 0.7 of the shorter one.
    pub(super) fn length_of_pair(dist: f64, angle1: f64, angle2: f64) -> f64 {
        let mut l1: f64 = Self::length_of_angle(dist, angle1);
        let mut l2: f64 = Self::length_of_angle(dist, angle2);
        if l1 < l2 {
            (l1, l2) = (l2, l1);
        }
        0.3 * l1 + 0.7 * l2
    }
    /// The protected `Compute(Dist, Angle)`: the chord, the circular arc of that
    /// angle, then linear and square-root growth beyond a half turn.
    pub(super) fn length_of_angle(dist: f64, angle: f64) -> f64 {
        if angle < precision::angular() {
            return dist;
        }
        if angle < FRAC_PI_2 {
            return angle * dist / angle.sin();
        }
        if angle > PI {
            return (angle * PI).sqrt() * dist;
        }
        angle * dist
    }
}
