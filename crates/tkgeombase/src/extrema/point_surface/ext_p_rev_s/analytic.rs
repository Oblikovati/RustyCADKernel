// Created on: 1999-09-21
// Created by: Edward AGAPOV
// Copyright (c) 1999-1999 Matra Datavision
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
// Rust translation of IsExtremum and the analytical part of Extrema_ExtPRevS::Perform
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The analytical case of Extrema_ExtPRevS: the point is rotated into the plane of
//! the basis conic (angle U), then into the opposite meridian (U + PI); each
//! extremum to the conic gives one on the surface, a parameter outside the V range
//! being brought back into it (by period for a circle or an ellipse) and kept only
//! when it is a local extremum there.
use super::{ExtPRevS, is_original_pnt, perform_ext_p_el_c};
use crate::extrema::ExtPElC;
use crate::extrema::pon::POnSurf;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{self, Axis1, Point3, Transform3, Vector3};
use std::f64::consts::PI;

// `IsExtremum`: the surface point at `(u, v)` and its square distance to `point`,
// and whether it is below (a minimum) or above (a maximum) the distances at
// `u + 1`, `u - 1` and one unit of V inside the range.
fn is_extremum(
    (u, v): (f64, f64),
    point: Point3,
    surface: &dyn Adaptor3dSurface,
    is_v_sup: bool,
    is_min: bool,
) -> Result<(bool, Point3, f64), Failure> {
    let e: Point3 = surface.value(u, v)?;
    let dist2: f64 = point.square_distance(&e);
    let d = |su: f64, sv: f64| -> Result<f64, Failure> { Ok(point.square_distance(&surface.value(su, sv)?)) };
    let inside_v: f64 = if is_v_sup { v - 1. } else { v + 1. };
    let extremum: bool = if is_min {
        dist2 < d(u + 1., v)? && dist2 < d(u - 1., v)? && dist2 < d(u, inside_v)?
    } else {
        dist2 > d(u + 1., v)? && dist2 > d(u - 1., v)? && dist2 > d(u, inside_v)?
    };
    Ok((extremum, e, dist2))
}

impl ExtPRevS {
    // The V parameter above the range brought back: to the upper bound, or for a
    // circle or an ellipse into `[vinf, vinf + 2 PI)` then clamped.
    fn above_range(&self, v: f64, periodic: bool) -> f64 {
        if !periodic {
            return self.vsup;
        }
        let mut new_v: f64 = elclib::in_period(v, self.vinf, self.vinf + 2. * PI);
        if new_v > self.vsup {
            new_v -= 2. * PI;
            if new_v + self.tolv < self.vinf {
                new_v = self.vsup;
            } else if new_v < self.vinf {
                new_v = self.vinf;
            }
        }
        new_v
    }
    // The V parameter below the range brought back.
    fn below_range(&self, v: f64, periodic: bool) -> f64 {
        if !periodic {
            return self.vinf;
        }
        let mut new_v: f64 = elclib::in_period(v, self.vsup - 2. * PI, self.vsup);
        if new_v < self.vinf {
            new_v += 2. * PI;
            if new_v - self.tolv > self.vsup {
                new_v = self.vinf;
            } else if new_v > self.vsup {
                new_v = self.vsup;
            }
        }
        new_v
    }

    // The extrema of one meridian: `an_ext` computed at the angle `u`.
    fn treat_meridian(&mut self, an_ext: &ExtPElC, u: f64, point: Point3, curve: &dyn Adaptor3dCurve) -> Result<(), Failure> {
        let surface_handle = self.s.clone();
        let surface: &dyn Adaptor3dSurface = Self::adaptor(&surface_handle)?;
        for i in 1..=an_ext.nb_ext()? {
            let poc = an_ext.point(i)?;
            let mut v: f64 = poc.parameter();
            let (e, dist2): (Point3, f64);
            if v > self.vsup || v < self.vinf {
                let is_v_sup: bool = v > self.vsup;
                let periodic: bool = matches!(curve.get_type()?, CurveType::Circle | CurveType::Ellipse);
                v = if is_v_sup { self.above_range(v, periodic) } else { self.below_range(v, periodic) };
                let (extremum, point_e, d2) = is_extremum((u, v), point, surface, is_v_sup, an_ext.is_min(i)?)?;
                if !extremum {
                    continue;
                }
                (e, dist2) = (point_e, d2);
            } else {
                e = surface.value(u, v)?;
                dist2 = point.square_distance(&e);
            }
            if is_original_pnt(&e, &self.points[..self.nb_ext as usize]) {
                let index: usize = self.nb_ext as usize;
                self.points[index] = POnSurf::new(u, v, e);
                self.sq_dist[index] = dist2;
                self.nb_ext += 1;
            }
        }
        Ok(())
    }

    // The angle of the point about the axis from the plane of the conic.
    fn point_angle(&self, point: Point3, axis: &Axis1) -> Result<Option<f64>, Failure> {
        let dir: Vector3 = Vector3::from(axis.direction());
        let z: Vector3 = Vector3::from(self.position.direction());
        let o: Point3 = axis.location();
        let op_dir: f64 = Vector3::between(o, point).dot(dir);
        let pp: Point3 = point.translated(dir * -op_dir);
        if o.is_equal(&pp, precision::confusion()) {
            // The point is on the axis of revolution.
            return Ok(None);
        }
        let op_pz: f64 = Vector3::between(o, pp).dot(z);
        let (ppp, mut u): (Point3, f64) = if op_pz.abs() <= gp::resolution() {
            (pp, 0.)
        } else {
            let ppp: Point3 = pp.translated(z * -op_pz);
            if o.is_equal(&ppp, precision::confusion()) {
                (ppp, PI / 2.)
            } else {
                (ppp, Vector3::between(o, ppp).angle_with_ref(Vector3::between(o, pp), dir)?)
            }
        };
        let adaptor = Self::adaptor(&self.s)?;
        let opp: Vector3 = Vector3::between(o, ppp);
        let mut opq: Vector3 = Vector3::between(o, adaptor.value(PI / 2., 0.)?);
        #[allow(clippy::float_cmp)]
        if u != PI / 2. {
            if opq.magnitude().abs() <= gp::resolution() {
                let last: f64 = adaptor.basis_curve()?.last_parameter()?;
                opq = Vector3::between(o, adaptor.value(PI / 2., last / 10.)?);
            }
            if opp.angle_with_ref(opq, dir)? < 0. {
                u += PI;
            }
        }
        Ok(Some(u))
    }

    /// The analytical Perform.
    pub(super) fn perform_analytic(&mut self, point: Point3) -> Result<(), Failure> {
        let adaptor = Self::adaptor(&self.s)?;
        let curve_handle = adaptor.basis_curve()?;
        let axis: Axis1 = adaptor.axe_of_revolution()?;
        let Some(mut u) = self.point_angle(point, &axis)? else {
            return Ok(());
        };
        let curve: &dyn Adaptor3dCurve = &*curve_handle;
        let mut t: Transform3 = Transform3::default();
        t.set_rotation(axis, -u)?;
        let mut p1: Point3 = point.transformed(t);
        let mut an_ext = ExtPElC::new();
        perform_ext_p_el_c(&mut an_ext, p1, curve, self.tolv)?;
        if an_ext.is_done() {
            self.done = true;
            self.treat_meridian(&an_ext, u, point, curve)?;
        }
        t.set_rotation(axis, PI)?;
        p1.transform(t);
        perform_ext_p_el_c(&mut an_ext, p1, curve, self.tolv)?;
        if an_ext.is_done() {
            self.done = true;
            u += PI;
            self.treat_meridian(&an_ext, u, point, curve)?;
        }
        Ok(())
    }
}
