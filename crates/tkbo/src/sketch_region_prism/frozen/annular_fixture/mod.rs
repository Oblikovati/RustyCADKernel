// SPDX-License-Identifier: GPL-2.0-only
//! Authored two-loop fixture names precede lookup; actual history only binds their semantic roles.
mod names;
use super::*;
use crate::sketch_region_prism::region_certificate::{annular_binding::*, two_loop::*};
pub(super) use names::authored;
use rustycad_tkernel::standard::Failure;

pub(super) fn with_bound<T>(
    run: impl FnOnce(
        &TwoLoopRegionRequest<'_, '_>,
        &mut crate::sketch_region_prism::PrismReceipt,
        &mut PrismPublicationBindings,
        AnnularPrismBinding,
    ) -> T,
) -> T {
    crate::sketch_region_prism::region_certificate::annular_binding::tests::with_actual(
        |request, region, receipt| {
            let binding =
                bind_annular_prism_v2(request, region, receipt, &mut ContinueAnnularFixture)
                    .unwrap_or_else(|_| panic!("actual annular binding"));
            let mut names = authored(request, receipt, &binding);
            run(request, receipt, &mut names, binding)
        },
    )
}

pub(super) struct ContinueAnnularFixture;
pub(super) fn project(
    receipt: &crate::sketch_region_prism::PrismReceipt,
    names: &PrismPublicationBindings,
) -> FrozenPrismProjection {
    super::tests::captured(freeze_prism_with_behavior(
        receipt,
        names,
        &mut ContinueAnnularFixture,
        PrismFreezeBehavior::OrderedTraversalV2,
    ))
}
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
            let capture = project(receipt, &names);
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
