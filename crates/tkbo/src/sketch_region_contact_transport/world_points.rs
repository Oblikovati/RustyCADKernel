// SPDX-License-Identifier: GPL-2.0-only
//! Reuse identical actual point queries while retaining every native/copy/directed role.
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ObservedPointRole {
    Native,
    Copied {
        index: usize,
        occurrence: PartitionOccurrenceIdentity,
    },
    Directed {
        area: usize,
        occurrence: PartitionOccurrenceIdentity,
    },
}
pub(super) struct ContactWorldPoint {
    pub vertex_index: i32,
    pub point: [f64; 3],
    pub roles: Vec<ObservedPointRole>,
}
pub(super) fn contact_world_points(contact: &ObservedPairedContact) -> Vec<ContactWorldPoint> {
    let mut points = Vec::new();
    for vertex in &contact.observed_vertices {
        append_world_point(
            &mut points,
            vertex.vertex_index,
            vertex.native_point,
            ObservedPointRole::Native,
        );
        for row in &vertex.generated_observations {
            append_generated_points(&mut points, vertex.vertex_index, row);
        }
    }
    points
}
fn append_generated_points(
    points: &mut Vec<ContactWorldPoint>,
    vertex: i32,
    row: &super::generated::GeneratedContactObservation,
) {
    if let Some(Ok(point)) = &row.copied_point {
        let role = ObservedPointRole::Copied {
            index: row.copied_index,
            occurrence: row.copied_image.clone(),
        };
        append_world_point(points, vertex, *point, role);
    }
    if let Some(Ok(point)) = &row.directed_point {
        let role = ObservedPointRole::Directed {
            area: row.area,
            occurrence: row.directed_occurrence.clone(),
        };
        append_world_point(points, vertex, *point, role);
    }
}
fn append_world_point(
    points: &mut Vec<ContactWorldPoint>,
    vertex_index: i32,
    point: [f64; 3],
    role: ObservedPointRole,
) {
    if let Some(previous) = points.iter_mut().find(|row| {
        row.vertex_index == vertex_index && row.point.map(f64::to_bits) == point.map(f64::to_bits)
    }) {
        if !previous.roles.contains(&role) {
            previous.roles.push(role);
        }
        return;
    }
    points.push(ContactWorldPoint {
        vertex_index,
        point,
        roles: vec![role],
    });
}
