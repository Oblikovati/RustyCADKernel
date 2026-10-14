// SPDX-License-Identifier: GPL-2.0-only
//! Lookup consumes existing authored full paths; neither row order nor shared role creates identity.
use super::*;
pub(super) fn observed(
    join: &AnnularFrozenDomain,
    key: &FrozenUseKey,
) -> AnnularDirectionCheck<Orientation> {
    let rows: Vec<_> = join
        .projection()
        .uses()
        .iter()
        .filter(|row| &row.key == key)
        .collect();
    count("one complete named directed occurrence", rows.len(), 1)?;
    let actual = rows[0].orientation;
    if !matches!(actual, Orientation::Forward | Orientation::Reversed) {
        return Err(AnnularDirectionCause::Unsupported {
            occurrence: Box::new(key.clone()),
            actual,
        });
    }
    Ok(actual)
}
pub(super) fn check_direction(
    join: &AnnularFrozenDomain,
    key: &FrozenUseKey,
    expected: Orientation,
) -> AnnularDirectionCheck<Orientation> {
    let actual = observed(join, key)?;
    if actual != expected {
        return Err(AnnularDirectionCause::Direction {
            occurrence: Box::new(key.clone()),
            actual,
            expected,
        });
    }
    Ok(actual)
}
pub(super) fn parents(
    join: &AnnularFrozenDomain,
    key: &FrozenUseKey,
) -> AnnularDirectionCheck<JoinedParentDispositions> {
    let rows: Vec<_> = join
        .parents()
        .iter()
        .filter(|row| &row.occurrence == key)
        .collect();
    count("one complete named parent-class record", rows.len(), 1)?;
    Ok(rows[0].clone())
}
pub(super) fn within_wire(key: &FrozenUseKey, wire: &FrozenUseKey) -> bool {
    key.0.len() == wire.0.len() + 1 && key.0.starts_with(&wire.0)
}
pub(super) fn distinct(mut keys: Vec<&FrozenUseKey>, expected: usize) -> AnnularDirectionCheck<()> {
    count("complete directed orientation rows", keys.len(), expected)?;
    keys.sort();
    keys.dedup();
    count(
        "distinct full directed orientation paths",
        keys.len(),
        expected,
    )
}
