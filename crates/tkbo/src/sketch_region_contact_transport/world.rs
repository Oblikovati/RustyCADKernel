// SPDX-License-Identifier: GPL-2.0-only
//! Exact original t brackets bound actual observed points without projecting or snapping them.
use super::*;
use crate::sketch_region_source_conformance::*;
use rustycad_tkmath::predicates::roots::{DyadicBound, RootIsolationOptions, metric_bounds::*};

pub(crate) struct ContactWorldMeasurement {
    pub vertex_index: i32,
    pub original: crate::sketch_region::OriginalMemberIdentity,
    pub point: [f64; 3],
    pub roles: Vec<super::world_points::ObservedPointRole>,
    pub requested_limit: f64,
    pub original_bounds: [DyadicBound; 2],
    pub report: Result<TransportedResidualReport, SourceConformanceCause>,
}
struct ContactMetricInputs<'a> {
    originals: &'a [OriginalCurveCapture],
    construction: &'a CapturedBoundaryConstruction,
    bounds: [DyadicBound; 2],
    requested: ContactTransportPolicy,
}
pub(super) fn retain_world_bounds(
    retained: &mut PairedContactTransaction,
    cancelled: &mut super::progress::ContactProgress<'_>,
) {
    for contact in 0..retained.inspection.contacts.len() {
        let rows = measure_contact_points(retained, contact, cancelled);
        retained.inspection.completed_bisections += rows
            .iter()
            .map(|row| row.report.as_ref().map_or(0, |report| report.bisections))
            .sum::<u64>();
        retained.inspection.contacts[contact].world_measurements = rows;
        if cancelled.stopped().is_some() {
            return;
        }
    }
}
fn measure_contact_points(
    retained: &PairedContactTransaction,
    contact: usize,
    cancelled: &mut super::progress::ContactProgress<'_>,
) -> Vec<ContactWorldMeasurement> {
    let Some(inputs) = contact_metric_inputs(retained, contact) else {
        return Vec::new();
    };
    let mut measured = Vec::new();
    let mut used = retained.inspection.completed_bisections;
    for vertex in &super::world_points::contact_world_points(&retained.inspection.contacts[contact])
    {
        append_vertex_bounds(&inputs, vertex, &mut measured, &mut used, cancelled);
        if cancelled.stopped().is_some() {
            break;
        }
    }
    measured
}
fn append_vertex_bounds(
    inputs: &ContactMetricInputs<'_>,
    vertex: &super::world_points::ContactWorldPoint,
    measured: &mut Vec<ContactWorldMeasurement>,
    used: &mut u64,
    cancelled: &mut super::progress::ContactProgress<'_>,
) {
    for parent in &inputs.construction.parents {
        let row = world_measurement(inputs, parent, vertex, *used, cancelled);
        *used += row.report.as_ref().map_or(0, |report| report.bisections);
        measured.push(row);
        if cancelled.stopped().is_some() {
            return;
        }
    }
}
fn contact_metric_inputs(
    retained: &PairedContactTransaction,
    contact: usize,
) -> Option<ContactMetricInputs<'_>> {
    let axes = super::topology::canonical_occurrences(retained, contact)?;
    let id = retained.census.inputs.segments[axes[0]].occurrence;
    let construction = retained
        .census
        .inputs
        .constructions
        .iter()
        .find(|row| row.occurrence == id)?;
    Some(ContactMetricInputs {
        construction,
        originals: &retained.census.inputs.originals,
        bounds: contact_bounds(retained, contact)?,
        requested: retained.requested,
    })
}
fn contact_bounds(retained: &PairedContactTransaction, contact: usize) -> Option<[DyadicBound; 2]> {
    let row = &retained.inspection.contacts[contact];
    let domain = &retained.census.domains[*row.occurrence_domains.first()?];
    let proof = crate::sketch_region_certification::domain_certificate(&retained.census, domain)?;
    let root = (0..proof.contact_count())
        .filter_map(|i| proof.contact(i))
        .find(|candidate| candidate.candidate_index == row.identity.candidate_root)?;
    let (first, last) = root.parameter.bounds();
    Some([first.clone(), last.clone()])
}
fn world_measurement(
    inputs: &ContactMetricInputs<'_>,
    parent: &crate::sketch_region::OriginalMemberIdentity,
    vertex: &super::world_points::ContactWorldPoint,
    used: u64,
    cancelled: &mut super::progress::ContactProgress<'_>,
) -> ContactWorldMeasurement {
    let report = certify_point(inputs, parent, vertex.point, used, cancelled);
    ContactWorldMeasurement {
        vertex_index: vertex.vertex_index,
        roles: vertex.roles.clone(),
        original: parent.clone(),
        point: vertex.point,
        requested_limit: inputs.requested.world_endpoint_limit,
        original_bounds: inputs.bounds.clone(),
        report,
    }
}
fn certify_point(
    inputs: &ContactMetricInputs<'_>,
    parent: &crate::sketch_region::OriginalMemberIdentity,
    point: [f64; 3],
    used: u64,
    cancelled: &mut super::progress::ContactProgress<'_>,
) -> Result<TransportedResidualReport, SourceConformanceCause> {
    let query = point_query(
        inputs,
        parent,
        point,
        inputs.requested.remaining_roots(used),
    )?;
    Ok(certify_transported_residual_on_bounds(
        query,
        inputs.bounds.clone(),
        || cancelled.poll(ContactTransportPhase::WorldConstruction),
    ))
}

fn point_query<'a>(
    inputs: &'a ContactMetricInputs<'a>,
    parent: &crate::sketch_region::OriginalMemberIdentity,
    point: [f64; 3],
    roots: RootIsolationOptions,
) -> Result<TransportedResidualQuery<'a>, SourceConformanceCause> {
    let query = super::exact_sources::exact_source_query(
        inputs.originals,
        inputs.construction,
        parent,
        roots,
    )?;
    Ok(point_comparator(
        query,
        point,
        inputs.requested.world_endpoint_limit,
    ))
}
fn point_comparator(
    mut query: TransportedResidualQuery<'_>,
    point: [f64; 3],
    limit: f64,
) -> TransportedResidualQuery<'_> {
    query.residual.pcurve = MetricCurve::Line {
        origin: [0.; 2],
        direction: [0.; 2],
    };
    query.residual.plane_embedding = MetricEmbedding {
        linear: [[0.; 2]; 3],
        origin: point,
        scale: 1.,
    };
    query.residual.plane_placement = identity_placement();
    query.residual.spatial_limit = limit;
    query
}
fn identity_placement() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: std::array::from_fn(|row| {
            std::array::from_fn(|column| if row == column { 1. } else { 0. })
        }),
        origin: [0.; 3],
        scale: 1.,
    }
}

/// Recompute a captured-source residual for a fresh point; `recertify_contact_point(bridge, 0, point, 0.)`.
pub(crate) fn recertify_contact_point(
    retained: &PairedContactTransaction,
    contact: usize,
    point: [f64; 3],
    limit: f64,
) -> Option<Result<TransportedResidualReport, SourceConformanceCause>> {
    let inputs = singleton_point_inputs(retained, contact)?;
    let [parent] = inputs.construction.parents.as_slice() else {
        return None;
    };
    let query = point_query(&inputs, parent, point, inputs.requested.remaining_roots(0));
    Some(query.map(|mut query| {
        // The region entry first proves the exact endpoint singleton; no root search is introduced.
        query.residual.spatial_limit = limit;
        certify_transported_residual_on_bounds(query, inputs.bounds.clone(), || false)
    }))
}

fn singleton_point_inputs(
    retained: &PairedContactTransaction,
    contact: usize,
) -> Option<ContactMetricInputs<'_>> {
    let inputs = contact_metric_inputs(retained, contact)?;
    let zero = inputs
        .bounds
        .iter()
        .all(|bound| bound.compare_f64(0.).is_ok_and(|order| order.is_eq()));
    // A fresh fixed endpoint proof cannot silently become a new unbudgeted contact search.
    zero.then_some(inputs)
}
