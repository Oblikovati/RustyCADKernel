// SPDX-License-Identifier: GPL-2.0-only
//! Numerical proof fields have explicit tags and exact scalar bits, independent of Debug output.
use super::*;
use crate::sketch_region_source_conformance::encoding::placement;
use rustycad_tkmath::predicates::roots::{RootIsolation, metric_bounds::*};

pub(super) fn policy(value: AnnularRegularityOptions, bytes: &mut Vec<u8>) {
    boundary_policy(value.boundary, bytes);
    bytes.extend(value.total_bisections.to_le_bytes());
}
fn boundary_policy(value: BezierRegularityOptions, bytes: &mut Vec<u8>) {
    bytes.extend(value.version.to_le_bytes());
    bytes.extend(value.max_stored_degree.to_le_bytes());
    crate::sketch_region_prism::frozen::proof_join::encode_joined_root_options(value.roots, bytes);
}
pub(super) fn encode(value: &AnnularRegularityReport, bytes: &mut Vec<u8>) {
    policy(value.requested, bytes);
    bytes.extend(value.total_bisections.to_le_bytes());
    for boundary in &value.boundaries {
        bytes.extend(boundary.source.0.to_le_bytes());
        boundary_policy(boundary.effective, bytes);
        report(&boundary.report, bytes);
    }
}
fn report(value: &BezierRegularityReport, bytes: &mut Vec<u8>) {
    source(value.source(), bytes);
    isolation(value.denominator_isolation(), bytes);
    count(value.denominator_degree(), bytes);
    bytes.push(u8::from(value.speed_degree().is_some()));
    if let Some(degree) = value.speed_degree() {
        count(degree, bytes);
    }
    decision(value.decision(), bytes);
    bytes.extend(value.bisections().to_le_bytes());
    // A report is constructible only for complete classification; failures are separate types.
    bytes.push(0);
}
fn source(value: &BezierRegularitySource, bytes: &mut Vec<u8>) {
    bytes.extend(value.curve_id().to_le_bytes());
    count(value.poles().len(), bytes);
    for pole in value.poles() {
        numbers(*pole, bytes);
    }
    bytes.push(u8::from(value.weights().is_some()));
    if let Some(weights) = value.weights() {
        count(weights.len(), bytes);
        numbers(weights.iter().copied(), bytes);
    }
    placement(value.placement(), bytes);
    numbers(value.interval(), bytes);
    boundary_policy(value.options(), bytes);
}
fn decision(value: &BezierRegularityDecision, bytes: &mut Vec<u8>) {
    match value {
        BezierRegularityDecision::Regular {
            zero_speed_isolation,
        } => {
            bytes.push(0);
            isolation(zero_speed_isolation, bytes);
        }
        BezierRegularityDecision::Stationary {
            zero_speed_isolation,
        } => {
            bytes.push(1);
            isolation(zero_speed_isolation, bytes);
        }
        BezierRegularityDecision::EverywhereStationary => bytes.push(2),
    }
}
fn isolation(value: &RootIsolation, bytes: &mut Vec<u8>) {
    bytes.extend(value.bisections().to_le_bytes());
    count(value.roots().len(), bytes);
    for root in value.roots() {
        let (lower, upper) = root.bounds();
        encode_joined_text(&lower.to_string(), bytes);
        encode_joined_text(&upper.to_string(), bytes);
        count(root.multiplicity(), bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::predicates::roots::RootIsolationOptions;
    fn certified(poles: &[[f64; 3]]) -> Vec<u8> {
        let options = BezierRegularityOptions::v1(RootIsolationOptions::new(1e-12));
        let proof =
            certify_bezier_regularity(BezierRegularityQuery::new(19, poles, options), || false)
                .unwrap();
        let mut bytes = Vec::new();
        report(&proof, &mut bytes);
        bytes
    }
    #[test]
    fn complete_regularity_variants_and_source_bits_have_distinct_replay() {
        let regular = certified(&[[0., 0., 0.], [1., 0., 0.]]);
        let stationary = certified(&[[0., 0., 0.], [1., 0., 0.], [0., 0., 0.]]);
        let constant = certified(&[[0., 0., 0.], [0., 0., 0.]]);
        assert_ne!(regular, stationary);
        assert_ne!(regular, constant);
        assert_ne!(stationary, constant);
        assert_ne!(regular, certified(&[[-0., 0., 0.], [1., 0., 0.]]));
    }
    #[test]
    fn aggregate_and_boundary_policy_have_independent_explicit_fields() {
        let requested = AnnularRegularityOptions {
            boundary: BezierRegularityOptions::v1(RootIsolationOptions::new(0.5)),
            total_bisections: 17,
        };
        let mut bytes = Vec::new();
        policy(requested, &mut bytes);
        let expected = [
            1u32.to_le_bytes().as_slice(),
            32u16.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            0.5f64.to_bits().to_le_bytes().as_slice(),
            4096u16.to_le_bytes().as_slice(),
            32u16.to_le_bytes().as_slice(),
            17u32.to_le_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(bytes, expected);
    }
}
