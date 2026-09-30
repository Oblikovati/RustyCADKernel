// Rust translation of ModelingData/TKBRep/BRep/BRep_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edge polygon queries of `BRep_Tool`: 2D polygons on surfaces and polygons on
//! triangulations.
use super::BRepTool;
use super::curves::representations;
use crate::brep::BRepCurveRepresentation;
use crate::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::poly::{Polygon2D, PolygonOnTriangulation, Triangulation};
use rustycad_tkmath::toploc::Location;

/// A 2D polygon of an edge with its surface and composed location.
pub type EdgePolygonOnSurface = (Handle<Polygon2D>, Handle<dyn GeomSurface>, Location);
/// A polygon on triangulation of an edge with its triangulation and composed location.
pub type EdgePolygonOnTriangulation = (
    Handle<PolygonOnTriangulation>,
    Handle<Triangulation>,
    Location,
);

/// The representation of rank `index` (1-based) among those accepted by `kind`; the
/// search stops once the count passes `index`.
fn ranked(
    list: &[Handle<dyn BRepCurveRepresentation>],
    index: i32,
    kind: impl Fn(&dyn BRepCurveRepresentation) -> bool,
) -> Option<&dyn BRepCurveRepresentation> {
    let mut rank = 0;
    for representation in list.iter().filter_map(|r| r.get()) {
        if !kind(representation) {
            continue;
        }
        rank += 1;
        if rank > index {
            return None;
        }
        if rank == index {
            return Some(representation);
        }
    }
    None
}

/// The outputs for a found 2D polygon on surface.
fn polygon_on_surface_of(
    edge: &Edge,
    representation: &dyn BRepCurveRepresentation,
) -> Result<EdgePolygonOnSurface, Failure> {
    Ok((
        representation.polygon()?,
        representation.surface()?,
        (edge.location() * &representation.location())?,
    ))
}

/// The outputs for a found polygon on triangulation.
fn polygon_on_triangulation_of(
    edge: &Edge,
    representation: &dyn BRepCurveRepresentation,
) -> Result<EdgePolygonOnTriangulation, Failure> {
    Ok((
        representation.polygon_on_triangulation()?,
        representation.triangulation()?,
        (edge.location() * &representation.location())?,
    ))
}

impl BRepTool {
    /// `PolygonOnSurface(E, F)`: the 2D polygon on the face's surface, for the edge
    /// reversed when the face is REVERSED; `BRepTool::polygon_on_face(&edge, &face)?`.
    pub fn polygon_on_face(edge: &Edge, face: &Face) -> Result<Handle<Polygon2D>, Failure> {
        let (surface, location) = Self::surface_with_location(face)?;
        let mut local = edge.clone();
        if face.orientation() == Orientation::Reversed {
            local.reverse();
        }
        Self::polygon_on_surface(&local, &surface, &location)
    }

    /// `PolygonOnSurface(E, S, L)`: the 2D polygon on (`surface`, `location`), the second
    /// one of a seam for a REVERSED edge; `BRepTool::polygon_on_surface(&edge, &surface, &location)?`.
    pub fn polygon_on_surface(
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<Handle<Polygon2D>, Failure> {
        let relative = location.predivided(edge.location())?;
        let reversed = edge.orientation() == Orientation::Reversed;
        for representation in representations(edge, "BRep_Tool::PolygonOnSurface")? {
            let Some(r) = representation
                .get()
                .filter(|r| r.is_polygon_on_surface_at(surface, &relative))
            else {
                continue;
            };
            return if r.is_polygon_on_closed_surface() && reversed {
                r.polygon2()
            } else {
                r.polygon()
            };
        }
        Ok(Handle::default())
    }

    /// `PolygonOnSurface(E, C, S, L)`: the first 2D polygon of the edge;
    /// `BRepTool::first_polygon_on_surface(&edge)?`.
    pub fn first_polygon_on_surface(edge: &Edge) -> Result<EdgePolygonOnSurface, Failure> {
        let list = representations(edge, "BRep_Tool::PolygonOnSurface")?;
        match list
            .iter()
            .filter_map(|r| r.get())
            .find(|r| r.is_polygon_on_surface())
        {
            Some(r) => polygon_on_surface_of(edge, r),
            None => Ok((Handle::default(), Handle::default(), Location::default())),
        }
    }

    /// `PolygonOnSurface(E, C, S, L, Index)`: the 2D polygon of rank `index` (a seam
    /// counts once); `BRepTool::polygon_on_surface_by_index(&edge, 1)?`.
    pub fn polygon_on_surface_by_index(
        edge: &Edge,
        index: i32,
    ) -> Result<EdgePolygonOnSurface, Failure> {
        let list = representations(edge, "BRep_Tool::PolygonOnSurface")?;
        match ranked(&list, index, |r| r.is_polygon_on_surface()) {
            Some(r) => polygon_on_surface_of(edge, r),
            None => Ok((Handle::default(), Handle::default(), Location::default())),
        }
    }

    /// `PolygonOnTriangulation(E, T, L)`: the node array on (`triangulation`,
    /// `location`), the second one of a seam for a REVERSED edge;
    /// `BRepTool::polygon_on_triangulation(&edge, &triangulation, &location)?`.
    pub fn polygon_on_triangulation(
        edge: &Edge,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> Result<Handle<PolygonOnTriangulation>, Failure> {
        let relative = location.predivided(edge.location())?;
        let reversed = edge.orientation() == Orientation::Reversed;
        for representation in representations(edge, "BRep_Tool::PolygonOnTriangulation")? {
            let Some(r) = representation
                .get()
                .filter(|r| r.is_polygon_on_triangulation_at(triangulation, &relative))
            else {
                continue;
            };
            return if r.is_polygon_on_closed_triangulation() && reversed {
                r.polygon_on_triangulation2()
            } else {
                r.polygon_on_triangulation()
            };
        }
        Ok(Handle::default())
    }

    /// `PolygonOnTriangulation(E, P, T, L)`: the first polygon on triangulation of the
    /// edge; `BRepTool::first_polygon_on_triangulation(&edge)?`.
    pub fn first_polygon_on_triangulation(
        edge: &Edge,
    ) -> Result<EdgePolygonOnTriangulation, Failure> {
        let list = representations(edge, "BRep_Tool::PolygonOnTriangulation")?;
        match list
            .iter()
            .filter_map(|r| r.get())
            .find(|r| r.is_polygon_on_triangulation())
        {
            Some(r) => polygon_on_triangulation_of(edge, r),
            None => Ok((Handle::default(), Handle::default(), Location::default())),
        }
    }

    /// `PolygonOnTriangulation(E, P, T, L, Index)`: the polygon on triangulation of rank
    /// `index` (a seam counts once); `BRepTool::polygon_on_triangulation_by_index(&edge, 1)?`.
    pub fn polygon_on_triangulation_by_index(
        edge: &Edge,
        index: i32,
    ) -> Result<EdgePolygonOnTriangulation, Failure> {
        let list = representations(edge, "BRep_Tool::PolygonOnTriangulation")?;
        match ranked(&list, index, |r| r.is_polygon_on_triangulation()) {
            Some(r) => polygon_on_triangulation_of(edge, r),
            None => Ok((Handle::default(), Handle::default(), Location::default())),
        }
    }
}
