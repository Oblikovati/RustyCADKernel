// SPDX-License-Identifier: GPL-2.0-only
use super::super::types::holds;
use super::*;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::{
    CapturedSpaceRepresentation, same_space_representation,
};

pub(super) fn actual_boundary(
    binding: &CurrentPrismBinding,
    options: BezierRegularityOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<BezierRegularityReport, PrismEmbeddingCause> {
    let side = domains::chart(binding, PrismImageRole::Generated)?;
    let source = current_source(binding, side)?;
    polynomial_regularity(binding.region.census_occurrence, source, options, cancelled)
}
fn polynomial_regularity(
    curve_id: u64,
    source: &CapturedSpaceRepresentation,
    options: BezierRegularityOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<BezierRegularityReport, PrismEmbeddingCause> {
    let poles = polynomial_poles(curve_id, source)?;
    let mut query = BezierRegularityQuery::new(curve_id, poles, options);
    query.interval = source.interval;
    query.placement = source.placement;
    let report = certify_bezier_regularity(query, cancelled)
        .map_err(|cause| PrismEmbeddingCause::Regularity(Box::new(cause)))?;
    require_regular(report)
}
fn polynomial_poles(
    curve_id: u64,
    source: &CapturedSpaceRepresentation,
) -> Result<&[[f64; 3]], PrismEmbeddingCause> {
    match &source.curve {
        CapturedMetricCurve::Bezier {
            poles,
            weights: None,
        } => Ok(poles),
        CapturedMetricCurve::Bezier {
            weights: Some(weights),
            ..
        } => Err(PrismEmbeddingCause::PolynomialRequired {
            curve_id,
            explicit_weights: weights.clone(),
        }),
        _ => Err(RegionCertificateCause::MissingEvidence {
            hypothesis: "actual polynomial Bezier generating boundary",
        }
        .into()),
    }
}

fn current_source<'a>(
    binding: &CurrentPrismBinding,
    side: &'a BoundPrismChart,
) -> Result<&'a CapturedSpaceRepresentation, PrismEmbeddingCause> {
    let CapturedChartSource::Side(source) = &side.captured.source else {
        return Err(RegionCertificateCause::MissingEvidence {
            hypothesis: "actual side source chart",
        }
        .into());
    };
    let boundary = &binding.region.copied.boundaries[0];
    holds(
        "regularity source is actual Jordan generating boundary",
        side.source == boundary.occurrence && same_space_representation(source, &boundary.space),
    )?;
    Ok(source)
}
fn require_regular(
    report: BezierRegularityReport,
) -> Result<BezierRegularityReport, PrismEmbeddingCause> {
    if !matches!(report.decision(), BezierRegularityDecision::Regular { .. }) {
        return Err(PrismEmbeddingCause::NonRegular(Box::new(report)));
    }
    Ok(report)
}
#[cfg(test)]
mod guard_tests {
    use super::*;
    use rustycad_tkmath::predicates::roots::RootIsolationOptions;
    fn source(poles: Vec<[f64; 3]>) -> CapturedSpaceRepresentation {
        CapturedSpaceRepresentation {
            curve: CapturedMetricCurve::Bezier {
                poles,
                weights: None,
            },
            interval: [0., 1.],
            placement: MetricEmbedding {
                linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
                origin: [0.; 3],
                scale: 1.,
            },
            same_parameter: true,
        }
    }
    #[test]
    fn nonregular_guard_retains_full_classification_without_a_jordan_claim() {
        for poles in [
            vec![[0.; 3]],
            vec![[1., 0., 0.], [-2., 0., 0.], [4., 0., 0.]],
        ] {
            let source = source(poles);
            let options = BezierRegularityOptions::v1(RootIsolationOptions::new(1e-6));
            let error = polynomial_regularity(7, &source, options, &mut || false)
                .err()
                .unwrap();
            let PrismEmbeddingCause::NonRegular(report) = error else {
                panic!("expected stationary classification")
            };
            assert!(!matches!(
                report.decision(),
                BezierRegularityDecision::Regular { .. }
            ));
        }
    }
    #[test]
    fn every_explicit_weight_representation_refuses_polynomial_v1() {
        for weights in [vec![1., 1.], vec![-1., -1.], vec![1., -1.]] {
            let mut source = source(vec![[0.; 3], [1., 0., 0.]]);
            source.curve = CapturedMetricCurve::Bezier {
                poles: vec![[0.; 3], [1., 0., 0.]],
                weights: Some(weights.clone()),
            };
            let error = polynomial_poles(7, &source).err().unwrap();
            assert!(matches!(error, PrismEmbeddingCause::PolynomialRequired {
                curve_id: 7, explicit_weights
            } if explicit_weights == weights));
        }
    }
    #[test]
    fn exact_one_third_stationary_root_preserves_typed_exhaustion() {
        // x=(3t-1)^2, so squared speed is 36(3t-1)^2; 1/3 is not dyadic.
        let source = source(vec![[1., 0., 0.], [-2., 0., 0.], [4., 0., 0.]]);
        let mut options = BezierRegularityOptions::v1(RootIsolationOptions::new(1e-6));
        options.roots.max_bisections = 0;
        let error = polynomial_regularity(7, &source, options, &mut || false)
            .err()
            .unwrap();
        let PrismEmbeddingCause::Regularity(failure) = error else {
            panic!("expected typed exhaustion")
        };
        assert_eq!(failure.completion(), BezierRegularityCompletion::Incomplete);
        assert_eq!(failure.source().options(), options);
    }
}
