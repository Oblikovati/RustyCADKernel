// SPDX-License-Identifier: GPL-2.0-only
use super::*;

pub(super) fn admit(
    join: &JoinedPrismProof,
    requested: JoinedPrismMeasureOptions,
) -> Result<[JoinedPrismFace; 3], JoinedPrismMeasureCause> {
    admit_version(requested.version)?;
    let binding = join.embedding().binding();
    count("captured polynomial prism charts", binding.charts.len(), 3)?;
    count(
        "complete single Jordan boundary",
        binding.region.copied.boundaries.len(),
        1,
    )?;
    check_chart_roles(join)?;
    check_extrusion(join)?;
    name_triplet(join.faces())
}
pub(super) fn name_triplet(
    faces: &[JoinedPrismFace],
) -> Result<[JoinedPrismFace; 3], JoinedPrismMeasureCause> {
    count("named polynomial prism faces", faces.len(), 3)?;
    Ok([
        named(faces, PrismImageRole::First)?,
        named(faces, PrismImageRole::Last)?,
        named(faces, PrismImageRole::Generated)?,
    ])
}
fn named(
    faces: &[JoinedPrismFace],
    role: PrismImageRole,
) -> Result<JoinedPrismFace, JoinedPrismMeasureCause> {
    let names: Vec<_> = faces.iter().filter(|face| face.role == role).collect();
    count("one authored face for measured role", names.len(), 1)?;
    Ok(names[0].clone())
}
fn check_extrusion(join: &JoinedPrismProof) -> Result<(), JoinedPrismMeasureCause> {
    let expected = join.projection().limits().extrusion.map(f64::to_bits);
    for chart in &join.embedding().binding().charts {
        let actual = chart.captured.extrusion.map(f64::to_bits);
        if actual != expected {
            return Err(JoinedPrismMeasureCause::Extrusion {
                role: chart.role,
                actual,
                expected,
            });
        }
    }
    Ok(())
}
fn count(
    subject: &'static str,
    actual: usize,
    expected: usize,
) -> Result<(), JoinedPrismMeasureCause> {
    if actual != expected {
        return Err(JoinedPrismMeasureCause::Count {
            subject,
            actual,
            expected,
        });
    }
    Ok(())
}

fn check_chart_roles(join: &JoinedPrismProof) -> Result<(), JoinedPrismMeasureCause> {
    for role in [
        PrismImageRole::First,
        PrismImageRole::Last,
        PrismImageRole::Generated,
    ] {
        let count_for_role = join
            .embedding()
            .binding()
            .charts
            .iter()
            .filter(|chart| chart.role == role)
            .count();
        count("one captured chart for measured role", count_for_role, 1)?;
    }
    Ok(())
}

fn admit_version(actual: u32) -> Result<(), JoinedPrismMeasureCause> {
    if actual != 1 {
        return Err(JoinedPrismMeasureCause::Version {
            actual,
            expected: 1,
        });
    }
    Ok(())
}
