// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) use crate::sketch_region_prism::frozen::proof_join::{
    JoinedPrismProof, operation_tests::JoinFixture,
};
pub(super) struct NeverCancel;
impl JoinedConformanceProgress for NeverCancel {
    fn cancelled(&mut self, _: JoinedConformancePhase) -> bool {
        false
    }
}
impl JoinedToleranceProgress for NeverCancel {
    fn cancelled(&mut self, _: JoinedTolerancePhase) -> bool {
        false
    }
}
pub(super) fn conformance(join: &JoinedPrismProof) -> JoinedPrismConformance {
    certify_joined_prism_conformance(join, &mut NeverCancel)
        .ok()
        .expect("actual joined exact geometry")
}
pub(super) fn tolerance(
    join: &JoinedPrismProof,
    geometry: &JoinedPrismConformance,
    cap: StoredToleranceCap,
) -> Result<JoinedPrismTolerance, JoinedToleranceFailure> {
    certify_joined_prism_tolerances(join, geometry, cap, &mut NeverCancel)
}
pub(super) fn zero() -> rustycad_tkmath::predicates::roots::DyadicBound {
    rustycad_tkmath::predicates::roots::DyadicBound::from_f64(0.).unwrap()
}
