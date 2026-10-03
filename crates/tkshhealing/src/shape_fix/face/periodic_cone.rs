use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Builder, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, StandardDirection2};
use rustycad_tkmath::toploc::Location;
use std::f64::consts::PI;

/// The UV bounds and winding direction of a periodic conical loop.
pub(super) struct ConicalLoopBounds {
    pub min_u: f64,
    pub max_u: f64,
    pub min_v: f64,
    pub max_v: f64,
    pub decreasing: bool,
}

/// The UV extents used for the two cone meridian edges.
struct PeriodicSeamBounds {
    min_u: f64,
    max_u: f64,
    apex_v: f64,
    ring_v: f64,
}

/// The direct wire children of a face with the orientations accepted by OCCT's fix.
pub(super) fn face_wires(face: &Face) -> Result<Vec<Wire>, Failure> {
    let mut wires = Vec::new();
    for child in ShapeIterator::new(&face.clone().into_shape(), false, true) {
        let shape = child;
        if shape.shape_type()? != ShapeEnum::Wire {
            continue;
        }
        if !matches!(
            shape.orientation(),
            Orientation::Forward | Orientation::Reversed
        ) {
            continue;
        }
        wires.push(topods::wire(&shape)?);
    }
    Ok(wires)
}

/// The bounds and winding of a wire's p-curves on the analyzed conical surface.
pub(super) fn conical_bounds_on_surface(
    wire: &Wire,
    surface: &Handle<dyn GeomSurface>,
    tolerance: f64,
) -> Result<Option<ConicalLoopBounds>, Failure> {
    conical_bounds_from_pcurves(wire, tolerance, |edge| {
        ShapeAnalysisEdge::new().pcurve_on_surface(edge, surface, &Location::default(), true)
    })
}
fn conical_bounds_from_pcurves<F>(
    wire: &Wire,
    tolerance: f64,
    pcurve: F,
) -> Result<Option<ConicalLoopBounds>, Failure>
where
    F: Fn(&rustycad_tkbrep::topods::Edge) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure>,
{
    let mut min_u = f64::MAX;
    let mut max_u = f64::MIN;
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    let (mut delta, mut abs_delta) = (0.0, 0.0);
    for child in ShapeIterator::new(&wire.clone().into_shape(), false, true) {
        let edge = topods::edge(&child)?;
        let (curve, first, last) = pcurve(&edge)?;
        let Some(curve) = curve.get() else {
            return Ok(None);
        };
        let (start, end) = (curve.value(first)?, curve.value(last)?);
        min_u = min_u.min(start.x()).min(end.x());
        max_u = max_u.max(start.x()).max(end.x());
        min_v = min_v.min(start.y()).min(end.y());
        max_v = max_v.max(start.y()).max(end.y());
        let step = end.x() - start.x();
        delta += step;
        abs_delta += step.abs();
    }
    let periodic = (abs_delta - 2.0 * PI).abs() <= tolerance;
    let spans_period = max_u - min_u > 2.0 * PI - tolerance;
    if periodic && spans_period {
        Ok(Some(ConicalLoopBounds {
            min_u,
            max_u,
            min_v,
            max_v,
            decreasing: delta < 0.0,
        }))
    } else {
        Ok(None)
    }
}

/// Creates the degenerated edge at the cone apex and inserts it into an empty face copy.
pub(super) fn build_apex_face(
    face: &Face,
    wire: &Wire,
    cone: &GeomConicalSurface,
    bounds: &ConicalLoopBounds,
    apex_v: f64,
    add_seam: bool,
    tolerance: f64,
) -> Result<Shape, Failure> {
    let (start_u, direction) = if apex_v < bounds.min_v {
        (
            bounds.min_u,
            Direction2::from_standard(StandardDirection2::X),
        )
    } else {
        (
            bounds.max_u,
            Direction2::from_standard(StandardDirection2::NegX),
        )
    };
    let mut fixed_wire = wire.clone();
    if (apex_v < bounds.min_v && !bounds.decreasing) || (apex_v > bounds.max_v && bounds.decreasing)
    {
        fixed_wire.reverse();
    }
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(start_u, apex_v),
        direction,
    )))
    .cast();
    let builder = BRepBuilder::default();
    let mut apex = rustycad_tkbrep::topods::Vertex::default();
    builder.make_vertex_at(&mut apex, cone.apex(), precision::confusion())?;
    let mut edge = rustycad_tkbrep::topods::Edge::default();
    builder.make_edge(&mut edge)?;
    builder.update_edge_pcurve_on_face(&edge, &line, face, tolerance)?;
    let mut edge_shape = edge.clone().into_shape();
    let topo_builder = Builder;
    topo_builder.add(&mut edge_shape, &apex.clone().into_shape())?;
    let mut reversed_apex = apex.clone();
    reversed_apex.reverse();
    topo_builder.add(&mut edge_shape, &reversed_apex.into_shape())?;
    builder.degenerated(&edge, true)?;
    builder.range(&edge, 0.0, (bounds.max_u - bounds.min_u).abs(), false)?;
    let apex_wire = rustycad_tktopalgo::brep_lib::BRepLibMakeWire::from_edge(&edge)?.wire()?;
    apex_wire.set_closed(true)?;
    let mut result_shape = face.empty_copied()?;
    result_shape.set_orientation(Orientation::Forward);
    topo_builder.add(&mut result_shape, &fixed_wire.clone().into_shape())?;
    topo_builder.add(&mut result_shape, &apex_wire.into_shape())?;
    let mut result = topods::face(&result_shape)?;
    result.set_orientation(face.orientation());
    if !add_seam {
        return Ok(result.into_shape());
    }
    build_missing_seam_face(
        &result,
        &fixed_wire,
        &edge,
        &apex,
        cone,
        PeriodicSeamBounds {
            min_u: bounds.min_u,
            max_u: bounds.max_u,
            apex_v,
            ring_v: bounds.max_v,
        },
    )
}

/// Connects the periodic loop and apex loop along the two cone meridians.
fn build_missing_seam_face(
    face: &Face,
    ring_wire: &Wire,
    apex_edge: &rustycad_tkbrep::topods::Edge,
    apex: &rustycad_tkbrep::topods::Vertex,
    cone: &GeomConicalSurface,
    bounds: PeriodicSeamBounds,
) -> Result<Shape, Failure> {
    let mut ring_edges = Vec::new();
    let mut ring_iterator = ShapeIterator::new(&ring_wire.clone().into_shape(), false, true);
    while let Ok(shape) = ring_iterator.value() {
        ring_edges.push(topods::edge(shape)?);
        ring_iterator.next();
    }
    if ring_edges.len() != 1 {
        return Ok(face.clone().into_shape());
    }
    let (ring_start, _) = topexp::vertices(&ring_edges[0], false);
    let left = make_meridian_edge(
        face,
        cone,
        bounds.min_u,
        bounds.apex_v,
        bounds.ring_v,
        apex,
        &ring_start,
    )?;
    let right = make_meridian_edge(
        face,
        cone,
        bounds.max_u,
        bounds.apex_v,
        bounds.ring_v,
        apex,
        &ring_start,
    )?;
    let reversed_ring = topods::edge(&ring_edges[0].oriented(Orientation::Reversed))?;
    let forward_apex = apex_edge.clone();
    let left = topods::edge(&left.oriented(Orientation::Reversed))?;
    let wire = rustycad_tktopalgo::brep_lib::BRepLibMakeWire::from_four_edges(
        &reversed_ring,
        &left,
        &forward_apex,
        &right,
    )?
    .wire()?;
    fix_wire_same_parameters(&wire)?;
    let builder = Builder;
    let mut result = face.empty_copied()?;
    builder.add(&mut result, &wire.into_shape())?;
    let mut result_face = topods::face(&result)?;
    result_face.set_orientation(face.orientation());
    Ok(result_face.into_shape())
}

/// Applies the final wire pass's same-parameter repair after the seam closes the loop.
fn fix_wire_same_parameters(wire: &Wire) -> Result<(), Failure> {
    let mut iterator = ShapeIterator::new(&wire.clone().into_shape(), false, true);
    while let Ok(shape) = iterator.value() {
        let edge = topods::edge(shape)?;
        rustycad_tktopalgo::brep_lib::same_parameter_edge(&edge, BRepTool::edge_tolerance(&edge)?)?;
        iterator.next();
    }
    Ok(())
}

/// Builds a face p-curve edge between the existing ring and apex vertices.
fn make_meridian_edge(
    face: &Face,
    cone: &GeomConicalSurface,
    u: f64,
    first_v: f64,
    last_v: f64,
    apex: &rustycad_tkbrep::topods::Vertex,
    ring: &rustycad_tkbrep::topods::Vertex,
) -> Result<rustycad_tkbrep::topods::Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = rustycad_tkbrep::topods::Edge::default();
    let curve = cone.u_iso(u)?;
    builder.make_edge_with_curve(&mut edge, &curve, precision::confusion())?;
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(u, first_v),
        Direction2::from_standard(StandardDirection2::Y),
    )))
    .cast();
    builder.update_edge_pcurve_on_face(&edge, &line, face, precision::confusion())?;
    builder.range(&edge, 0.0, last_v - first_v, false)?;
    let topo_builder = Builder;
    let mut forward_apex = apex.clone();
    forward_apex.set_orientation(Orientation::Forward);
    let mut edge_shape = edge.clone().into_shape();
    topo_builder.add(&mut edge_shape, &forward_apex.into_shape())?;
    let mut reversed_ring = ring.clone();
    reversed_ring.set_orientation(Orientation::Reversed);
    topo_builder.add(&mut edge_shape, &reversed_ring.into_shape())?;
    Ok(edge)
}
