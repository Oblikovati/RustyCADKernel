// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn name_faces(
    join: &JoinedPrismProof,
    evaluated: &PrismOrientationConclusions,
) -> JoinedDirectionCheck<Vec<JoinedFaceDirection>> {
    count("three named chart domains", join.faces().len(), 3)?;
    let mut result = Vec::new();
    for proof in evaluated.faces() {
        let face = named_face(join.faces(), proof.domain().role)?;
        result.push(named(&face.occurrence, name_face(join, face, proof))?);
    }
    result.sort_by(|a, b| a.named.occurrence.cmp(&b.named.occurrence));
    require_distinct_contexts(&result)?;
    Ok(result)
}
fn name_face(
    join: &JoinedPrismProof,
    face: &JoinedPrismFace,
    proof: &PrismFaceOrientation,
) -> JoinedDirectionCheck<JoinedFaceDirection> {
    naming::parent_count(face.parents.len(), proof.domain().parents.len())?;
    let (observed, expected) = proof.orientations();
    let actual = naming::actual_direction(join.projection(), &face.occurrence, observed)?;
    let trims = trims::name_trims(join, face, proof)?;
    Ok(JoinedFaceDirection {
        named: canonical_face(face),
        parents: naming::dispositions(join, &face.occurrence)?,
        actual,
        expected,
        trims,
    })
}

fn canonical_face(face: &JoinedPrismFace) -> JoinedPrismFace {
    let mut named = face.clone();
    named.trims.sort_by(|a, b| a.occurrence.cmp(&b.occurrence));
    named
}

fn require_distinct_contexts(rows: &[JoinedFaceDirection]) -> JoinedDirectionCheck<()> {
    naming::distinct_uses(rows.iter().map(|row| &row.named.occurrence).collect(), 3)?;
    let trims = rows
        .iter()
        .flat_map(|row| &row.trims)
        .map(|trim| &trim.named.occurrence)
        .collect();
    naming::distinct_uses(trims, 6)
}

pub(super) fn named_face(
    faces: &[JoinedPrismFace],
    role: crate::sketch_region_prism::PrismImageRole,
) -> JoinedDirectionCheck<&JoinedPrismFace> {
    let rows: Vec<_> = faces.iter().filter(|row| row.role == role).collect();
    naming::unique(&rows, "one named face for theorem role")
}
