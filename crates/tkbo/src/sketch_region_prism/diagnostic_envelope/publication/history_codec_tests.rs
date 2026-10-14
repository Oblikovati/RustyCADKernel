// SPDX-License-Identifier: GPL-2.0-only
//! Literal codec cases: every supported family has distinct exact bytes; others stay pending.
use super::*;
use crate::sketch_region_prism::frozen::{AuthoredPrismId, FreezePhase, PrismFreezeCause};
use rustycad_tkernel::standard::Failure;
use std::sync::Arc;

fn finding(cause: PrismHistoryCause) -> PrismHistoryFinding {
    PrismHistoryFinding {
        operation: AuthoredPrismId(77),
        phase: PrismHistoryPhase::Capture,
        entity: None,
        class: cause.failure_class(),
        cause,
    }
}
fn key() -> PrismHistoryUseKey {
    let member = PrismHistoryMemberKey {
        operation: AuthoredPrismId(77),
        stage: PrismHistoryStage::Partition,
        member: AuthoredPrismId(12004),
        kind: 6,
    };
    PrismHistoryUseKey(vec![(member, AuthoredPrismId(1704))])
}
fn at_use(cause: PrismHistoryCause) -> PrismHistoryCause {
    let (entity, cause) = (key(), Box::new(cause));
    PrismHistoryCause::AtUse { entity, cause }
}
fn count() -> PrismHistoryCause {
    let (required, actual, expected) = ("count", 1, 2);
    PrismHistoryCause::Count {
        required,
        actual,
        expected,
    }
}
fn parents() -> PrismHistoryCause {
    let (stage, actual, expected) = (PrismHistoryStage::Partition, Box::default(), Box::default());
    PrismHistoryCause::ParentsMismatch {
        stage,
        actual,
        expected,
    }
}
fn read() -> PrismHistoryCause {
    let (stage, cause) = (
        PrismHistoryStage::Original,
        HistoricalContextReadCause::NotReached,
    );
    PrismHistoryCause::ContextRead { stage, cause }
}
fn diagnostic() -> PrismHistoryCause {
    let (stage, kind) = (
        PrismHistoryStage::CandidateCopy,
        HistoricalDiagnosticKind::PartitionRead,
    );
    let (observation, retained_rows) = (Some(3), 4);
    PrismHistoryCause::UnsupportedDiagnostic {
        stage,
        kind,
        observation,
        retained_rows,
    }
}
fn limit_bounds() -> [HistoryBoundCause; 2] {
    use HistoryBoundCause as B;
    let (quantity, actual, minimum, maximum) = ("q", 0, 1, 2);
    let setting = B::Setting {
        quantity,
        actual,
        minimum,
        maximum,
    };
    let exhausted = B::Exhausted {
        quantity,
        actual: 3,
        maximum,
    };
    [setting, exhausted]
}
fn bounds() -> Vec<PrismHistoryCause> {
    use HistoryBoundCause as B;
    let [setting, exhausted] = limit_bounds();
    let (ancestor_depth, repeated_depth) = (1, 2);
    let cycle = B::AncestorCycle {
        ancestor_depth,
        repeated_depth,
    };
    let native = B::Native(Failure::new("codec native"));
    [setting, exhausted, cycle, native]
        .map(PrismHistoryCause::Bounds)
        .into()
}
fn supported() -> Vec<PrismHistoryCause> {
    use PrismHistoryCause as C;
    let mut result = bounds();
    let native = C::Native(Arc::new(Failure::new("codec native")));
    let cancelled = C::LineageCancelled(FreezePhase::Geometry);
    let (missing, changed) = (C::Missing { required: "m" }, C::Changed { operand: "c" });
    result.extend([
        count(),
        missing,
        changed,
        parents(),
        read(),
        native,
        cancelled,
    ]);
    result.extend([diagnostic(), at_use(count())]);
    result
}
fn encoded() -> Vec<Vec<u8>> {
    let bytes = supported().into_iter().map(|c| finding_bytes(&finding(c)));
    bytes.map(|b| b.expect("supported family")).collect()
}
#[test]
fn every_supported_history_family_has_distinct_deterministic_bytes() {
    let bytes = encoded();
    let distinct: std::collections::BTreeSet<_> = bytes.iter().collect();
    assert_eq!(distinct.len(), bytes.len());
    assert_eq!(bytes, encoded());
}
#[test]
fn missing_cause_has_the_exact_literal_layout() {
    let actual = finding_bytes(&finding(PrismHistoryCause::Missing { required: "x" })).unwrap();
    let mut expected = b"Martensite.PublicationHistoryDiagnostic/v1\0".to_vec();
    expected.extend(77u64.to_le_bytes());
    expected.extend([1, 1, 0, 2]);
    encode_joined_text("x", &mut expected);
    assert_eq!(actual, expected);
}
#[test]
fn owner_bearing_and_nested_untagged_families_remain_pending() {
    let frozen = || {
        let representation = "r";
        PrismHistoryCause::Frozen(Box::new(PrismFreezeCause::StaleCapture { representation }))
    };
    let pending = PublicationPendingPayload::History;
    assert_eq!(finding_bytes(&finding(frozen())).unwrap_err(), pending);
    assert_eq!(
        finding_bytes(&finding(at_use(frozen()))).unwrap_err(),
        pending
    );
}
