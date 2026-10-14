// SPDX-License-Identifier: GPL-2.0-only
//! Test-only domain assumptions. No region certificate or production topology can be issued here.
use super::super::*;
use super::{domain::FaceDomainPolicy, execution::TopologyWork, model::*, types::*};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub(super) enum AssumedParameterDomain {
    Disk(FrozenUseKey),
    Annulus {
        outer: FrozenUseKey,
        inner: FrozenUseKey,
    },
}
impl AssumedParameterDomain {
    fn boundaries(&self) -> Vec<&FrozenUseKey> {
        match self {
            Self::Disk(boundary) => vec![boundary],
            Self::Annulus { outer, inner } => vec![outer, inner],
        }
    }
    fn characteristic(&self) -> i64 {
        match self {
            Self::Disk(_) => 1,
            Self::Annulus { .. } => 0,
        }
    }
}

#[derive(Clone)]
pub(super) struct NamedAnnularAssumptions {
    pub(super) faces: BTreeMap<FrozenUseKey, AssumedParameterDomain>,
    source: Vec<u8>,
}
impl NamedAnnularAssumptions {
    pub(super) fn declared(source: &FrozenPrismProjection, cut_caps: bool) -> Self {
        let faces = (0..4)
            .map(|id| {
                let face = super::domain_fixture::authored_face(source, id);
                let domain = assumed_domain(face, id < 2 && !cut_caps);
                (face.key.clone(), domain)
            })
            .collect();
        Self {
            faces,
            source: source.replay_bytes(),
        }
    }
    fn binding(
        &self,
        source: &FrozenPrismProjection,
        index: &TopologyIndex<'_>,
    ) -> TopologyResult<()> {
        let faces: BTreeSet<_> = index
            .occurrences
            .keys()
            .filter(|key| key.0.len() == 3)
            .collect();
        if source.replay_bytes() != self.source || faces != self.faces.keys().collect() {
            return Err(CheckFailure::Unsupported(witness(
                None,
                TopologyActual::Keys(self.faces.keys().map(|key| key.0[2].0).collect()),
                "test assumptions bound to the identical full source and complete face domain",
            )));
        }
        Ok(())
    }
}
fn assumed_domain(face: &FrozenPrismUse, annular: bool) -> AssumedParameterDomain {
    if annular {
        return AssumedParameterDomain::Annulus {
            outer: face.children[0].clone(),
            inner: face.children[1].clone(),
        };
    }
    AssumedParameterDomain::Disk(face.children[0].clone())
}
impl FaceDomainPolicy for NamedAnnularAssumptions {
    fn check_face_children(
        &self,
        face: &FrozenPrismUse,
        _index: &TopologyIndex<'_>,
    ) -> TopologyResult<()> {
        let domain = self.faces.get(&face.key).ok_or_else(|| {
            CheckFailure::Unsupported(witness(
                Some(&face.key),
                TopologyActual::Count(0),
                "one explicitly assumed test face domain",
            ))
        })?;
        check_boundary_set(face, &domain.boundaries())
    }

    fn check_euler(
        &self,
        source: &FrozenPrismProjection,
        index: &TopologyIndex<'_>,
        work: &mut TopologyWork<'_>,
    ) -> TopologyResult<()> {
        work.poll(TopologyInvariant::FaceDomainEuler, 0)?;
        self.binding(source, index)?;
        check_characteristic(source, self.faces.values())
    }
}

fn check_boundary_set(face: &FrozenPrismUse, expected: &[&FrozenUseKey]) -> TopologyResult<()> {
    let actual: BTreeSet<_> = face.children.iter().collect();
    if actual != expected.iter().copied().collect() || actual.len() != expected.len() {
        return Err(invalid(
            Some(&face.key),
            TopologyActual::Occurrence(face.key.clone()),
            "bijection of the complete directed wire domain to the test assumptions",
        ));
    }
    Ok(())
}
fn check_characteristic<'a>(
    source: &FrozenPrismProjection,
    domains: impl Iterator<Item = &'a AssumedParameterDomain>,
) -> TopologyResult<()> {
    let face_sum: i64 = domains.map(AssumedParameterDomain::characteristic).sum();
    let characteristic =
        i64::from(source.graph.counts[0]) - i64::from(source.graph.counts[1]) + face_sum;
    if characteristic != 0 {
        return Err(invalid(
            None,
            TopologyActual::Euler(characteristic),
            "V-E+sum(parameter-domain Euler)=0 for the declared annular product test model",
        ));
    }
    Ok(())
}
