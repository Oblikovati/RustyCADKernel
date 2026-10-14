// SPDX-License-Identifier: GPL-2.0-only
//! Rank belongs to the exact captured chart; this fixed derivative determinant is no swept-volume proof.
use super::{failure::*, outcome::TwoLoopSourceProof, request::*, work::*};
use crate::sketch_region_geometry_capture::{captured_placement, captured_plane_frame};
use crate::sketch_region_source_conformance::CapturedPlaneSupport;
use rustycad_tkmath::{
    gp::Transform3,
    predicates::roots::{DyadicBound, metric_bounds::affine_prism_jacobian},
};
pub(super) fn inspect<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    sources: &[TwoLoopSourceProof],
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, DyadicBound> {
    let batch = super::super::borrowed_batch(request.bridge).ok_or_else(|| {
        work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "actual declared plane",
        })
    })?;
    let expected = CapturedPlaneSupport {
        frame: captured_plane_frame(batch.plane.position()),
        placement: captured_placement(Transform3::default()).expect("finite identity"),
    };
    for row in sources {
        work.holds(
            "declared and actual common plane",
            super::super::boundary::same_support(&expected, &row.support),
        )?;
    }
    let normal = *batch.plane.axis().direction().coordinates();
    let support = &sources[0].support;
    let mut cancelled = None;
    let result = affine_prism_jacobian(support.frame, support.placement, normal, || {
        cancelled = work.poll(TwoLoopRegionStage::Sources).err();
        cancelled.is_some()
    });
    if let Some(failure) = cancelled {
        return Err(failure);
    }
    let exact = result.map_err(|cause| work.failure(TwoLoopRegionCause::Residual(cause)))?;
    work.holds(
        "exact injective common plane chart",
        exact.compare_f64(0.).is_ok_and(|s| !s.is_eq()),
    )?;
    work.units("exact chart rank proof", 1)?;
    Ok(exact)
}
