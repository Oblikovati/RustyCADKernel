// SPDX-License-Identifier: GPL-2.0-only
//! Symbolic row operands only; these fixtures never construct or claim an accepted body.
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_prism::{
    frozen::*,
    owned_request::names::{body_map::*, *},
};
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;

pub(super) fn names() -> ResolvedPrismNames {
    fixture_unbound_symbolic_names()
}
pub(super) fn key(source: u64, role: FrozenSweepRole, kind: u8) -> FrozenPrismKey {
    FrozenPrismKey {
        operation: AuthoredPrismId(257),
        source: AuthoredPrismId(source),
        role,
        kind,
    }
}
pub(super) fn placement() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [9007199254740992., -0., 0.],
        scale: 1.,
    }
}
pub(super) fn space() -> CapturedSpaceRepresentation {
    CapturedSpaceRepresentation {
        curve: CapturedMetricCurve::Bezier {
            poles: vec![[1., -0., 0.], [2., 0., 0.]],
            weights: None,
        },
        interval: [-0., 1.],
        placement: placement(),
        same_parameter: true,
    }
}
pub(super) fn node(key: FrozenPrismKey) -> FrozenPrismNode {
    FrozenPrismNode {
        key,
        point: None,
        raw_tolerance: Some(0.),
        effective_tolerance: Some(1e-7),
        curve: Some(space()),
    }
}
pub(super) fn cap_endpoint(names: &ResolvedPrismNames) -> FrozenUseKey {
    names
        .use_bindings()
        .iter()
        .find(|row| {
            row.semantic.0.last().is_some_and(|step| {
                step.member.kind == 7 && step.directed_use.role == SemanticNameRole::StartUse
            }) && row.semantic.0.iter().any(|step| {
                step.member.kind == 4 && step.member.sweep_role == FrozenSweepRole::First
            })
        })
        .unwrap()
        .frozen
        .clone()
}
pub(super) fn symbolic_use(key: FrozenUseKey) -> FrozenPrismUse {
    FrozenPrismUse {
        key,
        orientation: rustycad_tkg3d::top_abs::Orientation::Forward,
        placement: placement(),
        children: vec![],
    }
}
pub(super) fn all_domain(
    names: &ResolvedPrismNames,
) -> (Vec<FrozenPrismNode>, Vec<FrozenPrismUse>) {
    let members = names
        .member_bindings()
        .iter()
        .map(|row| node(row.frozen))
        .collect();
    let uses = names
        .use_bindings()
        .iter()
        .map(|row| symbolic_use(row.frozen.clone()))
        .collect();
    (members, uses)
}
