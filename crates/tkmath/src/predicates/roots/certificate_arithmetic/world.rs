// SPDX-License-Identifier: GPL-2.0-only
use super::super::RootBracket;
use super::super::bezier_line::BezierLineQuery;
use crate::predicates::roots::certificate_arithmetic::{
    ExactRatio, RatioInterval, bernstein_enclosure, rounding,
};

pub(in crate::predicates::roots) struct HomogeneousCurve {
    coordinates: [Vec<ExactRatio>; 2],
    denominator: Vec<ExactRatio>,
}
pub(in crate::predicates::roots) struct WorldEnvelope {
    pub bounds: [[f64; 2]; 2],
    pub point: [f64; 2],
    pub achieved_error: f64,
    pub error_squared: ExactRatio,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Original exact world-enclosure noncompletion.
pub enum EnvelopeFailure {
    /// Caller cancellation before complete enclosure.
    Cancelled,
    /// Denominator interval contains zero; a regular point is not yet enclosed.
    DenominatorUnresolved,
    /// Finite binary64 enclosure or radius representation could not be constructed.
    Binary64Overflow,
}
impl HomogeneousCurve {
    pub(in crate::predicates::roots) fn from_validated(query: BezierLineQuery<'_>) -> Self {
        Self::from_controls(query.controls, query.weights)
    }
    pub(in crate::predicates::roots) fn from_controls(
        controls: &[[f64; 2]],
        weights: Option<&[f64]>,
    ) -> Self {
        let denominator: Vec<ExactRatio> = (0..controls.len())
            .map(|index| ExactRatio::from_finite(weights.map_or(1., |weights| weights[index])))
            .collect();
        let coordinates = std::array::from_fn(|axis| {
            controls
                .iter()
                .zip(&denominator)
                .map(|(point, weight)| ExactRatio::from_finite(point[axis]).multiply(weight))
                .collect()
        });
        Self {
            coordinates,
            denominator,
        }
    }
    pub(in crate::predicates::roots) fn enclose(
        &self,
        root: &RootBracket,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<WorldEnvelope, EnvelopeFailure> {
        let parameter = RatioInterval {
            lower: ExactRatio::from_dyadic(root.bounds().0),
            upper: ExactRatio::from_dyadic(root.bounds().1),
        };
        let denominator = bernstein_enclosure(&self.denominator, &parameter, cancelled)
            .ok_or(EnvelopeFailure::Cancelled)?;
        let reciprocal = denominator
            .reciprocal()
            .ok_or(EnvelopeFailure::DenominatorUnresolved)?;
        let x = self.coordinate(0, &parameter, &reciprocal, cancelled)?;
        let y = self.coordinate(1, &parameter, &reciprocal, cancelled)?;
        finish_envelope([x, y])
    }
    fn coordinate(
        &self,
        axis: usize,
        parameter: &RatioInterval,
        reciprocal: &RatioInterval,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<RatioInterval, EnvelopeFailure> {
        let numerator = bernstein_enclosure(&self.coordinates[axis], parameter, cancelled)
            .ok_or(EnvelopeFailure::Cancelled)?;
        Ok(numerator.multiply(reciprocal))
    }
}
struct AxisConstruction {
    bounds: [f64; 2],
    point: f64,
    radius: ExactRatio,
}
pub(in crate::predicates::roots) fn finish_envelope(
    intervals: [RatioInterval; 2],
) -> Result<WorldEnvelope, EnvelopeFailure> {
    let [x, y] = intervals;
    let x = construct_axis(x)?;
    let y = construct_axis(y)?;
    let error_squared = x
        .radius
        .multiply(&x.radius)
        .add(&y.radius.multiply(&y.radius));
    let achieved_error =
        rounding::outward_sqrt(&error_squared).ok_or(EnvelopeFailure::Binary64Overflow)?;
    Ok(WorldEnvelope {
        point: [x.point, y.point],
        bounds: [x.bounds, y.bounds],
        achieved_error,
        error_squared,
    })
}
fn construct_axis(interval: RatioInterval) -> Result<AxisConstruction, EnvelopeFailure> {
    let lower =
        rounding::outward_binary64(&interval.lower).ok_or(EnvelopeFailure::Binary64Overflow)?;
    let upper =
        rounding::outward_binary64(&interval.upper).ok_or(EnvelopeFailure::Binary64Overflow)?;
    let midpoint = interval
        .lower
        .add(&interval.upper)
        .divide(&ExactRatio::from_finite(2.));
    let candidates =
        rounding::outward_binary64(&midpoint).ok_or(EnvelopeFailure::Binary64Overflow)?;
    let (point, radius) = nearest_candidate(&interval, candidates);
    Ok(AxisConstruction {
        bounds: [lower[0], upper[1]],
        point,
        radius,
    })
}
fn nearest_candidate(interval: &RatioInterval, candidates: [f64; 2]) -> (f64, ExactRatio) {
    let left = endpoint_radius(interval, candidates[0]);
    let right = endpoint_radius(interval, candidates[1]);
    // The lower adjacent float wins exact ties, independently of platform rounding mode.
    if right < left {
        return (candidates[1], right);
    }
    (candidates[0], left)
}
fn endpoint_radius(interval: &RatioInterval, point: f64) -> ExactRatio {
    let exact = ExactRatio::from_finite(point);
    let lower = absolute(exact.subtract(&interval.lower));
    let upper = absolute(interval.upper.subtract(&exact));
    lower.max(upper)
}
fn absolute(value: ExactRatio) -> ExactRatio {
    if value.sign() == std::cmp::Ordering::Less {
        return value.negated();
    }
    value
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adjacent_float_distance_certifies_third_without_using_enclosure_width() {
        let third = ExactRatio::new(1.into(), 3.into());
        let axis = construct_axis(RatioInterval::point(third.clone())).unwrap();
        let distance = absolute(ExactRatio::from_finite(axis.point).subtract(&third));
        assert_eq!(axis.point.to_bits(), 0x3fd5_5555_5555_5555);
        assert_eq!(axis.bounds[1].to_bits(), 0x3fd5_5555_5555_5556);
        assert_eq!(axis.radius, distance);
        assert!(distance > ExactRatio::from_finite(1e-20));
        assert!(distance < ExactRatio::from_finite(3e-17));
        let other = absolute(ExactRatio::from_finite(axis.bounds[1]).subtract(&third));
        assert!(distance < other);
        assert!(other > ExactRatio::from_finite(3e-17));
    }
    #[test]
    fn candidate_ties_choose_lower_binary64_deterministically() {
        let tiny = ExactRatio::from_finite(f64::from_bits(1));
        let midpoint = tiny.divide(&ExactRatio::from_finite(2.));
        let axis = construct_axis(RatioInterval::point(midpoint.clone())).unwrap();
        assert_eq!(axis.point, 0.);
        assert_eq!(axis.radius, midpoint);
        let envelope = finish_envelope([
            RatioInterval::point(tiny),
            RatioInterval::point(ExactRatio::from_finite(0.)),
        ])
        .unwrap();
        assert_eq!(envelope.achieved_error, 0.);
    }
}
