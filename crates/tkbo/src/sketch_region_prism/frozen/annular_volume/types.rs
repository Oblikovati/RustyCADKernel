// SPDX-License-Identifier: GPL-2.0-only
//! Owned annular volume evidence: exact sign provenance plus the unchanged certified integral.
use super::*;
use crate::sketch_region_prism::frozen::AuthoredPrismId;
use rustycad_tkmath::predicates::roots::metric_bounds::{
    CertifiedPolynomialMeasure, PolynomialIntegralOptions,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularVolumeOperand {
    Orientation,
    Measures,
    /// The measured material normalization differs from the certified boundary directions.
    Boundaries,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularVolumeEvidence {
    /// Certified error-bounded sweep integral with an exact sign proof; never a mesh or sample.
    ExactSignTransportOfCertifiedIntegral,
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) enum AnnularVolumeCause {
    Version {
        actual: u32,
        expected: u32,
    },
    Binding(AnnularVolumeOperand),
    Cancelled(VolumeCheckpoint),
    SignFactors {
        jacobian: i8,
        boundaries: [AnnularBoundaryDirection; 2],
    },
    ScalarContradiction {
        signed_enclosure: [f64; 2],
        exact_sign: i8,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularVolumeFailure {
    pub operation: AuthoredPrismId,
    pub requested_version: u32,
    pub cause: AnnularVolumeCause,
}
impl AnnularVolumeFailure {
    /// Cancellation is distinct from incomplete evidence; `failure.cancelled()`.
    pub(in crate::sketch_region_prism) fn cancelled(&self) -> bool {
        matches!(self.cause, AnnularVolumeCause::Cancelled(_))
    }
}
pub(in crate::sketch_region_prism) struct AnnularPrismVolume {
    pub(super) jacobian_sign: i8,
    pub(super) boundaries: [AnnularBoundaryDirection; 2],
    pub(super) source: CertifiedPolynomialMeasure,
    pub(super) enclosure: [f64; 2],
    pub(super) representative: f64,
    pub(super) binding: Box<[u8]>,
    pub(super) replay: Vec<u8>,
}
impl AnnularPrismVolume {
    /// Read the nonnegative physical-volume enclosure; `volume.enclosure()[0] >= 0.`.
    pub(in crate::sketch_region_prism) fn enclosure(&self) -> [f64; 2] {
        self.enclosure
    }
    /// Read the transported representative; for a negative sign it is the UPPER endpoint.
    pub(in crate::sketch_region_prism) fn representative(&self) -> f64 {
        self.representative
    }
    /// Read the unchanged certified error of the source integral; `volume.absolute_error_bound()`.
    pub(in crate::sketch_region_prism) fn absolute_error_bound(&self) -> f64 {
        self.source.absolute_error_bound()
    }
    /// Read requested integration policy; `volume.requested()`.
    pub(in crate::sketch_region_prism) fn requested(&self) -> PolynomialIntegralOptions {
        self.source.requested_options()
    }
    /// Read effective integration policy, including clamps; `volume.effective()`.
    pub(in crate::sketch_region_prism) fn effective(&self) -> PolynomialIntegralOptions {
        self.source.effective_options()
    }
    /// Read the original signed integral without relabeling it; `volume.source_integral()`.
    pub(in crate::sketch_region_prism) fn source_integral(&self) -> &CertifiedPolynomialMeasure {
        &self.source
    }
    /// Read the exact determinant sign used for transport; `volume.jacobian_sign()`.
    pub(in crate::sketch_region_prism) fn jacobian_sign(&self) -> i8 {
        self.jacobian_sign
    }
    /// Read the certified boundary directions; `volume.boundaries()`.
    pub(in crate::sketch_region_prism) fn boundaries(&self) -> &[AnnularBoundaryDirection; 2] {
        &self.boundaries
    }
    /// Read the evidence kind; `volume.evidence()` grants no publication authority.
    pub(in crate::sketch_region_prism) fn evidence(&self) -> AnnularVolumeEvidence {
        AnnularVolumeEvidence::ExactSignTransportOfCertifiedIntegral
    }
    /// Compare complete join bytes; `volume.binds(join)` excludes foreign bindings.
    pub(in crate::sketch_region_prism) fn binds(&self, join: &AnnularFrozenDomain) -> bool {
        self.binding.as_ref() == join.replay_bytes()
    }
    /// Full replay of sign provenance, scalars and child receipts; `volume.replay_bytes()`.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
}
