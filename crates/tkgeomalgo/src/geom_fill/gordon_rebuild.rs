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
// Rust translation of GeomFill_GordonUtilities.pxx (rebuildCurve, equalizeParameters)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reconstruction of the network curves on shared averaged parameters.
use super::gordon_contacts::curve_bounds;
use super::gordon_params::{ParamTable, interpolation_degree, null_curve, parameter_resolution};
use super::gordon_reparam::{
    ScalarLaw, push_constraint, reparametrize_exactly, sampling_parameters,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::geom_lib::Interpolate as GeomLibInterpolate;
use rustycad_tkmath::gp::Point3;

/// The approximation state of an equalization: whether a rational curve may be (and
/// was) rebuilt approximately, and the largest measured rebuild deviation.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RebuildReport {
    pub(crate) allow_approximate_rational: bool,
    pub(crate) did_approximate: bool,
    pub(crate) max_deviation: f64,
}

// The constraints of a rebuild: the curve ends mapped to 0 and 1 and every interior
// contact (away from the ends and from 0 and 1) mapped to its target.
fn rebuild_constraints(
    (first, last): (f64, f64),
    the_source_params: &[f64],
    the_target_params: &[f64],
) -> (Vec<f64>, Vec<f64>) {
    let (mut source, mut target) = (Vec::new(), Vec::new());
    let tols = (
        parameter_resolution(first, last),
        parameter_resolution(0.0, 1.0),
    );
    push_constraint((&mut source, &mut target), (first, 0.0), tols);
    for (s, t) in the_source_params.iter().zip(the_target_params) {
        if (s - first).abs() <= tols.0
            || (s - last).abs() <= tols.0
            || t.abs() <= tols.1
            || (t - 1.0).abs() <= tols.1
        {
            continue;
        }
        push_constraint((&mut source, &mut target), (*s, *t), tols);
    }
    push_constraint((&mut source, &mut target), (last, 1.0), tols);
    (source, target)
}

// The samples of the law on the curve (a point within Precision::Confusion() of the
// previous one dropped), as 1-based points and parameters.
fn law_samples(
    adaptor: &GeomAdaptorCurve,
    law: &ScalarLaw,
    samples: &[f64],
) -> Result<(Array1<'static, Point3>, Array1<'static, f64>), Failure> {
    let (mut points, mut parameters): (Vec<Point3>, Vec<f64>) = (Vec::new(), Vec::new());
    for a_target_param in samples {
        let a_point: Point3 = adaptor.eval_d0(law.value(*a_target_param))?;
        if points
            .last()
            .is_some_and(|last: &Point3| a_point.distance(last) <= precision::confusion())
        {
            continue;
        }
        points.push(a_point);
        parameters.push(*a_target_param);
    }
    let mut point_array: Array1<'static, Point3> = Array1::with_bounds(1, points.len() as i32)
        .map_err(|e| e.into_failure("GeomFill_Gordon"))?;
    let mut param_array: Array1<'static, f64> = Array1::with_bounds(1, points.len() as i32)
        .map_err(|e| e.into_failure("GeomFill_Gordon"))?;
    for (i, (p, t)) in points.iter().zip(&parameters).enumerate() {
        point_array[i as i32 + 1] = *p;
        param_array[i as i32 + 1] = *t;
    }
    Ok((point_array, param_array))
}

/// `rebuildCurve`: the curve reparametrized exactly by the constraint law; otherwise
/// (a rational curve only when allowed, which flags the result approximate) a cubic
/// interpolation of law samples, whose deviation from the source is measured. False
/// when the law, the samples or the interpolation fail.
pub(crate) fn rebuild_curve(
    the_curve: &mut Handle<GeomBSplineCurve>,
    (the_source_params, the_target_params): (&[f64], &[f64]),
    report: &mut RebuildReport,
) -> Result<bool, Failure> {
    let bounds = curve_bounds(the_curve)?;
    let (source, target) = rebuild_constraints(bounds, the_source_params, the_target_params);
    let mut a_law = ScalarLaw::default();
    if !a_law.init(&source, &target) {
        return Ok(false);
    }
    if let Some(exact) = reparametrize_exactly(the_curve, (&source, &target), &a_law)? {
        *the_curve = exact;
        return Ok(true);
    }
    let curve = the_curve.get().ok_or_else(null_curve)?;
    if curve.is_rational() {
        if !report.allow_approximate_rational {
            return Ok(false);
        }
        report.did_approximate = true;
    }
    let a_curve_adaptor = GeomAdaptorCurve::with_curve(&the_curve.cast())?;
    let a_samples: Vec<f64> = sampling_parameters(the_target_params, curve.nb_poles() as usize);
    let (points, parameters) = law_samples(&a_curve_adaptor, &a_law, &a_samples)?;
    let a_nb_points: i32 = points.len() as i32;
    if a_nb_points < interpolation_degree() {
        return Ok(false);
    }
    let an_interpolator =
        GeomLibInterpolate::new(interpolation_degree(), a_nb_points, &points, &parameters)?;
    if !an_interpolator.is_done() {
        return Ok(false);
    }
    *the_curve = an_interpolator.curve();
    if the_curve.is_null() {
        return Ok(false);
    }
    let a_new_curve_adaptor = GeomAdaptorCurve::with_curve(&the_curve.cast())?;
    for a_target_param in &a_samples {
        let a_source_point: Point3 = a_curve_adaptor.eval_d0(a_law.value(*a_target_param))?;
        let a_new_point: Point3 = a_new_curve_adaptor.eval_d0(*a_target_param)?;
        report.max_deviation = report
            .max_deviation
            .max(a_source_point.distance(&a_new_point));
    }
    Ok(true)
}

// Reads the old contact `(curve, k)` of a family.
type ReadContact<'a> = &'a dyn Fn(usize, usize) -> f64;
// Writes the new contact `(curve, k)` of a family.
type WriteContact<'a> = &'a mut dyn FnMut(usize, usize, f64);

// One family equalized: each curve whose contacts are not all within
// Precision::PConfusion() of the targets is rebuilt and its contacts set to them.
fn equalize_family(
    curves: &mut [Handle<GeomBSplineCurve>],
    targets: &[f64],
    (read, write): (ReadContact<'_>, WriteContact<'_>),
    report: &mut RebuildReport,
) -> Result<bool, Failure> {
    for (curve_idx, curve) in curves.iter_mut().enumerate() {
        let old: Vec<f64> = (0..targets.len()).map(|k| read(curve_idx, k)).collect();
        let is_already_target = old
            .iter()
            .zip(targets)
            .all(|(o, t)| (o - t).abs() <= precision::p_confusion());
        if is_already_target {
            continue;
        }
        if !rebuild_curve(curve, (&old, targets), report)? {
            return Ok(false);
        }
        for (k, t) in targets.iter().enumerate() {
            write(curve_idx, k, *t);
        }
    }
    Ok(true)
}

/// `equalizeParameters`: every profile rebuilt so that its contacts sit at the mean
/// profile parameter of each guide column, then every guide so that its contacts sit
/// at the mean guide parameter of each profile row (the means are taken before any
/// rebuild).
pub(crate) fn equalize_parameters(
    (the_profiles, the_guides): (
        &mut [Handle<GeomBSplineCurve>],
        &mut [Handle<GeomBSplineCurve>],
    ),
    (the_profile_params, the_guide_params): (&mut ParamTable, &mut ParamTable),
    report: &mut RebuildReport,
) -> Result<bool, Failure> {
    let a_profile_targets: Vec<f64> = the_profile_params.column_means();
    let a_guide_targets: Vec<f64> = the_guide_params.row_means();
    let snapshot = the_profile_params.clone();
    let read = |p: usize, g: usize| snapshot.at(p, g);
    let mut write = |p: usize, g: usize, v: f64| the_profile_params.set(p, g, v);
    if !equalize_family(
        the_profiles,
        &a_profile_targets,
        (&read, &mut write),
        report,
    )? {
        return Ok(false);
    }
    let snapshot = the_guide_params.clone();
    let read = |g: usize, p: usize| snapshot.at(p, g);
    let mut write = |g: usize, p: usize, v: f64| the_guide_params.set(p, g, v);
    equalize_family(the_guides, &a_guide_targets, (&read, &mut write), report)
}
