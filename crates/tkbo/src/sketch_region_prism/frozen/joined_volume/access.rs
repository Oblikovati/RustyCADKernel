// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkmath::predicates::roots::metric_bounds::{
    CertifiedPolynomialMeasure, PolynomialIntegralOptions,
};
impl JoinedPrismVolume {
    /// Read outward volume bounds; `volume.enclosure()` follows the exact sign proof.
    pub(crate) fn enclosure(&self) -> [f64; 2] {
        self.enclosure
    }
    /// Read the transported representative; `volume.representative()` may be the UPPER bound.
    pub(crate) fn representative(&self) -> f64 {
        self.representative
    }
    /// Preserve the original certified error; `volume.absolute_error_bound()` changes no limit.
    pub(crate) fn absolute_error_bound(&self) -> f64 {
        self.source.absolute_error_bound()
    }
    /// Read unchanged requested volume-unit policy; `volume.requested()`.
    pub(crate) fn requested(&self) -> PolynomialIntegralOptions {
        self.source.requested_options()
    }
    /// Read unchanged effective volume-unit policy; `volume.effective()` has no new clamps.
    pub(crate) fn effective(&self) -> PolynomialIntegralOptions {
        self.source.effective_options()
    }
    /// Read the original signed scalar without relabeling it; `volume.source_integral()`.
    pub(crate) fn source_integral(&self) -> &CertifiedPolynomialMeasure {
        &self.source
    }
    /// Read exact orientation provenance; `volume.sign_factors()` never uses metric signs.
    pub(crate) fn sign_factors(&self) -> VolumeSignFactors {
        self.factors
    }
    /// Read the explicit evidence kind; `volume.evidence()` supplies no body publication authority.
    pub(crate) fn evidence(&self) -> JoinedVolumeEvidence {
        JoinedVolumeEvidence::ExactSignTransportOfCertifiedIntegral
    }
    /// Compare complete immutable operands; `volume.matches_join(join)` excludes foreign bindings.
    pub(crate) fn matches_join(&self, join: &JoinedPrismProof) -> bool {
        self.binding.as_ref() == join.replay_bytes()
    }
    /// Read owned full source binding; `volume.binding_bytes()` contains no process addresses.
    pub(crate) fn binding_bytes(&self) -> &[u8] {
        &self.binding
    }
    /// Encode all child operands and quantities; `volume.replay_bytes()` recomputes from owned fields.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        replay::encode(self)
    }
}
