// Rust translation of Is2DConnected and Is2DClosed (BRepLib_FindSurface.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Whether the edges of a wire form a closed loop in the parameter space of a surface.
use crate::brep_lib::make_face::BRepLibMakeFace;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Shape, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::toploc::Location;

/// `Is2DConnected`: whether the end of `edge1` and the start of `edge2` (by their
/// orientations) meet on the surface within the resolution of the tolerance of the
/// first vertex of `edge2`.
fn is_2d_connected(
    edge1: &Edge,
    edge2: &Edge,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<bool, Failure> {
    let end1 = pcurve_end(edge1, surface, location, true)?;
    let start2 = pcurve_end(edge2, surface, location, false)?;
    // compare 2D points
    let adaptor = GeomAdaptorSurface::with_surface(surface)?;
    let vertex = topexp::first_vertex(edge2, true);
    let tol3d: f64 = BRepTool::vertex_tolerance(&vertex)?;
    let tol2d: f64 = adaptor.u_resolution(tol3d)? + adaptor.v_resolution(tol3d)?;
    Ok(end1.square_distance(start2) < tol2d * tol2d)
}

/// The 2d end (`last`) or start of an edge on the surface, by its orientation.
fn pcurve_end(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
    last: bool,
) -> Result<Point2, Failure> {
    let found = BRepTool::curve_on_surface(edge, surface, location)?;
    let forward: bool = edge.orientation() == Orientation::Forward;
    let parameter: f64 = if forward == last {
        found.last
    } else {
        found.first
    };
    let pcurve = found.pcurve.get().ok_or_else(|| {
        crate::brep_lib::errors::null_object(
            "BRepLib_FindSurface: an edge without curve on the surface",
        )
    })?;
    pcurve.value(parameter)
}

/// `Is2DClosed`: whether the first wire of the shape, explored on a face of the
/// surface, has its edges connected in 2d and closes on its first vertex; any failure
/// answers false, as the native catch block does.
pub(super) fn is_2d_closed(
    shape: &Shape,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> bool {
    matches!(wire_closed_2d(shape, surface, location), Ok(true))
}

/// The body of [`is_2d_closed`].
fn wire_closed_2d(
    shape: &Shape,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<bool, Failure> {
    // get a wire theShape
    let wires = Explorer::new(shape, ShapeEnum::Wire, ShapeEnum::Shape)?;
    if !wires.more() {
        return Ok(false);
    }
    let wire = wires.current().clone();
    // a tmp face
    let face = BRepLibMakeFace::from_surface(surface, precision::p_confusion())?.face()?;
    // check topological closeness using wire explorer, if the wire is not closed
    // the 1st and the last vertices of wire are different
    let mut explorer = WireExplorer::with_wire_on_face(&Wire::unchecked(wire), &face)?;
    if !explorer.more() {
        return Ok(false);
    }
    // remember the 1st and the last edges of aWire
    let first: Edge = explorer.current().clone();
    let mut last: Edge = first.clone();
    explorer.next()?;
    while explorer.more() {
        let current: Edge = explorer.current().clone();
        if !is_2d_connected(&last, &current, surface, location)? {
            return Ok(false);
        }
        last = current;
        explorer.next()?;
    }
    // wire is closed if ( 1st vertex of aFisrtEdge ) == ( last vertex of aLastEdge ) in 2D
    let v1 = topexp::first_vertex(&first, true);
    let v2 = topexp::last_vertex(&last, true);
    Ok(v1.is_same(&v2) && is_2d_connected(&last, &first, surface, location)?)
}
