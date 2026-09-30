// Rust translation of BRepLib::ContinuityOfFaces and its SurfaceProperties helper
// (BRepLib.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::ContinuityOfFaces`: the continuity between two faces along an edge,
//! sampled at 21 points.
mod properties;
mod samples;

use properties::SurfaceProperties;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomElementarySurface, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::toploc::Location;

/// The edge as seen from each face and its curves on them.
struct EdgeCurves {
    in_face2: Edge,
    curve1: Handle<dyn Geom2dCurve>,
    curve2: Handle<dyn Geom2dCurve>,
}

/// The FORWARD-face occurrence of the edge in the first face (null when absent).
fn edge_in_face(edge: &Edge, face: &Face) -> Result<Edge, Failure> {
    let mut explorer = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        if explorer.current().is_same(edge) {
            return Ok(Edge::unchecked(explorer.current().clone()));
        }
        explorer.next()?;
    }
    Ok(Edge::default())
}

/// The curves of the edge on both faces: on two distinct faces for which the edge is a
/// seam, the occurrence found in the first face (and its reverse on the second face);
/// otherwise the edge itself (reversed on the second face for a seam of one face).
/// None when the first face lacks the seam edge.
fn edge_curves(edge: &Edge, face1: &Face, face2: &Face) -> Result<Option<EdgeCurves>, Failure> {
    let seam_on_both = !face1.is_same(face2)
        && BRepTool::is_closed_on_face(edge, face1)?
        && BRepTool::is_closed_on_face(edge, face2)?;
    if seam_on_both {
        // Find the edge in the face 1: this edge will have correct orientation
        let forward1 = Face::unchecked(face1.oriented(Orientation::Forward));
        let in_face1 = edge_in_face(edge, &forward1)?;
        if in_face1.is_null() {
            return Ok(None);
        }
        let curve1 = BRepTool::curve_on_face(&in_face1, &forward1)?.pcurve;
        let forward2 = Face::unchecked(face2.oriented(Orientation::Forward));
        let in_face2 = Edge::unchecked(in_face1.reversed());
        let curve2 = BRepTool::curve_on_face(&in_face2, &forward2)?.pcurve;
        return Ok(Some(EdgeCurves {
            in_face2,
            curve1,
            curve2,
        }));
    }
    // Obtaining of pcurves of edge on two faces.
    let curve1 = BRepTool::curve_on_face(edge, face1)?.pcurve;
    // For the case of seam edge
    let in_face2 = if face1.is_same(face2) {
        Edge::unchecked(edge.reversed())
    } else {
        edge.clone()
    };
    let curve2 = BRepTool::curve_on_face(&in_face2, face2)?.pcurve;
    Ok(Some(EdgeCurves {
        in_face2,
        curve1,
        curve2,
    }))
}

/// The surface of the face without its rectangular trimming, and the face location.
fn basis_surface(face: &Face) -> Result<(Handle<dyn GeomSurface>, Location), Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
        return Ok((trimmed.basis_surface(), location));
    }
    Ok((surface, location))
}

/// `BRepLib::ContinuityOfFaces(theEdge, theFace1, theFace2, theAngleTol)`: C0 when a
/// curve on surface is missing or the faces fold back; CN for a seam of one elementary
/// surface; otherwise the lowest continuity (G1, C1, G2, C2) found at 21 samples of the
/// edge range shrunk by 1%, starting from C2 (CN for two elementary surfaces, where C2
/// also becomes CN); `brep_lib::continuity_of_faces(&edge, &f1, &f2, 1e-10)?`.
pub fn continuity_of_faces(
    edge: &Edge,
    face1: &Face,
    face2: &Face,
    angle_tol: f64,
) -> Result<Continuity, Failure> {
    let is_seam: bool = face1.is_equal(face2);
    let Some(curves) = edge_curves(edge, face1, face2)? else {
        return Ok(Continuity::C0);
    };
    if curves.curve1.is_null() || curves.curve2.is_null() {
        return Ok(Continuity::C0);
    }
    let (surface1, location1) = basis_surface(face1)?;
    let (surface2, location2) = basis_surface(face2)?;
    // seam edge on elementary surface is always CN
    let is_elementary: bool = surface1.cast::<dyn GeomElementarySurface>().is_some()
        && surface2.cast::<dyn GeomElementarySurface>().is_some();
    if is_seam && is_elementary {
        return Ok(Continuity::CN);
    }
    let reversed1: bool = face1.orientation() == Orientation::Reversed;
    let reversed2: bool = face2.orientation() == Orientation::Reversed;
    let sp1 = SurfaceProperties::new(
        &surface1,
        location1.transformation(),
        &curves.curve1,
        reversed1,
    )?;
    let sp2 = SurfaceProperties::new(
        &surface2,
        location2.transformation(),
        &curves.curve2,
        reversed2,
    )?;
    let sampler = samples::Sampler {
        edge,
        face2,
        in_face2: &curves.in_face2,
        angle_tol,
        faces_reversed: (reversed1, reversed2),
    };
    let continuity: Continuity = sampler.run(sp1, sp2, is_elementary)?;
    // according to the list of supported elementary surfaces,
    // if the continuity is C2, than it is totally CN
    if is_elementary && continuity == Continuity::C2 {
        return Ok(Continuity::CN);
    }
    Ok(continuity)
}
