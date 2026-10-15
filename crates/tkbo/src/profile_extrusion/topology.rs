// SPDX-License-Identifier: GPL-2.0-only
//! Native construction of the certified profile: one vertex per caller vertex id, one edge
//! per caller curve (shared by every face that uses it), analytic line and circle carriers.
use super::admitted::{AdmittedCurve, AdmittedProfile, CircleCarrier, WorldFrame};
use super::loops::ProfileLoop;
use super::plane_math::{length2, sub2};
use super::profile::{ArcSense, ProfileCurveId, ProfileVertexId};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Builder, Edge, Face, Vertex, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use std::collections::BTreeMap;

/// Native vertices and edges keyed by caller identity.
pub(crate) struct ProfileTopology {
    pub vertices: BTreeMap<ProfileVertexId, Vertex>,
    pub edges: BTreeMap<ProfileCurveId, Edge>,
}

/// Build every vertex and edge of the admitted profile; `build_topology(&profile)?`.
pub(crate) fn build_topology(profile: &AdmittedProfile) -> Result<ProfileTopology, Failure> {
    let mut vertices = BTreeMap::new();
    for (id, point) in &profile.vertices {
        let tolerance = vertex_tolerance(profile, *id, *point);
        vertices.insert(*id, make_vertex(profile.frame.world(*point), tolerance)?);
    }
    let mut edges = BTreeMap::new();
    for curve in &profile.curves {
        edges.insert(curve.id(), make_edge(&profile.frame, curve, &vertices)?);
    }
    Ok(ProfileTopology { vertices, edges })
}

/// Confusion, raised to cover the sew gap between an arc end and its circle carrier.
fn vertex_tolerance(profile: &AdmittedProfile, id: ProfileVertexId, point: [f64; 2]) -> f64 {
    let gap =
        |carrier: &CircleCarrier| (length2(sub2(point, carrier.center)) - carrier.radius).abs();
    let arcs = profile.curves.iter().filter_map(|curve| match curve {
        AdmittedCurve::Arc { ends, carrier, .. } if ends.contains(&id) => Some(gap(carrier)),
        _ => None,
    });
    arcs.fold(
        rustycad_tkernel::precision::confusion(),
        |tolerance, gap| tolerance.max(2.0 * gap),
    )
}

fn make_vertex(point: [f64; 3], tolerance: f64) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    let [x, y, z] = point;
    BRepBuilder::default().make_vertex_at(&mut vertex, Point3::new(x, y, z), tolerance)?;
    Ok(vertex)
}

fn make_edge(
    frame: &WorldFrame,
    curve: &AdmittedCurve,
    vertices: &BTreeMap<ProfileVertexId, Vertex>,
) -> Result<Edge, Failure> {
    let ends = |ends: [ProfileVertexId; 2]| [&vertices[&ends[0]], &vertices[&ends[1]]];
    let maker = match *curve {
        AdmittedCurve::Line { ends: ids, .. } => {
            let [start, end] = ends(ids);
            BRepBuilderAPIMakeEdge::from_vertices(start, end)?
        }
        AdmittedCurve::Arc {
            ends: ids,
            carrier,
            sense,
            ..
        } => {
            let [start, end] = ends(ids);
            BRepBuilderAPIMakeEdge::from_circ_vertices(circle(frame, carrier, sense)?, start, end)?
        }
        AdmittedCurve::Circle { carrier, .. } => {
            BRepBuilderAPIMakeEdge::from_circ(circle(frame, carrier, ArcSense::CounterClockwise)?)?
        }
    };
    maker.edge()
}

/// World circle whose native parameter increases in the arc sense; parameter 0 lies on +x.
fn circle(frame: &WorldFrame, carrier: CircleCarrier, sense: ArcSense) -> Result<Circle3, Failure> {
    let [x, y, z] = frame.world(carrier.center);
    let normal = match sense {
        ArcSense::CounterClockwise => frame.normal,
        ArcSense::Clockwise => frame.normal.map(|c| -c),
    };
    let axis = Axis2::with_x_direction(
        Point3::new(x, y, z),
        direction(normal)?,
        direction(frame.x)?,
    )?;
    Circle3::new(axis, carrier.radius)
}

pub(crate) fn direction([x, y, z]: [f64; 3]) -> Result<Direction3, Failure> {
    Direction3::new(x, y, z)
}

/// The sketch plane as a native plane with the admitted frame.
pub(crate) fn native_plane(frame: &WorldFrame) -> Result<Plane3, Failure> {
    let [x, y, z] = frame.origin;
    let axis = Axis3::new(
        Point3::new(x, y, z),
        direction(frame.normal)?,
        direction(frame.x)?,
    )?;
    Ok(Plane3::from_position(axis))
}

/// A closed wire of `closed` in its certified traversal order: counterclockwise about the
/// plane normal for an outer loop, clockwise for a hole (ADR 0084 orbits).
pub(crate) fn loop_wire(topology: &ProfileTopology, closed: &ProfileLoop) -> Result<Wire, Failure> {
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    for used in &closed.uses {
        let orientation = [Orientation::Forward, Orientation::Reversed][usize::from(used.reversed)];
        Builder.add(
            &mut wire,
            &topology.edges[&used.curve].oriented(orientation),
        )?;
    }
    wire.set_closed(true)?;
    Ok(wire)
}

/// Planar face bounded by `outer` with `holes`; `region_face(&frame, &outer, &holes)?`.
pub(crate) fn region_face(
    frame: &WorldFrame,
    outer: &Wire,
    holes: &[Wire],
) -> Result<Face, Failure> {
    let mut maker = BRepBuilderAPIMakeFace::from_plane_wire(native_plane(frame)?, outer, true)?;
    for hole in holes {
        maker.add(hole)?;
    }
    maker.face()
}
