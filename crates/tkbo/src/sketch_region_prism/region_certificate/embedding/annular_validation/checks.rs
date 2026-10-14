// SPDX-License-Identifier: GPL-2.0-only
use super::*;
pub(super) fn domains(actual: &AnnularDomains, expected: &AnnularDomains) -> TableCheck {
    let observed = domain_rows(actual);
    for row in domain_rows(expected) {
        domain(&observed, row)?;
    }
    Ok(())
}
pub(super) fn domain_rows(domains: &AnnularDomains) -> [&AnnularFaceDomain; 4] {
    [
        &domains.first,
        &domains.last,
        &domains.outer,
        &domains.inner,
    ]
}
fn domain(actual: &[&AnnularFaceDomain; 4], expected: &AnnularFaceDomain) -> TableCheck {
    let found: Vec<_> = actual
        .iter()
        .filter(|row| row.identity == expected.identity)
        .collect();
    if found.len() != 1 {
        return Err(AnnularTableFailure::DomainMultiplicity {
            key: Box::new(expected.identity.clone()),
            actual: found.len(),
            expected: 1,
        });
    }
    if **found[0] != *expected {
        return Err(AnnularTableFailure::DomainCorrespondence {
            actual: Box::new((**found[0]).clone()),
            expected: Box::new(expected.clone()),
        });
    }
    Ok(())
}
pub(super) fn rows(
    domains: &AnnularDomains,
    actual: &[AnnularFaceIncidence],
    cancelled: &mut impl FnMut(usize) -> bool,
) -> TableCheck {
    poll(cancelled, 1)?;
    let expected = ten_rows(domains).map_err(reconstruction)?;
    let keys = domain_rows(domains).map(|row| row.identity.clone());
    let pairs: Vec<_> = actual.iter().map(|row| (&row.first, &row.second)).collect();
    check_annular_pair_keys(&keys, &pairs).map_err(AnnularTableFailure::Coverage)?;
    for (index, row) in expected.iter().enumerate() {
        poll(cancelled, index + 2)?;
        pair(actual, row)?;
    }
    poll(cancelled, expected.len() + 2)
}
fn pair(actual: &[AnnularFaceIncidence], expected: &AnnularFaceIncidence) -> TableCheck {
    // Complete multiplicity was checked before this lookup; order never supplies identity.
    let row = actual
        .iter()
        .find(|row| same_pair(row, expected))
        .expect("checked full pair census");
    if row != expected {
        return Err(AnnularTableFailure::Correspondence {
            actual: Box::new(row.clone()),
            expected: Box::new(expected.clone()),
        });
    }
    Ok(())
}
fn same_pair(actual: &AnnularFaceIncidence, expected: &AnnularFaceIncidence) -> bool {
    (actual.first == expected.first && actual.second == expected.second)
        || (actual.first == expected.second && actual.second == expected.first)
}
pub(super) fn poll(cancelled: &mut impl FnMut(usize) -> bool, completed: usize) -> TableCheck {
    if cancelled(completed) {
        return Err(AnnularTableFailure::Cancelled { completed });
    }
    Ok(())
}
