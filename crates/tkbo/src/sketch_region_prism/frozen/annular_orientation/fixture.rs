// SPDX-License-Identifier: GPL-2.0-only
//! Every variant uses the same retained arrangement, region, sweep, embedding and naming pipeline.
use super::super::annular_fixture::{ContinueAnnularFixture, authored};
use super::*;
use crate::sketch_region_prism::region_certificate::embedding::*;
use crate::sketch_region_prism::{
    annular_construction::retained_annular_sweep_with_request,
    region_certificate::{
        annular_binding::*,
        two_loop::{fixture::*, fixture_bindings, *},
    },
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::{
    gp::{Axis3, Direction3, Plane3, Point3, Vector3},
    predicates::roots::{RootIsolationOptions, metric_bounds::BezierRegularityOptions},
};
#[derive(Default, Clone, Copy)]
pub(super) struct Variant {
    pub stored_reverse: [bool; 2],
    pub traversed_reverse: [bool; 2],
    pub reflected: bool,
    pub negative_sweep: bool,
}
pub(super) fn with_variant<R>(
    variant: Variant,
    run: impl FnOnce(&AnnularFrozenDomain, &TwoLoopFixture) -> R,
) -> R {
    let fixture = source_fixture(variant);
    let sources = fixture.sources();
    let identities = fixture_bindings::identities(&fixture);
    let request = TwoLoopRegionRequest {
        bridge: &fixture.bridge,
        sources: &sources,
        identities: &identities,
        inner_interior: [1., 1.],
        material_seed: [0.25, 0.25],
        requested: fixture_bindings::options(),
    };
    let mut sweep = crate::sketch_region_prism::tests::request([0.25, 0.25]);
    sweep.extrusion = Vector3::new(0., 0., if variant.negative_sweep { -2. } else { 2. });
    let receipt = retained_annular_sweep_with_request(&fixture, sweep);
    let joined = join(&request, &receipt);
    run(&joined, &fixture)
}
fn source_fixture(variant: Variant) -> TwoLoopFixture {
    let outer = source(1., [0., 0.], 0, variant);
    let inner = source(0.5, [0.5, 0.5], 1, variant);
    TwoLoopFixture::for_edges_on_plane(outer, inner, plane(variant.reflected))
}
fn source(scale: f64, offset: [f64; 2], index: usize, variant: Variant) -> Shape {
    let mut points = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]].map(|p| {
        let x = offset[0] + scale * p[0];
        Point3::new(
            if variant.reflected { -x } else { x },
            offset[1] + scale * p[1],
            0.,
        )
    });
    if variant.stored_reverse[index] {
        points.reverse();
    }
    let shape = authored_cubic(points);
    shape.oriented(if variant.traversed_reverse[index] {
        Orientation::Reversed
    } else {
        Orientation::Forward
    })
}
fn plane(reflected: bool) -> Plane3 {
    if !reflected {
        return Plane3::default();
    }
    let axis = Axis3::new(
        Point3::new(0., 0., 0.),
        Direction3::new(0., 0., -1.).unwrap(),
        Direction3::new(-1., 0., 0.).unwrap(),
    )
    .unwrap();
    Plane3::from_position(axis)
}
fn join(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &crate::sketch_region_prism::PrismReceipt,
) -> AnnularFrozenDomain {
    let binding = current_binding(request, receipt);
    let names = authored(request, receipt, &binding);
    let proof =
        inspect_bound_annular_embedding(request, receipt, binding, embedding_options(), || false)
            .unwrap_or_else(|failure| panic!("variant embedding: {:?}", failure.classification()));
    join_annular_frozen_domain(
        request,
        receipt,
        &names,
        proof,
        join_options(),
        &mut ContinueJoin,
    )
    .unwrap_or_else(|failure| {
        panic!(
            "variant join: {:?}/{:?}",
            failure.phase,
            failure.classification()
        )
    })
}
fn embedding_options() -> AnnularEmbeddingOptions {
    AnnularEmbeddingOptions {
        version: 2,
        regularity: AnnularRegularityOptions {
            boundary: BezierRegularityOptions::v1(RootIsolationOptions::new(1e-12)),
            total_bisections: 0,
        },
    }
}

struct ContinueJoin;
impl AnnularFrozenProgress for ContinueJoin {
    fn checkpoint(
        &mut self,
        _: AnnularFrozenPhase,
    ) -> Result<bool, rustycad_tkernel::standard::Failure> {
        Ok(false)
    }
}

fn current_binding(
    request: &TwoLoopRegionRequest<'_, '_>,
    receipt: &crate::sketch_region_prism::PrismReceipt,
) -> AnnularPrismBinding {
    let region =
        crate::sketch_region_prism::region_certificate::annular_binding::tests::fresh_region(
            request,
        );
    bind_annular_prism_v2(request, region, receipt, &mut ContinueAnnularFixture).unwrap_or_else(
        |cause| {
            panic!(
                "variant binding at {:?}: {:?}",
                cause.stage,
                cause.classification()
            )
        },
    )
}
fn join_options() -> AnnularFrozenOptions {
    AnnularFrozenOptions {
        version: 2,
        maximum_name_units: 1024,
    }
}
