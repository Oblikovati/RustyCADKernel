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
// Rust translation of GeomFill_GordonUtilities.pxx (ScalarLaw, reparametrizeExactly,
// pushConstraint, samplingParameters) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exact reparametrization of a network curve by a piecewise-linear law.
use super::gordon_params::{
    add_interval_midpoints, append_unit_parameter, compact_parameters, interpolation_degree,
    null_curve,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::bsplclib::{self, EvaluatorFunction, FunctionTarget};
use rustycad_tkmath::gp::{self, Point3};

/// `ScalarLaw`: the strictly increasing piecewise-linear map from target parameters
/// back to source parameters (a `BSplCLib_EvaluatorFunction`).
#[derive(Clone, Debug, Default)]
pub(crate) struct ScalarLaw {
    my_source: Vec<f64>,
    my_target: Vec<f64>,
}

impl ScalarLaw {
    /// `Init(Source, Target, NbValues)`: false for fewer than two values or values that
    /// do not increase by more than Precision::PConfusion() in both lists.
    pub(crate) fn init(&mut self, the_source: &[f64], the_target: &[f64]) -> bool {
        if the_source.len() < 2 {
            return false;
        }
        self.my_source.clear();
        self.my_target.clear();
        for (source, target) in the_source.iter().zip(the_target) {
            let not_increasing = match (self.my_source.last(), self.my_target.last()) {
                (Some(last_s), Some(last_t)) => {
                    *source <= last_s + precision::p_confusion()
                        || *target <= last_t + precision::p_confusion()
                }
                _ => false,
            };
            if not_increasing {
                return false;
            }
            self.my_source.push(*source);
            self.my_target.push(*target);
        }
        true
    }
    /// `Value(TargetParameter)`: the linear interpolation, clamped to the source ends.
    pub(crate) fn value(&self, the_target_parameter: f64) -> f64 {
        let (Some(first_s), Some(last_s)) = (self.my_source.first(), self.my_source.last()) else {
            return 0.0;
        };
        if the_target_parameter <= self.my_target[0] {
            return *first_s;
        }
        if the_target_parameter >= *self.my_target.last().unwrap_or(&0.0) {
            return *last_s;
        }
        for i in 1..self.my_target.len() {
            if the_target_parameter <= self.my_target[i] {
                let (a_t0, a_t1) = (self.my_target[i - 1], self.my_target[i]);
                let a_weight: f64 = (the_target_parameter - a_t0) / (a_t1 - a_t0);
                return self.my_source[i - 1]
                    + a_weight * (self.my_source[i] - self.my_source[i - 1]);
            }
        }
        *last_s
    }
    // The slope of the segment holding `t` (the last one beyond it).
    fn slope(&self, t: f64) -> f64 {
        for i in 1..self.my_target.len() {
            if t <= self.my_target[i] {
                return (self.my_source[i] - self.my_source[i - 1])
                    / (self.my_target[i] - self.my_target[i - 1]);
            }
        }
        let n: usize = self.my_source.len();
        (self.my_source[n - 1] - self.my_source[n - 2])
            / (self.my_target[n - 1] - self.my_target[n - 2])
    }
}

impl EvaluatorFunction for ScalarLaw {
    fn evaluate(
        &self,
        derivative_request: i32,
        _start_end: &[f64; 2],
        parameter: f64,
        result: &mut f64,
        error_code: &mut i32,
    ) {
        *error_code = 0;
        *result = match derivative_request {
            0 => self.value(parameter),
            1 => self.slope(parameter),
            _ => 0.0,
        };
    }
}

// The knots of the reparametrized curve: each constraint before a source knot becomes
// a knot of multiplicity Degree, each source knot is mapped by the constraint law, an
// interior constraint on a source knot raises its multiplicity to the degree.
fn mapped_knots(
    curve: &GeomBSplineCurve,
    (source, target): (&[f64], &[f64]),
) -> Result<(Vec<f64>, Vec<i32>), Failure> {
    let (a_degree, nb) = (curve.degree(), source.len());
    let (knots, mults) = (curve.knots(), curve.multiplicities());
    let (mut values, mut new_mults): (Vec<f64>, Vec<i32>) = (Vec::new(), Vec::new());
    let mut ci: usize = 0;
    for k in knots.lower()..=knots.upper() {
        let a_source_knot: f64 = knots[k];
        while ci < nb && source[ci] < a_source_knot - precision::p_confusion() {
            values.push(target[ci]);
            new_mults.push(a_degree);
            ci += 1;
        }
        let mut a_target_knot: f64 = target[nb - 1];
        for i in 0..nb - 1 {
            if a_source_knot <= source[i + 1] + precision::p_confusion() {
                a_target_knot = target[i]
                    + (a_source_knot - source[i]) * (target[i + 1] - target[i])
                        / (source[i + 1] - source[i]);
                break;
            }
        }
        let mut a_multiplicity: i32 = mults[k];
        if ci < nb && (source[ci] - a_source_knot).abs() <= precision::p_confusion() {
            if ci > 0 && ci + 1 < nb {
                a_multiplicity = a_multiplicity.max(a_degree);
            }
            ci += 1;
        }
        values.push(a_target_knot);
        new_mults.push(a_multiplicity);
    }
    Ok((values, new_mults))
}

// `NCollection_Array1` views of the new knots (1-based).
fn one_based<T: Copy + Default>(values: &[T]) -> Result<Array1<'static, T>, Failure> {
    let mut array: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32)
        .map_err(|e| e.into_failure("GeomFill_Gordon"))?;
    for (i, v) in values.iter().enumerate() {
        array[i as i32 + 1] = *v;
    }
    Ok(array)
}

/// `reparametrizeExactly`: the curve rebuilt on the mapped knots by
/// BSplCLib::FunctionReparameterise of the law (homogeneously for a rational curve);
/// None when there are fewer than two constraints, the resampling fails or a new
/// weight vanishes.
pub(crate) fn reparametrize_exactly(
    the_curve: &Handle<GeomBSplineCurve>,
    constraints: (&[f64], &[f64]),
    the_law: &ScalarLaw,
) -> Result<Option<Handle<GeomBSplineCurve>>, Failure> {
    if constraints.0.len() < 2 {
        return Ok(None);
    }
    let curve = the_curve.get().ok_or_else(null_curve)?;
    let a_degree: i32 = curve.degree();
    let (values, mults) = mapped_knots(curve, constraints)?;
    let (a_new_knots, a_new_mults) = (one_based(&values)?, one_based(&mults)?);
    let a_nb_new_poles: i32 = bsplclib::nb_poles(a_degree, false, &a_new_mults)?;
    let mut a_source_flat: Array1<'static, f64> =
        one_based(&vec![0.0; (curve.nb_poles() + a_degree + 1) as usize])?;
    let mut a_target_flat: Array1<'static, f64> =
        one_based(&vec![0.0; (a_nb_new_poles + a_degree + 1) as usize])?;
    bsplclib::knot_sequence(
        &curve.knots(),
        &curve.multiplicities(),
        &mut a_source_flat,
        false,
    )?;
    bsplclib::knot_sequence(&a_new_knots, &a_new_mults, &mut a_target_flat, false)?;
    let target = FunctionTarget {
        bspline_degree: a_degree,
        bspline_flat_knots: &a_source_flat,
        flat_knots: &a_target_flat,
        new_degree: a_degree,
    };
    let built = if curve.is_rational() {
        rational_poles(curve, the_law, &target, a_nb_new_poles)?.map(|(poles, weights)| {
            GeomBSplineCurve::with_weights(
                &poles,
                &weights,
                &a_new_knots,
                &a_new_mults,
                a_degree,
                false,
            )
        })
    } else {
        polynomial_poles(curve, the_law, &target, a_nb_new_poles)?
            .map(|poles| GeomBSplineCurve::new(&poles, &a_new_knots, &a_new_mults, a_degree, false))
    };
    match built {
        None => Ok(None),
        Some(curve) => Ok(Some(Handle::new(curve?))),
    }
}

// The reparametrized poles of a polynomial curve (None on a resampling failure).
fn polynomial_poles(
    curve: &GeomBSplineCurve,
    the_law: &ScalarLaw,
    target: &FunctionTarget<'_, '_>,
    nb_new: i32,
) -> Result<Option<Array1<'static, Point3>>, Failure> {
    let mut a_new_poles: Array1<'static, Point3> =
        one_based(&vec![Point3::default(); nb_new as usize])?;
    let mut a_status: i32 = 0;
    bsplclib::function_reparameterise(
        the_law,
        target,
        &curve.poles(),
        &mut a_new_poles,
        &mut a_status,
    )?;
    Ok((a_status == 0).then_some(a_new_poles))
}

/// Poles with their weights.
type WeightedPoles = (Array1<'static, Point3>, Array1<'static, f64>);

// The reparametrized poles and weights of a rational curve, resampled as homogeneous
// 4D poles (None on a resampling failure or a new weight within gp::Resolution()).
fn rational_poles(
    curve: &GeomBSplineCurve,
    the_law: &ScalarLaw,
    target: &FunctionTarget<'_, '_>,
    nb_new: i32,
) -> Result<Option<WeightedPoles>, Failure> {
    let poles = curve.poles();
    let weights = curve.weights_array(|w| w.clone());
    let mut homogeneous: Vec<f64> = Vec::with_capacity(4 * poles.len());
    for i in poles.lower()..=poles.upper() {
        let (p, w) = (poles[i], weights[i]);
        homogeneous.extend([p.x() * w, p.y() * w, p.z() * w, w]);
    }
    let mut new_homogeneous: Vec<f64> = vec![0.0; 4 * nb_new as usize];
    let mut a_status: i32 = 0;
    bsplclib::function_reparameterise_flat(
        the_law,
        target,
        4,
        &homogeneous,
        &mut new_homogeneous,
        &mut a_status,
    )?;
    if a_status != 0 {
        return Ok(None);
    }
    let mut a_new_poles: Array1<'static, Point3> =
        one_based(&vec![Point3::default(); nb_new as usize])?;
    let mut a_new_weights: Array1<'static, f64> = one_based(&vec![0.0; nb_new as usize])?;
    for (i, h) in new_homogeneous.chunks(4).enumerate() {
        let a_weight: f64 = h[3];
        if a_weight <= gp::resolution() {
            return Ok(None);
        }
        a_new_poles[i as i32 + 1] = Point3::new(h[0] / a_weight, h[1] / a_weight, h[2] / a_weight);
        a_new_weights[i as i32 + 1] = a_weight;
    }
    Ok(Some((a_new_poles, a_new_weights)))
}

/// `pushConstraint`: a source/target pair appended unless it is within the
/// tolerances of the previous one; the target is clamped to `[0, 1]`.
pub(crate) fn push_constraint(
    (the_source, the_target): (&mut Vec<f64>, &mut Vec<f64>),
    (the_source_param, the_target_param): (f64, f64),
    (the_source_tol, the_target_tol): (f64, f64),
) {
    let repeated = match (the_source.last(), the_target.last()) {
        (Some(s), Some(t)) => {
            (the_source_param - s).abs() <= the_source_tol
                || (the_target_param - t).abs() <= the_target_tol
        }
        _ => false,
    };
    if repeated {
        return;
    }
    the_source.push(the_source_param);
    the_target.push(0.0_f64.max(1.0_f64.min(the_target_param)));
}

/// `samplingParameters`: 0, 1 and the targets, compacted, then refined by interval
/// midpoints up to max(degree + 1, 2 * poles, targets + 2) samples.
pub(crate) fn sampling_parameters(the_targets: &[f64], the_nb_source_poles: usize) -> Vec<f64> {
    let mut a_samples: Vec<f64> = Vec::new();
    append_unit_parameter(&mut a_samples, 0.0);
    append_unit_parameter(&mut a_samples, 1.0);
    for target in the_targets {
        append_unit_parameter(&mut a_samples, *target);
    }
    compact_parameters(&mut a_samples);
    // The rebuilt curve is parameter-driven; extra samples are deterministic.
    let minimum: usize = (interpolation_degree() as usize + 1)
        .max((the_nb_source_poles * 2).max(the_targets.len() + 2));
    add_interval_midpoints(&mut a_samples, minimum);
    a_samples
}
