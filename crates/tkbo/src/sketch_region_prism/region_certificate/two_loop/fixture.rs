// SPDX-License-Identifier: GPL-2.0-only
//! Authored poles and identities precede every native query; ADR0046 independent fixture.
use crate::sketch_region::*;
use crate::sketch_region_candidates::observe_candidate_copies;
use crate::sketch_region_certification::*;
use crate::sketch_region_contact_transport::*;
use crate::sketch_region_partition_bindings::bind_partition_outputs;
use crate::sketch_region_source_conformance::*;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve};
use rustycad_tkmath::{
    gp::{Plane3, Point3},
    predicates::roots::RootIsolationOptions,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

pub(super) const OUTER: RegionSourceId = RegionSourceId(101);
pub(super) const INNER: RegionSourceId = RegionSourceId(202);
pub(super) struct TwoLoopFixture {
    pub outer: Shape,
    pub inner: Shape,
    pub bridge: PairedContactTransaction,
}
impl TwoLoopFixture {
    pub fn nested() -> Self {
        let outer = cubic(1., [0., 0.]);
        let inner = cubic(0.5, [0.5, 0.5]);
        Self::for_edges(outer, inner)
    }
    pub fn for_edges(outer: Shape, inner: Shape) -> Self {
        let bridge = bridge(&authored_sources(&outer, &inner));
        Self {
            outer,
            inner,
            bridge,
        }
    }
    pub fn sources(&self) -> [ArrangementSource<'_>; 2] {
        authored_sources(&self.outer, &self.inner)
    }
}
fn authored_sources<'a>(outer: &'a Shape, inner: &'a Shape) -> [ArrangementSource<'a>; 2] {
    [
        ArrangementSource {
            identity: OUTER,
            shape: outer,
        },
        ArrangementSource {
            identity: INNER,
            shape: inner,
        },
    ]
}
pub(super) fn cubic(scale: f64, offset: [f64; 2]) -> Shape {
    let points = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]]
        .map(|p| Point3::new(offset[0] + scale * p[0], offset[1] + scale * p[1], 0.));
    let poles = Array1::from_shared(&points, 1);
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomBezierCurve::new(&poles).unwrap()).cast();
    BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap()
        .into()
}
fn roots() -> RootIsolationOptions {
    RootIsolationOptions::new(0.001)
}
pub(super) fn census(sources: &[ArrangementSource<'_>]) -> BoundaryCensus {
    let inputs = capture_boundary_inputs(sources, Plane3::default())
        .unwrap_or_else(|_| panic!("authored loops; expected source capture"));
    let policy = CensusPolicy {
        roots: roots(),
        max_world_error: 1e-8,
    };
    let source_policy = SourceConformancePolicy {
        roots: roots(),
        original_plane_limit: 0.,
        construction_limit: 0.,
    };
    enumerate_source_bound_census(inputs, policy, source_policy, || false)
        .unwrap_or_else(|_| panic!("authored loops; expected retained census"))
}
fn mapped(
    sources: &[ArrangementSource<'_>],
) -> crate::sketch_region_partition_bindings::PartitionBoundOutputs {
    let observed = observe_partition_space(sources, &mut |_| false, capture_space_representation);
    let ArrangementCompletion::Arranged(owned) = &observed.completion else {
        panic!("authored loops; expected actual owned arrangement");
    };
    let copied = observe_candidate_copies(
        owned,
        Plane3::default(),
        &mut |_| false,
        PartitionOccurrenceIdentity::capture,
    );
    let originals = capture_originals(sources).unwrap();
    bind_partition_outputs(observed, Some(copied), originals, sources)
}
fn bridge(sources: &[ArrangementSource<'_>]) -> PairedContactTransaction {
    let policy = ContactTransportPolicy {
        native_parameter_limits: [1e-8; 2],
        world_endpoint_limit: 1e-8,
        roots: roots(),
    };
    transport_partition_contacts(census(sources), mapped(sources), policy, sources, |_| false)
        .unwrap_or_else(|_| panic!("authored loops; expected retained bridge"))
}
