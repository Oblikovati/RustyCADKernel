// SPDX-License-Identifier: GPL-2.0-only
//! Expected native source roles and points come from authored fixture poles, never output order.
use super::{tests::*, *};
use crate::sketch_region_prism::{
    exact_tolerance::{frames::*, scalar::*, test_support::*},
    frozen::annular_join::operation_tests::joined,
};
use rustycad_tkmath::predicates::roots::{DyadicBound, metric_bounds::*};
#[test]
fn all_twenty_closure_contexts_match_independently_authored_source_roles() {
    let join = joined();
    let proof = conformance(&join);
    for source in [101, 202] {
        assert_source_roles(&proof, source);
    }
    assert_closure_points(&proof);
}
fn assert_source_roles(proof: &AnnularPrismConformance, source: u64) {
    let (selected, rejected, rejected_wire) = if source == 101 {
        (401, 303, 404)
    } else {
        (402, 302, 403)
    };
    for stage in source_stages(selected, rejected, rejected_wire) {
        let rows: Vec<_> = proof
            .evidence()
            .closures
            .iter()
            .filter(|row| row.context.source.0 == source && row.context.stage == stage)
            .collect();
        assert_eq!(rows.len(), 2);
        for row in rows {
            assert_reuse(row, selected);
        }
    }
}
fn assert_reuse(row: &AnnularClosureEvidence, selected: u64) {
    if matches!(row.context.stage, AnnularClosureStage::PrismCopy { .. }) {
        assert_eq!(
            row.reused_from,
            Some(AnnularSourceContext {
                stage: AnnularClosureStage::Candidate {
                    candidate: 301,
                    wire: selected
                },
                ..row.context
            })
        );
    } else {
        assert_eq!(row.reused_from, None);
    }
}
pub(super) fn endpoint_proof() -> TwoLoopEndpointProof {
    TwoLoopEndpointProof {
        parameter: 0.,
        orientation: rustycad_tkg3d::top_abs::Orientation::Forward,
        local_point: [0.; 3],
        placement: identity(),
        residual: ResidualDecision::Within {
            proven_squared_upper: DyadicBound::from_f64(0.).unwrap(),
        },
        bisections: 7,
        raw_tolerance: 0.,
    }
}
#[test]
fn retained_zero_cannot_authorize_singular_raw_point_or_curve_placement() {
    let curve = constant_curve([0.; 3]);
    let mut endpoint = endpoint_proof();
    endpoint.placement.linear[0] = [0.; 3];
    assert!(matches!(
        super::source_closures::retained_endpoint(&curve, &endpoint),
        Err(ExactEndpointFailure::Frame {
            role: EndpointFrameRole::VertexOccurrence,
            cause: ToleranceFrameFailure::Singular { .. }
        })
    ));
    endpoint.placement = identity();
    let mut singular = curve;
    singular.placement.scale = 0.;
    assert!(matches!(
        super::source_closures::retained_endpoint(&singular, &endpoint),
        Err(ExactEndpointFailure::Frame {
            role: EndpointFrameRole::CurveRepresentation,
            ..
        })
    ));
}
#[test]
fn retained_bound_must_be_exact_zero_and_keeps_terminal_work() {
    let curve = constant_curve([0.; 3]);
    let mut endpoint = endpoint_proof();
    let zero = super::source_closures::retained_endpoint(&curve, &endpoint)
        .ok()
        .unwrap();
    assert_eq!(zero.bisections, 7);
    endpoint.residual = ResidualDecision::Within {
        proven_squared_upper: DyadicBound::from_f64(0.125).unwrap(),
    };
    let Err(ExactEndpointFailure::Residual(report)) =
        super::source_closures::retained_endpoint(&curve, &endpoint)
    else {
        panic!("nonzero bound is incomplete")
    };
    assert_eq!(report.bisections, 7);
    assert!(
        matches!(report.decision, Ok(ResidualDecision::Within { proven_squared_upper }) if proven_squared_upper == DyadicBound::from_f64(0.125).unwrap())
    );
}
#[test]
fn original_source_requested_effective_and_work_reports_are_retained_without_requery() {
    let join = joined();
    let proof = conformance(&join);
    for row in &proof.evidence().source_reports {
        let source = join
            .embedding()
            .binding()
            .region()
            .sources()
            .iter()
            .find(|source| source.name == row.source)
            .unwrap();
        let original = source
            .stages
            .iter()
            .find(|stage| stage.stage == row.stage)
            .unwrap();
        assert_report_unchanged(row, original);
    }
}

fn assert_closure_points(proof: &AnnularPrismConformance) {
    for row in &proof.evidence().closures {
        let point = if row.context.source.0 == 101 {
            [0.; 3]
        } else {
            [0.5, 0.5, 0.]
        };
        assert_endpoint_geometry(row, point);
    }
}

fn source_stages(selected: u64, rejected: u64, rejected_wire: u64) -> [AnnularClosureStage; 5] {
    [
        AnnularClosureStage::Original,
        AnnularClosureStage::Census,
        AnnularClosureStage::Candidate {
            candidate: 301,
            wire: selected,
        },
        AnnularClosureStage::Candidate {
            candidate: rejected,
            wire: rejected_wire,
        },
        AnnularClosureStage::PrismCopy {
            candidate: 301,
            wire: selected,
        },
    ]
}

fn assert_endpoint_geometry(row: &AnnularClosureEvidence, point: [f64; 3]) {
    assert_eq!(row.local_point, point);
    assert_eq!(
        row.certificate.proven_squared_upper,
        DyadicBound::from_f64(0.).unwrap()
    );
    assert_eq!(
        (
            row.certificate.requested_spatial,
            row.certificate.effective_spatial
        ),
        (0., 0.)
    );
    assert_eq!(
        row.parameter,
        if row.context.native == Some(CanonicalEndpointSide::First) {
            0.
        } else {
            1.
        }
    );
}

fn assert_report_unchanged(row: &AnnularSourceReport, original: &TwoLoopSourceStage) {
    assert_eq!(
        row.requested_limit.to_bits(),
        original.requested_limit.to_bits()
    );
    assert_eq!(
        row.effective_limit.to_bits(),
        original.effective_limit.to_bits()
    );
    assert_eq!(row.bisections, original.bisections);
    assert_eq!(
        format!("{:?}", row.residual),
        format!("{:?}", original.residual)
    );
    assert_eq!(
        format!("{:?}", row.effective_roots),
        format!("{:?}", original.effective_roots)
    );
}
