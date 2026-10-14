// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn name_trims(
    join: &JoinedPrismProof,
    face: &JoinedPrismFace,
    proof: &PrismFaceOrientation,
) -> JoinedDirectionCheck<Vec<JoinedTrimDirection>> {
    count(
        "all authored trims of named face",
        face.trims.len(),
        proof.trims().len(),
    )?;
    let mut result = Vec::new();
    for expected in proof.trims() {
        let trim = named_trim(face, expected.authored().role)?;
        result.push(named(&trim.occurrence, name_trim(join, trim, expected))?);
    }
    result.sort_by(|a, b| a.named.occurrence.cmp(&b.named.occurrence));
    Ok(result)
}
fn name_trim(
    join: &JoinedPrismProof,
    trim: &JoinedPrismTrim,
    proof: &PrismTrimOrientation,
) -> JoinedDirectionCheck<JoinedTrimDirection> {
    naming::parent_count(trim.parents.len(), proof.authored().parents.len())?;
    naming::interval(trim.interval, proof.authored().interval)?;
    let (observed, expected) = proof.orientations();
    let actual = naming::actual_direction(join.projection(), &trim.occurrence, observed)?;
    Ok(JoinedTrimDirection {
        named: trim.clone(),
        parents: naming::dispositions(join, &trim.occurrence)?,
        actual,
        expected,
    })
}

pub(super) fn named_trim(
    face: &JoinedPrismFace,
    role: crate::sketch_region_prism::surface_charts::PrismTrimRole,
) -> JoinedDirectionCheck<&JoinedPrismTrim> {
    let rows: Vec<_> = face.trims.iter().filter(|row| row.role == role).collect();
    naming::unique(&rows, "one authored trim for theorem role")
}
