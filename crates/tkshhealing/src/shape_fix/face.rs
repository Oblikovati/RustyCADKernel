// Rust translation of the context and periodic-cone path in ShapeFix_Face.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Context preservation and apex-edge repair for periodic loops on conical faces.
use super::{FixRoot, FixRootData};
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisSurface};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_construct::ShapeConstructProjectCurveOnSurface;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Builder, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, StandardDirection2};
use std::f64::consts::PI;

/// The context and periodic conical-loop path of `ShapeFix_Face`.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkshhealing::shape_fix::ShapeFixFace;
/// let fixer = ShapeFixFace::new(Shape::default());
/// assert!(!fixer.perform().unwrap());
/// ```
#[derive(Default)]
pub struct ShapeFixFace {
    parent: StandardTransient,
    root: FixRootData,
    result: SharedState<Shape>,
    status: SharedState<i32>,
}

impl_transient!(ShapeFixFace, "ShapeFix_Face", StandardTransient, parent, interfaces: [dyn FixRoot]);

impl ShapeFixFace {
    /// `ShapeFix_Face(face)`: the input also initializes the result;
    /// `ShapeFixFace::new(face.into_shape())`.
    pub fn new(face: Shape) -> Self {
        let fixer = Self::default();
        fixer.init(face);
        fixer
    }

    /// `Init(face)`: replace the input and current result with `face`.
    pub fn init(&self, face: Shape) {
        self.root.with_mut(|fields| fields.shape = face.clone());
        *self.result.write() = face;
        *self.status.write() = 0;
    }

    /// `Perform()`: preserve context replacements and repair a single periodic cone loop.
    pub fn perform(&self) -> Result<bool, Failure> {
        *self.status.write() = 0;
        let (input, context, tolerance) = self.root.with(|fields| {
            (
                fields.shape.clone(),
                fields.context.clone(),
                fields.precision,
            )
        });
        let applied = apply_context(&input, &context)?;
        if !is_face(&applied)? {
            *self.result.write() = applied;
            return Ok(false);
        }
        let face = topods::face(&applied)?;
        if let Some(split) = super::face_loops::split_bowtie_face(&face, tolerance, &context)? {
            *self.status.write() = ShapeExtend::encode_status(ShapeExtendStatus::Done7);
            *self.result.write() = split.clone();
            replace_context(&context, &face, &split)?;
            return Ok(true);
        }
        let fixed = repair_periodic_cone(&face, tolerance)?;
        let result = fixed.clone().unwrap_or(applied);
        *self.result.write() = result.clone();
        replace_context(&context, &face, &result)?;
        Ok(false)
    }

    /// `Result()`: return the last performed shape, including non-face context results.
    pub fn result(&self) -> Shape {
        self.result.read().clone()
    }

    /// `Face()`: return the last result as a face when it has face topology.
    pub fn face(&self) -> Result<Face, Failure> {
        topods::face(&self.result.read())
    }

    /// `Status(status)`: decode a flag from the last `Perform()`.
    pub fn status(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(*self.status.read(), status)
    }
}

impl FixRoot for ShapeFixFace {
    fn fix_root_data(&self) -> &FixRootData {
        &self.root
    }
}

/// Applies a non-null reshape context using the full-shape limit of ShapeFix_Face.
fn apply_context(input: &Shape, context: &Handle<ShapeBuildReShape>) -> Result<Shape, Failure> {
    let Some(context) = context.get() else {
        return Ok(input.clone());
    };
    rustycad_tkbrep::brep_tools::ReShape::apply(context, input, ShapeEnum::Shape)
}

/// Returns whether context application left a face, including the null case.
fn is_face(shape: &Shape) -> Result<bool, Failure> {
    if shape.is_null() {
        return Ok(false);
    }
    Ok(shape.shape_type()? == ShapeEnum::Face)
}

/// Records a repaired face in the supplied reshape context.
fn replace_context(
    context: &Handle<ShapeBuildReShape>,
    original: &Face,
    result: &Shape,
) -> Result<(), Failure> {
    let Some(context) = context.get() else {
        return Ok(());
    };
    rustycad_tkbrep::brep_tools::ReShape::replace(context, &original.clone().into_shape(), result)
}

/// The UV bounds and winding direction of a periodic conical loop.
struct ConicalLoopBounds {
    min_u: f64,
    max_u: f64,
    min_v: f64,
    max_v: f64,
    decreasing: bool,
}

/// The UV extents used for the two cone meridian edges.
struct PeriodicSeamBounds {
    min_u: f64,
    max_u: f64,
    apex_v: f64,
    ring_v: f64,
}

/// The direct wire children of a face with the orientations accepted by OCCT's fix.
fn face_wires(face: &Face) -> Result<Vec<Wire>, Failure> {
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

/// Adds missing p-curves to a conical wire using OCCT's curve-on-surface projector.
fn add_wire_pcurves(
    face: &Face,
    wire: &Wire,
    surface: &Handle<dyn GeomSurface>,
    tolerance: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let analysis_surface = Handle::new(ShapeAnalysisSurface::new(surface)?);
    let projector = ShapeConstructProjectCurveOnSurface::new();
    projector.init_analysis(&analysis_surface, tolerance);
    let analysis = ShapeAnalysisEdge::new();
    for child in ShapeIterator::new(&wire.clone().into_shape(), false, true) {
        let edge = topods::edge(&child)?;
        if analysis.has_pcurve_on_face(&edge, face)? {
            continue;
        }
        let (curve, first, last) = BRepTool::curve(&edge)?;
        let tolerances = endpoint_tolerances(&analysis, &edge)?;
        let mut pcurve = Handle::default();
        projector.perform(&curve, (first, last), &mut pcurve, tolerances)?;
        builder.same_parameter(&edge, false)?;
        builder.update_edge_pcurve_on_face(&edge, &pcurve, face, tolerance)?;
    }
    Ok(())
}

/// The endpoint tolerances passed to ShapeFix_Edge::FixAddPCurve.
fn endpoint_tolerances(
    analysis: &ShapeAnalysisEdge,
    edge: &rustycad_tkbrep::topods::Edge,
) -> Result<(f64, f64), Failure> {
    let first = analysis.first_vertex(edge);
    let last = analysis.last_vertex(edge);
    Ok((vertex_tolerance(&first)?, vertex_tolerance(&last)?))
}

/// Native uses -1 for a missing endpoint vertex.
fn vertex_tolerance(vertex: &rustycad_tkbrep::topods::Vertex) -> Result<f64, Failure> {
    if vertex.is_null() {
        return Ok(-1.0);
    }
    BRepTool::vertex_tolerance(vertex)
}

/// The bounds and winding of a wire's p-curves on a conical face.
fn conical_loop_bounds(
    face: &Face,
    wire: &Wire,
    tolerance: f64,
) -> Result<Option<ConicalLoopBounds>, Failure> {
    let mut min_u = f64::MAX;
    let mut max_u = f64::MIN;
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    let (mut delta, mut abs_delta) = (0.0, 0.0);
    let analysis = ShapeAnalysisEdge::new();
    for child in ShapeIterator::new(&wire.clone().into_shape(), false, true) {
        let edge = topods::edge(&child)?;
        let (curve, first, last) = analysis.pcurve_on_face(&edge, face, true)?;
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

/// Builds OCCT's apex p-curve edge and returns a face copy containing both wires.
fn repair_periodic_cone(face: &Face, tolerance: f64) -> Result<Option<Shape>, Failure> {
    let wires = face_wires(face)?;
    if wires.len() != 1 {
        return Ok(None);
    }
    let (surface, _) = BRepTool::surface_with_location(face)?;
    let cone = Handle::<GeomConicalSurface>::down_cast(&surface);
    let Some(cone) = cone.get() else {
        return Ok(None);
    };
    add_wire_pcurves(face, &wires[0], &surface, tolerance)?;
    let bounds = conical_loop_bounds(face, &wires[0], tolerance)?;
    let Some(bounds) = bounds else {
        return Ok(None);
    };
    let apex_v = -cone.ref_radius() / cone.semi_angle().sin();
    if (apex_v - bounds.min_v).abs() <= tolerance
        || (apex_v - bounds.max_v).abs() <= tolerance
        || (apex_v < bounds.max_v && apex_v > bounds.min_v)
    {
        return Ok(None);
    }
    build_apex_face(face, &wires[0], cone, &bounds, apex_v).map(Some)
}

/// Creates the degenerated edge at the cone apex and inserts it into an empty face copy.
fn build_apex_face(
    face: &Face,
    wire: &Wire,
    cone: &GeomConicalSurface,
    bounds: &ConicalLoopBounds,
    apex_v: f64,
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
    builder.update_edge_pcurve_on_face(&edge, &line, face, precision::confusion())?;
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
    topo_builder.add(&mut result_shape, &fixed_wire.clone().into_shape())?;
    topo_builder.add(&mut result_shape, &apex_wire.into_shape())?;
    let mut result = topods::face(&result_shape)?;
    result.set_orientation(face.orientation());
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
