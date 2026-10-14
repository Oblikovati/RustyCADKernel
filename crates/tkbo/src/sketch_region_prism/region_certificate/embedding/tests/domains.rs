// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::super::tests::request;
use super::super::super::measurements::tests::domain_tests::with_request;
use super::*;
use crate::sketch_region_certification::census_bezier_fixture;

#[test]
fn exact_trimmed_domains_retain_authored_parents_and_corner_seam() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let report = embedded(&bridge, &shapes, &receipt);
    assert_domain_kinds(&report);
    assert_six_conclusions(&report);
    assert_original_source(&report);
    for row in report.incidences() {
        for domain in [&row.first, &row.second] {
            assert!(!domain.parents.is_empty());
            assert!(
                domain
                    .trims
                    .iter()
                    .all(|trim| !trim.parents.is_empty() && trim.face == domain.face)
            );
        }
    }
}
fn assert_domain_kinds(report: &PrismEmbeddingEvidence) {
    let [first, last, side] = report.domains();
    assert!(matches!(
        first.kind,
        PrismDomainKind::Cap {
            sweep: 0.,
            winding: 1,
            ..
        }
    ));
    assert!(matches!(
        last.kind,
        PrismDomainKind::Cap {
            sweep: 1.,
            winding: 1,
            ..
        }
    ));
    assert!(matches!(
        side.kind,
        PrismDomainKind::Side {
            native: [0., 1.],
            stored_sweep: [-2., 0.],
            normalized_sweep: [0., 1.],
            height: 2.,
            ..
        }
    ));
    assert_eq!(
        [first.trims.len(), last.trims.len(), side.trims.len()],
        [1, 1, 4]
    );
}
fn assert_six_conclusions(report: &PrismEmbeddingEvidence) {
    let rows = report.incidences();
    assert!(matches!(
        rows[0].conclusion,
        PrismIncidenceConclusion::InjectivePlanarDomain
    ));
    assert!(matches!(
        rows[1].conclusion,
        PrismIncidenceConclusion::InjectivePlanarDomain
    ));
    assert_seam(&rows[2].conclusion);
    assert!(matches!(
        rows[3].conclusion,
        PrismIncidenceConclusion::Disjoint
    ));
    for (row, expected) in [(&rows[4], 0.), (&rows[5], 1.)] {
        let PrismIncidenceConclusion::ExactlyRim {
            cap,
            side,
            native,
            normalized_sweep,
        } = &row.conclusion
        else {
            panic!("expected rim")
        };
        assert_eq!(*native, [0., 1.]);
        assert_eq!(*normalized_sweep, expected);
        assert!(super::super::super::history::same_physical(
            &cap.output,
            &side.output
        ));
        assert_eq!(cap.interval, [0., 1.]);
        assert_eq!(side.interval, [0., 1.]);
    }
}
fn assert_seam(conclusion: &PrismIncidenceConclusion) {
    let PrismIncidenceConclusion::InjectiveModuloSeam {
        start,
        end,
        native_endpoints,
        normalized_sweep,
    } = conclusion
    else {
        panic!("expected seam quotient")
    };
    assert_eq!(*native_endpoints, [0., 1.]);
    assert_eq!(*normalized_sweep, [0., 1.]);
    assert_eq!(start.role, PrismTrimRole::Start);
    assert_eq!(end.role, PrismTrimRole::End);
    assert_eq!(start.interval, [0., 2.]);
    assert_eq!(end.interval, [0., 2.]);
    assert_ne!(start.output.orientation, end.output.orientation);
}
fn assert_original_source(report: &PrismEmbeddingEvidence) {
    // Tangents (12,0,0) and (0,-12,0) differ: accepted closure is explicitly a corner.
    let source = report.regularity().source();
    assert_eq!(
        source.poles(),
        &[[0.; 3], [4., 0., 0.], [0., 4., 0.], [0.; 3]]
    );
    assert_eq!(source.curve_id(), report.binding().region.census_occurrence);
    assert_eq!(source.interval(), [0., 1.]);
    assert!(source.weights().is_none());
    assert_closure_contexts(report);
}
fn assert_closure_contexts(report: &PrismEmbeddingEvidence) {
    use super::super::super::ClosurePointStage::*;
    // Physical reuse groups differ by stage; every directed authored use must remain present.
    let contexts: Vec<_> = report
        .binding()
        .region
        .closure_points
        .iter()
        .flat_map(|row| &row.contexts)
        .collect();
    assert_eq!(contexts.len(), 12);
    for stage in [
        Original,
        Census,
        Partition,
        CandidateFirst,
        CandidateSecond,
        PrismCopy,
    ] {
        assert_eq!(contexts.iter().filter(|row| row.stage == stage).count(), 2);
    }
}
#[test]
fn scaled_reversed_polynomial_sources_use_the_same_theorem() {
    for (scale, reverse) in [(0.5, false), (1., true), (2., false), (2., true)] {
        let shape = crate::closed_analytic_cubic::closed_analytic_cubic(scale, reverse);
        let (bridge, shapes) = fixture_for(vec![shape]);
        let receipt = with_request(&bridge, &shapes, request([scale, scale]));
        assert_eq!(embedded(&bridge, &shapes, &receipt).incidences().len(), 6);
    }
}
#[test]
fn rational_jordan_source_is_explicitly_outside_polynomial_theorem_v1() {
    let shape = census_bezier_fixture(
        &[[0., 0.], [4., 0.], [0., 4.], [0., 0.]],
        Some(&[1., 2., 2., 1.]),
    );
    let (bridge, shapes) = fixture_for(vec![shape]);
    let receipt = run(&bridge, &shapes);
    assert!(receipt.region.as_ref().unwrap().decision.is_ok());
    let error = attempt(&bridge, &shapes, &receipt, options(), || false)
        .err()
        .unwrap();
    assert_eq!(error.phase, PrismEmbeddingPhase::Regularity);
    let PrismEmbeddingCause::PolynomialRequired {
        curve_id,
        explicit_weights,
    } = error.cause
    else {
        panic!("expected explicit polynomial scope refusal")
    };
    assert_eq!(curve_id, bridge.census.inputs.segments[0].occurrence);
    assert_eq!(explicit_weights, [1., 2., 2., 1.]);
}

#[test]
fn role_order_and_historical_status_do_not_supply_theorem_identity() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    let before = embedded(&bridge, &shapes, &receipt);
    receipt.region = None;
    let diagnostic = receipt.diagnostic.as_mut().unwrap();
    diagnostic.surface_charts = None;
    diagnostic.bindings.reverse();
    diagnostic.relations.reverse();
    let after = embedded(&bridge, &shapes, &receipt);
    assert_eq!(before.incidences(), after.incidences());
    let mut domains = after.domains().clone();
    domains.reverse();
    assert_eq!(&table::six_rows(&domains).unwrap(), after.incidences());
}
#[test]
fn translated_last_cap_and_negative_sweep_retain_exact_domain_map() {
    for height in [4., -2.] {
        let (bridge, shapes) = fixture();
        let mut requested = request([1., 1.]);
        requested.extrusion = rustycad_tkmath::gp::Vector3::new(0., 0., height);
        let receipt = with_request(&bridge, &shapes, requested);
        let report = embedded(&bridge, &shapes, &receipt);
        assert_eq!(
            report
                .binding()
                .signed_jacobian
                .compare_f64(height)
                .unwrap(),
            std::cmp::Ordering::Equal
        );
        assert!(
            matches!(report.domains()[2].kind,PrismDomainKind::Side{height:h,stored_sweep:[a,0.],..} if h==height.abs() && a == -height.abs())
        );
        let last = domains::chart(report.binding(), PrismImageRole::Last).unwrap();
        let CapturedPrismSurface::Plane(support) = &last.captured.actual else {
            panic!("expected placed cap")
        };
        assert_eq!(support.placement.origin, [0., 0., height]);
    }
}
#[test]
fn placed_source_keeps_exact_closure_point_and_regular_boundary() {
    use rustycad_tkmath::{
        gp::{Transform3, Vector3},
        toploc::Location,
    };
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., true);
    let mut placement = Transform3::default();
    placement.set_translation(Vector3::new(8., -4., 0.));
    let source = source.located(&Location::new(placement), true).unwrap();
    let (bridge, shapes) = fixture_for(vec![source]);
    let receipt = with_request(&bridge, &shapes, request([9., -3.]));
    let report = embedded(&bridge, &shapes, &receipt);
    assert!(
        report
            .binding()
            .region
            .closure_points
            .iter()
            .all(|point| point.point == [8., -4., 0.])
    );
    assert_eq!(report.regularity().speed_degree(), Some(4));
    assert_eq!(
        report.binding().signed_jacobian.compare_f64(2.).unwrap(),
        std::cmp::Ordering::Equal
    );
}
