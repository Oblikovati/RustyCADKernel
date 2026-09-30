// Rust translation of the static RebuildShape functions of
// ModelingAlgorithms/TKShHealing/ShapeUpgrade/ShapeUpgrade_RemoveLocations.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The faces, edges and vertices of `ShapeUpgrade_RemoveLocations` rebuilt without
//! location.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;

/// `RebuildShape(theFace, theNewFace)`: a face on the surface transformed by the face
/// location, with the face tolerance; false (the new face untouched) for a face
/// without location.
pub(super) fn rebuild_face(face: &Face, new_face: &mut Face) -> Result<bool, Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    if location.is_identity() {
        return Ok(false);
    }
    let stored = surface
        .get()
        .ok_or_else(|| null_geometry("the face has no surface"))?;
    let moved = stored
        .transformed(location.transformation())?
        .cast::<dyn GeomSurface>();
    BRepBuilder::default().make_face_on_surface(
        new_face,
        &moved,
        BRepTool::face_tolerance(face)?,
    )?;
    Ok(true)
}

/// `RebuildShape(theEdge, theNewEdge, theFace, theNewFace, isBound)`: when the edge is
/// not bound yet, a new edge on the 3D curve transformed by its location, with the edge
/// range, orientation, tolerance and degenerated flag; then, off a plane, the p-curves
/// on `face` put on `new_face` (both of a seam, ordered by the orientation of the new
/// edge in the face) with their range. True when the edge was built.
pub(super) fn rebuild_edge(
    edge: &Edge,
    new_edge: &mut Edge,
    face: &Face,
    new_face: &Face,
    bound: bool,
) -> Result<bool, Failure> {
    let rebuilt = !bound;
    if rebuilt {
        new_located_free_edge(edge, new_edge)?;
    }
    if face.is_null() {
        return Ok(rebuilt);
    }
    let surface = BRepTool::surface(face)?;
    if surface.is_null() {
        return Err(null_geometry("the face has no surface"));
    }
    if !surface.cast::<GeomPlane>().is_null() {
        return Ok(rebuilt);
    }
    copy_pcurves(edge, new_edge, face, new_face)?;
    Ok(rebuilt)
}

/// The first part of `RebuildShape(edge)`: the new edge, its 3D curve without location,
/// its range, orientation and degenerated flag.
fn new_located_free_edge(edge: &Edge, new_edge: &mut Edge) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let found = BRepTool::curve_with_location(edge)?;
    builder.make_edge(new_edge)?;
    if let Some(curve) = found.curve.get() {
        let mut c3d: Handle<dyn GeomCurve> = found.curve.clone();
        if !found.location.is_identity() {
            c3d = curve.transformed(found.location.transformation())?.cast();
        }
        builder.update_edge_curve(new_edge, &c3d, BRepTool::edge_tolerance(edge)?)?;
        builder.range(new_edge, found.first, found.last, false)?;
    }
    new_edge.set_orientation(edge.orientation());
    if BRepTool::degenerated(edge)? {
        builder.degenerated(new_edge, true)?;
    }
    Ok(())
}

/// The p-curve part of `RebuildShape(edge)`: on a closed edge whose new edge is not
/// closed yet, both p-curves (the one of the reversed edge second or first by the
/// orientation of the new edge, flipped for a REVERSED face); otherwise the p-curve of
/// the edge; then the range of the last p-curve read, when one was found.
fn copy_pcurves(edge: &Edge, new_edge: &Edge, face: &Face, new_face: &Face) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut found = BRepTool::curve_on_face(edge, face)?;
    let c2d = found.pcurve.clone();
    let mut c2d1: Handle<dyn Geom2dCurve> = Handle::default();
    if BRepTool::is_closed_on_face(edge, face)? {
        if !BRepTool::is_closed_on_face(new_edge, new_face)? {
            let reversed = topods::edge(&edge.reversed())?;
            found = BRepTool::curve_on_face(&reversed, face)?;
            c2d1 = found.pcurve.clone();
            let mut or_edge = new_edge.orientation();
            if face.orientation() == Orientation::Reversed {
                or_edge = if or_edge == Orientation::Forward {
                    Orientation::Reversed
                } else {
                    Orientation::Forward
                };
            }
            let pair = if or_edge == Orientation::Forward {
                (&c2d, &c2d1)
            } else {
                (&c2d1, &c2d)
            };
            builder.update_edge_pcurves_on_face(new_edge, pair, new_face, 0.0)?;
        }
    } else {
        builder.update_edge_pcurve_on_face(new_edge, &c2d, new_face, 0.0)?;
    }
    if !c2d.is_null() || !c2d1.is_null() {
        builder.range_on_face(new_edge, new_face, found.first, found.last)?;
    }
    Ok(())
}

/// `RebuildShape(theVertex, theNewVertex)`: a new vertex at the located point, with the
/// orientation and tolerance of the vertex; always true.
pub(super) fn rebuild_vertex(vertex: &Vertex, new_vertex: &mut Vertex) -> Result<bool, Failure> {
    let builder = BRepBuilder::default();
    builder.make_vertex(new_vertex);
    new_vertex.set_orientation(vertex.orientation());
    let point = BRepTool::pnt(vertex)?;
    builder.update_vertex_point(new_vertex, point, BRepTool::vertex_tolerance(vertex)?)?;
    Ok(true)
}

/// The null dereference of a missing surface or curve.
fn null_geometry(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("ShapeUpgrade_RemoveLocations: {what}"),
        "",
    )
}
