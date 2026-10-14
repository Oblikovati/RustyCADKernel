// SPDX-License-Identifier: GPL-2.0-only
use super::super::annular_fixture::{project, with_bound};
use super::*;
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions, metric_bounds::BezierRegularityOptions,
};

pub(super) fn embedding_options() -> AnnularEmbeddingOptions {
    AnnularEmbeddingOptions {
        version: 2,
        regularity: AnnularRegularityOptions {
            boundary: BezierRegularityOptions::v1(RootIsolationOptions::new(1e-12)),
            total_bisections: 0,
        },
    }
}
pub(super) fn options() -> AnnularFrozenOptions {
    AnnularFrozenOptions {
        version: 2,
        maximum_name_units: 1024,
    }
}
#[test]
fn named_annular_domains_bind_both_same_role_cap_rims_and_actual_side_seams() {
    with_bound(|request, receipt, authored, binding| {
        assert!(names::inspect(request, &binding, authored).is_ok());
        let rows = captured_domains(request, receipt, authored, binding);
        assert_eq!(rows.len(), 4);
        assert_eq!(
            rows.iter()
                .filter(|r| matches!(r.domain, NamedAnnularDomain::Cap { .. }))
                .count(),
            2
        );
        for row in rows {
            assert_domain(row);
        }
    });
}
fn captured_domains(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &PrismReceipt,
    authored: &PrismPublicationBindings,
    binding: AnnularPrismBinding,
) -> Vec<NamedAnnularFace> {
    let evidence =
        inspect_bound_annular_embedding(request, receipt, binding, embedding_options(), || false)
            .unwrap_or_else(|_| panic!("actual embedding"));
    let projection = project(receipt, authored);
    let bound = bind_projection(receipt, authored).unwrap();
    links::capture(&evidence, &bound, &projection)
        .unwrap_or_else(|_| panic!("complete named annular domains"))
}

fn assert_domain(row: NamedAnnularFace) {
    match row.domain {
        NamedAnnularDomain::Cap { outer, inner } => {
            assert_eq!([outer.source.0, inner.source.0], [101, 202]);
            assert_ne!(outer.occurrence, inner.occurrence);
            assert_eq!(row.chart.trims.len(), 2);
            assert_eq!(row.chart.trims[0].role, row.chart.trims[1].role);
        }
        NamedAnnularDomain::Side { boundary } => {
            assert!([101, 202].contains(&boundary.source.0));
            assert_eq!(row.chart.trims.len(), 4);
        }
    }
}
#[test]
fn annular_name_census_counts_all_nested_rows_and_refuses_one_short() {
    with_bound(|_, _, authored, _| {
        let measured =
            census::inspect(authored, options()).unwrap_or_else(|_| panic!("bounded names"));
        let expected = expected_units(authored);
        assert_eq!(measured.measured_units, expected);
        assert_exact_budget(authored, expected);
        let short = AnnularFrozenOptions {
            maximum_name_units: expected - 1,
            ..options()
        };
        assert!(matches!(
            census::inspect(authored, short),
            Err(AnnularFrozenCause::NameBudget { .. })
        ));
    });
}
fn expected_units(authored: &PrismPublicationBindings) -> usize {
    2 + authored.originals.len()
        + authored.sources.len()
        + authored.directed.len()
        + authored
            .directed
            .iter()
            .map(|b| b.actual.ancestors.len() + b.actual.child_slots.len())
            .sum::<usize>()
}
fn assert_exact_budget(authored: &PrismPublicationBindings, expected: usize) {
    let exact = AnnularFrozenOptions {
        maximum_name_units: expected,
        ..options()
    };
    assert_eq!(
        census::inspect(authored, exact)
            .unwrap_or_else(|_| panic!("inclusive limit"))
            .effective,
        exact
    );
}

#[test]
fn annular_name_bijection_refuses_duplicates_and_omitted_original_roles() {
    with_bound(|request, _, authored, binding| {
        let row = &authored.originals[0];
        let duplicate = PrismOriginalBinding {
            actual: row.actual.clone(),
            member: row.member,
            directed_use: row.directed_use,
        };
        authored.originals[1] = duplicate;
        assert!(names::inspect(request, &binding, authored).is_err());
    });
}
