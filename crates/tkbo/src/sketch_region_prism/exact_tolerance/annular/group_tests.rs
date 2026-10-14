// SPDX-License-Identifier: GPL-2.0-only
//! Malformed scalar groups prove refusal only; no test constructor issues a sealed annular body.
use super::{progress::AnnularExactWork, source_tests::endpoint_proof, tests::*, *};
use crate::sketch_region_prism::exact_tolerance::{frames::ToleranceFrameFailure, test_support::*};
fn context() -> AnnularSourceContext {
    AnnularSourceContext {
        source: crate::sketch_region::RegionSourceId(101),
        stage: AnnularClosureStage::Original,
        native: None,
    }
}
fn endpoints() -> [TwoLoopEndpointProof; 2] {
    let first = endpoint_proof();
    let mut last = endpoint_proof();
    last.parameter = 1.;
    last.orientation = rustycad_tkg3d::top_abs::Orientation::Reversed;
    [first, last]
}
#[test]
fn singular_support_refuses_a_previously_zero_report_with_source_context() {
    let mut work = AnnularExactWork::new();
    let mut frame = identity();
    frame.scale = 0.;
    let result = super::source_closures::group(
        context(),
        &constant_curve([0.; 3]),
        frame,
        &endpoints(),
        &mut work,
        &mut Continue,
    );
    assert!(matches!(
        result,
        Err(AnnularConformanceCause::SourceFrame(
            ToleranceFrameFailure::Scale { actual: 0. }
        ))
    ));
    assert_eq!(work.source_context, Some(context()));
    assert!(work.upstream.closures.is_empty());
}
#[test]
fn duplicate_missing_noncanonical_parameters_and_duplicate_source_groups_refuse() {
    let curve = constant_curve([0.; 3]);
    let mut work = AnnularExactWork::new();
    let rows = endpoints();
    assert!(
        super::source_closures::group(
            context(),
            &curve,
            identity(),
            &rows[..1],
            &mut work,
            &mut Continue
        )
        .is_err()
    );
    for parameter in [0., -0., 0.5] {
        let mut malformed = endpoints();
        malformed[1].parameter = parameter;
        assert!(
            super::source_closures::group(
                context(),
                &curve,
                identity(),
                &malformed,
                &mut AnnularExactWork::new(),
                &mut Continue
            )
            .is_err()
        );
    }
    super::source_closures::group(
        context(),
        &curve,
        identity(),
        &rows,
        &mut work,
        &mut Continue,
    )
    .ok()
    .unwrap();
    assert!(
        super::source_closures::group(
            context(),
            &curve,
            identity(),
            &rows,
            &mut work,
            &mut Continue
        )
        .is_err()
    );
}
