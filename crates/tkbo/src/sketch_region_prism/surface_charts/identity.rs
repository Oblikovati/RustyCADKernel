// SPDX-License-Identifier: GPL-2.0-only
use super::types::*;
use rustycad_tkmath::predicates::roots::metric_bounds::*;

pub(super) fn chain<'a, const DIM: usize>(
    curve: MetricCurve<'a, DIM>,
    embedding: MetricEmbedding<DIM>,
    placements: &'a [MetricEmbedding<3>],
) -> MetricCurveChain<'a, DIM> {
    MetricCurveChain {
        curve,
        embedding,
        placements,
    }
}
pub(super) fn linear_part(mut embedding: MetricEmbedding<3>) -> MetricEmbedding<3> {
    // A derivative transports through the linear coefficients, never the affine origin.
    embedding.origin = [0.; 3];
    embedding
}

pub(super) fn spatial_identity() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
pub(super) fn planar_identity() -> MetricEmbedding<2> {
    MetricEmbedding {
        linear: [[1., 0.], [0., 1.], [0., 0.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
pub(super) fn translated(extrusion: [f64; 3]) -> MetricEmbedding<3> {
    MetricEmbedding {
        origin: extrusion,
        ..spatial_identity()
    }
}
pub(super) struct ProofContext<'a> {
    pub cancelled: &'a mut dyn FnMut() -> bool,
    pub proof: &'a mut PrismSurfaceProof,
}
impl ProofContext<'_> {
    pub(super) fn equal<const FIRST: usize, const SECOND: usize>(
        &mut self,
        first: MetricCurveChain<'_, FIRST>,
        second: MetricCurveChain<'_, SECOND>,
        stage: PrismChartStage,
    ) -> Result<(), PrismSurfaceCause> {
        let decision = compare_curve_chains(first, second, &mut || (self.cancelled)())?;
        if decision != CurveIdentityDecision::Identical {
            return Err(PrismSurfaceCause::Identity { stage, decision });
        }
        self.proof.identities.push(stage);
        Ok(())
    }
}
pub(super) fn line<const DIM: usize>(
    origin: [f64; DIM],
    direction: [f64; DIM],
) -> MetricCurve<'static, DIM> {
    // Preserve coefficients directly: constructing rounded endpoints would change the map.
    MetricCurve::Line { origin, direction }
}
pub(super) fn require_range(actual: [f64; 2], expected: [f64; 2]) -> Result<(), PrismSurfaceCause> {
    if actual != expected || actual.iter().any(|v| !v.is_finite()) || actual[0] >= actual[1] {
        return Err(PrismSurfaceCause::Range { actual, expected });
    }
    Ok(())
}
pub(super) fn require_count(
    entity: &'static str,
    actual: usize,
    expected: usize,
) -> Result<(), PrismSurfaceCause> {
    if actual != expected {
        return Err(PrismSurfaceCause::Count {
            entity,
            actual,
            expected,
        });
    }
    Ok(())
}
