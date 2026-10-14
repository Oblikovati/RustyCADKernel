// SPDX-License-Identifier: GPL-2.0-only
//! Retained allocation observations, not a transient allocator profiler or release benchmark.
use super::*;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_prism::{owned_request::*, surface_charts::CapturedPrismSurface};
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use std::mem::size_of;

#[derive(Default)]
struct RetainedBodyStorage {
    blocks: usize,
    capacity_bytes: usize,
    curve_scalar_bytes: usize,
}
impl RetainedBodyStorage {
    fn vector<T>(&mut self, values: &Vec<T>) {
        self.blocks += usize::from(values.capacity() != 0);
        self.capacity_bytes += values.capacity() * size_of::<T>();
    }
    fn key(&mut self, value: &names::SemanticUseKey) {
        self.vector(&value.0);
    }
    fn keys(&mut self, values: &Vec<names::SemanticUseKey>) {
        self.vector(values);
        for value in values {
            self.key(value);
        }
    }
    fn curve<const D: usize>(&mut self, value: &CapturedMetricCurve<D>) {
        if let CapturedMetricCurve::Bezier { poles, weights } = value {
            self.vector(poles);
            self.curve_scalar_bytes += poles.len() * size_of::<[f64; D]>();
            if let Some(weights) = weights {
                self.vector(weights);
                self.curve_scalar_bytes += weights.len() * size_of::<f64>();
            }
        }
    }
    fn space(&mut self, value: &CapturedSpaceRepresentation) {
        self.curve(&value.curve);
    }
}
fn observe(body: &OwnedNamedPrismBody) -> RetainedBodyStorage {
    let mut cost = RetainedBodyStorage::default();
    cost.vector(&body.naming_binding);
    cost.key(&body.root);
    members(body, &mut cost);
    occurrences(body, &mut cost);
    faces(body, &mut cost);
    incidence(body, &mut cost);
    cost
}
fn members(body: &OwnedNamedPrismBody, cost: &mut RetainedBodyStorage) {
    cost.vector(&body.rows.members);
    for row in &body.rows.members {
        if let Some(curve) = &row.curve {
            cost.space(curve);
        }
    }
}
fn occurrences(body: &OwnedNamedPrismBody, cost: &mut RetainedBodyStorage) {
    cost.vector(&body.rows.occurrences);
    for row in &body.rows.occurrences {
        cost.key(&row.key);
        cost.keys(&row.children);
    }
}
fn faces(body: &OwnedNamedPrismBody, cost: &mut RetainedBodyStorage) {
    cost.vector(&body.rows.faces);
    for row in &body.rows.faces {
        if let CapturedPrismSurface::Extrusion { basis, .. } = &row.surface {
            cost.curve(basis);
        }
        cost.vector(&row.trims);
        for trim in &row.trims {
            cost.key(&trim.occurrence);
            cost.curve(&trim.pcurve);
            cost.space(&trim.space);
        }
    }
}
fn incidence(body: &OwnedNamedPrismBody, cost: &mut RetainedBodyStorage) {
    cost.vector(&body.rows.endpoints);
    cost.vector(&body.rows.coedges);
    for row in &body.rows.coedges {
        cost.key(&row.occurrence);
    }
    cost.vector(&body.rows.wires);
    for row in &body.rows.wires {
        cost.key(&row.occurrence);
        cost.keys(&row.coedges);
    }
}
#[test]
fn actual_body_retained_storage_and_projection_latency_observations() {
    let fixture = default_resolved_prism_fixture();
    let body = project_named_prism_body(&fixture.publication, &fixture.names).unwrap();
    let observed = observe(&body);
    assert!(observed.blocks > 26);
    assert!(observed.capacity_bytes >= body.naming_binding.len());
    assert!(observed.curve_scalar_bytes > 0);
    let elapsed = repeated_projection(&fixture);
    print_observation(&body, &observed, elapsed);
}
fn repeated_projection(fixture: &ResolvedPrismFixture) -> u128 {
    let started = std::time::Instant::now();
    for _ in 0..32 {
        std::hint::black_box(
            project_named_prism_body(&fixture.publication, &fixture.names).unwrap(),
        );
    }
    started.elapsed().as_nanos()
}
fn print_observation(body: &OwnedNamedPrismBody, observed: &RetainedBodyStorage, nanos: u128) {
    // The dictionary's private Vec capacity is unavailable; report payload separately without guessing.
    let dictionary_payload = std::mem::size_of_val(body.plan.entries());
    println!(
        "{{\"scope\":\"actual-body-retained-vectors-excluding-dictionary-capacity\",\"blocks\":{},\"capacity_bytes\":{},\"curve_scalar_bytes\":{},\"dictionary_payload_bytes\":{},\"binding_bytes\":{},\"replay_bytes\":{},\"projection_iterations\":32,\"elapsed_ns\":{}}}",
        observed.blocks,
        observed.capacity_bytes,
        observed.curve_scalar_bytes,
        dictionary_payload,
        body.naming_binding.len(),
        body.replay_bytes().len(),
        nanos
    );
}
