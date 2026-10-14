// SPDX-License-Identifier: GPL-2.0-only
//! Full joined source/chart identities supply trim zeros; no sampling or new surface evaluator.
use super::{joined_progress::*, joined_types::*};
use crate::sketch_region_prism::{
    frozen::proof_join::*, frozen::*, surface_charts::PrismChartStage,
};
use rustycad_tkmath::predicates::roots::DyadicBound;
use std::collections::BTreeSet;

pub(super) fn certify_joined_trims(
    join: &JoinedPrismProof,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    count("complete named faces", join.faces().len(), 3)?;
    for face in join.faces() {
        append_face_trims(join, face, work, progress)?;
    }
    complete_trim_contexts(join, work)
}
fn complete_trim_contexts(join: &JoinedPrismProof, work: &ConformanceWork) -> ConformanceCheck {
    let contexts = trim_contexts(&work.evidence.trims);
    count(
        "complete directed trim proofs",
        work.evidence.trims.len(),
        6,
    )?;
    let coedges = &join.projection().incidence().coedges;
    let expected: BTreeSet<_> = coedges.iter().map(|row| row.occurrence.clone()).collect();
    if contexts.len() != 6 || contexts != expected {
        return Err(JoinedConformanceCause::Mismatch {
            quantity: "joined trim and actual coedge bijection",
        });
    }
    Ok(())
}
fn trim_contexts(rows: &[JoinedTrimEvidence]) -> BTreeSet<FrozenUseKey> {
    rows.iter().map(|row| row.occurrence.clone()).collect()
}

fn append_face_trims(
    join: &JoinedPrismProof,
    face: &JoinedPrismFace,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    work.entity = Some(face.definition);
    work.occurrence = Some(face.occurrence.clone());
    let actual = bound_face(join, face.definition)?;
    trim_identity_coverage(actual, face.trims.len())?;
    for trim in &face.trims {
        append_trim(actual, trim, work, progress)?;
    }
    Ok(())
}
fn bound_face(join: &JoinedPrismProof, key: FrozenPrismKey) -> ConformanceCheck<&FrozenPrismFace> {
    let faces: Vec<_> = join
        .projection()
        .faces()
        .iter()
        .filter(|row| row.key == key)
        .collect();
    one(&faces, "one captured face for joined trim identities").copied()
}
fn append_trim(
    actual: &FrozenPrismFace,
    trim: &JoinedPrismTrim,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    work.occurrence = Some(trim.occurrence.clone());
    work.poll(progress)?;
    let matching: Vec<_> = actual
        .trims
        .iter()
        .filter(|row| row.occurrence == trim.occurrence)
        .collect();
    one(&matching, "one actual trim for joined exact identity")?;
    work.evidence
        .trims
        .push(zero_trim(actual.key, trim.occurrence.clone()));
    Ok(())
}
fn zero_trim(face: FrozenPrismKey, occurrence: FrozenUseKey) -> JoinedTrimEvidence {
    JoinedTrimEvidence {
        face,
        occurrence,
        squared_upper: DyadicBound::from_f64(0.).expect("exact zero"),
    }
}

fn trim_identity_coverage(face: &FrozenPrismFace, expected: usize) -> ConformanceCheck {
    count(
        "actual pcurve identity coverage",
        stage_count(face, PrismChartStage::TrimPCurve),
        expected,
    )?;
    count(
        "actual trim role coverage",
        face.proof.trim_roles.len(),
        expected,
    )?;
    space_identity_coverage(stage_count(face, PrismChartStage::TrimSpaceCurve), expected)
}
fn space_identity_coverage(actual: usize, expected: usize) -> ConformanceCheck {
    if actual < expected {
        return Err(JoinedConformanceCause::Count {
            quantity: "actual space-curve identity coverage",
            actual,
            expected,
        });
    }
    Ok(())
}
fn stage_count(face: &FrozenPrismFace, stage: PrismChartStage) -> usize {
    face.proof
        .identities
        .iter()
        .filter(|actual| **actual == stage)
        .count()
}
