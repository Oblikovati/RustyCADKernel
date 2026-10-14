// SPDX-License-Identifier: GPL-2.0-only
//! Actual region construction, copy and sweep supply every domain and trim operand.
use super::super::super::annular_binding::tests::actual_binding;
use super::super::{
    annular_coverage::check_annular_pair_keys, annular_regularity::*, annular_table::*,
};
use super::*;
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions, metric_bounds::BezierRegularityOptions,
};

#[test]
fn actual_annular_domains_keep_both_cap_rims_and_both_source_specific_sides() {
    let binding = actual_binding();
    let domains = capture_annular_domains(&binding).unwrap();
    assert_eq!(domains.first.trims.len(), 2);
    assert_eq!(domains.last.trims.len(), 2);
    assert_eq!(domains.outer.trims.len(), 4);
    assert_eq!(domains.inner.trims.len(), 4);
    assert_ne!(domains.outer.identity, domains.inner.identity);
    assert_eq!(domains.outer.identity.role, domains.inner.identity.role);
    assert_cap_kind(&domains.first, &binding, 0.);
    assert_cap_kind(&domains.last, &binding, 1.);
    assert_side_kind(&domains.outer, binding.region().nesting().outer);
    assert_side_kind(&domains.inner, binding.region().nesting().inner);
}
fn assert_cap_kind(domain: &AnnularFaceDomain, binding: &AnnularPrismBinding, sweep: f64) {
    let nesting = binding.region().nesting();
    assert_eq!(
        domain.kind,
        AnnularDomainKind::Cap {
            region: binding.region().selected_candidate_id(),
            outer: nesting.outer,
            inner: nesting.inner,
            normalized_sweep: sweep,
        }
    );
}
fn assert_side_kind(domain: &AnnularFaceDomain, expected_source: RegionSourceId) {
    let AnnularDomainKind::Side {
        source,
        boundary,
        native,
        stored_sweep,
        normalized_sweep,
        height,
    } = &domain.kind
    else {
        panic!("actual generated side domain")
    };
    assert_eq!(*source, expected_source);
    assert_eq!(*boundary, domain.identity.source);
    assert_eq!(*native, [0., 1.]);
    assert_eq!(*stored_sweep, [-2., 0.]);
    assert_eq!(*normalized_sweep, [0., 1.]);
    assert_eq!(*height, 2.);
}
#[test]
fn actual_four_domains_produce_ten_exactly_covered_pair_claims() {
    let domains = capture_annular_domains(&actual_binding()).unwrap();
    let rows = ten_rows(&domains).unwrap();
    let keys = [
        &domains.first,
        &domains.last,
        &domains.outer,
        &domains.inner,
    ]
    .map(|d| d.identity.clone());
    let pairs: Vec<_> = rows.iter().map(|r| (&r.first, &r.second)).collect();
    assert_eq!(check_annular_pair_keys(&keys, &pairs), Ok(()));
    assert_eq!(claim_counts(&rows), [2, 1, 1, 6]);
    assert_trim_claims(&rows);
}
fn claim_counts(rows: &[AnnularFaceIncidence]) -> [usize; 4] {
    let mut counts = [0; 4];
    for row in rows {
        let slot = match row.conclusion {
            AnnularIncidenceConclusion::InjectiveAnnularDomain => 0,
            AnnularIncidenceConclusion::DisjointCaps => 1,
            AnnularIncidenceConclusion::DisjointCertifiedBoundaries => 2,
            AnnularIncidenceConclusion::TrimIncidence { .. } => 3,
        };
        counts[slot] += 1;
    }
    counts
}
fn assert_trim_claims(rows: &[AnnularFaceIncidence]) {
    let trims: Vec<_> = rows
        .iter()
        .filter_map(|row| match &row.conclusion {
            AnnularIncidenceConclusion::TrimIncidence {
                first,
                second,
                conclusion,
            } => Some((first, second, conclusion)),
            _ => None,
        })
        .collect();
    assert_eq!(trims.len(), 6);
    for (first, second, conclusion) in trims {
        assert_directed_incidence(first, second, conclusion);
    }
}
fn assert_directed_incidence(
    first: &AnnularAuthoredTrim,
    second: &AnnularAuthoredTrim,
    conclusion: &super::super::PrismIncidenceConclusion,
) {
    assert_ne!(first.identity.output, second.identity.output);
    assert_eq!(first.identity.output.occurrence, first.operand.output);
    assert_eq!(second.identity.output.occurrence, second.operand.output);
    assert!(matches!(
        conclusion,
        super::super::PrismIncidenceConclusion::ExactlyRim { .. }
            | super::super::PrismIncidenceConclusion::InjectiveModuloSeam { .. }
    ));
}
#[test]
fn actual_source_regularity_uses_certified_roles_and_exact_copied_coefficients() {
    let binding = actual_binding();
    let operands = bound_regularity_operands(&binding).unwrap();
    assert_eq!(
        operands.each_ref().map(|o| o.source),
        [RegionSourceId(101), RegionSourceId(202)]
    );
    let requested = AnnularRegularityOptions {
        boundary: BezierRegularityOptions::v1(RootIsolationOptions::new(1e-12)),
        total_bisections: 0,
    };
    let report = certify_boundaries(operands, requested, &mut || false)
        .unwrap_or_else(|_| panic!("both polynomial fixture boundaries regular"));
    assert_eq!(report.total_bisections, 0);
    assert_eq!(
        report.boundaries.each_ref().map(|r| r.source),
        [RegionSourceId(101), RegionSourceId(202)]
    );
}
