// Rust translation of the vertex part of InternalUpdateTolerances (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertex tolerances of `BRepLib::UpdateTolerances`: at least the tolerances of the
//! edges and the distance to their curves at the vertex parameter.
use super::ShapeTolerances;
use crate::brep_lib::errors::null_object;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point3, Transform3};
use std::collections::HashSet;

/// The edge tolerance above which the curves of an edge are not measured.
const BIG_TOL: f64 = 1.0e10;

/// The member named in the failures of the vertex part.
const MEMBER: &str = "BRepLib::UpdateTolerances";

/// "Vertices are processed": the tolerance of each vertex is the largest tolerance of
/// its edges (the recorded one first) and the distance to their curves, plus twice its
/// epsilon; it is recorded when above the vertex tolerance or, with the verification,
/// when it differs on the first use of the vertex TShape.
pub(super) fn vertex_tolerances(
    shape: &Shape,
    is_verify_tolerance: bool,
    sh_to_tol: &mut ShapeTolerances,
) -> Result<(), Failure> {
    let mut parents = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_unique_ancestors(
        shape,
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        &mut parents,
        false,
    )?;
    // `Initialized`: the vertex TShapes met, compared by address as natively.
    let mut initialized: HashSet<usize> = HashSet::new();
    for index in 1..=parents.extent() {
        let vertex = Vertex::unchecked(parents.find_key(index)?.clone());
        let mut tol: f64 = edge_bound(&vertex, parents.find_from_index(index)?, sh_to_tol)?;
        tol += 2. * epsilon(tol);
        let v_tol: f64 = BRepTool::vertex_tolerance(&vertex)?;
        // Attention to sharing of the vertex by other shapes
        let to_add: bool =
            is_verify_tolerance && initialized.insert(vertex.tshape_address()) && v_tol != tol;
        if tol > v_tol || to_add {
            sh_to_tol.bind(vertex.into_shape(), tol);
        }
    }
    Ok(())
}

/// The largest edge tolerance and distance from the vertex to the curves of its
/// SameRange edges (an edge above [`BIG_TOL`] is not measured).
fn edge_bound(
    vertex: &Vertex,
    edges: &List<Shape>,
    sh_to_tol: &ShapeTolerances,
) -> Result<f64, Failure> {
    let point: Point3 = BRepTool::pnt(vertex)?;
    let (mut tol, mut max_dist): (f64, f64) = (0., 0.);
    for shape in edges.iter() {
        let edge = Edge::unchecked(shape.clone());
        let edge_tol: f64 = match sh_to_tol.seek(shape) {
            Some(&recorded) => recorded,
            None => BRepTool::edge_tolerance(&edge)?,
        };
        tol = std_max(tol, edge_tol);
        if tol > BIG_TOL || !BRepTool::same_range(&edge)? {
            continue;
        }
        max_dist = curves_distance(vertex, &edge, point, max_dist)?;
    }
    Ok(std_max(tol, max_dist.sqrt()))
}

/// The largest of `max_dist` and the square distances from the vertex point to each 3d
/// curve and curve on surface of the edge at the vertex parameter.
fn curves_distance(
    vertex: &Vertex,
    edge: &Edge,
    point: Point3,
    mut max_dist: f64,
) -> Result<f64, Failure> {
    let par: f64 = BRepTool::parameter(vertex, edge)?;
    for cr in edge_tshape(edge, MEMBER)?.curves().iter() {
        let cr: &dyn BRepCurveRepresentation =
            cr.get().expect("curve representations are non-null");
        let transform: Transform3 = edge.location().multiplied(&cr.location())?.transformation();
        for p3d in representation_points(cr, par)? {
            let distance: f64 = p3d.transformed(transform).square_distance(&point);
            if distance > max_dist {
                max_dist = distance;
            }
        }
    }
    Ok(max_dist)
}

/// The points of a representation at `par`: the 3d curve point, or the surface points
/// of the one or two p-curves; none for a null 3d curve or another representation.
fn representation_points(
    cr: &dyn BRepCurveRepresentation,
    par: f64,
) -> Result<Vec<Point3>, Failure> {
    if cr.is_curve3d() {
        let curve = cr.curve3d()?;
        // edge non degenerated
        return match curve.get() {
            Some(curve) => Ok(vec![curve.value(par)?]),
            None => Ok(Vec::new()),
        };
    }
    if !cr.is_curve_on_surface() {
        return Ok(Vec::new());
    }
    let surface = cr.surface()?;
    let mut points = vec![pcurve_point(&surface, &cr.pcurve()?, par)?];
    if cr.is_curve_on_closed_surface() {
        let pcurve2 = cr.pcurve2()?;
        if pcurve2.is_some() {
            points.push(pcurve_point(&surface, &pcurve2, par)?);
        }
    }
    Ok(points)
}

/// The surface point of a p-curve at `par` (a null p-curve or surface, a native crash,
/// is a failure).
fn pcurve_point(
    surface: &Handle<dyn GeomSurface>,
    pcurve: &Handle<dyn Geom2dCurve>,
    par: f64,
) -> Result<Point3, Failure> {
    let (Some(surface), Some(pcurve)) = (surface.get(), pcurve.get()) else {
        return Err(null_object(&format!(
            "{MEMBER}: a curve on surface without its surface or p-curve (expected both non-null)"
        )));
    };
    let uv = pcurve.value(par)?;
    surface.value(uv.x(), uv.y())
}
