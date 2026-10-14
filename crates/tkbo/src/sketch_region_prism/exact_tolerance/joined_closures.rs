// SPDX-License-Identifier: GPL-2.0-only
//! Semantic source contexts retain original maps and both directed raw operands.
use super::{closure_scalar::certify_exact_source_point, joined_progress::*, joined_types::*};
use crate::sketch_region_prism::frozen::proof_join::{JoinedClosurePointOperand, JoinedPrismProof};
use crate::sketch_region_source_conformance::same_embedding;
use rustycad_tkg3d::top_abs::Orientation;

pub(super) fn certify_source_closures(
    join: &JoinedPrismProof,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    count(
        "all semantic raw closure contexts",
        join.raw_closure_points().len(),
        12,
    )?;
    for stage in [
        "original",
        "census",
        "partition",
        "selected",
        "complement",
        "copied",
    ] {
        work.poll(progress)?;
        let rows = closure_rows(join.raw_closure_points(), stage)?;
        append_closure(join, rows, work)?;
    }
    Ok(())
}

pub(super) fn closure_rows<'a>(
    raw: &'a [JoinedClosurePointOperand],
    stage: &'static str,
) -> ConformanceCheck<[&'a JoinedClosurePointOperand; 2]> {
    let rows: Vec<_> = raw.iter().filter(|row| row.stage == stage).collect();
    if rows.len() != 2 {
        return Err(JoinedConformanceCause::ClosureCensus {
            stage,
            actual: rows.len(),
            expected: 2,
        });
    }
    let forward = rows
        .iter()
        .find(|row| row.orientation == Orientation::Forward);
    let reversed = rows
        .iter()
        .find(|row| row.orientation == Orientation::Reversed);
    let (Some(forward), Some(reversed)) = (forward, reversed) else {
        return Err(missing(
            "one forward and one reversed semantic closure context",
        ));
    };
    equivalent_closures(forward, reversed)?;
    Ok([*forward, *reversed])
}

fn equivalent_closures(
    first: &JoinedClosurePointOperand,
    last: &JoinedClosurePointOperand,
) -> ConformanceCheck {
    if first.contact != last.contact
        || first.original != last.original
        || first.local_point.map(f64::to_bits) != last.local_point.map(f64::to_bits)
        || !same_embedding(&first.placement, &last.placement)
    {
        return Err(JoinedConformanceCause::ClosureOperands {
            stage: first.stage,
            contact: first.contact,
            original: first.original.clone(),
        });
    }
    Ok(())
}

fn append_closure(
    join: &JoinedPrismProof,
    rows: [&JoinedClosurePointOperand; 2],
    work: &mut ConformanceWork,
) -> ConformanceCheck {
    let raw = rows[0];
    let query = join.closure_query();
    let certificate = certify_exact_source_point(
        query.source_query(),
        query.bounds.clone(),
        raw.local_point,
        raw.placement,
    )
    .map_err(|cause| JoinedConformanceCause::Closure {
        contact: raw.contact,
        original: raw.original.clone(),
        stage: raw.stage,
        cause: Box::new(cause),
    })?;
    work.evidence.closures.push(JoinedClosureEvidence {
        contact: raw.contact,
        original: raw.original.clone(),
        stage: raw.stage,
        orientations: rows.iter().map(|row| row.orientation).collect(),
        local_point: raw.local_point,
        certificate,
    });
    Ok(())
}
