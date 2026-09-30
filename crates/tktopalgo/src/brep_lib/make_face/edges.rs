// Rust translation of the edges and wires of BRepLib_MakeFace::Init(S, UMin, UMax, VMin,
// VMax, TolDegen) (BRepLib_MakeFace.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges of the finite sides of the parameter box and the wires that bound the face.
use super::sides::{Side, UMAX, UMIN, VMAX, VMIN};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Face, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::geom::SurfaceBounds;
use rustycad_tkg3d::top_abs::Orientation;

/// The p-curves of a side edge: one line, or the two lines of a seam.
enum SidePCurves<'a> {
    Single(&'a Handle<dyn Geom2dCurve>),
    Seam(&'a Handle<dyn Geom2dCurve>, &'a Handle<dyn Geom2dCurve>),
}

/// The edge of one finite side: on its iso curve (empty when degenerated), with its
/// p-curves on the face, its end vertices (FORWARD then REVERSED, where the crossing
/// sides are finite) and its range.
fn side_edge(
    face: &Face,
    side: &Side,
    (pcurves, tol): (SidePCurves<'_>, f64),
    ends: [Option<&Vertex>; 2],
    (first, last): (f64, f64),
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    if side.degenerated {
        builder.make_edge(&mut edge)?;
    } else {
        builder.make_edge_with_curve(&mut edge, &side.curve, side.tol)?;
    }
    match pcurves {
        SidePCurves::Seam(c1, c2) => {
            builder.update_edge_pcurves_on_face(&edge, (c1, c2), face, tol)?
        }
        SidePCurves::Single(c) => builder.update_edge_pcurve_on_face(&edge, c, face, tol)?,
    }
    builder.degenerated(&edge, side.degenerated)?;
    for (vertex, orientation) in ends
        .into_iter()
        .zip([Orientation::Forward, Orientation::Reversed])
    {
        if let Some(vertex) = vertex {
            builder.add(&mut edge, &vertex.oriented(orientation))?;
        }
    }
    builder.range(&edge, first, last, false)?;
    Ok(edge)
}

/// The vertex when the crossing side is finite.
fn end<'v>(sides: &[Side; 4], crossing: usize, vertex: &'v Vertex) -> Option<&'v Vertex> {
    (!sides[crossing].infinite).then_some(vertex)
}

/// "make the edges": eumin, eumax, evmin, evmax (null for an infinite side); a closed
/// direction shares its first edge as the seam (U: the lines of UMax then UMin, V: of
/// VMin then VMax, as natively).
pub(super) fn side_edges(
    face: &Face,
    sides: &[Side; 4],
    [v00, v10, v11, v01]: &[Vertex; 4],
    (umin, umax, vmin, vmax): SurfaceBounds,
    (uclosed, vclosed): (bool, bool),
) -> Result<[Edge; 4], Failure> {
    let [su0, su1, sv0, sv1] = sides;
    let mut edges: [Edge; 4] = Default::default();
    if !su0.infinite {
        let pcurves = match uclosed {
            true => (
                SidePCurves::Seam(&su1.line, &su0.line),
                std_max(su0.tol, su1.tol),
            ),
            false => (SidePCurves::Single(&su0.line), su0.tol),
        };
        let ends = [end(sides, VMIN, v00), end(sides, VMAX, v01)];
        edges[UMIN] = side_edge(face, su0, pcurves, ends, (vmin, vmax))?;
    }
    if !su1.infinite {
        edges[UMAX] = match uclosed {
            true => edges[UMIN].clone(),
            false => {
                let ends = [end(sides, VMIN, v10), end(sides, VMAX, v11)];
                side_edge(
                    face,
                    su1,
                    (SidePCurves::Single(&su1.line), su1.tol),
                    ends,
                    (vmin, vmax),
                )?
            }
        };
    }
    if !sv0.infinite {
        let pcurves = match vclosed {
            true => (
                SidePCurves::Seam(&sv0.line, &sv1.line),
                std_max(sv0.tol, sv1.tol),
            ),
            false => (SidePCurves::Single(&sv0.line), sv0.tol),
        };
        let ends = [end(sides, UMIN, v00), end(sides, UMAX, v10)];
        edges[VMIN] = side_edge(face, sv0, pcurves, ends, (umin, umax))?;
    }
    if !sv1.infinite {
        edges[VMAX] = match vclosed {
            true => edges[VMIN].clone(),
            false => {
                let ends = [end(sides, UMIN, v01), end(sides, UMAX, v11)];
                side_edge(
                    face,
                    sv1,
                    (SidePCurves::Single(&sv1.line), sv1.tol),
                    ends,
                    (umin, umax),
                )?
            }
        };
    }
    Ok(edges)
}

/// A wire of the given edges, added to the face.
fn add_wire(face: &mut Face, edges: &[&Edge]) -> Result<Wire, Failure> {
    let builder = BRepBuilder::default();
    let mut wire = Wire::default();
    builder.make_wire(&mut wire);
    for edge in edges {
        builder.add(&mut wire, edge)?;
    }
    builder.add(face, &wire)?;
    Ok(wire)
}

/// "make the wires and add them to the face": eumin and evmax reversed; two wires for
/// a band bounded in U only or in V only, else one wire of the finite sides (closed when
/// all four are finite); the face is Closed as its closed directions allow.
pub(super) fn add_wires(
    face: &mut Face,
    sides: &[Side; 4],
    mut edges: [Edge; 4],
    (uclosed, vclosed): (bool, bool),
) -> Result<(), Failure> {
    edges[UMIN].set_orientation(Orientation::Reversed);
    edges[VMAX].set_orientation(Orientation::Reversed);
    let finite = [UMIN, UMAX, VMIN, VMAX].map(|i| !sides[i].infinite);
    if finite == [true, true, false, false] {
        // two wires in u
        add_wire(face, &[&edges[UMIN]])?;
        add_wire(face, &[&edges[UMAX]])?;
        return face.set_closed(uclosed);
    }
    if finite == [false, false, true, true] {
        // two wires in v
        add_wire(face, &[&edges[VMIN]])?;
        add_wire(face, &[&edges[VMAX]])?;
        return face.set_closed(vclosed);
    }
    if finite == [false; 4] {
        return Ok(());
    }
    // one wire
    let order = [UMIN, VMIN, UMAX, VMAX];
    let chosen: Vec<&Edge> = order
        .iter()
        .filter(|&&i| finite[i])
        .map(|&i| &edges[i])
        .collect();
    let wire = add_wire(face, &chosen)?;
    wire.set_closed(finite == [true; 4])?;
    face.set_closed(uclosed && vclosed)
}
