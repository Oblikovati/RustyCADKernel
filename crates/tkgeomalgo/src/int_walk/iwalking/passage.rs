// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of TestArretPassage (closed line) and TestPassedSolutionWithNegativeState of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The stop test of a closed line (in parameters normalized by the estimated section
//! ranges) and the test of an open line on the path points already processed.
use super::perform::Multiples;
use super::points::evaluate_at;
use super::{IWalkingTypes, IntWalkIWalking};
use crate::int_walk::IntWalkWalkingData;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::math::Vector;

/// The normalized quantities of the closed-line test: the previous and current
/// points, the tolerances and the squared step between the points.
struct Normalized {
    up: f64,
    vp: f64,
    uv1: f64,
    uv2: f64,
    tolu: f64,
    tolv: f64,
    deltau: f64,
    deltav: f64,
    d_previous_current: f64,
}

impl Normalized {
    /// The crossing verdict for interior point `k` at the normalized `(utest, vtest)`:
    /// `Some(true)` arrives, `Some(false)` marks the point, `None` leaves it.
    fn loop_verdict(&self, is_start: bool, (utest, vtest): (f64, f64)) -> Option<bool> {
        let (tolu2, tolv2) = (self.tolu + self.tolu, self.tolv + self.tolv);
        let (uv1m, uv2m) = (self.uv1 - utest, self.uv2 - vtest);
        if (uv1m < tolu2 && uv1m > -tolu2) && (uv2m < tolv2 && uv2m > -tolv2) {
            return Some(is_start);
        }
        let (upm, vpm) = (self.up - utest, self.vp - vtest);
        let d_previous_start: f64 = (upm) * (upm) + (vpm) * (vpm);
        let d_current_start: f64 = uv1m * uv1m + uv2m * uv2m;
        let scal: f64 = (upm) * (uv1m) + (vpm) * (uv2m);
        if upm.abs() < self.tolu && vpm.abs() < self.tolv {
            return if is_start { None } else { Some(false) };
        }
        if scal < 0.0 && (d_previous_start + d_current_start < self.d_previous_current) {
            return Some(is_start);
        }
        if is_start {
            return None;
        }
        self.near_verdict(d_previous_start, d_current_start, (utest, vtest))
    }
    /// The last tests on another interior point: near the previous point, near the
    /// current one, or near the middle of the step.
    fn near_verdict(
        &self,
        d_previous_start: f64,
        d_current_start: f64,
        (utest, vtest): (f64, f64),
    ) -> Option<bool> {
        if d_previous_start < self.d_previous_current * 0.25 {
            return Some(false);
        }
        if d_current_start < self.d_previous_current * 0.25 {
            return Some(false);
        }
        let u_mid: f64 = 0.5 * (self.uv1 + self.up) - utest;
        let v_mid: f64 = 0.5 * (self.uv2 + self.vp) - vtest;
        let d_middle_start: f64 = u_mid * u_mid + v_mid * v_mid;
        (d_middle_start < self.d_previous_current * 0.5).then_some(false)
    }
    /// Whether the step passes by (or ends near) the normalized point `(utest, vtest)`.
    fn crosses(&self, (utest, vtest): (f64, f64)) -> bool {
        ((self.up - utest) * (self.uv1 - utest) + (self.vp - vtest) * (self.uv2 - vtest) < 0.0)
            || ((self.uv1 - utest).abs() < self.tolu && (self.uv2 - vtest).abs() < self.tolv)
    }
}

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// `TestArretPassage` of a closed line started on interior point `index`: marks
    /// the interior points the line passes by, detects its return to its start, and
    /// sets `irang` to the last passing path point crossed (0 when none).
    ///
    /// The parameters are divided by the estimated section ranges (at least 1 unless
    /// the parametrization is highly anisotropic).
    pub(super) fn test_arret_passage_closed(
        &mut self,
        mult: &Multiples,
        uv: &Vector<'_>,
        index: i32,
        irang: &mut i32,
    ) -> Result<bool, Failure> {
        let norm: Normalized = self.normalized(uv);
        let mut arrive: bool = false;
        for k in 1..self.wd2.len() {
            let data: IntWalkWalkingData = self.wd2[k];
            if data.etat <= 0 {
                continue;
            }
            let test: (f64, f64) = (data.ustart / norm.deltau, data.vstart / norm.deltav);
            match norm.loop_verdict(index == k as i32, test) {
                Some(true) => arrive = true,
                Some(false) => self.wd2[k].etat = -self.wd2[k].etat,
                None => {}
            }
        }
        *irang = 0;
        for i in 1..self.wd1.len() {
            let data: IntWalkWalkingData = self.wd1[i];
            if data.etat <= 0 || data.etat >= 11 {
                continue;
            }
            let crossed: bool =
                norm.crosses((data.ustart / norm.deltau, data.vstart / norm.deltav));
            if crossed
                || (self.nb_multiplicities[i] > 0
                    && self.normalized_multiple_crossed(mult, i, &norm)?)
            {
                *irang = i as i32;
            }
        }
        Ok(arrive)
    }
    /// The normalized previous and current points (`TestArretPassage`, closed line).
    fn normalized(&self, uv: &Vector<'_>) -> Normalized {
        let (up, vp) = self.surface_parameters(&self.previous_point);
        let (tolu, tolv) = (self.tolerance[1], self.tolerance[2]);
        // A normalizing factor below 1 would expand the range, so it is limited, unless
        // the parametrization is highly anisotropic.
        let anisotropic: bool = std_max(tolu, tolv) > 1000. * std_min(tolu, tolv);
        let delta = |range: &rustycad_tkmath::bnd::Range1, whole: f64| -> f64 {
            if range.is_void() {
                whole
            } else if anisotropic {
                range.delta()
            } else {
                std_max(range.delta(), 1.0)
            }
        };
        let deltau: f64 = delta(&self.my_s_range_u, self.um_last - self.um);
        let deltav: f64 = delta(&self.my_s_range_v, self.vm_last - self.vm);
        let (up, uv1, vp, uv2) = (up / deltau, uv[1] / deltau, vp / deltav, uv[2] / deltav);
        let (tolu, tolv) = (tolu / deltau, tolv / deltav);
        let d_previous_current: f64 = (up - uv1) * (up - uv1) + (vp - uv2) * (vp - uv2);
        Normalized {
            up,
            vp,
            uv1,
            uv2,
            tolu,
            tolv,
            deltau,
            deltav,
            d_previous_current,
        }
    }
    /// Whether the step passes by one of the (normalized) multiplicity points of
    /// path point `i`.
    fn normalized_multiple_crossed(
        &self,
        mult: &Multiples,
        i: usize,
        norm: &Normalized,
    ) -> Result<bool, Failure> {
        let first: i32 = self.nb_multiplicities[1..i].iter().sum();
        for j in first + 1..=first + self.nb_multiplicities[i] {
            let (umj, vmj) = mult.at(j)?;
            if norm.crosses((umj / norm.deltau, vmj / norm.deltav)) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// `TestPassedSolutionWithNegativeState` (NIZHNY-MKK 2000): whether the step of an open
/// line from `(prev_up, prev_vp)` to `uv` passes by a path point already processed
/// (state below -11) or one of its multiplicity points; on a stop, `uv` becomes that
/// point, `irang` its index, and the function is evaluated there.
pub(super) fn test_passed_solution_with_negative_state<F: super::TheIWFunction>(
    wd: &[IntWalkWalkingData],
    mult: &Multiples,
    (prev_up, prev_vp): (f64, f64),
    (nb_multiplicities, tolerance): (&[i32], &Vector<'_>),
    sp: &mut F,
    uv: &mut Vector<'_>,
    irang: &mut i32,
) -> Result<bool, Failure> {
    let (tolu, tolv) = (tolerance[1], tolerance[2]);
    for i in 1..wd.len() {
        let data: IntWalkWalkingData = wd[i];
        if data.etat >= -11 {
            continue;
        }
        let (dup, dvp) = (prev_up - data.ustart, prev_vp - data.vstart);
        if !(dup.abs() >= tolu || dvp.abs() >= tolv) {
            continue;
        }
        let (uv1m, uv2m) = (uv[1] - data.ustart, uv[2] - data.vstart);
        let crossed: bool =
            ((dup * uv1m + dvp * uv2m) < 0.0) || (uv1m.abs() < tolu && uv2m.abs() < tolv);
        let arrive: bool = crossed
            || (nb_multiplicities[i] > 0
                && multiple_passed(
                    mult,
                    (nb_multiplicities, i),
                    (prev_up, prev_vp),
                    uv,
                    (tolu, tolv),
                )?);
        if arrive {
            *irang = i as i32;
            (uv[1], uv[2]) = (data.ustart, data.vstart);
            evaluate_at(sp, uv)?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether the step from the previous point to `uv` passes by (or ends on) one of the
/// multiplicity points of path point `i`.
pub(super) fn multiple_passed(
    mult: &Multiples,
    (nb_multiplicities, i): (&[i32], usize),
    (prev_up, prev_vp): (f64, f64),
    uv: &Vector<'_>,
    (tolu, tolv): (f64, f64),
) -> Result<bool, Failure> {
    let first: i32 = nb_multiplicities[1..i].iter().sum();
    for j in first + 1..=first + nb_multiplicities[i] {
        let (umj, vmj) = mult.at(j)?;
        if ((prev_up - umj) * (uv[1] - umj) + (prev_vp - vmj) * (uv[2] - vmj) < 0.0)
            || ((uv[1] - umj).abs() < tolu && (uv[2] - vmj).abs() < tolv)
        {
            return Ok(true);
        }
    }
    Ok(false)
}
