// SPDX-License-Identifier: GPL-2.0-only
//! Owned baseline remains unchanged when the new private inspector is composed later.
use super::super::{
    PrismCompletion,
    contexts::body_contexts,
    tests::{cubic_receipt, progress},
};
use crate::sketch_region_validation::StageCoverage;
use rustycad_tkg3d::top_abs::ShapeEnum;

#[test]
fn legacy_prism_keeps_unknown_achieved_error_and_blocked_tolerance_stage() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    assert_eq!(receipt.limits.achieved_error, None);
    assert_eq!(receipt.limits.requested.spatial_limit, 1e-8);
    let diagnostic = receipt.diagnostic.as_ref().unwrap();
    let coverage = diagnostic.coverage.as_ref().unwrap();
    assert_eq!(coverage.tolerance, StageCoverage::Blocked);
    assert!(!coverage.complete());
}

#[test]
fn original_path_has_all_authored_directed_incidence_obligations() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let body = &receipt.diagnostic.as_ref().unwrap().body;
    let contexts = body_contexts(body)
        .ok()
        .expect("complete acyclic prism contexts");
    for (kind, expected) in [
        (ShapeEnum::Solid, 1),
        (ShapeEnum::Shell, 1),
        (ShapeEnum::Face, 3),
        (ShapeEnum::Wire, 3),
        (ShapeEnum::Edge, 6),
        (ShapeEnum::Vertex, 12),
    ] {
        let actual = contexts
            .iter()
            .filter(|row| row.occurrence.shape_type().unwrap() == kind)
            .count();
        assert_eq!(actual, expected, "authored {kind:?} context multiplicity");
    }
}
