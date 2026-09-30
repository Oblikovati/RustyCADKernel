// Rust translation of the members that BRepSweep_Translation.cxx and
// BRepSweep_Rotation.cxx write identically (SetParameters, SetGeneratingParameter, the
// copy of a generating edge and the static SetThePCurve) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry helpers shared by the translation and the rotation sweeps.
use crate::sweep::SweepNumShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

/// `SetParameters`: the parameters of the generating vertex on the generating face,
/// glued on the new vertex of the new cap face at Precision::PConfusion().
pub(crate) fn set_cap_parameters(
    builder: BRepBuilder,
    new_face: &Shape,
    new_vertex: &Shape,
    gen_f: &Shape,
    gen_v: &Shape,
) -> Result<(), Failure> {
    let uv = BRepTool::parameters(&topods::vertex(gen_v)?, &topods::face(gen_f)?)?;
    builder.update_vertex_on_face(
        &topods::vertex(new_vertex)?,
        uv.x(),
        uv.y(),
        &topods::face(new_face)?,
        precision::p_confusion(),
    )
}

/// `SetGeneratingParameter`: the parameter of the generating vertex on the generating
/// edge, on the new vertex (with the orientation of the generating vertex) and the new
/// edge.
pub(crate) fn set_generating_vertex_parameter(
    builder: BRepBuilder,
    new_edge: &Shape,
    new_vertex: &Shape,
    gen_e: &Shape,
    gen_v: &Shape,
) -> Result<(), Failure> {
    let vbid = topods::vertex(&new_vertex.oriented(gen_v.orientation()))?;
    let parameter = BRepTool::parameter(&topods::vertex(gen_v)?, &topods::edge(gen_e)?)?;
    builder.update_vertex_on_edge(
        &vbid,
        parameter,
        &topods::edge(new_edge)?,
        precision::p_confusion(),
    )
}

/// `MakeEmptyGeneratingEdge` without its copy check: a degenerated edge with the
/// generating tolerance, or an edge on a copy of the located 3D curve of the generating
/// edge, moved by `location` at the last directing vertex (a null curve stays null).
pub(crate) fn copied_generating_edge(
    builder: BRepBuilder,
    gen_e: &Shape,
    dir_v: &SweepNumShape,
    location: &Location,
) -> Result<Shape, Failure> {
    let gen_e = topods::edge(gen_e)?;
    let mut new_e = Edge::default();
    if BRepTool::degenerated(&gen_e)? {
        builder.make_edge(&mut new_e)?;
        builder.update_edge_tolerance(&new_e, BRepTool::edge_tolerance(&gen_e)?)?;
        builder.degenerated(&new_e, true)?;
        return Ok(new_e.into());
    }
    let found = BRepTool::curve_with_location(&gen_e)?;
    let mut curve: Handle<dyn GeomCurve> = found.curve.clone();
    if let Some(source) = found.curve.get() {
        curve = source.copy()?.cast();
        let copy = curve
            .get()
            .ok_or_else(|| null_curve("MakeEmptyGeneratingEdge"))?;
        copy.transform(found.location.transformation())?;
        if dir_v.index() == 2 {
            copy.transform(location.transformation())?;
        }
    }
    builder.make_edge_with_curve(&mut new_e, &curve, BRepTool::edge_tolerance(&gen_e)?)?;
    Ok(new_e.into())
}

/// The Standard_NullObject of a member that dereferences a missing curve (a crash
/// natively).
pub(crate) fn null_curve(member: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!("BRepSweep::{member}: the edge has no 3D curve; expected a geometric edge"),
    )
}

/// The static `SetThePCurve` of the translation and the rotation: `pcurve` becomes the
/// p-curve of `edge` on `face`, or, when the edge already has one on a non-planar face,
/// the other p-curve of a seam (first for a REVERSED edge). `tolerance` gives the
/// tolerance of the update from the new p-curve.
pub(crate) fn set_the_pcurve(
    builder: BRepBuilder,
    edge: &Shape,
    face: &Shape,
    orientation: Orientation,
    pcurve: &Handle<dyn Geom2dCurve>,
    tolerance: impl Fn(&Edge, &Face, &Handle<dyn Geom2dCurve>) -> Result<f64, Failure>,
) -> Result<(), Failure> {
    let (edge, face) = (topods::edge(edge)?, topods::face(face)?);
    let (surface, _) = BRepTool::surface_with_location(&face)?;
    let plane: Handle<GeomPlane> = surface.cast();
    let mut other: Handle<dyn Geom2dCurve> = Handle::default();
    if plane.is_null() {
        other = BRepTool::curve_on_face(&edge, &face)?.pcurve;
    }
    if other.is_null() {
        let tol = tolerance(&edge, &face, pcurve)?;
        return builder.update_edge_pcurve_on_face(&edge, pcurve, &face, tol);
    }
    let tol = tolerance(&edge, &face, pcurve)?;
    let pair = if orientation == Orientation::Reversed {
        (&other, pcurve)
    } else {
        (pcurve, &other)
    };
    builder.update_edge_pcurves_on_face(&edge, pair, &face, tol)
}
