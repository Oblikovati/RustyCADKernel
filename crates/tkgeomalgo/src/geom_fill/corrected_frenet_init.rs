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
// Rust translation of Init, InitInterval, CalcAngleAT and the static smoothlaw and
// FindPlane of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_CorrectedFrenet.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction of the angle law of `GeomFill_CorrectedFrenet`: the angle
//! between consecutive Frenet normals transported along the curve, sampled with an
//! adaptive step on each Frenet interval and interpolated by a smoothed B-spline law.
use super::corrected_frenet::{AngleHistory, GeomFillCorrectedFrenet};
use super::snglr_func::GeomFillSnglrFunc;
use super::sweep_failures::live;
use super::trihedron_law::{GeomFillTrihedronLaw, TrihedronD0};
use crate::law::{LawBSpFunc, LawBSpline, LawComposite, LawConstant, LawFunction, LawInterpolate};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::bnd_lib::Add3dCurve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

/// The running values of one `InitInterval` call that the caller passes on to the
/// next interval (`startAng`, `prevTangent`, `prevNormal`, `aT`, `aN`) and the step
/// records it appends.
pub(crate) struct IntervalWalk {
    pub start_ang: f64,
    pub prev_tangent: Vector3,
    pub prev_normal: Vector3,
    pub at: Vector3,
    pub an: Vector3,
    pub history: AngleHistory,
}

/// `CalcAngleAT(Tangent, Normal, prevTangent, prevNormal)`: the angle from the
/// previous normal to the current one, once the current frame is turned back onto
/// the previous tangent.
pub(crate) fn calc_angle_at(
    tangent: Vector3,
    normal: Vector3,
    prev_tangent: Vector3,
    prev_normal: Vector3,
) -> Result<f64, Failure> {
    let angle = tangent.angle(prev_tangent)?;
    let normal_rot =
        if angle.abs() > precision::angular() && angle.abs() < PI - precision::angular() {
            let cross = tangent.crossed(prev_tangent).normalized()?;
            normal
                + angle.sin() * cross.crossed(normal)
                + (1.0 - angle.cos()) * cross.crossed(cross.crossed(normal))
        } else {
            normal
        };
    let mut angle_at = normal_rot.angle(prev_normal)?;
    if angle_at > precision::angular()
        && PI - angle_at > precision::angular()
        && normal_rot
            .crossed(prev_normal)
            .is_opposite(prev_tangent, precision::angular())?
    {
        angle_at = -angle_at;
    }
    Ok(angle_at)
}

/// The largest distance between `law` and the samples, and whether it stays within
/// `tol` (the check loop of `smoothlaw`, which stops at the first excess).
fn check_law(
    law: &LawBSpline,
    points: &[f64],
    params: &[f64],
    tol: f64,
) -> Result<(bool, f64), Failure> {
    let (mut max, mut ok) = (0.0, true);
    for (point, param) in points.iter().zip(params) {
        if !ok {
            break;
        }
        let d = (law.value(*param)? - point).abs();
        if d > max {
            max = d;
        }
        ok = max <= tol;
    }
    Ok((ok, max))
}

/// Removes every inner knot within `tol`, from the last one; whether one went.
fn remove_knots(law: &LawBSpline, tol: f64) -> Result<bool, Failure> {
    let mut removed = false;
    for ii in (2..law.nb_knots()).rev() {
        if law.remove_knot(ii, 0, tol)? {
            removed = true;
        }
    }
    Ok(removed)
}

/// `smoothlaw(Law, Points, Param, Tol)`: two passes of knot removal (at `Tol / 10`,
/// then at the remaining margin) kept when the law stays within `Tol` of the
/// samples. The second pass edits the copy that the first pass may already have
/// stored in `law`, as natively.
fn smoothlaw(
    law: &mut Handle<LawBSpline>,
    points: &[f64],
    params: &[f64],
    tol_max: f64,
) -> Result<(), Failure> {
    let bs = live(law, "smoothlaw", "Law")?.copy()?;
    let b = live(&bs, "smoothlaw", "BS")?;
    let mut ok = remove_knots(b, tol_max / 10.0)?;
    let tol = if ok {
        let (within, max) = check_law(b, points, params, tol_max)?;
        ok = within;
        if !ok {
            return Ok(());
        }
        (tol_max - max) / 2.0
    } else {
        tol_max / 2.0
    };
    if ok {
        *law = bs.clone();
    }
    ok = remove_knots(b, tol)?;
    if ok {
        ok = check_law(b, points, params, tol_max)?.0;
    }
    if ok {
        *law = bs;
    }
    Ok(())
}

/// The sample points of `FindPlane`: the poles of a Bezier or B-spline curve of more
/// than two poles, `15 + NbIntervals(C3)` points of another curve, none for a line,
/// a conic (whose plane is known) or a two-pole curve; `None` when no plane exists.
fn plane_samples(c: &dyn Adaptor3dCurve) -> Result<Option<Option<Vec<Point3>>>, Failure> {
    use rustycad_tkmath::geom_abs::CurveType;
    let poles = match c.get_type()? {
        CurveType::Line => return Ok(None),
        CurveType::Circle | CurveType::Ellipse | CurveType::Hyperbola | CurveType::Parabola => {
            return Ok(Some(None));
        }
        CurveType::BezierCurve => c.bezier()?.get().map(|b| b.poles()),
        CurveType::BSplineCurve => c.bspline()?.get().map(|b| b.poles()),
        _ => return Ok(Some(Some(sampled_points(c)?))),
    };
    match poles {
        Some(p) if p.len() > 2 => Ok(Some(Some(p.as_slice().to_vec()))),
        _ => Ok(None),
    }
}

fn sampled_points(c: &dyn Adaptor3dCurve) -> Result<Vec<Point3>, Failure> {
    let nbp = 15 + c.nb_intervals(Shape::C3)?;
    let (f, l) = (c.first_parameter()?, c.last_parameter()?);
    let inv = 1.0 / f64::from(nbp - 1);
    let mut points = Vec::with_capacity(nbp as usize);
    for ii in 1..=nbp {
        let mut t = f * f64::from(nbp - ii) + l * f64::from(ii - 1);
        t *= inv;
        points.push(c.value(t)?);
    }
    Ok(points)
}

/// `FindPlane(C, P)`: whether the curve lies in a plane (within Confusion of the
/// plane of inertia of its samples).
fn find_plane(curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
    let c = live(curve, "FindPlane", "theC")?;
    let samples = match plane_samples(c)? {
        None => return Ok(false),
        Some(None) => return Ok(true),
        Some(Some(samples)) => samples,
    };
    let mut tab_p = one_based(samples.len() as i32)?;
    tab_p.as_mut_slice().copy_from_slice(&samples);
    let inertia = rustycad_tkgeombase::geom_lib::axe_of_inertia(&tab_p, 1.0e-7)?;
    if inertia.is_singular {
        return Ok(false);
    }
    let plane =
        rustycad_tkg3d::geom::GeomPlane::new(rustycad_tkmath::gp::Axis3::from_axis2(inertia.axe));
    let [a, b, cc, d] = plane.coefficients()?;
    Ok(samples.iter().all(|p| {
        let xyz = p.xyz();
        (a * xyz.x() + b * xyz.y() + cc * xyz.z() + d).abs() <= precision::confusion()
    }))
}

impl GeomFillCorrectedFrenet {
    /// `Init()`: builds `EvolAroundT` interval by interval of the Frenet law.
    pub(crate) fn init(&self) -> Result<(), Failure> {
        let composite = Handle::new(LawComposite::new());
        let frenet = self.frenet();
        let nb_i = frenet.nb_intervals(Shape::C0)?;
        let mut t = one_based(nb_i + 1)?;
        frenet.intervals(&mut t, Shape::C0)?;
        let trimmed = self.trihedron_base().trimmed();
        let tc = live(&trimmed, "GeomFill_CorrectedFrenet::Init", "myTrimmed")?;
        let mut frame = TrihedronD0::default();
        frenet.d0(tc.first_parameter()?, &mut frame)?;
        let av_step = (tc.last_parameter()? - tc.first_parameter()?) / 10.0;
        let mut walk = IntervalWalk {
            start_ang: 0.0,
            prev_tangent: frame.tangent,
            prev_normal: frame.normal,
            at: Vector3::default(),
            an: Vector3::default(),
            history: AngleHistory::default(),
        };
        for i in 1..=nb_i {
            let nb_step = (((t[i + 1] - t[i]) / av_step) as i32).max(3);
            let step = (t[i + 1] - t[i]) / f64::from(nb_step);
            let (is_zero, func) = self.init_interval(t[i], t[i + 1], step, &mut walk)?;
            if !is_zero {
                self.state.write().is_frenet = false;
            }
            composite.change_laws(|laws| {
                laws.append(func);
            });
        }
        if tc.is_periodic()? {
            composite.set_periodic();
        }
        self.store_init(composite.cast(), walk);
        Ok(())
    }

    fn store_init(&self, law: Handle<dyn LawFunction>, walk: IntervalWalk) {
        let mut state = self.state.write();
        state.evol_around_t = law.clone();
        state.t_law = law;
        state.at = walk.at;
        state.an = walk.an;
        if !walk.history.poles.is_empty() {
            state.history = Some(walk.history);
        }
    }

    /// `InitInterval(First, Last, Step, ...)`: walks `[first, last]` and returns
    /// whether the angle stays null (`isZero`) and the angle law of the interval.
    fn init_interval(
        &self,
        first: f64,
        last: f64,
        step: f64,
        walk: &mut IntervalWalk,
    ) -> Result<(bool, Handle<dyn LawFunction>), Failure> {
        let frenet = self.frenet();
        frenet.set_interval(first, last)?;
        let curve = self.trihedron_base().curve();
        let cs = GeomFillSnglrFunc::new(&curve);
        let mut bounds = BoundingBox3::default();
        Add3dCurve::add_range(&cs, first, last, 1.0e-2, &mut bounds)?;
        let length_min = bounds.gap() * 1.0e-4;
        walk.at = Vector3::default();
        walk.an = Vector3::default();
        let is_planar = if self.for_evaluation {
            false
        } else {
            find_plane(&curve)?
        };
        let steps = self.walk_interval(&cs, [first, last, step, length_min], is_planar, walk)?;
        let (parameters, evol_at, is_zero, is_const) = steps;
        let n = parameters.len() as i32;
        if !is_planar {
            walk.at /= f64::from(n - 1);
            walk.an /= f64::from(n - 1);
        }
        let angle_at = evol_at.last().copied().unwrap_or(0.0);
        walk.start_ang = angle_at;
        if is_const || is_planar {
            let law = LawConstant::new();
            law.set(angle_at, first, last);
            return Ok((is_zero, Handle::new(law).cast()));
        }
        let func = interpolated_law(&parameters, &evol_at, first, last)?;
        Ok((is_zero, func))
    }
}

/// The smoothed interpolation of the angles at the parameters, as a `Law_BSpFunc`.
fn interpolated_law(
    parameters: &[f64],
    evol_at: &[f64],
    first: f64,
    last: f64,
) -> Result<Handle<dyn LawFunction>, Failure> {
    let n = parameters.len() as i32;
    let mut pararr = one_based(n)?;
    let mut anglearr = one_based(n)?;
    pararr.as_mut_slice().copy_from_slice(parameters);
    anglearr.as_mut_slice().copy_from_slice(evol_at);
    let points = Handle::new(HArray1::from_owned(anglearr));
    let params = Handle::new(HArray1::from_owned(pararr));
    let mut law_at =
        LawInterpolate::with_parameters(&points, &params, false, precision::p_confusion())?;
    law_at.perform()?;
    let mut bs = law_at.curve()?;
    smoothlaw(&mut bs, evol_at, parameters, 0.1)?;
    Ok(Handle::new(LawBSpFunc::with_curve(&bs, first, last)).cast())
}
