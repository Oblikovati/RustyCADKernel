// SPDX-License-Identifier: GPL-2.0-only
//! Real aggregate copy and General Fuse history reaches the private validation adapter.
use super::outcome::{BodyKind, StageCoverage};
use crate::sketch_region::{
    ArrangementCompletion, ArrangementSource, OwnedArrangement, RegionSourceId,
    arrange_owned_sources,
};

fn actual_owned_arrangement() -> OwnedArrangement {
    let original = crate::sketch_region::tests::closed_cubic();
    let sources = [ArrangementSource {
        identity: RegionSourceId(42),
        shape: &original,
    }];
    match arrange_owned_sources(&sources, &mut |_| false) {
        ArrangementCompletion::Arranged(output) => output,
        _ => panic!("expected completed arrangement stage, not validated body"),
    }
}

#[test]
fn actual_owned_history_reaches_validation_without_final_operation_credit() {
    let owned = actual_owned_arrangement();
    let report = super::validate_owned_arrangement(&owned, BodyKind::PlanarRegion);
    let root = owned
        .relations
        .iter()
        .find(|row| row.output.is_equal(&owned.arrangement))
        .unwrap();
    let finding = report.findings.first().unwrap();
    let parents = finding.affected.original_parents.as_ref().unwrap();
    assert_eq!(parents.preserved, root.preserved);
    assert_eq!(parents.modified, root.modified);
    assert_eq!(parents.generated, root.generated);
    assert_eq!(finding.affected.source_parents, Some(vec![42]));
    assert_eq!(report.topology, StageCoverage::Failed);
    assert_eq!(report.geometry, StageCoverage::Blocked);
    assert!(!report.complete());
    assert!(!owned.incidences_certified && owned.achieved_error.is_none());
}
