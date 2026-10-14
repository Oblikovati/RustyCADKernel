// SPDX-License-Identifier: GPL-2.0-only
//! Callback encoding preserves child phase, ordinal and native failure independently of outer status.
use super::{super::*, callback, native};
use crate::sketch_region_prism::PrismPhase;
use rustycad_tkernel::standard::Failure;
#[test]
fn absent_and_exhausted_callbacks_have_distinct_exact_receipts() {
    let mut bytes = Vec::new();
    callback(None, &mut bytes);
    assert_eq!(bytes, [0]);
    let stop = OwnedPreparationCallback {
        checkpoint: OwnedPrismCheckpoint {
            phase: OwnedPrismPhase::Prism(PrismPhase::Sweep),
            ordinal: usize::MAX,
        },
        cause: OwnedDiagnosticCallbackCause::Exhausted,
    };
    bytes.clear();
    callback(Some(&stop), &mut bytes);
    let mut expected = vec![1, 7, 3];
    expected.extend((usize::MAX as u64).to_le_bytes());
    expected.push(2);
    assert_eq!(bytes, expected);
}
#[test]
fn callback_failure_composes_exact_native_payload_after_child_checkpoint() {
    let failure = Failure::with_type_bytes("X", &[255], &[254]);
    let stop = OwnedPreparationCallback {
        checkpoint: OwnedPrismCheckpoint {
            phase: OwnedPrismPhase::Census,
            ordinal: 19,
        },
        cause: OwnedDiagnosticCallbackCause::Failed(failure.clone()),
    };
    let mut expected = vec![1, 3];
    expected.extend(19_u64.to_le_bytes());
    expected.push(1);
    native::append(&failure, &mut expected);
    let mut actual = Vec::new();
    callback(Some(&stop), &mut actual);
    assert_eq!(actual, expected);
}
