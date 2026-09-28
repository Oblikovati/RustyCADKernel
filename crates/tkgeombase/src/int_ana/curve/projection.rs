// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of IntAna_Curve::FindParameter at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! IntAna_Curve::FindParameter: the parameters of a point on the curve.
use super::Curve;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point3;
use std::f64::consts::PI;

/// The angular gap (1e-8) within which the angle of the point snaps to a bound.
const EPS_ANG: f64 = 1.0e-8;
/// The square distance (1e-8) that accepts the projected parameter itself.
const INTERNAL_PRECISION: f64 = 1.0e-8;

impl Curve {
    /// FindParameter(P, Params): appends to `params`, in ascending order, the
    /// candidate parameters (domain bounds, the angle of `point` on the cylinder or
    /// the cone and, for two curves, its mirror and the far bound) whose curve point
    /// is within the tolerance of `point`: 1e-8 (square) for the angle and its
    /// mirror, Precision::SquareConfusion() for the bounds. Only the first four
    /// candidates are sorted, as natively. Nothing for another quadric; a
    /// candidate outside the domain raises Standard_DomainError (it cannot happen
    /// for a valid trim). `curve.find_parameter(point, &mut params)?`.
    pub fn find_parameter(&self, point: Point3, params: &mut List<f64>) -> Result<(), Failure> {
        let Some(theta) = self.point_angle(point) else {
            return Ok(());
        };
        let theta: f64 = self.snap_angle(theta);
        let (inf, sup) = (self.domain_inf, self.domain_sup);
        let mut candidates: [f64; 5] = [
            inf,
            sup,
            theta,
            if self.two_curves {
                sup + sup - theta
            } else {
                f64::MAX
            },
            if self.two_curves {
                sup + sup - inf
            } else {
                f64::MAX
            },
        ];
        insertion_sort(&mut candidates[..4]);
        for i in 0..candidates.len() {
            let candidate: f64 = candidates[i];
            if candidate > self.last_parameter {
                break;
            }
            if candidate < self.first_parameter {
                continue;
            }
            if i != 0 && (candidate - candidates[i - 1]) < precision::p_confusion() {
                continue;
            }
            let uv = self.internal_uv_value(candidate, 0.0)?;
            let on_curve: Point3 = self.internal_value(uv.u, uv.v);
            let square_tolerance: f64 =
                if candidate == theta || (self.two_curves && candidate == sup + sup - theta) {
                    INTERNAL_PRECISION
                } else {
                    precision::square_confusion()
                };
            if on_curve.square_distance(&point) < square_tolerance {
                params.append(candidate);
            }
        }
        Ok(())
    }

    /// The angle of `point` on the cylinder or the cone, `None` for another quadric.
    fn point_angle(&self, point: Point3) -> Option<f64> {
        match self.typequadric {
            SurfaceType::Cylinder => {
                Some(elslib::cylinder_parameters(self.ax3, self.rcyl, point).0)
            }
            SurfaceType::Cone => {
                Some(elslib::cone_parameters(self.ax3, self.rcyl, self.angle, point).0)
            }
            _ => None,
        }
    }

    /// Snaps an angle just outside a closed bound onto it, then brings it into the
    /// domain by one period.
    fn snap_angle(&self, angle: f64) -> f64 {
        let (inf, sup) = (self.domain_inf, self.domain_sup);
        let mut theta: f64 = angle;
        if !self.firstbounded && (inf > theta) && ((inf - theta) <= EPS_ANG) {
            theta = inf;
        } else if !self.lastbounded && (theta > sup) && ((theta - sup) <= EPS_ANG) {
            theta = sup;
        }
        if theta < inf {
            theta += PI + PI;
        } else if theta > sup {
            theta -= PI + PI;
        }
        theta
    }
}

/// std::sort of a few doubles as libstdc++ runs it below 16 elements: a stable
/// insertion sort on `<`, which fixes the order of signed zeros and NaN.
fn insertion_sort(values: &mut [f64]) {
    for i in 1..values.len() {
        let value: f64 = values[i];
        if value < values[0] {
            values.copy_within(0..i, 1);
            values[0] = value;
            continue;
        }
        let mut j: usize = i;
        while j > 0 && value < values[j - 1] {
            values[j] = values[j - 1];
            j -= 1;
        }
        values[j] = value;
    }
}
