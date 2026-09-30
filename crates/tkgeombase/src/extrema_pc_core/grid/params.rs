// Rust translation of BuildUniformParams, BuildCurveAwareParams, IsClosedDomain,
// insertParameter and buildPartition of ExtremaPC2d_GridEvaluator.hxx and
// ExtremaPC_GridEvaluator.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameter partitions of the grid evaluator.
use super::PcGridEvaluator;
use crate::extrema_pc_core::space::{PcPoint, PcSpace};
use crate::extrema_pc_core::validity::{
    THE_OTHER_CURVE_NB_SAMPLES, THE_PARAM_TOLERANCE, is_closed_periodic_domain,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Vector;
use rustycad_tkmath::math_utils::Domain1D;

impl<S: PcSpace> PcGridEvaluator<S> {
    /// A uniform partition of `[u_min, u_max]` with `nb_samples` values, the last one set
    /// to `u_max` exactly (`BuildUniformParams`); Standard_RangeError below 2 samples;
    /// `GridEvaluator::build_uniform_params(0.0, 1.0, 5)?`.
    pub fn build_uniform_params(
        u_min: f64,
        u_max: f64,
        nb_samples: usize,
    ) -> Result<Vector<'static>, Failure> {
        if nb_samples < 2 {
            let message = format!(
                "{}::BuildUniformParams: {nb_samples} samples; expected >= 2",
                S::GRID_EVALUATOR_NAME
            );
            return Err(Failure::with_type("Standard_RangeError", &message, ""));
        }
        let mut params: Vector<'static> = Vector::with_size(nb_samples);
        let step: f64 = (u_max - u_min) / (nb_samples - 1) as f64;
        for index in 0..nb_samples {
            params[index as i32] = u_min + index as f64 * step;
        }
        params[(nb_samples - 1) as i32] = u_max;
        Ok(params)
    }
    /// A deflection-aware partition within each C2-smooth interval of the curve
    /// (`BuildCurveAwareParams`); any failure gives the uniform partition of
    /// THE_OTHER_CURVE_NB_SAMPLES values; `GridEvaluator::build_curve_aware_params(&c, &d)?`.
    pub fn build_curve_aware_params(
        curve: &S::Curve,
        domain: &Domain1D,
    ) -> Result<Vector<'static>, Failure> {
        match curve_aware_parameters::<S>(curve, domain) {
            Ok(parameters) => {
                let mut result: Vector<'static> = Vector::with_size(parameters.len());
                for (index, parameter) in parameters.iter().enumerate() {
                    result[index as i32] = *parameter;
                }
                Ok(result)
            }
            Err(_) => {
                Self::build_uniform_params(domain.min, domain.max, THE_OTHER_CURVE_NB_SAMPLES)
            }
        }
    }
    /// True when the ends of a finite domain are the same curve point: a whole number of
    /// periods, or exactly equal end points (`IsClosedDomain`); false on any failure;
    /// `GridEvaluator::is_closed_domain(&curve, &domain)`.
    pub fn is_closed_domain(curve: &S::Curve, domain: &Domain1D) -> bool {
        if !domain.is_finite(1.0e100) || domain.min == domain.max {
            return false;
        }
        closed_domain::<S>(curve, domain).unwrap_or(false)
    }
    /// `buildPartition`: the cached parameters merged with the C2 bounds; the interior
    /// bounds are also recorded as continuity bounds. An adaptor without interval
    /// support keeps the cached sampling.
    pub(super) fn build_partition(&mut self, curve: &S::Curve, domain: &Domain1D) {
        self.partition.extend_from_slice(&self.params);
        let Ok(bounds) = c2_bounds::<S>(curve) else {
            return;
        };
        for bound in bounds {
            let parameter: f64 = domain.clamp(bound);
            insert_parameter(&mut self.partition, parameter);
            if parameter > domain.min + THE_PARAM_TOLERANCE
                && parameter < domain.max - THE_PARAM_TOLERANCE
            {
                insert_parameter(&mut self.continuity_bounds, parameter);
            }
        }
    }
}

// The body of IsClosedDomain inside its try block.
fn closed_domain<S: PcSpace>(curve: &S::Curve, domain: &Domain1D) -> Result<bool, Failure> {
    if S::is_periodic(curve)?
        && is_closed_periodic_domain(domain, S::period(curve)?, THE_PARAM_TOLERANCE)
    {
        return Ok(true);
    }
    let first = S::value(curve, domain.min)?;
    Ok(first.square_distance_to(&S::value(curve, domain.max)?) == 0.0)
}

/// `NbIntervals(GeomAbs_C2)` then `Intervals`: the C2 bounds of the curve.
fn c2_bounds<S: PcSpace>(curve: &S::Curve) -> Result<Vec<f64>, Failure> {
    let count: i32 = S::nb_c2_intervals(curve)?;
    let mut bounds: Array1<'static, f64> =
        Array1::with_bounds(1, count + 1).map_err(|error| error.into_failure("Intervals"))?;
    S::c2_intervals(curve, &mut bounds)?;
    Ok(bounds.iter().copied().collect())
}

// The body of BuildCurveAwareParams inside its try block.
fn curve_aware_parameters<S: PcSpace>(
    curve: &S::Curve,
    domain: &Domain1D,
) -> Result<Vec<f64>, Failure> {
    let mut parameters: Vec<f64> = Vec::new();
    insert_parameter(&mut parameters, domain.min);
    insert_parameter(&mut parameters, domain.max);
    let bounds: Vec<f64> = c2_bounds::<S>(curve)?;
    for pair in bounds.windows(2) {
        let lower: f64 = domain.clamp(pair[0]);
        let upper: f64 = domain.clamp(pair[1]);
        if upper - lower <= THE_PARAM_TOLERANCE {
            continue;
        }
        insert_parameter(&mut parameters, lower);
        insert_parameter(&mut parameters, upper);
        add_deflection_parameters::<S>(curve, (lower, upper), &mut parameters)?;
    }
    Ok(parameters)
}

// The DeflCurvIntervals parameters of one C2 interval, clamped to it.
fn add_deflection_parameters<S: PcSpace>(
    curve: &S::Curve,
    (lower, upper): (f64, f64),
    parameters: &mut Vec<f64>,
) -> Result<(), Failure> {
    let local = S::trim(curve, lower, upper, precision::p_confusion())?;
    let Some(local) = local.get() else {
        return Ok(());
    };
    let interval = Domain1D::new(lower, upper);
    for parameter in S::defl_curv_intervals(local)? {
        insert_parameter(parameters, interval.clamp(parameter));
    }
    Ok(())
}

/// `insertParameter`: inserts in ascending order unless a value within
/// THE_PARAM_TOLERANCE of a neighbor of the insertion point is present.
pub(super) fn insert_parameter(parameters: &mut Vec<f64>, parameter: f64) {
    let (mut lower, mut upper) = (0, parameters.len());
    while lower < upper {
        let middle: usize = lower + (upper - lower) / 2;
        if parameters[middle] < parameter {
            lower = middle + 1;
        } else {
            upper = middle;
        }
    }
    let near_next: bool =
        lower < parameters.len() && (parameters[lower] - parameter).abs() <= THE_PARAM_TOLERANCE;
    let near_previous: bool =
        lower > 0 && (parameters[lower - 1] - parameter).abs() <= THE_PARAM_TOLERANCE;
    if near_next || near_previous {
        return;
    }
    parameters.insert(lower, parameter);
}
