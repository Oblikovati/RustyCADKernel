// SPDX-License-Identifier: GPL-2.0-only
//! Parameter construction bounds are exact metric evidence, never root or topological identity.
use super::super::{
    certificate_arithmetic::{ExactRatio, RatioInterval},
    metric_bounds::{MetricParameterMap, ResidualError},
};
use super::{BezierPairCertificate, ExactBezierPairParameterEnclosure};
use num_bigint::BigInt;
use std::cmp::Ordering;

/// Completion of an exact interval error comparison, not a contact identity decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairParameterDecision {
    /// Every value in the retained enclosure meets the requested construction limit.
    Within,
    /// Every value in the retained enclosure exceeds the requested construction limit.
    Exceeds,
    /// A conservative enclosure cannot decide the requested threshold.
    Inconclusive,
}
/// Exact original measurement failure with its canonical report parameter axis.
#[derive(Debug)]
pub struct PairParameterConstructionFailure {
    /// Canonical t=0 or u=1 source slot whose measurement could not complete.
    pub axis: usize,
    /// Original unflattened scalar, map or arithmetic cause.
    pub cause: ResidualError,
}
impl std::fmt::Display for PairParameterConstructionFailure {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let axis = self.axis;
        write!(
            output,
            "parameter axis={axis}: {}; expected two canonical source axes",
            self.cause
        )
    }
}
impl std::error::Error for PairParameterConstructionFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        let retained = &self.cause;
        Some(retained)
    }
}
/// Immutable native-unit construction measurement from an actual completed contact.
pub struct MeasuredPairParameter {
    /// Original observed scalar, preserved without snapping.
    native_parameter: f64,
    /// Caller-requested native-parameter limit, never widened.
    requested_limit: f64,
    /// Proven threshold decision or explicitly inconclusive enclosure.
    decision: PairParameterDecision,
    native_enclosure: RatioInterval,
    error_enclosure: RatioInterval,
}
impl MeasuredPairParameter {
    /// Read the unchanged observed scalar; example: `measured.native_parameter().to_bits()`.
    pub fn native_parameter(&self) -> f64 {
        self.native_parameter
    }
    /// Read the original construction limit; example: `assert_eq!(measured.requested_limit(), 1e-8)`.
    pub fn requested_limit(&self) -> f64 {
        self.requested_limit
    }
    /// Read the certified comparison; example: `measured.decision() == PairParameterDecision::Within`.
    pub fn decision(&self) -> PairParameterDecision {
        self.decision
    }
    /// Borrow native-unit rational bounds; example: `measured.native_enclosure().compare_lower(0.5)?`.
    pub fn native_enclosure(&self) -> ExactBezierPairParameterEnclosure<'_> {
        let retained = &self.native_enclosure;
        ExactBezierPairParameterEnclosure::new(retained)
    }
    /// Borrow rigorous error bounds, without claiming a tight measured error; example: `measured.error_enclosure().compare_upper(1e-8)?`.
    pub fn error_enclosure(&self) -> ExactBezierPairParameterEnclosure<'_> {
        let retained = &self.error_enclosure;
        ExactBezierPairParameterEnclosure::new(retained)
    }
}
impl BezierPairCertificate<'_> {
    /// Measure both native parameter constructions on one actual contact; this does not prove identity.
    /// Example: `proof.measure_native_parameters(0, maps, [0.5,0.5], [1e-8,1e-8])?`.
    pub fn measure_native_parameters(
        &self,
        contact_index: usize,
        maps: [MetricParameterMap; 2],
        native: [f64; 2],
        limits: [f64; 2],
    ) -> Result<Option<[MeasuredPairParameter; 2]>, PairParameterConstructionFailure> {
        let Some(contact) = self.contact(contact_index) else {
            return Ok(None);
        };
        let first = super::graph_enclosure::parameter_interval(contact.parameter);
        let bounds = [&first, contact.exact_paired_parameter.interval()];
        measure_captured_axes(bounds, self.captured_curves(), maps, native, limits).map(Some)
    }
}
struct AxisConstructionSettings {
    map: MetricParameterMap,
    native: f64,
    limit: f64,
    axis: usize,
}
fn measure_captured_axes(
    bounds: [&RatioInterval; 2],
    sources: [super::BezierPairCurve<'_>; 2],
    maps: [MetricParameterMap; 2],
    native: [f64; 2],
    limits: [f64; 2],
) -> Result<[MeasuredPairParameter; 2], PairParameterConstructionFailure> {
    let [first, second] = std::array::from_fn(|axis| {
        let settings = AxisConstructionSettings {
            map: maps[axis],
            native: native[axis],
            limit: limits[axis],
            axis,
        };
        measure_axis(bounds[axis], sources[axis].interval, settings)
            .map_err(|cause| PairParameterConstructionFailure { axis, cause })
    });
    Ok([first?, second?])
}
fn measure_axis(
    bounds: &RatioInterval,
    trim: [f64; 2],
    settings: AxisConstructionSettings,
) -> Result<MeasuredPairParameter, ResidualError> {
    let [supplied, threshold] = admitted_axis(&settings)?;
    let native_enclosure = native_image(&inside_source_domain(bounds, trim), settings.map)?;
    let error_enclosure = distance_enclosure(&native_enclosure, &supplied);
    let decision = threshold_decision(&error_enclosure, &threshold);
    Ok(owned_measurement(
        settings,
        native_enclosure,
        error_enclosure,
        decision,
    ))
}
fn admitted_axis(settings: &AxisConstructionSettings) -> Result<[ExactRatio; 2], ResidualError> {
    let supplied = admitted_value(
        settings.native,
        "native parameter construction",
        settings.axis,
        false,
    )?;
    let threshold = admitted_value(
        settings.limit,
        "native parameter limit",
        settings.axis,
        true,
    )?;
    Ok([supplied, threshold])
}

fn owned_measurement(
    settings: AxisConstructionSettings,
    native_enclosure: RatioInterval,
    error_enclosure: RatioInterval,
    decision: PairParameterDecision,
) -> MeasuredPairParameter {
    MeasuredPairParameter {
        native_parameter: settings.native,
        requested_limit: settings.limit,
        decision,
        native_enclosure,
        error_enclosure,
    }
}

fn admitted_value(
    value: f64,
    role: &'static str,
    index: usize,
    nonnegative: bool,
) -> Result<ExactRatio, ResidualError> {
    if !value.is_finite() || nonnegative && value < 0. {
        return Err(ResidualError::Value { role, index, value });
    }
    Ok(ExactRatio::from_finite(value))
}
fn inside_source_domain(bounds: &RatioInterval, trim: [f64; 2]) -> RatioInterval {
    // Exact membership in this same certificate proves the actual root lies in the original trim.
    RatioInterval {
        lower: bounds.lower.clone().max(ExactRatio::from_finite(trim[0])),
        upper: bounds.upper.clone().min(ExactRatio::from_finite(trim[1])),
    }
}
fn native_image(
    bounds: &RatioInterval,
    map: MetricParameterMap,
) -> Result<RatioInterval, ResidualError> {
    let [origin, span] = map.exact_coefficients()?;
    let scale: BigInt = BigInt::from(1) << 1074_usize;
    let origin = RatioInterval::point(ExactRatio::new(origin, scale.clone()));
    let span = RatioInterval::point(ExactRatio::new(span, scale));
    Ok(origin.add(&bounds.multiply(&span)))
}
fn distance_enclosure(bounds: &RatioInterval, native: &ExactRatio) -> RatioInterval {
    let left = absolute(bounds.lower.subtract(native));
    let right = absolute(bounds.upper.subtract(native));
    let minimum = if native < &bounds.lower {
        left.clone()
    } else if native > &bounds.upper {
        right.clone()
    } else {
        ExactRatio::from_finite(0.)
    };
    RatioInterval {
        lower: minimum,
        upper: left.max(right),
    }
}
fn absolute(value: ExactRatio) -> ExactRatio {
    if value.sign() == Ordering::Less {
        return value.negated();
    }
    value
}
fn threshold_decision(error: &RatioInterval, limit: &ExactRatio) -> PairParameterDecision {
    if &error.upper <= limit {
        return PairParameterDecision::Within;
    }
    if &error.lower > limit {
        return PairParameterDecision::Exceeds;
    }
    PairParameterDecision::Inconclusive
}
