// Rust translation of the curve preparation and pcurves shared by BRepFill::Face and
// BRepFill::Shell (ModelingAlgorithms/TKBool/BRepFill/BRepFill.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ruled surface between two edges as `BRepFill::Face` and `BRepFill::Shell` build
//! it: the curves, trimmed and oriented, and the GeomFill_Generator surface.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_fill::GeomFillGenerator;
use rustycad_tkmath::gp::{Direction2, Point2};
use rustycad_tkmath::toploc::Location;

/// The ruled surface between two edges, its bounds, the vertices of each edge in its
/// orientation and the common location when both edges share it.
pub(super) struct RuledPatch {
    pub(super) surface: Handle<dyn GeomSurface>,
    pub(super) bounds: (f64, f64, f64, f64),
    pub(super) v1: (Vertex, Vertex),
    pub(super) v2: (Vertex, Vertex),
    /// `Some(L)` when both curves have the location `L` (`SameLoc`): the face is built
    /// at the identity and moved by `L` afterwards.
    pub(super) same_location: Option<Location>,
}

/// A side curve of the ruled surface and the vertices of its edge in its orientation.
type OrientedSide = (Handle<dyn GeomCurve>, (Vertex, Vertex));

/// The curve of an edge trimmed to its range (a copy when the range is the whole
/// curve), moved by its location unless the locations are shared, reversed for a
/// REVERSED edge, and the vertices of the edge in its orientation.
fn side_curve(
    edge: &Edge,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    location: Option<&Location>,
) -> Result<OrientedSide, Failure> {
    let Some(stored) = curve.get() else {
        return Err(null_curve(edge));
    };
    let (f, l) = range;
    let trim: bool = (f - stored.first_parameter()).abs() > precision::p_confusion()
        || (l - stored.last_parameter()).abs() > precision::p_confusion();
    let copy: Handle<dyn GeomCurve> = if trim {
        Handle::new(GeomTrimmedCurve::new(curve, f, l, true, true)?).cast::<dyn GeomCurve>()
    } else {
        stored.copy()?.cast::<dyn GeomCurve>()
    };
    let Some(owned) = copy.get() else {
        return Err(null_curve(edge));
    };
    if let Some(location) = location {
        owned.transform(location.transformation())?;
    }
    let (first, last) = topexp::vertices(edge, false);
    if edge.orientation() == Orientation::Reversed {
        owned.reverse()?;
        return Ok((copy, (last, first)));
    }
    Ok((copy, (first, last)))
}

/// Standard_NullObject for an edge without 3d curve (a null dereference natively).
fn null_curve(edge: &Edge) -> Failure {
    let message =
        format!("BRepFill: the edge {edge:?} has no 3d curve; expected an edge with a 3d curve");
    Failure::with_type("Standard_NullObject", &message, "BRepFill")
}

/// The GeomFill_Generator surface between the two edges, as `BRepFill::Face` and
/// `BRepFill::Shell` compute it; `let patch = ruled_patch(&e1, &e2)?;`.
pub(super) fn ruled_patch(edge1: &Edge, edge2: &Edge) -> Result<RuledPatch, Failure> {
    let c1 = BRepTool::curve_with_location(edge1)?;
    let c2 = BRepTool::curve_with_location(edge2)?;
    // Compute the location: equal locations are applied to the face, not the curves.
    let same_location: Option<Location> = (c1.location == c2.location).then(|| c1.location.clone());
    let (l1, l2) = match same_location {
        Some(_) => (None, None),
        None => (Some(&c1.location), Some(&c2.location)),
    };
    let (curve1, v1) = side_curve(edge1, &c1.curve, (c1.first, c1.last), l1)?;
    let (curve2, v2) = side_curve(edge2, &c2.curve, (c2.first, c2.last), l2)?;
    let mut generator = GeomFillGenerator::new();
    generator.add_curve(&curve1)?;
    generator.add_curve(&curve2)?;
    generator.perform(precision::p_confusion())?;
    let surface: Handle<dyn GeomSurface> = generator.surface();
    let bounds = match surface.get() {
        Some(s) => s.bounds(),
        None => return Err(null_curve(edge1)),
    };
    Ok(RuledPatch {
        surface,
        bounds,
        v1,
        v2,
        same_location,
    })
}

/// A 2d line through `(u, v)` (`new Geom2d_Line(gp_Pnt2d(u, v), direction)`).
pub(super) fn line2d(u: f64, v: f64, direction: Direction2) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(Point2::new(u, v), direction)).cast::<dyn Geom2dCurve>()
}

/// The iso V = `v` p-curve of a section edge on the face: along -U on `[-l1, -f1]` for
/// a REVERSED edge, along +U on `[f1, l1]` otherwise.
pub(super) fn section_pcurve(
    edge: &Edge,
    face: &Face,
    u_range: (f64, f64),
    v: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let t: f64 = precision::confusion();
    let (f1, l1) = u_range;
    if edge.orientation() == Orientation::Reversed {
        builder.update_edge_pcurve_on_face(edge, &line2d(0., v, Direction2::NEG_X), face, t)?;
        return builder.range_on_face(edge, face, -l1, -f1);
    }
    builder.update_edge_pcurve_on_face(edge, &line2d(0., v, Direction2::X), face, t)?;
    builder.range_on_face(edge, face, f1, l1)
}

/// The lateral edge at the iso U = `u` between two vertices: the iso curve when its
/// ends are farther apart than the larger vertex tolerance, a degenerated edge
/// otherwise; FORWARD at `start`, REVERSED at `end`, on `[v1, v2]`.
pub(super) fn iso_edge(
    surface: &Handle<dyn GeomSurface>,
    u: f64,
    v_range: (f64, f64),
    start: &Vertex,
    end: &Vertex,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let Some(geometry) = surface.get() else {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "BRepFill: null ruled surface; expected a surface",
            "BRepFill",
        ));
    };
    let iso: Handle<dyn GeomCurve> = geometry.u_iso(u)?;
    let tol: f64 = BRepTool::vertex_tolerance(start)?.max(BRepTool::vertex_tolerance(end)?);
    let mut edge = Edge::default();
    let Some(iso_curve) = iso.get() else {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "BRepFill: null iso curve; expected a curve",
            "BRepFill",
        ));
    };
    let (v1, v2) = v_range;
    if iso_curve.value(v1)?.distance(&iso_curve.value(v2)?) > tol {
        builder.make_edge_with_curve(&mut edge, &iso, precision::confusion())?;
    } else {
        builder.make_edge(&mut edge)?;
        builder.degenerated(&edge, true)?;
    }
    builder.add(&mut edge, &start.oriented(Orientation::Forward))?;
    builder.add(&mut edge, &end.oriented(Orientation::Reversed))?;
    builder.range(&edge, v1, v2, false)?;
    Ok(edge)
}

/// The SameParameter and SameRange flags of the four edges of a face cleared.
pub(super) fn clear_same_flags(edges: [&Edge; 4]) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    for edge in edges {
        builder.same_parameter(edge, false)?;
    }
    for edge in edges {
        builder.same_range(edge, false)?;
    }
    Ok(())
}
