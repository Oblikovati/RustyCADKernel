// Rust translation of ModelingData/TKBRep/BRep/BRep_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edge flag, range and UV point queries of `BRep_Tool`.
use super::BRepTool;
use super::curve_on_plane::plane_of;
use super::curves::representations;
use super::face::floored_tolerance;
use crate::brep::access::edge_tshape;
use crate::brep::builder::update_curves::as_gcurve;
use crate::brep::{BRepCurveOnClosedSurface, BRepCurveOnSurface, BRepCurveRepresentation};
use crate::topexp;
use crate::topods::{Edge, Face, TShape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Plane3, Point2};
use rustycad_tkmath::toploc::Location;

/// The edge reversed when the face is REVERSED, as the face overloads use it.
fn edge_on_face(edge: &Edge, face: &Face) -> Edge {
    let mut local = edge.clone();
    if face.orientation() == Orientation::Reversed {
        local.reverse();
    }
    local
}

/// The UV point pair of a curve on surface: the second pair of a seam for a REVERSED edge.
fn stored_uv_points(
    representation: &Handle<dyn BRepCurveRepresentation>,
    reversed: bool,
) -> (Point2, Point2) {
    let closed = representation.cast::<BRepCurveOnClosedSurface>();
    if let Some(closed) = closed.get().filter(|_| reversed) {
        return closed.uv_points2();
    }
    representation
        .cast::<BRepCurveOnSurface>()
        .get()
        .map_or_else(Default::default, |r| r.uv_points())
}

/// The UV parameters of a vertex projected on a plane, `(0, 0)` for a null vertex.
fn vertex_on_plane(vertex: &Vertex, inverse: &Location, plane: &Plane3) -> Result<Point2, Failure> {
    if vertex.is_null() {
        return Ok(Point2::new(0.0, 0.0));
    }
    let moved = Vertex::unchecked(vertex.moved(inverse, false)?);
    let (u, v) = elslib::plane_parameters(plane.position(), BRepTool::pnt(&moved)?);
    Ok(Point2::new(u, v))
}

impl BRepTool {
    /// `Tolerance(E)`: the stored tolerance, at least `Precision::Confusion()`;
    /// `BRepTool::edge_tolerance(&edge)?`.
    pub fn edge_tolerance(edge: &Edge) -> Result<f64, Failure> {
        Ok(floored_tolerance(
            edge_tshape(edge, "BRep_Tool::Tolerance")?.tolerance(),
        ))
    }

    /// `SameParameter(E)`; `BRepTool::same_parameter(&edge)?`.
    pub fn same_parameter(edge: &Edge) -> Result<bool, Failure> {
        Ok(edge_tshape(edge, "BRep_Tool::SameParameter")?.same_parameter())
    }

    /// `SameRange(E)`; `BRepTool::same_range(&edge)?`.
    pub fn same_range(edge: &Edge) -> Result<bool, Failure> {
        Ok(edge_tshape(edge, "BRep_Tool::SameRange")?.same_range())
    }

    /// `Degenerated(E)`; `BRepTool::degenerated(&edge)?`.
    pub fn degenerated(edge: &Edge) -> Result<bool, Failure> {
        Ok(edge_tshape(edge, "BRep_Tool::Degenerated")?.degenerated())
    }

    /// `Range(E, First, Last)`: the range of the first non-null 3D curve or curve on
    /// surface, `0, 0` without one; `let (first, last) = BRepTool::range(&edge)?;`.
    pub fn range(edge: &Edge) -> Result<(f64, f64), Failure> {
        for representation in representations(edge, "BRep_Tool::Range")? {
            let Some(r) = representation.get() else {
                continue;
            };
            let gcurve = as_gcurve(&representation);
            let Some(gcurve) = gcurve.get() else { continue };
            if r.is_curve3d() && r.curve3d()?.is_some()
                || !r.is_curve3d() && r.is_curve_on_surface()
            {
                return Ok(gcurve.range());
            }
        }
        Ok((0.0, 0.0))
    }

    /// `Range(E, S, L, First, Last)`: the range of the p-curve on (`surface`,
    /// `location`), else `Range(E)`. It also sets the Modified flag of the edge, as
    /// natively; `BRepTool::range_on_surface(&edge, &surface, &location)?`.
    pub fn range_on_surface(
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(f64, f64), Failure> {
        let relative = location.predivided(edge.location())?;
        let list = representations(edge, "BRep_Tool::Range")?;
        let found = list.iter().find(|r| {
            r.get()
                .is_some_and(|r| r.is_curve_on_surface_at(surface, &relative))
        });
        let range = match found.map(as_gcurve) {
            Some(gcurve) => gcurve.get().map_or((0.0, 0.0), |g| g.range()),
            None => Self::range(edge)?,
        };
        edge_tshape(edge, "BRep_Tool::Range")?.set_modified(true);
        Ok(range)
    }

    /// `Range(E, F, First, Last)`: the range on the face's located surface;
    /// `BRepTool::range_on_face(&edge, &face)?`.
    pub fn range_on_face(edge: &Edge, face: &Face) -> Result<(f64, f64), Failure> {
        let (surface, location) = Self::surface_with_location(face)?;
        Self::range_on_surface(edge, &surface, &location)
    }

    /// `UVPoints(E, S, L, PFirst, PLast)`: the stored UV points of the curve on
    /// (`surface`, `location`) (the second pair of a seam for a REVERSED edge); without
    /// one, on a plane, the vertices projected on the plane; else `(0, 0)` twice;
    /// `BRepTool::uv_points(&edge, &surface, &location)?`.
    pub fn uv_points(
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<(Point2, Point2), Failure> {
        let relative = location.predivided(edge.location())?;
        let reversed = edge.orientation() == Orientation::Reversed;
        let list = representations(edge, "BRep_Tool::UVPoints")?;
        if let Some(found) = list.iter().find(|r| {
            r.get()
                .is_some_and(|r| r.is_curve_on_surface_at(surface, &relative))
        }) {
            return Ok(stored_uv_points(found, reversed));
        }
        let plane = plane_of(surface);
        let Some(plane) = plane.get() else {
            return Ok((Point2::new(0.0, 0.0), Point2::new(0.0, 0.0)));
        };
        let (first, last) = topexp::vertices(edge, false);
        let inverse = location.inverted()?;
        let pln = plane.pln();
        Ok((
            vertex_on_plane(&first, &inverse, &pln)?,
            vertex_on_plane(&last, &inverse, &pln)?,
        ))
    }

    /// `UVPoints(E, F, PFirst, PLast)`: the UV points on the face's surface, for the edge
    /// reversed when the face is REVERSED; `BRepTool::uv_points_on_face(&edge, &face)?`.
    pub fn uv_points_on_face(edge: &Edge, face: &Face) -> Result<(Point2, Point2), Failure> {
        let (surface, location) = Self::surface_with_location(face)?;
        Self::uv_points(&edge_on_face(edge, face), &surface, &location)
    }

    /// `SetUVPoints(E, S, L, PFirst, PLast)`: sets the UV points of EVERY curve on
    /// (`surface`, `location`) (the second pair of a seam for a REVERSED edge);
    /// `BRepTool::set_uv_points(&edge, &surface, &location, first, last)?`.
    pub fn set_uv_points(
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        first: Point2,
        last: Point2,
    ) -> Result<(), Failure> {
        let relative = location.predivided(edge.location())?;
        let reversed = edge.orientation() == Orientation::Reversed;
        for representation in representations(edge, "BRep_Tool::SetUVPoints")? {
            if !representation
                .get()
                .is_some_and(|r| r.is_curve_on_surface_at(surface, &relative))
            {
                continue;
            }
            let closed = representation.cast::<BRepCurveOnClosedSurface>();
            match closed.get().filter(|_| reversed) {
                Some(closed) => closed.set_uv_points2(first, last),
                None => set_first_uv_points(&representation, first, last),
            }
        }
        Ok(())
    }

    /// `SetUVPoints(E, F, PFirst, PLast)`: on the face's located surface, for the edge
    /// reversed when the face is REVERSED; `BRepTool::set_uv_points_on_face(&edge, &face, first, last)?`.
    pub fn set_uv_points_on_face(
        edge: &Edge,
        face: &Face,
        first: Point2,
        last: Point2,
    ) -> Result<(), Failure> {
        let (surface, location) = Self::surface_with_location(face)?;
        Self::set_uv_points(&edge_on_face(edge, face), &surface, &location, first, last)
    }
}

/// `static_cast<BRep_CurveOnSurface*>(cr)->SetUVPoints(...)`.
fn set_first_uv_points(
    representation: &Handle<dyn BRepCurveRepresentation>,
    first: Point2,
    last: Point2,
) {
    if let Some(curve) = representation.cast::<BRepCurveOnSurface>().get() {
        curve.set_uv_points(first, last);
    }
}
