// SPDX-License-Identifier: GPL-2.0-only
use super::{fixtures::*, *};
#[test]
fn real_two_root_pair_retains_each_physical_endpoint_once_without_contact_assignment() {
    let edges = two_root_edges();
    let snapshot = actual_binding_snapshot(&edges);
    let inspection = inspect_partition_bindings(&snapshot);
    let cuts = shared_endpoints(&inspection);
    assert_two_root_binding(&inspection, &cuts);
}

fn shared_endpoints(inspection: &PartitionBindingInspection) -> Vec<&PartitionEndpointBinding> {
    inspection
        .endpoints
        .iter()
        .filter(|row| {
            row.incidences.iter().any(|edge| edge.source_index == 0)
                && row.incidences.iter().any(|edge| edge.source_index == 3)
        })
        .collect()
}
fn assert_two_root_binding(
    inspection: &PartitionBindingInspection,
    cuts: &[&PartitionEndpointBinding],
) {
    assert_eq!(
        cuts.len(),
        2,
        "actual two-root pair; expected two unique physical endpoint bindings"
    );
    assert_ne!(cuts[0].vertex_index, cuts[1].vertex_index);
    assert_eq!(
        inspection.canonical_contact,
        CanonicalContactGap::MissingPairedSourceProof
    );
}

#[test]
fn actual_native_singleton_cut_stays_distinct_from_exact_half() {
    let edges = single_root_edges();
    let snapshot = actual_binding_snapshot(&edges);
    let cut = snapshot.sources[0].blocks[0].own_range[1];
    assert_eq!(cut, 0.49999999999996131);
    assert_ne!(cut, 0.5);
    let observed = inspect_partition_bindings(&snapshot);
    assert_eq!(shared_endpoints(&observed).len(), 1);
    assert_eq!(
        observed.canonical_contact,
        CanonicalContactGap::MissingPairedSourceProof
    );
}

#[test]
fn unsplit_original_endpoints_bind_actual_source_children_without_invented_paves() {
    let mut edges = two_root_edges();
    edges.truncate(1);
    let snapshot = actual_binding_snapshot(&edges);
    assert!(snapshot.sources[0].blocks.is_empty());
    let inspected = inspect_partition_bindings(&snapshot);
    assert_eq!(inspected.endpoints.len(), 2);
    for endpoint in inspected.endpoints {
        assert_eq!(endpoint.incidences.len(), 1);
        assert_eq!(
            endpoint.incidences[0].origin,
            EndpointIncidenceOrigin::UnreferencedSourceChild
        );
        assert_eq!(endpoint.incidences[0].own_edge, 0);
        assert!(endpoint.incidences[0].representative.is_none());
    }
}

#[test]
fn closed_unsplit_source_retains_both_directed_native_end_parameters() {
    let edges = vec![crate::closed_analytic_cubic::closed_analytic_cubic(
        1., false,
    )];
    let snapshot = actual_binding_snapshot(&edges);
    assert_closed_source_children(&snapshot.sources[0]);
    let inspected = inspect_partition_bindings(&snapshot);
    assert_eq!(inspected.endpoints.len(), 1);
    assert_eq!(inspected.endpoints[0].incidences.len(), 2);
}

fn assert_closed_source_children(source: &crate::sketch_region::SourcePartitionObservation) {
    assert_eq!(source.source_vertices.len(), 2);
    assert_eq!(
        source.source_vertices[0].shape_index,
        source.source_vertices[1].shape_index
    );
    assert_ne!(
        source.source_vertices[0].occurrence.orientation,
        source.source_vertices[1].occurrence.orientation
    );
    assert_eq!(
        [
            source.source_vertices[0].parameter,
            source.source_vertices[1].parameter
        ],
        [0., 1.]
    );
}
