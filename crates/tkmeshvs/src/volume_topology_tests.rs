// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use std::error::Error;

#[test]
fn cloned_connectivity_has_no_shared_mutable_storage() {
    for factory in [create_prism_topology, create_pyramid_topology] {
        let original = factory(4).unwrap();
        let mut clone = original.clone();
        assert_ne!(original.nodes.as_ptr(), clone.nodes.as_ptr());
        assert_ne!(original.face_offsets.as_ptr(), clone.face_offsets.as_ptr());
        clone.nodes[0] = 99;
        clone.face_offsets[1] = 0;
        assert_eq!(original, factory(4).unwrap());
        drop(clone);
        assert_eq!(original.face_count(), factory(4).unwrap().face_count());
    }
}

#[test]
fn maximum_index_sizes_are_checked_without_allocating() {
    assert_maximum_sizes(MeshVSTopologyOperation::Prism, i32::MAX / 2 + 1, 6, 3);
    assert_maximum_sizes(MeshVSTopologyOperation::Pyramid, i32::MAX - 1, 4, 2);
}

fn assert_maximum_sizes(
    operation: MeshVSTopologyOperation,
    maximum: i32,
    entries: usize,
    extra: usize,
) {
    let sizes = topology_sizes(maximum, operation);
    if usize::BITS >= 64 {
        assert_eq!(
            sizes.unwrap(),
            (maximum as usize * entries, maximum as usize + extra)
        );
    } else {
        assert!(matches!(
            sizes,
            Err(MeshVSTopologyError::IndexOverflow { .. })
        ));
    }
    let excessive = topology_sizes(maximum + 1, operation);
    assert!(matches!(
        excessive,
        Err(MeshVSTopologyError::IndexOverflow { .. })
    ));
}

#[test]
fn count_failure_includes_operation_and_expected_shape() {
    let invalid = create_prism_topology(2).unwrap_err();
    let message = invalid.to_string();
    assert!(message.contains("Prism base_points=2; expected at least 3"));
    assert!(invalid.source().is_none());
}

#[test]
fn overflow_failure_includes_operation_and_expected_shape() {
    let overflow = create_pyramid_topology(i32::MAX).unwrap_err();
    let message = overflow.to_string();
    assert!(message.contains("Pyramid base_points=2147483647; expected"));
    assert!(overflow.source().is_none());
}

#[test]
fn allocation_failure_includes_operation_count_and_original_cause() {
    let allocation =
        reserve_topology_buffer::<i32>(usize::MAX, 3, MeshVSTopologyOperation::Prism).unwrap_err();
    let message = allocation.to_string();
    assert!(message.contains("Prism base_points=3; expected allocatable"));
    assert!(allocation.source().is_some());
}
