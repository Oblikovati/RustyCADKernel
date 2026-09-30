// Rust translation of ModelingData/TKBRep/BRep/BRep_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The closedness queries of `BRep_Tool`.
use super::BRepTool;
use super::curves::representations;
use crate::topexp::{self, Explorer};
use crate::topods::{Edge, Face, Shape};
use crate::toptools::MapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::poly::{Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;

/// The static `IsPlane`: a plane, or the plane basis of an offset or rectangular
/// trimmed surface (NIZNHY-PKV, 2008-10-17).
fn is_plane(surface: &Handle<dyn GeomSurface>) -> bool {
    let offset = surface.cast::<GeomOffsetSurface>();
    if let Some(offset) = offset.get() {
        return offset.basis_surface().cast::<GeomPlane>().is_some();
    }
    let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
    if let Some(trimmed) = trimmed.get() {
        return trimmed.basis_surface().cast::<GeomPlane>().is_some();
    }
    surface.cast::<GeomPlane>().is_some()
}

/// Whether an INTERNAL or EXTERNAL orientation hides the boundary element.
fn is_inner_or_outer(orientation: Orientation) -> bool {
    orientation == Orientation::Internal || orientation == Orientation::External
}

/// The shell and wire test of `IsClosed(S)`: every boundary element of type `kind`
/// (skipped when `skipped` says so) is met an even number of times, and one is met.
fn has_no_free_boundary(
    shape: &Shape,
    kind: ShapeEnum,
    skipped: impl Fn(&Shape) -> Result<bool, Failure>,
) -> Result<bool, Failure> {
    let mut boundary = MapOfShape::default();
    let mut has_bound = false;
    let mut explorer = Explorer::new(
        &shape.oriented(Orientation::Forward),
        kind,
        ShapeEnum::Shape,
    )?;
    while explorer.more() {
        let element = explorer.current().clone();
        explorer.next()?;
        // Degenerated is read first, as natively (it raises for a non-BRep edge).
        if skipped(&element)? || is_inner_or_outer(element.orientation()) {
            continue;
        }
        has_bound = true;
        if !boundary.add(element.clone()) {
            boundary.remove(&element);
        }
    }
    Ok(has_bound && boundary.is_empty())
}

impl BRepTool {
    /// `IsClosed(S)`: a shell without free edges, a wire without free vertices
    /// (INTERNAL and EXTERNAL elements and degenerated edges ignored), an edge whose
    /// two vertices are the same, or else the shape's Closed flag;
    /// `BRepTool::is_closed(&shape)?`.
    pub fn is_closed(shape: &Shape) -> Result<bool, Failure> {
        match shape.shape_type()? {
            ShapeEnum::Shell => has_no_free_boundary(shape, ShapeEnum::Edge, |edge| {
                Self::degenerated(&Edge::unchecked(edge.clone()))
            }),
            ShapeEnum::Wire => has_no_free_boundary(shape, ShapeEnum::Vertex, |_| Ok(false)),
            ShapeEnum::Edge => {
                let (first, last) = topexp::vertices(&Edge::unchecked(shape.clone()), false);
                Ok(!first.is_null() && first.is_same(&last))
            }
            _ => shape.closed(),
        }
    }

    /// `IsClosed(E, F)`: the edge is a seam of the face's surface or of its active
    /// triangulation; `BRepTool::is_closed_on_face(&edge, &face)?`.
    pub fn is_closed_on_face(edge: &Edge, face: &Face) -> Result<bool, Failure> {
        let (surface, location) = Self::surface_with_location(face)?;
        if Self::is_closed_on_surface(edge, &surface, &location)? {
            return Ok(true);
        }
        let (triangulation, location) = Self::triangulation(face, mesh_purpose::NONE)?;
        Self::is_closed_on_triangulation(edge, &triangulation, &location)
    }

    /// `IsClosed(E, S, L)`: false on a plane; else whether the edge has a seam
    /// representation on (`surface`, `location`); `BRepTool::is_closed_on_surface(&edge, &surface, &location)?`.
    pub fn is_closed_on_surface(
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<bool, Failure> {
        if is_plane(surface) {
            return Ok(false);
        }
        let relative = location.predivided(edge.location())?;
        let list = representations(edge, "BRep_Tool::IsClosed")?;
        Ok(list.iter().filter_map(|r| r.get()).any(|r| {
            r.is_curve_on_surface_at(surface, &relative) && r.is_curve_on_closed_surface()
        }))
    }

    /// `IsClosed(E, T, L)`: whether the edge has two node arrays on (`triangulation`,
    /// `location`); false for a null triangulation;
    /// `BRepTool::is_closed_on_triangulation(&edge, &triangulation, &location)?`.
    pub fn is_closed_on_triangulation(
        edge: &Edge,
        triangulation: &Handle<Triangulation>,
        location: &Location,
    ) -> Result<bool, Failure> {
        if triangulation.is_null() {
            return Ok(false);
        }
        let relative = location.predivided(edge.location())?;
        let list = representations(edge, "BRep_Tool::IsClosed")?;
        Ok(list.iter().filter_map(|r| r.get()).any(|r| {
            r.is_polygon_on_triangulation_at(triangulation, &relative)
                && r.is_polygon_on_closed_triangulation()
        }))
    }
}
