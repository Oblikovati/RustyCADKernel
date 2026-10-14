// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use std::collections::BTreeSet;

pub(super) struct BoundAnnularPolicy<'a>(pub(super) &'a AnnularFrozenDomain);
impl FaceDomainPolicy for BoundAnnularPolicy<'_> {
    fn check_face_children(
        &self,
        face: &FrozenPrismUse,
        _index: &TopologyIndex<'_>,
    ) -> TopologyResult<()> {
        let domain = self.face_domain(face)?;
        let expected = domain.boundary_uses();
        let actual: BTreeSet<_> = face.children.iter().collect();
        if face.children.len() != expected.len()
            || actual != expected.iter().copied().collect()
            || actual.len() != expected.len()
        {
            return Err(invalid(
                Some(&face.key),
                TopologyActual::Occurrence(face.key.clone()),
                "exact directed wire bijection to certified outer/inner or seam-cut disk domain",
            ));
        }
        Ok(())
    }
    fn check_euler(
        &self,
        source: &FrozenPrismProjection,
        index: &TopologyIndex<'_>,
        work: &mut TopologyWork<'_>,
    ) -> TopologyResult<()> {
        work.poll(TopologyInvariant::FaceDomainEuler, 0)?;
        self.face_universe(source, index)?;
        let sum: i64 = self
            .0
            .faces()
            .iter()
            .map(|f| i64::from(matches!(f.domain, NamedAnnularDomain::Side { .. })))
            .sum();
        let actual = i64::from(source.graph.counts[0]) - i64::from(source.graph.counts[1]) + sum;
        if actual != 0 {
            return Err(invalid(
                None,
                TopologyActual::Euler(actual),
                "V-E+sum(certified parameter-domain Euler)=0 for the actual annular sweep",
            ));
        }
        Ok(())
    }
}
impl BoundAnnularPolicy<'_> {
    fn face_domain(&self, face: &FrozenPrismUse) -> TopologyResult<&NamedAnnularDomain> {
        let found: Vec<_> = self
            .0
            .faces()
            .iter()
            .filter(|f| f.chart.occurrence == face.key)
            .collect();
        count(
            Some(&face.key),
            found.len(),
            1,
            "one full-context certified annular parameter domain",
        )?;
        Ok(&found[0].domain)
    }
    fn face_universe(
        &self,
        source: &FrozenPrismProjection,
        index: &TopologyIndex<'_>,
    ) -> TopologyResult<()> {
        let expected: BTreeSet<_> = self
            .0
            .faces()
            .iter()
            .map(|face| &face.chart.occurrence)
            .collect();
        let actual = indexed_faces(index);
        if expected != actual
            || expected.len() != self.0.faces().len()
            || source.replay_bytes() != self.0.projection().replay_bytes()
        {
            return Err(CheckFailure::Unsupported(witness(
                None,
                TopologyActual::Count(actual.len()),
                "complete same-projection face-domain bijection",
            )));
        }
        Ok(())
    }
}

fn indexed_faces<'a>(index: &'a TopologyIndex<'_>) -> BTreeSet<&'a FrozenUseKey> {
    index
        .occurrences
        .keys()
        .filter(|key| {
            key.0
                .last()
                .is_some_and(|(key, _)| key.kind == rustycad_tkg3d::top_abs::ShapeEnum::Face as u8)
        })
        .collect()
}
