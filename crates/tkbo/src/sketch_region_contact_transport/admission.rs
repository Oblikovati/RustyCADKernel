// SPDX-License-Identifier: GPL-2.0-only
//! Report-local construction conformance never establishes cross-report contact equality.
use super::*;
use crate::sketch_region::OriginalMemberIdentity;
use rustycad_tkmath::predicates::roots::{
    bezier_pair::PairParameterDecision, metric_bounds::ResidualDecision,
};

pub(crate) struct BoundedContactConstruction {
    pub vertex_index: i32,
    pub identity: ReportContactIdentity,
    pub directed_contexts: Vec<(usize, crate::sketch_region::PartitionOccurrenceIdentity)>,
}
pub(super) fn inspect_contact_admission(retained: &mut PairedContactTransaction) {
    inspect_report_cardinalities(retained);
    for index in 0..retained.inspection.contacts.len() {
        let decision = bounded_construction(retained, index);
        let contact = &mut retained.inspection.contacts[index];
        contact.missing.clear();
        match decision {
            Ok(bounded) => contact.bounded_construction = Some(bounded),
            Err(cause) => {
                contact.bounded_construction = None;
                contact.missing.push(cause);
            }
        }
    }
    reject_cross_report_aliases(retained);
}
fn inspect_report_cardinalities(retained: &mut PairedContactTransaction) {
    // WHY: admission can be repeated after evidence repair; old observations are not current proof.
    retained
        .inspection
        .missing
        .retain(|gap| !matches!(gap, ContactTransportGap::ContactSetCardinality { .. }));
    let mut seen = Vec::new();
    for (index, domain) in retained.census.domains.iter().enumerate() {
        let Some(report) = paired_report_index(domain) else {
            continue;
        };
        if seen.contains(&report) {
            continue;
        }
        seen.push(report);
        if let Some(gap) = current_cardinality_gap(retained, index, report) {
            retained.inspection.missing.push(gap);
        }
    }
}
fn paired_report_index(
    domain: &crate::sketch_region_certification::BoundaryDomain,
) -> Option<usize> {
    match domain.evidence {
        crate::sketch_region_certification::DomainEvidence::PairedReport { report, .. } => {
            Some(report)
        }
        _ => None,
    }
}
fn current_cardinality_gap(
    retained: &PairedContactTransaction,
    domain_index: usize,
    report: usize,
) -> Option<ContactTransportGap> {
    let domain = &retained.census.domains[domain_index];
    let proof = crate::sketch_region_certification::domain_certificate(&retained.census, domain)?;
    let rows = super::topology::observed_domain_vertices(retained, domain_index);
    let observed = unique_vertex_indices(&rows).len();
    (observed != proof.contact_count()).then_some(ContactTransportGap::ContactSetCardinality {
        report,
        expected: proof.contact_count(),
        observed,
    })
}
fn unique_vertex_indices(rows: &[ObservedContactVertex]) -> Vec<i32> {
    let mut indices = Vec::new();
    for row in rows {
        if !indices.contains(&row.vertex_index) {
            indices.push(row.vertex_index);
        }
    }
    indices
}
fn bounded_construction(
    retained: &PairedContactTransaction,
    index: usize,
) -> Result<BoundedContactConstruction, ContactTransportGap> {
    check_contact_prerequisites(retained, index)?;
    if let Some(cause) = additional_source_gap(retained, index) {
        return Err(cause);
    }
    let contact = &retained.inspection.contacts[index];
    let eligible = eligible_vertices(retained, index);
    let selected = choose_eligible_vertex(contact.identity, &eligible)?;
    Ok(bounded_vertex(contact.identity, selected))
}
fn bounded_vertex(
    identity: ReportContactIdentity,
    selected: &ObservedContactVertex,
) -> BoundedContactConstruction {
    BoundedContactConstruction {
        vertex_index: selected.vertex_index,
        identity,
        directed_contexts: selected
            .generated_observations
            .iter()
            .map(|row| (row.area, row.directed_occurrence.clone()))
            .collect(),
    }
}
fn bounded_observed_vertices(
    contact: &ObservedPairedContact,
) -> impl Iterator<Item = &ObservedContactVertex> {
    contact
        .observed_vertices
        .iter()
        .filter(|row| vertex_bounds_hold(contact, row))
}

fn eligible_vertices(
    retained: &PairedContactTransaction,
    index: usize,
) -> Vec<&ObservedContactVertex> {
    let contact = &retained.inspection.contacts[index];
    bounded_observed_vertices(contact)
        .filter(|row| generated_parents_hold(retained, index, row))
        .collect()
}
fn choose_eligible_vertex<'a>(
    identity: ReportContactIdentity,
    eligible: &[&'a ObservedContactVertex],
) -> Result<&'a ObservedContactVertex, ContactTransportGap> {
    if eligible.is_empty() {
        return Err(ContactTransportGap::ConstructionConformanceUnproved { identity });
    }
    unique_eligible_vertex(eligible).ok_or(ContactTransportGap::AmbiguousPairIncidence {
        identity,
        count: eligible.len(),
    })
}

fn check_contact_prerequisites(
    retained: &PairedContactTransaction,
    index: usize,
) -> Result<(), ContactTransportGap> {
    check_source_identity(retained, index)?;
    let contact = &retained.inspection.contacts[index];
    if report_cardinality_missing(retained, contact.identity.report) {
        return Err(ContactTransportGap::ReportContactCoverageUnproved {
            report: contact.identity.report,
        });
    }
    check_simple_root(retained, index)?;
    if contact.observed_vertices.is_empty() {
        return Err(ContactTransportGap::MissingActualPairIncidence {
            identity: contact.identity,
        });
    }
    Ok(())
}
fn check_source_identity(
    retained: &PairedContactTransaction,
    index: usize,
) -> Result<(), ContactTransportGap> {
    if source_identities_exact(retained, index) {
        return Ok(());
    }
    let contact = &retained.inspection.contacts[index];
    let occurrence = retained.census.inputs.segments
        [retained.census.domains[contact.occurrence_domains[0]].occurrences[0]]
        .occurrence;
    Err(ContactTransportGap::MissingExactSourceCorrespondence { occurrence })
}

fn report_cardinality_missing(retained: &PairedContactTransaction, report: usize) -> bool {
    retained.inspection.missing.iter().any(|gap| {
        matches!(gap,
        ContactTransportGap::ContactSetCardinality { report: actual, .. } if *actual == report)
    })
}
fn unique_eligible_vertex<'a>(
    rows: &[&'a ObservedContactVertex],
) -> Option<&'a ObservedContactVertex> {
    let first = *rows.first()?;
    if rows
        .iter()
        .any(|row| row.vertex_index != first.vertex_index)
    {
        return None;
    }
    // WHY: two distinct parameter tuples at the same vertex remain ambiguous even within one report.
    rows.iter()
        .all(|row| {
            row.native_parameters.map(f64::to_bits) == first.native_parameters.map(f64::to_bits)
        })
        .then_some(first)
}
fn source_identities_exact(retained: &PairedContactTransaction, index: usize) -> bool {
    let Some(axes) = super::topology::canonical_occurrences(retained, index) else {
        return false;
    };
    axes.into_iter().all(|axis| {
        let source = &retained.census.inputs.segments[axis];
        !source.parents.is_empty()
            && source.parents.iter().all(|parent| {
                retained.exact_sources.iter().any(|row| {
                    row.occurrence == source.occurrence
                        && row.original == *parent
                        && matches!(&row.decision,
                    Ok(ResidualDecision::Within { proven_squared_upper })
                    if proven_squared_upper.compare_f64(0.).is_ok_and(|order| order.is_eq()))
                })
            })
    })
}
fn vertex_bounds_hold(contact: &ObservedPairedContact, vertex: &ObservedContactVertex) -> bool {
    if !parameter_bounds_hold(vertex) {
        return false;
    }
    let world = contact
        .world_measurements
        .iter()
        .filter(|row| row.vertex_index == vertex.vertex_index)
        .collect::<Vec<_>>();
    !world.is_empty()
        && world.iter().all(|row| {
            matches!(&row.report, Ok(report)
        if matches!(report.decision, Ok(ResidualDecision::Within { .. })))
        })
}
fn parameter_bounds_hold(vertex: &ObservedContactVertex) -> bool {
    let Some(Ok(Some(axes))) = &vertex.parameter_measurement else {
        return false;
    };
    axes.iter()
        .all(|axis| axis.decision() == PairParameterDecision::Within)
}

fn generated_parents_hold(
    retained: &PairedContactTransaction,
    index: usize,
    vertex: &ObservedContactVertex,
) -> bool {
    let Some(axes) = super::topology::canonical_occurrences(retained, index) else {
        return false;
    };
    let parents = source_axis_parents(retained, axes);
    super::topology::only_axis_incidences(retained, axes, vertex.vertex_index)
        && !vertex.generated_observations.is_empty()
        && vertex.generated_observations.iter().all(|row| {
            actual_points_read(row)
                && copied_history_preserved(row)
                && (same_parent_set(&parents, &row.native_history.generated)
                    || original_endpoint_history(retained, axes, vertex, row))
        })
}

fn source_axis_parents(
    retained: &PairedContactTransaction,
    axes: [usize; 2],
) -> Vec<&OriginalMemberIdentity> {
    axes.into_iter()
        .flat_map(|axis| &retained.census.inputs.segments[axis].parents)
        .collect()
}

fn actual_points_read(row: &super::generated::GeneratedContactObservation) -> bool {
    matches!(row.copied_point, Some(Ok(_)))
        && matches!(row.directed_point, Some(Ok(_)))
        && row.copied_index != 0
}
fn copied_history_preserved(row: &super::generated::GeneratedContactObservation) -> bool {
    let modified = row
        .native_history
        .preserved
        .iter()
        .chain(&row.native_history.modified)
        .collect::<Vec<_>>();
    row.copied_history.preserved.is_empty()
        && same_parent_set(&modified, &row.copied_history.modified)
        && same_parent_set(
            &row.native_history.generated.iter().collect::<Vec<_>>(),
            &row.copied_history.generated,
        )
}
fn same_parent_set(
    expected: &[&OriginalMemberIdentity],
    actual: &[OriginalMemberIdentity],
) -> bool {
    expected.iter().all(|parent| actual.contains(parent))
        && actual.iter().all(|parent| expected.contains(&parent))
}
fn reject_cross_report_aliases(retained: &mut PairedContactTransaction) {
    let selected = retained
        .inspection
        .contacts
        .iter()
        .filter_map(|row| row.bounded_construction.as_ref())
        .map(|row| (row.identity, row.vertex_index))
        .collect::<Vec<_>>();
    for contact in &mut retained.inspection.contacts {
        let Some(bounded) = &contact.bounded_construction else {
            continue;
        };
        if let Some(gap) = selected_alias_gap(&selected, bounded) {
            contact.missing.push(gap);
            contact.bounded_construction = None;
        }
    }
}
fn selected_alias_gap(
    selected: &[(ReportContactIdentity, i32)],
    bounded: &BoundedContactConstruction,
) -> Option<ContactTransportGap> {
    let aliases = same_vertex_aliases(selected, bounded);
    if aliases.is_empty() {
        return None;
    }
    let cross_report = aliases
        .iter()
        .any(|(id, _)| id.report != bounded.identity.report);
    if cross_report {
        return Some(ContactTransportGap::CrossReportEqualityUnproved {
            vertex_index: bounded.vertex_index,
        });
    }
    let identity = bounded.identity;
    let count = aliases.len() + 1;
    Some(ContactTransportGap::AmbiguousPairIncidence { identity, count })
}

fn same_vertex_aliases<'a>(
    selected: &'a [(ReportContactIdentity, i32)],
    bounded: &BoundedContactConstruction,
) -> Vec<&'a (ReportContactIdentity, i32)> {
    selected
        .iter()
        .filter(|(id, vertex)| *vertex == bounded.vertex_index && *id != bounded.identity)
        .collect()
}

fn original_endpoint_history(
    retained: &PairedContactTransaction,
    axes: [usize; 2],
    vertex: &ObservedContactVertex,
    row: &super::generated::GeneratedContactObservation,
) -> bool {
    let native = row
        .native_history
        .preserved
        .iter()
        .chain(&row.native_history.modified)
        .collect::<Vec<_>>();
    !vertex.original_parents.is_empty()
        && same_parent_set(&native, &vertex.original_parents)
        && matches_source_endpoints(retained, axes, vertex)
}

fn matches_source_endpoints(
    retained: &PairedContactTransaction,
    axes: [usize; 2],
    vertex: &ObservedContactVertex,
) -> bool {
    axes.into_iter().enumerate().all(|(axis, occurrence)| {
        super::topology::original_source_endpoint(
            retained,
            occurrence,
            vertex.vertex_index,
            vertex.native_parameters[axis],
        )
    })
}

fn check_simple_root(
    retained: &PairedContactTransaction,
    index: usize,
) -> Result<(), ContactTransportGap> {
    let multiplicity = retained_root_multiplicity(retained, index)?;
    if multiplicity != 1 {
        return Err(ContactTransportGap::RepeatedRootCorrespondence {
            identity: retained.inspection.contacts[index].identity,
            multiplicity,
        });
    }
    Ok(())
}
fn retained_root_multiplicity(
    retained: &PairedContactTransaction,
    index: usize,
) -> Result<usize, ContactTransportGap> {
    let contact = &retained.inspection.contacts[index];
    let domain_index = contact.occurrence_domains[0];
    let domain = &retained.census.domains[domain_index];
    let proof = crate::sketch_region_certification::domain_certificate(&retained.census, domain)
        .ok_or(ContactTransportGap::MissingSourceBoundDomain {
            domain: domain_index,
        })?;
    let root = (0..proof.contact_count())
        .filter_map(|i| proof.contact(i))
        .find(|root| root.candidate_index == contact.identity.candidate_root)
        .ok_or(ContactTransportGap::MissingRetainedContactRoot {
            identity: contact.identity,
            contact_count: proof.contact_count(),
        })?;
    Ok(root.parameter.multiplicity())
}

fn additional_source_gap(
    retained: &PairedContactTransaction,
    index: usize,
) -> Option<ContactTransportGap> {
    let contact = &retained.inspection.contacts[index];
    let axes = super::topology::canonical_occurrences(retained, index)?;
    for vertex in bounded_observed_vertices(contact) {
        let sources =
            super::topology::unrelated_source_indices(retained, axes, vertex.vertex_index)?;
        if !sources.is_empty() {
            return Some(ContactTransportGap::AdditionalIncidentSources {
                identity: contact.identity,
                vertex_index: vertex.vertex_index,
                sources,
            });
        }
    }
    None
}

#[cfg(test)]
mod reuse_tests {
    use super::*;

    #[test]
    fn admission_recomputes_cardinality_instead_of_retaining_stale_observations() {
        let mut retained = super::super::test_fixture::actual_bridge();
        let report = retained.inspection.contacts[0].identity.report;
        retained
            .inspection
            .missing
            .push(ContactTransportGap::ContactSetCardinality {
                report,
                expected: 2,
                observed: 1,
            });
        inspect_contact_admission(&mut retained);
        assert!(!report_cardinality_missing(&retained, report));
        assert!(!super::super::test_fixture::bounded_contacts(&retained).is_empty());
        inspect_contact_admission(&mut retained);
        assert!(!report_cardinality_missing(&retained, report));
    }
}
