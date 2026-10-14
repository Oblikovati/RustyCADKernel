// SPDX-License-Identifier: GPL-2.0-only
//! Requested binary64 operands use their bits, independently of diagnostic formatting.
use super::*;
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions,
    metric_bounds::{BezierRegularityOptions, MetricParameterMap},
};

pub(super) fn roots(value: RootIsolationOptions, bytes: &mut Vec<u8>) {
    bytes.extend(value.version.to_le_bytes());
    encoding::numbers([value.max_width], bytes);
    bytes.extend(value.max_bisections.to_le_bytes());
    bytes.extend(value.max_degree.to_le_bytes());
}
pub(super) fn regularity(value: BezierRegularityOptions, bytes: &mut Vec<u8>) {
    bytes.extend(value.version.to_le_bytes());
    bytes.extend(value.max_stored_degree.to_le_bytes());
    roots(value.roots, bytes);
}
pub(super) fn embedding(value: PrismEmbeddingOptions, bytes: &mut Vec<u8>) {
    bytes.extend(value.version.to_le_bytes());
    regularity(value.regularity, bytes);
}
pub(super) fn parameter_maps(values: [MetricParameterMap; 2], bytes: &mut Vec<u8>) {
    for value in values {
        match value {
            MetricParameterMap::Identity => bytes.push(0),
            MetricParameterMap::Endpoints(endpoints) => {
                bytes.push(1);
                encoding::numbers(endpoints, bytes);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoding_preserves_signed_zero_and_nan_payload_bits_without_admitting_them() {
        let nan1 = f64::from_bits(0x7ff8000000000001);
        let nan2 = f64::from_bits(0x7ff8000000000002);
        let encode = |value| {
            let mut bytes = Vec::new();
            parameter_maps([MetricParameterMap::Endpoints([value, 1.]); 2], &mut bytes);
            bytes
        };
        assert_ne!(encode(0.), encode(-0.));
        assert_ne!(encode(nan1), encode(nan2));
        let root_bytes = |value| {
            let mut bytes = Vec::new();
            roots(RootIsolationOptions::new(value), &mut bytes);
            bytes
        };
        assert_ne!(root_bytes(nan1), root_bytes(nan2));
    }
}
