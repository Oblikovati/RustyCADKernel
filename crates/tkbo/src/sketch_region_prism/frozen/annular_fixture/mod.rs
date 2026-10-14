// SPDX-License-Identifier: GPL-2.0-only
//! Authored two-loop fixture names precede lookup; actual history only binds their semantic roles.
mod names;
use super::*;
use crate::sketch_region_prism::region_certificate::{annular_binding::*, two_loop::*};
pub(super) use names::authored;
use rustycad_tkernel::standard::Failure;

pub(super) struct ContinueAnnularFixture;
impl AnnularBindingProgress for ContinueAnnularFixture {
    fn checkpoint(&mut self, _: AnnularBindingStage) -> Result<bool, Failure> {
        Ok(false)
    }
}
impl FreezeProgress for ContinueAnnularFixture {
    fn checkpoint(&mut self, _: FreezePhase) -> Result<bool, Failure> {
        Ok(false)
    }
}
#[test]
fn actual_authored_annular_projection_keeps_all_faces_wires_and_directed_trims() {
    crate::sketch_region_prism::region_certificate::annular_binding::tests::with_actual(
        |request, region, receipt| {
            let binding =
                bind_annular_prism_v2(request, region, receipt, &mut ContinueAnnularFixture)
                    .unwrap_or_else(|_| panic!("expected actual annular binding"));
            let names = names::authored(request, receipt, &binding);
            let capture = super::tests::captured(freeze_prism_with_behavior(
                receipt,
                &names,
                &mut ContinueAnnularFixture,
                PrismFreezeBehavior::OrderedTraversalV2,
            ));
            assert_eq!(capture.graph.counts, [4, 6, 12, 6, 4, 1, 1]);
            assert_eq!(capture.faces().len(), 4);
            assert_eq!(
                capture.faces().iter().map(|f| f.trims.len()).sum::<usize>(),
                12
            );
            assert!(!capture.is_valid_body());
        },
    );
}
