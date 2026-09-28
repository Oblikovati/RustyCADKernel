// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of EvaluateDu, PerformLinear and PerformCircular of
// ModelingData/TKGeomBase/GCPnts/GCPnts_TangentialDeflection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The line and circle cases of GCPnts_TangentialDeflection and the curvature step.
use super::{DeflectionCurve, TangentialDeflection, arc_angular_step};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::Point3;

/// The outputs of `EvaluateDu`: the point is always written, the step only when the
/// curvature is significant (then `not_done` is cleared).
pub(super) struct StepEstimate {
    pub point: Point3,
    pub du: f64,
    pub not_done: bool,
}

impl TangentialDeflection {
    /// `EvaluateDu(theC, theU, theP, theDu, theNotDone)`: the parameter step whose
    /// chord deflection matches the curvature at `u`, starting from `du`/`not_done`.
    pub(super) fn evaluate_du(
        &self,
        curve: &impl DeflectionCurve,
        u: f64,
        du: f64,
        not_done: bool,
    ) -> Result<StepEstimate, Failure> {
        let (point, t, n) = curve.lifted_d2(u)?;
        let mut estimate = StepEstimate {
            point,
            du,
            not_done,
        };
        let lt: f64 = t.magnitude();
        let ltol: f64 = precision::confusion();
        if lt > ltol && n.magnitude() > ltol {
            let lc: f64 = n.cross_magnitude(t);
            let ln: f64 = lc / lt;
            if ln > ltol {
                estimate.du = (8.0 * std_max(self.curvature_deflection, self.min_len) / ln).sqrt();
                estimate.not_done = false;
            }
        }
        Ok(estimate)
    }
    /// `PerformLinear`: the end points and `myMinNbPnts - 2` equidistant interior points.
    pub(super) fn perform_linear(&mut self, curve: &impl DeflectionCurve) -> Result<(), Failure> {
        let first: Point3 = curve.lifted_d0(self.first_u)?;
        self.append(self.first_u, first);
        if self.min_nb_pnts > 2 {
            let du: f64 = (self.last_u - self.first_u) / f64::from(self.min_nb_pnts);
            let mut u: f64 = self.first_u + du;
            for _ in 2..self.min_nb_pnts {
                let point: Point3 = curve.lifted_d0(u)?;
                self.append(u, point);
                u += du;
            }
        }
        let last: Point3 = curve.lifted_d0(self.last_u)?;
        self.append(self.last_u, last);
        Ok(())
    }
    /// `PerformCircular`: equal angular steps from ArcAngularStep.
    pub(super) fn perform_circular(&mut self, curve: &impl DeflectionCurve) -> Result<(), Failure> {
        // akm 8/01/02 : check the radius before divide by it
        let radius: f64 = curve.circle_radius()?;
        let mut du: f64 = arc_angular_step(
            radius,
            self.curvature_deflection,
            self.angular_deflection,
            self.min_len,
        )?;
        let diff: f64 = self.last_u - self.first_u;
        // Round up number of points to satisfy curvatureDeflection more precisely
        let mut nb_points: i32 = std_min((diff / du).ceil(), 1.0e6) as i32;
        nb_points = nb_points.max(self.min_nb_pnts - 1);
        du = diff / f64::from(nb_points);
        let mut u: f64 = self.first_u;
        for _ in 1..=nb_points {
            let point: Point3 = curve.lifted_d0(u)?;
            self.append(u, point);
            u += du;
        }
        let last: Point3 = curve.lifted_d0(self.last_u)?;
        self.append(self.last_u, last);
        Ok(())
    }
}
