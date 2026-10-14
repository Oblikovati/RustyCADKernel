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
    let contexts: BTreeSet<_> = work
        .evidence
        .trims
        .iter()
        .map(|row| row.occurrence.clone())
        .collect();
    count(
        "complete directed trim proofs",
        work.evidence.trims.len(),
        6,
    )?;
    let expected: BTreeSet<_> = join
        .projection()
        .incidence()
        .coedges
        .iter()
        .map(|row| row.occurrence.clone())
        .collect();
    if contexts.len() != 6 || contexts != expected {
        return Err(JoinedConformanceCause::Mismatch {
            quantity: "joined trim and actual coedge bijection",
        });
    }
    Ok(())
}

fn append_face_trims(
    join: &JoinedPrismProof,
    face: &JoinedPrismFace,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    work.entity = Some(face.definition);
    work.occurrence = Some(face.occurrence.clone());
    let actual: Vec<_> = join
        .projection()
        .faces()
        .iter()
        .filter(|row| row.key == face.definition)
        .collect();
    let actual = *one(&actual, "one captured face for joined trim identities")?;
    trim_identity_coverage(actual, face.trims.len())?;
    for trim in &face.trims {
        work.occurrence = Some(trim.occurrence.clone());
        work.poll(progress)?;
        let matching: Vec<_> = actual
            .trims
            .iter()
            .filter(|row| row.occurrence == trim.occurrence)
            .collect();
        one(&matching, "one actual trim for joined exact identity")?;
        work.evidence.trims.push(JoinedTrimEvidence {
            face: face.definition,
            occurrence: trim.occurrence.clone(),
            squared_upper: DyadicBound::from_f64(0.).expect("exact zero"),
        });
    }
    Ok(())
}

fn trim_identity_coverage(face: &FrozenPrismFace, expected: usize) -> ConformanceCheck {
    let pcurves = face
        .proof
        .identities
        .iter()
        .filter(|stage| **stage == PrismChartStage::TrimPCurve)
        .count();
    let curves = face
        .proof
        .identities
        .iter()
        .filter(|stage| **stage == PrismChartStage::TrimSpaceCurve)
        .count();
    count("actual pcurve identity coverage", pcurves, expected)?;
    count(
        "actual trim role coverage",
        face.proof.trim_roles.len(),
        expected,
    )?;
    if curves < expected {
        return Err(JoinedConformanceCause::Count {
            quantity: "actual space-curve identity coverage",
            actual: curves,
            expected,
        });
    }
    Ok(())
}
