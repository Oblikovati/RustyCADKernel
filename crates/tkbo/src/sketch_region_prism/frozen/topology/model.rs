// SPDX-License-Identifier: GPL-2.0-only
//! Operation-local indexes borrow the existing immutable projection; no second graph owner.
use super::super::{graph::FrozenCoedgeIncidence, *};
use super::types::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct TopologyIndex<'a> {
    pub(super) traversals: BTreeMap<FrozenUseKey, &'a [FrozenUseKey]>,
    pub(super) definitions: BTreeSet<FrozenPrismKey>,
    pub(super) occurrences: BTreeMap<FrozenUseKey, &'a FrozenPrismUse>,
    pub(super) coedges: BTreeMap<FrozenUseKey, &'a FrozenCoedgeIncidence>,
    pub(super) endpoints: BTreeMap<FrozenPrismKey, [FrozenPrismKey; 2]>,
}
impl<'a> TopologyIndex<'a> {
    pub(super) fn new() -> Self {
        Self {
            traversals: BTreeMap::new(),
            definitions: BTreeSet::new(),
            occurrences: BTreeMap::new(),
            coedges: BTreeMap::new(),
            endpoints: BTreeMap::new(),
        }
    }
    pub(super) fn occurrence(&self, key: &FrozenUseKey) -> TopologyResult<&'a FrozenPrismUse> {
        self.occurrences.get(key).copied().ok_or_else(|| {
            invalid(
                Some(key),
                TopologyActual::Occurrence(key.clone()),
                "one existing occurrence",
            )
        })
    }
}
pub(super) fn definition(key: &FrozenUseKey) -> TopologyResult<FrozenPrismKey> {
    key.0.last().map(|pair| pair.0).ok_or_else(|| {
        invalid(
            Some(key),
            TopologyActual::Count(0),
            "nonempty authored occurrence ancestry",
        )
    })
}
pub(super) fn invalid(
    occurrence: Option<&FrozenUseKey>,
    actual: TopologyActual,
    expected: &'static str,
) -> CheckFailure {
    CheckFailure::Invalid(witness(occurrence, actual, expected))
}
pub(super) fn witness(
    occurrence: Option<&FrozenUseKey>,
    actual: TopologyActual,
    expected: &'static str,
) -> TopologyWitness {
    TopologyWitness {
        entities: occurrence
            .map(|key| key.0.iter().map(|pair| pair.0).collect())
            .unwrap_or_default(),
        occurrence: occurrence.cloned(),
        actual,
        expected,
    }
}
pub(super) fn count(
    occurrence: Option<&FrozenUseKey>,
    actual: usize,
    expected: usize,
    shape: &'static str,
) -> TopologyResult<()> {
    if actual != expected {
        return Err(invalid(occurrence, TopologyActual::Count(actual), shape));
    }
    Ok(())
}
pub(super) fn keys(actual: Vec<FrozenPrismKey>, expected: &'static str) -> CheckFailure {
    let mut result = witness(None, TopologyActual::Keys(actual.clone()), expected);
    result.entities = actual;
    CheckFailure::Invalid(result)
}
