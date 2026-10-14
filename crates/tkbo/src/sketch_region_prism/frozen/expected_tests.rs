// SPDX-License-Identifier: GPL-2.0-only
//! Owned closed_cubic_v1.json predates production edits; coefficient formulas are independent.
use super::super::surface_charts::{CapturedPrismSurface, PrismTrimRole};
use super::{tests::*, *};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;

#[test]
fn complete_frozen_geometry_matches_owned_cubic_fixture() {
    let projection = fresh_projection();
    assert_eq!(projection.incidence().counts, [2, 3, 6, 3, 3, 1, 1]);
    for face in projection.faces() {
        match &face.surface {
            CapturedPrismSurface::Plane(plane) => check_cap(face, plane),
            CapturedPrismSurface::Extrusion {
                basis, direction, ..
            } => check_side(face, basis, direction),
        }
    }
    assert_eq!(projection.endpoint_proofs().len(), 6);
    assert!(
        projection
            .endpoint_proofs()
            .iter()
            .all(|p| p.requested_spatial == 0.)
    );
    assert!(!projection.is_valid_body());
}
fn check_cap(
    face: &FrozenPrismFace,
    plane: &crate::sketch_region_source_conformance::CapturedPlaneSupport,
) {
    let height = match face.key.role {
        FrozenSweepRole::First => 0.,
        FrozenSweepRole::Last => 2.,
        _ => panic!("expected cap role"),
    };
    assert_eq!(plane.frame.linear, [[1., 0.], [0., 1.], [0., 0.]]);
    assert_eq!(plane.frame.origin, [0.; 3]);
    assert_eq!(plane.placement.origin, [0., 0., height]);
    assert_eq!(face.trims.len(), 1);
    assert_eq!(face.trims[0].interval, [0., 1.]);
}
fn check_side(face: &FrozenPrismFace, basis: &CapturedMetricCurve<3>, direction: &[f64; 3]) {
    let CapturedMetricCurve::Bezier { poles, weights } = basis else {
        panic!("expected cubic")
    };
    assert_eq!(
        poles,
        &[[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 0.]]
    );
    assert!(weights.is_none());
    assert_eq!(*direction, [0., 0., -1.]);
    assert_eq!(face.proof.side_height, Some(2.));
    assert_eq!(face.trims.len(), 4);
    for (role, trim) in face.proof.trim_roles.iter().zip(&face.trims) {
        check_side_trim(*role, trim);
    }
}
fn check_side_trim(role: PrismTrimRole, trim: &FrozenPrismTrim) {
    let expected = match role {
        PrismTrimRole::First => ([0., 0.], [1., 0.], [0., 1.]),
        PrismTrimRole::Last => ([0., -2.], [1., 0.], [0., 1.]),
        PrismTrimRole::Start => ([0., 0.], [0., -1.], [0., 2.]),
        PrismTrimRole::End => ([1., 0.], [0., -1.], [0., 2.]),
    };
    let CapturedMetricCurve::Line { origin, direction } = &trim.pcurve else {
        panic!("expected line")
    };
    assert_eq!((*origin, *direction, trim.interval), expected);
}
#[test]
fn coverage_history_and_effective_limits_remain_explicit() {
    use crate::sketch_region_validation::StageCoverage;
    let projection = fresh_projection();
    assert_eq!(
        projection.coverage().current,
        [StageCoverage::Incomplete; 4]
    );
    assert_eq!(projection.limits().extrusion, [0., 0., 2.]);
    assert_eq!(projection.limits().effective_extrusion, Some([0., 0., 2.]));
    assert!(projection.limits().achieved_error.is_none());
    assert_eq!(projection.history().derived.len(), projection.uses().len());
    assert!(
        projection
            .history()
            .derived
            .iter()
            .all(|r| !r.generated.is_empty() || !r.modified.is_empty() || !r.preserved.is_empty())
    );
    assert!(!projection.history().rejected_unbound.is_empty());
    assert!(!projection.history().originals.is_empty());
}
