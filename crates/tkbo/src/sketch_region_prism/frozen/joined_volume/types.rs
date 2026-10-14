// SPDX-License-Identifier: GPL-2.0-only
use super::AuthoredPrismId;
use rustycad_tkmath::predicates::roots::metric_bounds::CertifiedPolynomialMeasure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VolumeOperand {
    Orientation,
    Measures,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VolumeCheckpoint {
    Begin,
    Finish,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinedVolumeEvidence {
    ExactSignTransportOfCertifiedIntegral,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JoinedVolumeFailureClass {
    Incomplete,
    Cancelled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VolumeSignFactors {
    pub candidate_facing: i8,
    pub boundary_traversal: i8,
    pub selected_winding: i64,
    pub increasing_parameter: i8,
    pub jacobian: i8,
}
#[derive(Debug, PartialEq)]
pub(crate) struct JoinedPrismVolumeFailure {
    pub operation: AuthoredPrismId,
    pub requested_version: u32,
    pub entity: super::super::FrozenOriginalKey,
    pub class: JoinedVolumeFailureClass,
    pub cause: JoinedPrismVolumeCause,
}
#[derive(Debug, PartialEq)]
pub(crate) enum JoinedPrismVolumeCause {
    Version {
        expected: u32,
    },
    Binding(VolumeOperand),
    Cancelled(VolumeCheckpoint),
    SignFactors(VolumeSignFactors),
    ScalarContradiction {
        signed_enclosure: [f64; 2],
        exact_sign: i8,
    },
}
pub(crate) struct JoinedPrismVolume {
    pub(super) factors: VolumeSignFactors,
    pub(super) source: CertifiedPolynomialMeasure,
    pub(super) enclosure: [f64; 2],
    pub(super) representative: f64,
    pub(super) binding: Box<[u8]>,
    pub(super) directions: Vec<u8>,
    pub(super) measures: Vec<u8>,
}
