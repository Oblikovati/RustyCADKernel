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

fn spline(id: u64, poles: &[[f64; 2]], weights: Option<&[f64]>, knots: &[f64]) -> ProfileCurve {
    ProfileCurve::Spline {
        id: ProfileCurveId(id),
        start: ProfileVertexId(0),
        end: ProfileVertexId(1),
        degree: 2,
        poles: poles.to_vec(),
        weights: weights.map(<[f64]>::to_vec),
        knots: knots.to_vec(),
    }
}

/// A spline from vertex 0 to 1 closed by the line back.
fn spline_segment(curve: ProfileCurve, ends: [[f64; 2]; 2]) -> SketchProfile {
    let vertices = (0..2).map(|i| ProfileVertex {
        id: ProfileVertexId(i),
        point: ends[i as usize],
    });
    let closing = ProfileCurve::Line {
        id: ProfileCurveId(9),
        start: ProfileVertexId(1),
        end: ProfileVertexId(0),
    };
    SketchProfile {
        plane: ProfilePlane::world_xy(),
        vertices: vertices.collect(),
        curves: vec![curve, closing],
    }
}

/// Independent oracle (ADR 0083): `body_properties` integrates the extruded B-rep itself
/// (adaptive quadrature on the elliptic or spline side faces); volume and centroid must
/// agree with the closed forms within its bound, and with the operation's volume.
fn assert_body(profile: &SketchProfile, height: f64, volume: f64, centroid: [f64; 3]) {
    use crate::properties::body_properties;
    let set = build_planar_regions(profile, resolution()).unwrap();
    let options = ExtrudeOptions {
        resolution: resolution(),
    };
    let result = extrude(&set.regions()[0], [0., 0., height], &options).unwrap();
    assert!(result.volume().encloses(volume), "{:?}", result.volume());
    let measured = body_properties(result.solid()).unwrap();
    let slack = measured.volume.error_bound.max(1e-9 * volume);
    assert!(
        (measured.volume.value - volume).abs() <= slack,
        "{measured:?}"
    );
    let distance: f64 = (0..3)
        .map(|k| (measured.centroid.point[k] - centroid[k]).powi(2))
        .sum::<f64>()
        .sqrt();
    assert!(
        distance <= measured.centroid.error_bound.max(1e-9),
        "{measured:?}"
    );
}

#[test]
fn ellipse_and_spline_prisms_match_body_properties() {
    use std::f64::consts::{FRAC_1_SQRT_2, PI};
    let ellipse = SketchProfile {
        plane: ProfilePlane::world_xy(),
        vertices: Vec::new(),
        curves: vec![ProfileCurve::Ellipse {
            id: ProfileCurveId(3),
            center: [1., -2.],
            major_axis: [1.2, 1.6],
            minor_radius: 0.5,
        }],
    };
    assert_body(&ellipse, 3.0, 3.0 * PI, [1., -2., 1.5]);
    let knots = [0., 0., 0., 0.5, 1., 1., 1.];
    let parabola = spline(1, &[[0., 0.], [1., 2.], [3., 2.], [4., 0.]], None, &knots);
    let parabola = spline_segment(parabola, [[0., 0.], [4., 0.]]);
    assert_body(&parabola, 1.5, 8.0, [2., 0.8, 0.75]);
    let h = FRAC_1_SQRT_2;
    let poles = [[1., 0.], [1., 1.], [0., 1.], [-1., 1.], [-1., 0.]];
    let knots = [0., 0., 0., 0.5, 0.5, 1., 1., 1.];
    let arc = spline(1, &poles, Some(&[1., h, 1., h, 1.]), &knots);
    let half_disc = spline_segment(arc, [[1., 0.], [-1., 0.]]);
    assert_body(&half_disc, 2.0, PI, [0., 4. / (3. * PI), 1.]);
}
