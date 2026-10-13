// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::RootBracket;
use super::{
    domain::DomainIsolation,
    refine::{ConstructedRoot, RootConstruction},
    types::*,
};
use crate::predicates::roots::certificate_arithmetic::world::HomogeneousCurve;

pub(super) struct ReportConstruction {
    source: BezierIncidenceSource,
    domain: DomainIsolation,
    curve: HomogeneousCurve,
    in_domain: Vec<bool>,
    bisections: u16,
    contacts: Vec<CertifiedBezierIncidence>,
}
impl ReportConstruction {
    pub(super) fn new(
        query: BezierIncidenceQuery<'_>,
        source: BezierIncidenceSource,
        domain: DomainIsolation,
    ) -> Self {
        let in_domain = classifications(query.domain, &domain);
        let curve = HomogeneousCurve::from_validated(query.curve);
        let bisections = domain.isolation.bisections();
        Self {
            source,
            domain,
            curve,
            in_domain,
            bisections,
            contacts: Vec::new(),
        }
    }
    pub(super) fn construct(
        mut self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<BezierIncidenceReport, BezierIncidenceFailure> {
        for index in 0..self.in_domain.len() {
            if !self.in_domain[index] {
                continue;
            }
            self.construct_contact(index, cancelled)?;
        }
        Ok(self.into_report())
    }
    fn construct_contact(
        &mut self,
        index: usize,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), BezierIncidenceFailure> {
        let constructor = RootConstruction::new(
            &self.curve,
            &self.domain.normal,
            self.source.root_options,
            self.source.requested_world_error,
        );
        let root = self.domain.isolation.roots()[index].clone();
        let constructed = constructor
            .construct(root, &mut self.bisections, cancelled)
            .map_err(|(cause, root)| self.failed_construction(cause, *root))?;
        self.contacts.push(contact(index, constructed));
        Ok(())
    }
    fn failed_construction(
        &self,
        cause: BezierIncidenceCause,
        root: RootBracket,
    ) -> BezierIncidenceFailure {
        super::failure(
            self.source.clone(),
            cause,
            Some(self.domain.isolation.clone()),
            Some(root),
        )
    }
    fn into_report(self) -> BezierIncidenceReport {
        BezierIncidenceReport {
            source: self.source,
            degrees: self.domain.degrees,
            denominator_isolation: self.domain.denominator_isolation,
            support_isolation: self.domain.isolation,
            in_domain: self.in_domain,
            contacts: self.contacts,
            bisections: self.bisections,
        }
    }
}
fn classifications(policy: BezierLineDomain, domain: &DomainIsolation) -> Vec<bool> {
    use std::cmp::Ordering::{Greater, Less};
    if policy == BezierLineDomain::Support {
        return vec![true; domain.isolation.roots().len()];
    }
    domain
        .origin_signs
        .iter()
        .zip(&domain.end_signs)
        .map(|(origin, end)| *origin != Less && *end != Greater)
        .collect()
}
fn contact(index: usize, constructed: ConstructedRoot) -> CertifiedBezierIncidence {
    CertifiedBezierIncidence {
        support_root_index: index,
        parameter: constructed.root,
        world_bounds: constructed.envelope.bounds,
        point: constructed.envelope.point,
        achieved_world_error: constructed.envelope.achieved_error,
    }
}
