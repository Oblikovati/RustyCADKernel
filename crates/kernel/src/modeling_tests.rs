// SPDX-License-Identifier: GPL-2.0-only
//! `OperationResult` entity numbers (ADR 0068) agree with `BodyTopology` (ADR 0076).
use crate::modeling::*;
use crate::topology::{BodyTopology, EdgeIndex, FaceIndex, VertexIndex};

fn resolution() -> Resolution {
    let display = DisplayTolerance {
        chordal: 1e-3,
        angular: 0.1,
    };
    Resolution::new(1e-6, 1e-7, 1e-9, display, ToleranceCap::Refuse(1e-6))
}

/// A 4 x 2 slot (two lines, two half circles) with a circular hole.
fn slot_with_hole() -> SketchProfile {
    let corners = [[0., 0.], [4., 0.], [4., 2.], [0., 2.]];
    let vertices = (0..4).map(|i| ProfileVertex {
        id: ProfileVertexId(i),
        point: corners[i as usize],
    });
    let (v, c) = (ProfileVertexId, ProfileCurveId);
    let ccw = ArcSense::CounterClockwise;
    let curves = vec![
        ProfileCurve::Line {
            id: c(10),
            start: v(0),
            end: v(1),
        },
        ProfileCurve::Arc {
            id: c(11),
            start: v(1),
            end: v(2),
            center: [4., 1.],
            sense: ccw,
        },
        ProfileCurve::Line {
            id: c(12),
            start: v(2),
            end: v(3),
        },
        ProfileCurve::Arc {
            id: c(13),
            start: v(3),
            end: v(0),
            center: [0., 1.],
            sense: ccw,
        },
        ProfileCurve::Circle {
            id: c(14),
            center: [2., 1.],
            radius: 0.5,
        },
    ];
    SketchProfile {
        plane: ProfilePlane::world_xy(),
        vertices: vertices.collect(),
        curves,
    }
}

fn extruded(profile: &SketchProfile) -> OperationResult {
    let set = build_planar_regions(profile, resolution()).unwrap();
    let options = ExtrudeOptions {
        resolution: resolution(),
    };
    extrude(&set.regions()[0], [0., 0., 1.5], &options).unwrap()
}

#[test]
fn extrusion_entity_numbers_agree_with_body_topology() {
    let result = extruded(&slot_with_hole());
    let topology = BodyTopology::enumerate(result.solid()).unwrap();
    assert_eq!(topology.face_count(), result.faces().len());
    assert_eq!(topology.edge_count(), result.edges().len());
    assert_eq!(topology.vertex_count(), result.vertices().len());
    for (index, face) in result.faces().iter().enumerate() {
        assert_eq!(topology.face_index(face), Some(FaceIndex(index as u32)));
    }
    for (index, edge) in result.edges().iter().enumerate() {
        assert_eq!(topology.edge_index(edge), Some(EdgeIndex(index as u32)));
    }
    for (index, vertex) in result.vertices().iter().enumerate() {
        assert_eq!(
            topology.vertex_index(vertex),
            Some(VertexIndex(index as u32))
        );
    }
}

#[test]
fn history_face_records_resolve_through_body_topology() {
    let result = extruded(&slot_with_hole());
    let topology = BodyTopology::enumerate(result.solid()).unwrap();
    for record in &result.history().records {
        let ResultEntity::Face(index) = record.result else {
            continue;
        };
        let face = topology.face(FaceIndex(index as u32)).unwrap();
        assert!(face.is_same(&result.faces()[index]), "{record:?}");
    }
}

/// ADR 0081: the extrusion volume and `body_properties` report one `Measured` type; both
/// enclose the closed form `(8 + pi - pi / 4) * 1.5` and their intervals overlap.
#[test]
fn operation_volume_is_a_body_properties_measure() {
    use crate::properties::{Evidence, IntegrationMethod, Measured, body_properties};
    let result = extruded(&slot_with_hole());
    let volume: Measured = result.volume();
    let expected = (8.0 + std::f64::consts::PI * 0.75) * 1.5;
    assert!(volume.encloses(expected), "{volume:?}");
    assert_eq!(volume.method, IntegrationMethod::RepresentationEnclosure);
    assert_eq!(volume.evidence(), Evidence::Estimated);
    let measured = body_properties(result.solid()).unwrap().volume;
    assert!(measured.encloses(expected), "{measured:?}");
    let gap = (measured.value - volume.value).abs();
    assert!(gap <= measured.error_bound + volume.error_bound, "{gap}");
}
