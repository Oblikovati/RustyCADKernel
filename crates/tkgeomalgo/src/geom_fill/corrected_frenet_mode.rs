// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of GetAngleAT, EvaluateBestMode, the D2 correction and the static
// ComputeTorsion, corr2PI_PI and diffAng of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_CorrectedFrenet.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The evaluation of the angle law of `GeomFill_CorrectedFrenet` (with the branch
//! correction of OCC78) and the choice of the best trihedron mode.
use super::corrected_frenet::{AngleHistory, GeomFillCorrectedFrenet};
use super::corrected_frenet_init::calc_angle_at;
use super::pipe_error::GeomFillTrihedron;
use super::sweep_failures::{live, null_object};
use super::trihedron_law::{GeomFillTrihedronLaw, TrihedronD0, TrihedronD2};
use crate::law::LawFunction;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{self, Point2, Vector2, Vector3};
use std::f64::consts::PI;

/// `ComputeTorsion(Param, aCurve)`: `(C' ^ C'') . C''' / |C' ^ C''|^2`, 0 when the
/// squared norm is below gp::Resolution().
fn compute_torsion(param: f64, curve: &dyn Adaptor3dCurve) -> Result<f64, Failure> {
    let (_, dc1, dc2, dc3) = curve.d3(param)?;
    let dc1_cross_dc2 = dc1 ^ dc2;
    let norm = dc1_cross_dc2.magnitude();
    let dc1_dc2_dc3 = dc1_cross_dc2 * dc3;
    let square_norm = norm * norm;
    if square_norm <= gp::resolution() {
        return Ok(0.0);
    }
    Ok(dc1_dc2_dc3 / square_norm)
}

/// `corr2PI_PI(Ang)`: an angle of `[0, 2PI[` in `[-PI, PI[`.
fn corr2pi_pi(ang: f64) -> f64 {
    if ang < PI { ang } else { ang - 2.0 * PI }
}

/// `diffAng(A, Ao)`: the difference of two angles in `[-PI, PI]`.
fn diff_ang(a: f64, ao: f64) -> f64 {
    let da = (a - ao) - ((a - ao) / 2.0 / PI).floor() * 2.0 * PI;
    if da >= 0.0 {
        corr2pi_pi(da)
    } else {
        -corr2pi_pi(-da)
    }
}

/// The step `iC` of the history whose interval holds `param`, by bisection, or
/// `None` when `param` is a recorded parameter (the law value is then kept). A
/// parameter below the first record loops forever natively, and here.
fn history_step(poles: &[f64], param: f64) -> Option<usize> {
    let pole = |i: usize| poles[i - 1];
    let (mut ib, mut ie) = (1, poles.len());
    let mut ic = (ie + ib) / 2;
    if param == pole(ib) {
        return None;
    }
    if param > pole(ie) {
        ic = ie;
    }
    if ic < ie {
        while pole(ic) > param || param > pole(ic + 1) {
            if pole(ic) < param {
                ib = ic;
            } else {
                ie = ic;
            }
            ic = (ie + ib) / 2;
        }
        if pole(ic) == param || param == pole(ic + 1) {
            return None;
        }
    }
    Some(ic)
}

impl GeomFillCorrectedFrenet {
    /// `GetAngleAT(Param)`: the angle law at `param`, replaced by the transported
    /// angle from the nearest record when they differ by more than PI/2 (OCC78).
    pub(crate) fn get_angle_at(&self, param: f64) -> Result<f64, Failure> {
        let (t_law, history) = {
            let state = self.state.read();
            (state.t_law.clone(), state.history.clone())
        };
        let law = live(&t_law, "GeomFill_CorrectedFrenet::GetAngleAT", "TLaw")?;
        let history = history
            .ok_or_else(|| null_object("GeomFill_CorrectedFrenet::GetAngleAT", "HArrPoles"))?;
        let Some(ic) = history_step(&history.poles, param) else {
            return law.value(param);
        };
        self.corrected_angle(law, &history, ic, param)
    }

    fn corrected_angle(
        &self,
        law: &dyn LawFunction,
        history: &AngleHistory,
        ic: usize,
        param: f64,
    ) -> Result<f64, Failure> {
        let mut ang_p = law.value(param)?;
        let ang_po = history.angle[ic - 1];
        let d_ang_law = ang_p - ang_po;
        let mut frame = TrihedronD0::default();
        self.frenet().d0(param, &mut frame)?;
        let d_ang = calc_angle_at(
            frame.tangent,
            frame.normal,
            history.tangent[ic - 1],
            history.normal[ic - 1],
        )?;
        if diff_ang(d_ang, d_ang_law).abs() > PI / 2.0 {
            ang_p = ang_po + d_ang;
        }
        Ok(ang_p)
    }

    /// The body of `EvaluateBestMode` once `EvolAroundT` exists.
    pub(crate) fn best_mode_of(
        &self,
        law: &Handle<dyn LawFunction>,
    ) -> Result<GeomFillTrihedron, Failure> {
        let evol = live(
            law,
            "GeomFill_CorrectedFrenet::EvaluateBestMode",
            "EvolAroundT",
        )?;
        let nb_int = evol.nb_intervals(Shape::CN)?;
        let mut int = one_based(nb_int + 1)?;
        evol.intervals(&mut int, Shape::CN)?;
        let trimmed = self.trihedron_base().trimmed();
        let curve = live(
            &trimmed,
            "GeomFill_CorrectedFrenet::EvaluateBestMode",
            "myTrimmed",
        )?;
        let mut walk = AngleWalk {
            old: Point2::default(),
            prev_vec: Vector2::default(),
            k: 1,
        };
        for i in 1..=nb_int {
            let (tmin, tmax) = (int[i], int[i + 1]);
            if compute_torsion(tmin, curve)?.abs() > 100.0 {
                return Ok(GeomFillTrihedron::IsDiscreteTrihedron);
            }
            let trimmed_law = evol.trim(tmin, tmax, precision::p_confusion() / 2.0)?;
            let step = (int[i + 1] - int[i]) / 10.0;
            if walk.turns_too_fast(
                live(&trimmed_law, "EvaluateBestMode", "trimmedlaw")?,
                tmin,
                step,
            )? {
                return Ok(GeomFillTrihedron::IsDiscreteTrihedron);
            }
        }
        Ok(GeomFillTrihedron::IsCorrectedFrenet)
    }
}

/// The samples of the angle law as points `(u, angle)` of the plane and the turn
/// between consecutive chords (`old`, `PrevVec`, `k` of EvaluateBestMode).
struct AngleWalk {
    old: Point2,
    prev_vec: Vector2,
    k: i32,
}

impl AngleWalk {
    /// Whether a chord of the eleven samples of one interval turns by more than
    /// 3PI/4 from the previous chord.
    fn turns_too_fast(
        &mut self,
        law: &dyn LawFunction,
        tmin: f64,
        step: f64,
    ) -> Result<bool, Failure> {
        for j in 0..=10 {
            let u = tmin + f64::from(j) * step;
            let point2d = Point2::new(u, law.value(u)?);
            if j != 0 {
                let a_vec = Vector2::from(point2d.xy() - self.old.xy());
                if self.k > 2 && self.prev_vec.angle(a_vec)?.abs() > 3.0 * PI / 4.0 {
                    return Ok(true);
                }
                self.prev_vec = a_vec;
            }
            self.old = point2d;
            self.k += 1;
        }
        Ok(false)
    }
}

/// The D2 correction of the Frenet frame by the angle and its two derivatives.
pub(crate) fn corrected_d2(frame: &mut TrihedronD2, angle_at: f64, d_angle: f64, d2_angle: f64) {
    let (sina, cosa) = (angle_at.sin(), angle_at.cos());
    let f = *frame;
    let cross = f.tangent.crossed(f.normal);
    let dcross = scaled_sum(f.d_tangent.crossed(f.normal), f.tangent.crossed(f.d_normal));
    let d2cross = three(
        [1.0, 2.0],
        [
            f.d2_tangent.crossed(f.normal),
            f.d_tangent.crossed(f.d_normal),
            f.tangent.crossed(f.d2_normal),
        ],
    );
    let tcross = f.tangent.crossed(cross);
    let dtcross = scaled_sum(f.d_tangent.crossed(cross), f.tangent.crossed(dcross));
    let d2tcross = three(
        [1.0, 2.0],
        [
            f.d2_tangent.crossed(cross),
            f.d_tangent.crossed(dcross),
            f.tangent.crossed(d2cross),
        ],
    );
    let mut aux = Vector3::default();
    aux.set_linear_form_three(
        [
            sina,
            2.0 * cosa * d_angle,
            cosa * d2_angle - sina * d_angle * d_angle,
        ],
        [d2cross, dcross, cross],
    );
    let prev = aux;
    aux.set_linear_form_three_offset(
        [
            1.0 - cosa,
            2.0 * sina * d_angle,
            cosa * d_angle * d_angle + sina * d2_angle,
        ],
        [d2tcross, dtcross, tcross],
        prev,
    );
    frame.d2_normal += aux;
    aux.set_linear_form(sina, dcross, cosa * d_angle, cross);
    let prev = aux;
    aux.set_linear_form_offset(1.0 - cosa, dtcross, sina * d_angle, tcross, prev);
    frame.d_normal += aux;
    let normal = frame.normal;
    frame
        .normal
        .set_linear_form_offset(sina, cross, 1.0 - cosa, tcross, normal);
    frame.binormal = frame.tangent.crossed(frame.normal);
    frame.d_binormal = scaled_sum(
        frame.d_tangent.crossed(frame.normal),
        frame.tangent.crossed(frame.d_normal),
    );
    frame.d2_binormal = three(
        [1.0, 2.0],
        [
            frame.d2_tangent.crossed(frame.normal),
            frame.d_tangent.crossed(frame.d_normal),
            frame.tangent.crossed(frame.d2_normal),
        ],
    );
}

/// `SetLinearForm(1, V1, V2)`.
fn scaled_sum(v1: Vector3, v2: Vector3) -> Vector3 {
    let mut out = Vector3::default();
    out.set_scaled_sum(1.0, v1, v2);
    out
}

/// `SetLinearForm(a, V1, b, V2, V3)` with `[a, b]` and `[V1, V2, V3]`.
fn three(weights: [f64; 2], terms: [Vector3; 3]) -> Vector3 {
    let mut out = Vector3::default();
    out.set_linear_form_offset(weights[0], terms[0], weights[1], terms[1], terms[2]);
    out
}
