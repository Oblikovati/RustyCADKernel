// SPDX-License-Identifier: GPL-2.0-only
//! Finite four-domain coverage only; geometric conclusions require the same bound annular proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AnnularPairCoverageFailure<K> {
    RowCount {
        actual: usize,
        expected: usize,
    },
    RepeatedDomain {
        key: Box<K>,
        actual: usize,
        expected: usize,
    },
    ForeignPair {
        first: Box<K>,
        second: Box<K>,
    },
    PairMultiplicity {
        first: Box<K>,
        second: Box<K>,
        actual: usize,
        expected: usize,
    },
}

/// Check exact context-key multiplicities; `check_annular_pair_keys(&keys,&pairs)` adds no geometry authority.
/// Four domains and ten rows bound all work; the enclosing operation polls before and after this callback-free check.
pub(super) fn check_annular_pair_keys<K: Clone + Eq>(
    domains: &[K; 4],
    pairs: &[(&K, &K)],
) -> Result<(), AnnularPairCoverageFailure<K>> {
    if pairs.len() != 10 {
        return Err(AnnularPairCoverageFailure::RowCount {
            actual: pairs.len(),
            expected: 10,
        });
    }
    unique_domain_keys(domains)?;
    known_pair_keys(domains, pairs)?;
    for (offset, first) in domains.iter().enumerate() {
        for second in &domains[offset..] {
            unique_pair_key(pairs, first, second)?;
        }
    }
    Ok(())
}

fn unique_domain_keys<K: Clone + Eq>(
    domains: &[K; 4],
) -> Result<(), AnnularPairCoverageFailure<K>> {
    for key in domains {
        let actual = domains.iter().filter(|row| *row == key).count();
        if actual != 1 {
            return Err(AnnularPairCoverageFailure::RepeatedDomain {
                key: Box::new(key.clone()),
                actual,
                expected: 1,
            });
        }
    }
    Ok(())
}

fn known_pair_keys<K: Clone + Eq>(
    domains: &[K; 4],
    pairs: &[(&K, &K)],
) -> Result<(), AnnularPairCoverageFailure<K>> {
    for (first, second) in pairs {
        if !domains.contains(first) || !domains.contains(second) {
            return Err(AnnularPairCoverageFailure::ForeignPair {
                first: Box::new((*first).clone()),
                second: Box::new((*second).clone()),
            });
        }
    }
    Ok(())
}

fn unique_pair_key<K: Clone + Eq>(
    pairs: &[(&K, &K)],
    first: &K,
    second: &K,
) -> Result<(), AnnularPairCoverageFailure<K>> {
    let actual = pairs
        .iter()
        .filter(|(a, b)| (*a == first && *b == second) || (*a == second && *b == first))
        .count();
    if actual != 1 {
        return Err(AnnularPairCoverageFailure::PairMultiplicity {
            first: Box::new(first.clone()),
            second: Box::new(second.clone()),
            actual,
            expected: 1,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
