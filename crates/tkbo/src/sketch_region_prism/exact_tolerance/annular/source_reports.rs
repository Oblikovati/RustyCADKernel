// SPDX-License-Identifier: GPL-2.0-only
//! Reused source bounds retain their original requested/effective policy and actual work.
use super::{progress::AnnularExactWork, *};
use crate::sketch_region_source_conformance::ConstructionStage;
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;
pub(super) fn retain(
    source: &TwoLoopSourceProof,
    work: &mut AnnularExactWork,
) -> AnnularExactCheck {
    use ConstructionStage::*;
    count(
        "source exact conformance stage roster",
        source.stages.len(),
        3,
    )?;
    for expected in [OriginalPlane, CopiedPlane, CapturedBasis] {
        work.source_context = Some(report_context(source.name, expected));
        let rows: Vec<_> = source
            .stages
            .iter()
            .filter(|row| row.stage == expected)
            .collect();
        let row = *one(&rows, "one exact source conformance stage")?;
        require_zero(row)?;
        work.upstream
            .source_reports
            .push(owned_report(source.name, row));
    }
    Ok(())
}
fn require_zero(row: &TwoLoopSourceStage) -> AnnularExactCheck {
    if !matches!(&row.residual, ResidualDecision::Within { proven_squared_upper }
        if proven_squared_upper.compare_f64(0.).is_ok_and(|order| order.is_eq()))
    {
        return Err(AnnularConformanceCause::SourceReport {
            stage: row.stage,
            actual: row.residual.clone(),
        });
    }
    Ok(())
}
fn owned_report(
    source: crate::sketch_region::RegionSourceId,
    row: &TwoLoopSourceStage,
) -> AnnularSourceReport {
    AnnularSourceReport {
        source,
        stage: row.stage,
        requested_limit: row.requested_limit,
        effective_limit: row.effective_limit,
        effective_roots: row.effective_roots,
        bisections: row.bisections,
        residual: row.residual.clone(),
    }
}

fn report_context(
    source: crate::sketch_region::RegionSourceId,
    expected: ConstructionStage,
) -> AnnularSourceContext {
    use ConstructionStage::OriginalPlane;
    AnnularSourceContext {
        source,
        native: None,
        stage: if expected == OriginalPlane {
            AnnularClosureStage::Original
        } else {
            AnnularClosureStage::Census
        },
    }
}
