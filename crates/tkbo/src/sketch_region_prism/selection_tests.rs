// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use crate::sketch_region_candidates::{
    CandidateCause, CandidateClassification, MissingCertificate, SeedSelection,
    tests::{candidates, rectangle},
};
use rustycad_tkg3d::top_abs::Orientation;
#[test]
fn nested_hole_and_inner_regions_use_identical_transaction() {
    for seed in [[1., 1.], [3., 3.], [2., 3.]] {
        let mut edges = rectangle(0., 0., 8., 8.);
        edges.extend(rectangle(2., 2., 4., 4.));
        let result =
            retain_selected_prism(candidates(&edges), request(seed), &mut progress(None, None));
        assert_nested_selection(result, seed == [2., 3.]);
    }
}
#[test]
fn touching_boundary_and_duplicate_candidates_do_not_choose_first_match() {
    let mut edges = rectangle(0., 0., 2., 2.);
    edges.extend(rectangle(2., 2., 2., 2.));
    let touched = retain_selected_prism(
        candidates(&edges),
        request([2., 2.]),
        &mut progress(None, None),
    );
    assert!(
        matches!(touched.completion,PrismCompletion::NoBody(SeedSelection::Boundary { occurrences }) if occurrences.len()>=2)
    );
}
#[test]
fn duplicate_candidates_have_distinct_ambiguous_disposition() {
    let mut batch = candidates(&rectangle(0., 0., 4., 3.));
    let repeated = candidates(&rectangle(0., 0., 4., 3.));
    batch.candidates.extend(repeated.candidates);
    let ambiguous = retain_selected_prism(batch, request([2., 1.]), &mut progress(None, None));
    assert!(
        matches!(ambiguous.completion,PrismCompletion::NoBody(SeedSelection::Ambiguous { candidates }) if candidates.len()==2)
    );
    assert!(ambiguous.diagnostic.is_none());
    assert_eq!(ambiguous.input.rejected.len(), 4);
}
#[test]
fn shared_reversed_aliases_preserve_both_original_caller_ids() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let reversed = source.oriented(Orientation::Reversed);
    let result = retain_selected_prism(
        candidates(&[source, reversed]),
        request([1., 1.]),
        &mut progress(None, None),
    );
    let diagnostic = result.diagnostic.unwrap();
    for row in &diagnostic.relations {
        let sources: std::collections::BTreeSet<_> = row
            .generated
            .iter()
            .chain(&row.modified)
            .map(|key| key.source.0)
            .collect();
        assert_eq!(sources, [1, 2].into_iter().collect());
    }
}
#[test]
fn exhausted_classifier_keeps_original_typed_query_and_actual_work() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let mut options = request([1., 1.]);
    options.roots.max_bisections = 1;
    let result = retain_selected_prism(candidates(&[source]), options, &mut progress(None, None));
    let PrismCompletion::Failed(PrismCause::Classification(query)) = result.completion else {
        panic!("expected original query failure");
    };
    let CandidateCause::CapturedOperation(failure) = *query else {
        panic!("expected captured query prefix");
    };
    assert_eq!(failure.retained.work.operation_bisections, 1);
    assert!(matches!(
        *failure.cause,
        CandidateCause::CapturedNumerical(_)
    ));
    assert!(result.diagnostic.is_none());
}
#[test]
fn terminal_receipt_keeps_requested_limits_and_exact_numeric_capture() {
    let result = cubic_receipt([1., 1.], &mut progress(None, None));
    let CandidateClassification::Incomplete {
        captured, missing, ..
    } = result.classification.unwrap();
    assert_eq!(captured.work.reused_certificates, 1);
    assert_eq!(
        result.limits.effective.roots,
        Some(result.limits.requested.roots)
    );
    assert!(result.limits.effective.spatial_limit.is_none());
    assert!(result.limits.effective.parameter_limit.is_none());
    assert!(missing.contains(&MissingCertificate::Construction));
    assert!(missing.contains(&MissingCertificate::SelfIntersection));
    assert!(missing.contains(&MissingCertificate::Tolerance));
}

fn assert_nested_selection(result: PrismReceipt, boundary: bool) {
    if boundary {
        assert!(matches!(
            result.completion,
            PrismCompletion::NoBody(SeedSelection::Boundary { .. })
        ));
        return;
    }
    assert!(matches!(result.completion, PrismCompletion::Incomplete));
    assert!(result.diagnostic.is_some());
    assert!(result.gaps.iter().any(|gap| matches!(
        gap,
        PrismGap::Candidate(MissingCertificate::SearchCompleteness)
    )));
}
