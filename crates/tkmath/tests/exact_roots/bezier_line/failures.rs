// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use std::error::Error;
fn refusal(request: BezierLineQuery<'_>) -> BezierLineError {
    let error = intersect_bezier_line(request, || false).unwrap_err();
    assert!(error.to_string().contains("expected"));
    error
}
#[test]
fn bezier_denominator_zeros_are_refused_before_numerator_coincidence() {
    let controls = [[0., 0.], [1., 0.], [2., 0.]];
    for (weights, expected) in [([1., -1., 1.], "2,-1:2"), ([0., 0., 1.], "1,0:2")] {
        let mut request = query(&controls);
        request.weights = Some(&weights);
        let BezierLineError::DenominatorRoots(roots) = refusal(request) else {
            panic!("expected denominator roots");
        };
        assert_eq!(roots.roots().len(), 1);
        super::super::oracle::certify_interval(&roots.roots()[0], expected);
    }
    let mut request = query(&controls);
    request.weights = Some(&[0., 0., 0.]);
    assert_eq!(refusal(request), BezierLineError::ZeroDenominator);
}
#[test]
fn bezier_signed_weights_are_accepted_only_on_denominator_regular_intervals() {
    let controls = [[0., -1.], [1., 1.]];
    let mut request = query(&controls);
    request.weights = Some(&[1., -1.]);
    assert!(matches!(
        refusal(request),
        BezierLineError::DenominatorRoots(_)
    ));
    request.interval = [0., 0.25];
    assert!(contacts(request).roots().is_empty());
    request.weights = Some(&[1., 2.]);
    request.interval = [-1., -1.];
    assert!(matches!(
        refusal(request),
        BezierLineError::DenominatorRoots(_)
    ));
}
#[test]
fn bezier_shape_and_coordinate_errors_name_the_offending_input() {
    assert_eq!(refusal(query(&[])), BezierLineError::EmptyControls);
    let controls = [[0., 1.], [1., -1.]];
    let mut request = query(&controls);
    request.weights = Some(&[1.]);
    assert_eq!(
        refusal(request),
        BezierLineError::WeightCount {
            controls: 2,
            weights: 1
        }
    );
    request.weights = None;
    request.line = [[1., 2.]; 2];
    assert_eq!(
        refusal(request),
        BezierLineError::DegenerateLine(request.line)
    );
    check_nonfinite_inputs();
}
fn check_nonfinite_inputs() {
    let points = [[0., f64::INFINITY]];
    let request = query(&points);
    let error = refusal(request);
    assert!(matches!(
        error,
        BezierLineError::Coordinate {
            input: BezierInput::Control,
            cause: rustycad_tkmath::predicates::NonFiniteCoordinate {
                point: 0,
                axis: 1,
                ..
            }
        }
    ));
    assert!(error.source().is_some());
}
#[test]
fn bezier_nonfinite_line_coordinates_name_the_line_owner() {
    let mut request = query(&[[0., 1.]]);
    request.line[1][0] = f64::NAN;
    assert!(matches!(
        refusal(request),
        BezierLineError::Coordinate {
            input: BezierInput::Line,
            ..
        }
    ));
}
#[test]
fn bezier_nonfinite_weights_name_the_control_index() {
    let mut request = query(&[[0., 1.]]);
    request.weights = Some(&[f64::NEG_INFINITY]);
    assert!(matches!(
        refusal(request),
        BezierLineError::Weight {
            index: 0,
            value: f64::NEG_INFINITY
        }
    ));
}
#[test]
fn bezier_work_limits_name_the_stage_and_preserve_the_budget_failure() {
    let controls = [[0., 1.], [1., -2.], [2., 1.]];
    let mut request = query(&controls);
    request.options.max_bisections = 0;
    let cause = RootIsolationError::SubdivisionLimit { limit: 0 };
    assert_root_stage(refusal(request), BezierStage::Numerator, cause.clone());
    request.weights = Some(&[1., -2., 1.]);
    assert_root_stage(refusal(request), BezierStage::Denominator, cause);
}
#[test]
fn bezier_degree_and_version_errors_preserve_the_original_cause() {
    let controls = [[0., 1.], [1., -2.], [2., 1.]];
    let mut request = query(&controls);
    request.options.version = 2;
    assert_root_stage(
        refusal(request),
        BezierStage::Input,
        RootIsolationError::OptionsVersion(2),
    );
    request.options = options(0.0001);
    request.options.max_degree = 1;
    let cause = RootIsolationError::Degree {
        actual: 2,
        limit: 1,
    };
    request.weights = Some(&[1., -2., 1.]);
    assert_root_stage(refusal(request), BezierStage::Denominator, cause.clone());
    request.weights = None;
    assert_root_stage(refusal(request), BezierStage::Numerator, cause);
}
fn assert_root_stage(error: BezierLineError, stage: BezierStage, cause: RootIsolationError) {
    assert!(error.source().is_some());
    assert_eq!(error, BezierLineError::Root { stage, cause });
}
