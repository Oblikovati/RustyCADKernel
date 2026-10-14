// SPDX-License-Identifier: GPL-2.0-only
//! The independent binary fixture covers every V1 admission case and original scalar bit pattern.
use super::{super::*, admission};
#[test]
fn typed_admission_encoding_matches_independent_every_variant_fixture() {
    let mut actual = Vec::new();
    for value in cases() {
        admission::append(&value, &mut actual);
    }
    assert_eq!(actual, include_bytes!("fixtures/admission-v1.bin"));
}
fn cases() -> Vec<OwnedPrismAdmissionCause> {
    let mut values = count_cases();
    values.extend(geometry_cases());
    values.extend(
        name_cases()
            .into_iter()
            .map(OwnedPrismAdmissionCause::Names),
    );
    values
}
fn count_cases() -> Vec<OwnedPrismAdmissionCause> {
    use OwnedPrismAdmissionCause::*;
    vec![
        EmptyInput,
        UnsupportedVersion {
            actual: 99,
            expected: 1,
        },
        MultipleBoundaries {
            actual: vec![CallerBoundaryId(7), CallerBoundaryId(9001)],
            supported: 1,
        },
        PoleCount {
            source: CallerBoundaryId(7),
            actual: 0,
            minimum: 2,
            maximum: 26,
        },
    ]
}
fn geometry_cases() -> Vec<OwnedPrismAdmissionCause> {
    use OwnedPrismAdmissionCause::*;
    vec![
        NonfinitePole {
            source: CallerBoundaryId(7),
            pole: 3,
            axis: 2,
            actual: f64::from_bits(0x7ff8_0000_0000_0123),
        },
        OpenBoundary {
            source: CallerBoundaryId(7),
            first: [-0., 1., 2.],
            last: [3., 4., 5.],
        },
        InvalidExtrusion {
            actual: [1., -0., f64::INFINITY],
        },
        nonfinite_plane(),
    ]
}
fn name_cases() -> Vec<SemanticNameFailure> {
    vec![
        SemanticNameFailure::EqualEndpointUses {
            actual: CallerDirectedUseId(901),
        },
        SemanticNameFailure::NonfiniteSeed {
            axis: 1,
            actual: f64::NEG_INFINITY,
        },
        SemanticNameFailure::UnknownToken {
            actual: 9001,
            version: 2,
        },
    ]
}

fn nonfinite_plane() -> OwnedPrismAdmissionCause {
    OwnedPrismAdmissionCause::NonfinitePlane {
        component: DeclaredPlaneComponent::V,
        axis: 2,
        actual: f64::from_bits(0x7ff8_0000_0000_0456),
    }
}
#[test]
fn typed_plane_components_have_explicit_distinct_tags() {
    use DeclaredPlaneComponent::*;
    for (component, tag) in [(Origin, 0), (Normal, 1), (U, 2), (V, 3)] {
        let value = OwnedPrismAdmissionCause::NonfinitePlane {
            component,
            axis: 1,
            actual: f64::INFINITY,
        };
        let mut bytes = Vec::new();
        admission::append(&value, &mut bytes);
        assert_eq!(&bytes[..2], &[7, tag]);
    }
}
