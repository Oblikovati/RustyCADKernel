// Rust translation of BRepTools::UVBounds and BRepTools::AddUVBounds
// (ModelingData/TKBRep/BRepTools/BRepTools.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The UV bounds of a face, a wire on a face or an edge on a face.
use super::uv_periodicity::{Axis, EdgeBox, is_bspline_periodic};
use crate::brep::BRepTool;
use crate::brep::errors::null_object;
use crate::topexp::Explorer;
use crate::topods::{self, Edge, Face, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;

/// UV bounds in the `UVBounds` output order `(UMin, UMax, VMin, VMax)`.
pub type UVBounds = (f64, f64, f64, f64);

/// The `(UMin, UMax, VMin, VMax)` of a box, all zero for a void box.
fn bounds_of(bounds: &BoundingBox2) -> Result<UVBounds, Failure> {
    if bounds.is_void() {
        return Ok((0.0, 0.0, 0.0, 0.0));
    }
    let [umin, vmin, umax, vmax] = bounds.get()?;
    Ok((umin, umax, vmin, vmax))
}

/// `BRepTools::UVBounds(F, UMin, UMax, VMin, VMax)`: the bounds of `AddUVBounds(F)`,
/// all zero when it adds nothing; `let (umin, umax, vmin, vmax) = uv_bounds(&face)?;`.
pub fn uv_bounds(face: &Face) -> Result<UVBounds, Failure> {
    let mut bounds = BoundingBox2::default();
    add_uv_bounds(face, &mut bounds)?;
    bounds_of(&bounds)
}

/// `BRepTools::UVBounds(F, W, ...)`: the bounds of the edges of `wire` on `face`;
/// `uv_bounds_of_wire(&face, &wire)?`.
pub fn uv_bounds_of_wire(face: &Face, wire: &Wire) -> Result<UVBounds, Failure> {
    let mut bounds = BoundingBox2::default();
    add_uv_bounds_of_wire(face, wire, &mut bounds)?;
    bounds_of(&bounds)
}

/// `BRepTools::UVBounds(F, E, ...)`: the bounds of the p-curve of `edge` on `face`;
/// `uv_bounds_of_edge(&face, &edge)?`.
pub fn uv_bounds_of_edge(face: &Face, edge: &Edge) -> Result<UVBounds, Failure> {
    let mut bounds = BoundingBox2::default();
    add_uv_bounds_of_edge(face, edge, &mut bounds)?;
    bounds_of(&bounds)
}

/// `BRepTools::AddUVBounds(F, B)`: adds the bounds of the edges of the FORWARD face, or
/// the natural bounds of its surface when the edges add nothing (nothing when the
/// face has no surface); `add_uv_bounds(&face, &mut bounds)?`.
pub fn add_uv_bounds(face: &Face, bounds: &mut BoundingBox2) -> Result<(), Failure> {
    let forward = topods::face(&face.oriented(Orientation::Forward))?;
    let mut face_box = BoundingBox2::default();
    for edge in Explorer::new(&forward, ShapeEnum::Edge, ShapeEnum::Shape)? {
        add_uv_bounds_of_edge(&forward, &topods::edge(&edge)?, &mut face_box)?;
    }
    if face_box.is_void() {
        let (surface, _) = BRepTool::surface_with_location(&forward)?;
        let Some(surface) = surface.get() else {
            return Ok(());
        };
        let (umin, umax, vmin, vmax) = surface.bounds();
        face_box.update_bounds([umin, vmin], [umax, vmax]);
    }
    bounds.add_box(&face_box);
    Ok(())
}

/// `BRepTools::AddUVBounds(F, W, B)`: adds the bounds of each edge of `wire` on `face`;
/// `add_uv_bounds_of_wire(&face, &wire, &mut bounds)?`.
pub fn add_uv_bounds_of_wire(
    face: &Face,
    wire: &Wire,
    bounds: &mut BoundingBox2,
) -> Result<(), Failure> {
    for edge in Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)? {
        add_uv_bounds_of_edge(face, &topods::edge(&edge)?, bounds)?;
    }
    Ok(())
}

/// `BRepTools::AddUVBounds(F, E, B)`: adds the box of the p-curve of `edge` on `face`,
/// clamped to the surface bounds in each non-periodic direction (a B-spline surface
/// whose points repeat across its bounds counts as periodic). An edge without a
/// p-curve adds nothing; `add_uv_bounds_of_edge(&face, &edge, &mut bounds)?`.
pub fn add_uv_bounds_of_edge(
    face: &Face,
    edge: &Edge,
    bounds: &mut BoundingBox2,
) -> Result<(), Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    if found.pcurve.is_null() {
        return Ok(());
    }
    let mut curve_box = BoundingBox2::default();
    Add2dCurve::add_curve_range(&found.pcurve, found.first, found.last, 0.0, &mut curve_box)?;
    let [xmin, ymin, xmax, ymax] = if curve_box.is_void() {
        [0.0; 4]
    } else {
        curve_box.get()?
    };
    let edge_box = clamped_edge_box(face, EdgeBox::new(xmin, ymin, xmax, ymax))?;
    let mut surface_box = BoundingBox2::default();
    surface_box.update_bounds(
        [edge_box.xmin, edge_box.ymin],
        [edge_box.xmax, edge_box.ymax],
    );
    bounds.add_box(&surface_box);
    Ok(())
}

/// The p-curve box clamped to the bounds of the face's surface (its basis for a
/// rectangular trimmed surface, whose bounds are read before the unwrap).
fn clamped_edge_box(face: &Face, mut edge_box: EdgeBox) -> Result<EdgeBox, Failure> {
    let (surface, _) = BRepTool::surface_with_location(face)?;
    let Some(stored) = surface.get() else {
        return Err(null_object(
            "BRepTools::AddUVBounds: the face has no surface (a null dereference natively)",
        ));
    };
    let surface_bounds = stored.bounds();
    let basis = trimmed_basis(&surface);
    let Some(basis_ref) = basis.get() else {
        return Err(null_object("BRepTools::AddUVBounds: null basis surface"));
    };
    for axis in [Axis::U, Axis::V] {
        if axis.is_periodic(basis_ref)
            || is_bspline_periodic(&basis, axis, surface_bounds, &edge_box)?
        {
            continue;
        }
        edge_box.clamp(axis, surface_bounds);
    }
    Ok(edge_box)
}

/// The basis surface of a `Geom_RectangularTrimmedSurface`, the surface otherwise.
fn trimmed_basis(surface: &Handle<dyn GeomSurface>) -> Handle<dyn GeomSurface> {
    match surface.cast::<GeomRectangularTrimmedSurface>().get() {
        Some(trimmed) => trimmed.basis_surface(),
        None => surface.clone(),
    }
}
