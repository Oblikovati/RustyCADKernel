// SPDX-License-Identifier: GPL-2.0-only
//! Roles select existing authored names; no order, address or coordinate creates a name.
use super::*;
pub(super) fn unique<'a, T>(rows: &[&'a T], subject: &'static str) -> JoinedDirectionCheck<&'a T> {
    count(subject, rows.len(), 1)?;
    let row = rows[0];
    Ok(row)
}
pub(super) fn actual_direction(
    projection: &FrozenPrismProjection,
    key: &FrozenUseKey,
    expected: Orientation,
) -> JoinedDirectionCheck<Orientation> {
    let rows: Vec<_> = projection
        .uses()
        .iter()
        .filter(|row| &row.key == key)
        .collect();
    let actual = unique(&rows, "one named cumulative use")?.orientation;
    if actual != expected {
        return Err(JoinedPrismOrientationCause::Direction { actual, expected });
    }
    Ok(actual)
}
pub(super) fn dispositions(
    join: &JoinedPrismProof,
    key: &FrozenUseKey,
) -> JoinedDirectionCheck<JoinedParentDispositions> {
    let rows: Vec<_> = join
        .parent_dispositions()
        .iter()
        .filter(|row| &row.occurrence == key)
        .collect();
    Ok(unique(&rows, "one complete named parent disposition")?.clone())
}
pub(super) fn parent_count(actual: usize, expected: usize) -> JoinedDirectionCheck<()> {
    if actual == expected {
        return Ok(());
    }
    Err(JoinedPrismOrientationCause::ParentMultiplicity { actual, expected })
}
pub(super) fn interval(actual: [f64; 2], expected: [f64; 2]) -> JoinedDirectionCheck<()> {
    let actual = actual.map(f64::to_bits);
    let expected = expected.map(f64::to_bits);
    if actual == expected {
        return Ok(());
    }
    Err(JoinedPrismOrientationCause::Interval { actual, expected })
}

pub(super) fn distinct_uses(
    mut keys: Vec<&FrozenUseKey>,
    expected: usize,
) -> JoinedDirectionCheck<()> {
    keys.sort();
    keys.dedup();
    count("distinct authored directed contexts", keys.len(), expected)
}
