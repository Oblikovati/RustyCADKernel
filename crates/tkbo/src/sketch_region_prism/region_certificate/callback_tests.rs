// SPDX-License-Identifier: GPL-2.0-only
//! Callbacks can mutate shared source handles; post-copy admission must inspect current geometry.
use super::super::{isolation_tests::set_pole, tests::request};
use super::{tests::*, *};
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkbrep::topods::{Shape, edge};
use rustycad_tkernel::standard::Failure;

struct EditOriginalAtPhase {
    source: Shape,
    phase: PrismPhase,
    edited: bool,
}
impl PrismProgress for EditOriginalAtPhase {
    fn checkpoint(&mut self, phase: PrismPhase) -> Result<bool, Failure> {
        if phase == self.phase && !self.edited {
            set_pole(&edge(&self.source)?, 5.);
            self.edited = true;
        }
        Ok(false)
    }
}
#[test]
fn callback_source_mutation_after_selection_never_reuses_old_certificate() {
    for phase in [
        PrismPhase::Selection,
        PrismPhase::Copy,
        PrismPhase::History,
        PrismPhase::Validation,
    ] {
        assert_callback_refusal(phase);
    }
}

fn assert_callback_refusal(phase: PrismPhase) {
    let (bridge, shapes) = fixture();
    let mut callback = EditOriginalAtPhase {
        source: shapes[0].clone(),
        phase,
        edited: false,
    };
    let receipt = retain_source_bound_prism(
        &bridge,
        &binding_sources(&shapes),
        request([1., 1.]),
        &mut callback,
    )
    .ok()
    .unwrap();
    assert!(callback.edited);
    assert_edited_receipt(receipt);
}
fn assert_edited_receipt(receipt: PrismReceipt) {
    assert_four_present(&receipt);
    assert!(matches!(
        receipt.region.unwrap().decision,
        Err(RegionCertificateCause::Hypothesis {
            name: "current original coefficients/range/placement",
            ..
        })
    ));
}
