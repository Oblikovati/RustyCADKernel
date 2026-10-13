// SPDX-License-Identifier: GPL-2.0-only
//! Missing/failed checker coverage is not a geometric defect certificate.
use super::outcome::{StageCoverage, ValidationCause};
use rustycad_tktopalgo::brep_check::{BRepCheckStatus, ListOfStatus};

pub(super) fn status_cause(statuses: ListOfStatus) -> Option<(StageCoverage, ValidationCause)> {
    if statuses.is_empty() {
        return Some((StageCoverage::Incomplete, ValidationCause::EmptyStatuses));
    }
    let defects: Vec<BRepCheckStatus> = statuses
        .iter()
        .copied()
        .filter(|status| *status != BRepCheckStatus::NoError)
        .collect();
    if defects.is_empty() {
        return None;
    }
    let coverage = if defects.contains(&BRepCheckStatus::CheckFail) {
        StageCoverage::Incomplete
    } else {
        StageCoverage::Failed
    };
    Some((coverage, ValidationCause::NativeStatuses(defects)))
}
