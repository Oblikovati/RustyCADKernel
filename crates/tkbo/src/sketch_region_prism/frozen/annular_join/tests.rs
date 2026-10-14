// SPDX-License-Identifier: GPL-2.0-only
use super::super::annular_fixture::{ContinueAnnularFixture, authored};
use super::*;
use crate::sketch_region_prism::region_certificate::annular_binding::tests::with_actual;
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
    with_actual(|request, region, receipt| {
        let binding = bind_annular_prism_v2(request, region, receipt, &mut ContinueAnnularFixture)
            .unwrap_or_else(|_| panic!("actual annular binding"));
        let authored = authored(request, receipt, &binding);
        assert!(names::inspect(request, &binding, &authored).is_ok());
        let evidence =
            inspect_bound_annular_embedding(request, receipt, binding, embedding_options(), || {
                false
            })
            .unwrap_or_else(|_| panic!("actual embedding"));
        let projection = super::super::tests::captured(super::super::freeze_prism_with_behavior(
            receipt,
            &authored,
            &mut ContinueAnnularFixture,
            PrismFreezeBehavior::OrderedTraversalV2,
        ));
        let bound = bind_projection(receipt, &authored).unwrap();
        let rows = links::capture(&evidence, &bound, &projection)
            .unwrap_or_else(|_| panic!("complete named annular domains"));
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
    with_actual(|request, region, receipt| {
        let binding = bind_annular_prism_v2(request, region, receipt, &mut ContinueAnnularFixture)
            .unwrap_or_else(|_| panic!("binding"));
        let authored = authored(request, receipt, &binding);
        let measured =
            census::inspect(&authored, options()).unwrap_or_else(|_| panic!("bounded names"));
        let expected = 2
            + authored.originals.len()
            + authored.sources.len()
            + authored.directed.len()
            + authored
                .directed
                .iter()
                .map(|b| b.actual.ancestors.len() + b.actual.child_slots.len())
                .sum::<usize>();
        assert_eq!(measured.measured_units, expected);
        let exact = AnnularFrozenOptions {
            maximum_name_units: expected,
            ..options()
        };
        assert_eq!(
            census::inspect(&authored, exact)
                .unwrap_or_else(|_| panic!("inclusive limit"))
                .effective,
            exact
        );
        assert!(matches!(
            census::inspect(
                &authored,
                AnnularFrozenOptions {
                    maximum_name_units: expected - 1,
                    ..exact
                }
            ),
            Err(AnnularFrozenCause::NameBudget { .. })
        ));
    });
}
#[test]
fn annular_name_bijection_refuses_duplicates_and_omitted_original_roles() {
    with_actual(|request, region, receipt| {
        let binding = bind_annular_prism_v2(request, region, receipt, &mut ContinueAnnularFixture)
            .unwrap_or_else(|_| panic!("binding"));
        let mut authored = authored(request, receipt, &binding);
        let row = &authored.originals[0];
        let duplicate = PrismOriginalBinding {
            actual: row.actual.clone(),
            member: row.member,
            directed_use: row.directed_use,
        };
        authored.originals[1] = duplicate;
        assert!(names::inspect(request, &binding, &authored).is_err());
    });
}
