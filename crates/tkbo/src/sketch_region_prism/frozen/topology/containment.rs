// SPDX-License-Identifier: GPL-2.0-only
use super::super::*;
use super::{domain::FaceDomainPolicy, execution::TopologyWork, model::*, types::*};
use std::collections::BTreeMap;

pub(super) fn check(
    index: &TopologyIndex<'_>,
    domain: &impl FaceDomainPolicy,
    work: &mut TopologyWork<'_>,
) -> TopologyResult<FrozenPrismKey> {
    let mut references = BTreeMap::new();
    let mut definitions = BTreeMap::new();
    for (position, occurrence) in index.occurrences.values().enumerate() {
        work.poll(
            TopologyInvariant::ClosedContainment,
            index.occurrences.len() + position + 1,
        )?;
        children(index, occurrence, &mut references, domain)?;
        *definitions.entry(definition(&occurrence.key)?).or_insert(0) += 1;
    }
    definition_counts(index, &definitions)?;
    references_count(index, &references)?;
    root(index)
}
fn root(index: &TopologyIndex<'_>) -> TopologyResult<FrozenPrismKey> {
    let roots: Vec<_> = index
        .definitions
        .iter()
        .filter(|key| key.kind == 2)
        .copied()
        .collect();
    count(None, roots.len(), 1, "exactly one solid root definition")?;
    let shells = index.definitions.iter().filter(|key| key.kind == 3).count();
    count(None, shells, 1, "exactly one shell definition")?;
    Ok(roots[0])
}
fn children(
    index: &TopologyIndex<'_>,
    parent: &FrozenPrismUse,
    references: &mut BTreeMap<FrozenUseKey, usize>,
    domain: &impl FaceDomainPolicy,
) -> TopologyResult<()> {
    let kind = definition(&parent.key)?.kind;
    if kind == 4 {
        domain.check_face_children(parent, index)?;
    } else {
        child_count(parent, kind)?;
    }
    for key in &parent.children {
        let child = index.occurrence(key)?;
        direct_child(parent, key, kind)?;
        *references.entry(child.key.clone()).or_insert(0) += 1;
    }
    Ok(())
}
fn direct_child(parent: &FrozenPrismUse, key: &FrozenUseKey, kind: u8) -> TopologyResult<()> {
    if key.0.len() != parent.key.0.len() + 1
        || key.0[..key.0.len() - 1] != parent.key.0
        || definition(key)?.kind != kind + 1
    {
        return Err(invalid(
            Some(key),
            TopologyActual::Occurrence(parent.key.clone()),
            "direct child ancestry with next Solid..Vertex kind",
        ));
    }
    Ok(())
}
fn child_count(parent: &FrozenPrismUse, kind: u8) -> TopologyResult<()> {
    let actual = parent.children.len();
    let expected = match kind {
        2 => Some(1),
        6 => Some(2),
        7 => Some(0),
        _ => None,
    };
    if let Some(expected) = expected {
        return count(Some(&parent.key), actual, expected, child_shape(kind));
    }
    if actual == 0 || actual > super::super::domain::MAXIMUM_USES {
        return Err(invalid(
            Some(&parent.key),
            TopologyActual::Count(actual),
            "shell/wire with 1..=4096 direct children",
        ));
    }
    Ok(())
}
fn child_shape(kind: u8) -> &'static str {
    match kind {
        2 | 4 => "one shell per solid / one wire per face",
        6 => "two directed endpoint uses per edge occurrence",
        _ => "no vertex children",
    }
}
fn definition_counts(
    index: &TopologyIndex<'_>,
    counts: &BTreeMap<FrozenPrismKey, usize>,
) -> TopologyResult<()> {
    for key in &index.definitions {
        let actual = counts.get(key).copied().unwrap_or(0);
        if actual == 0 || (key.kind < 6 && actual != 1) {
            return Err(keys(
                vec![*key],
                "reachable definition, exactly one occurrence for each container",
            ));
        }
    }
    Ok(())
}
fn references_count(
    index: &TopologyIndex<'_>,
    references: &BTreeMap<FrozenUseKey, usize>,
) -> TopologyResult<()> {
    for key in index.occurrences.keys() {
        let root = key.0.len() == 1;
        let expected = usize::from(!root);
        count(
            Some(key),
            references.get(key).copied().unwrap_or(0),
            expected,
            "zero parent references for root; exactly one for every other occurrence",
        )?;
    }
    Ok(())
}
