// Rust translation of BRepTools::Triangulation, LoadTriangulation, UnloadTriangulation,
// ActivateTriangulation, LoadAllTriangulations and UnloadAllTriangulations
// (ModelingData/TKBRep/BRepTools/BRepTools.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The mesh queries of BRepTools and the deferred triangulations of faces.
//!
//! The native load members take an `OSD_FileSystem` (the default one when null) and pass
//! it to `Poly_Triangulation::LoadDeferredData`. The TKMath port's deferred storage owns
//! its input instead (docs/tkmath-progress.md), so the file system chosen here is not
//! forwarded; the argument is kept for the API.
use crate::brep::errors::program_error;
use crate::brep::{BRepBuilder, BRepTool};
use crate::topexp::Explorer;
use crate::topods::{self, Face, Shape};
use rustycad_tkernel::osd::FileSystem;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::poly::{Triangulation, mesh_purpose};
use rustycad_tkmath::toploc::Location;

/// The faces of `shape`, in exploration order.
fn faces_of(shape: &Shape) -> Result<Vec<Face>, Failure> {
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?
        .map(|face| topods::face(&face))
        .collect()
}

/// `BRepTools::Triangulation(theShape, theLinDefl, theToCheckFreeEdges)`: every face has
/// a triangulation of deflection at most `lin_defl` with a polygon on it for each of its
/// edges; with `check_free_edges`, every edge outside a face has a 3D polygon, or a
/// polygon on a triangulation, of deflection at most `lin_defl`;
/// `triangulation(&shape, 0.1, false)?`.
pub fn triangulation(
    shape: &Shape,
    lin_defl: f64,
    check_free_edges: bool,
) -> Result<bool, Failure> {
    for face in faces_of(shape)? {
        if !is_face_meshed(&face, lin_defl)? {
            return Ok(false);
        }
    }
    if !check_free_edges {
        return Ok(true);
    }
    let mut dummy_location = Location::default();
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Face)? {
        if !is_free_edge_meshed(&topods::edge(&edge)?, lin_defl, &mut dummy_location)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn is_face_meshed(face: &Face, lin_defl: f64) -> Result<bool, Failure> {
    let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
    let Some(stored) = mesh.get() else {
        return Ok(false);
    };
    if stored.deflection() > lin_defl {
        return Ok(false);
    }
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&edge)?;
        if BRepTool::polygon_on_triangulation(&edge, &mesh, &location)?.is_null() {
            return Ok(false);
        }
    }
    Ok(true)
}

/// A free edge is meshed by a 3D polygon of deflection at most `lin_defl`. Natively the
/// fallback asks `BRep_Tool::PolygonOnTriangulation(E, T, L)` for the polygon on the
/// still-null `anEdgeTri` (the lookup overload, not the one that returns the
/// triangulation), so an edge without a 3D polygon is never meshed; the lookup is kept
/// for its exceptions.
fn is_free_edge_meshed(
    edge: &crate::topods::Edge,
    lin_defl: f64,
    dummy_location: &mut Location,
) -> Result<bool, Failure> {
    let (polygon, location) = BRepTool::polygon3d(edge)?;
    *dummy_location = location;
    if let Some(polygon) = polygon.get() {
        return Ok(polygon.deflection() <= lin_defl);
    }
    let edge_mesh: Handle<Triangulation> = Handle::default();
    let polygon = BRepTool::polygon_on_triangulation(edge, &edge_mesh, dummy_location)?;
    let Some(mesh) = edge_mesh.get() else {
        return Ok(false);
    };
    Ok(!polygon.is_null() && mesh.deflection() <= lin_defl)
}

/// `Standard_ASSERT_RAISE(theTriangulationIdx >= minimum, ...)`.
fn check_index(index: i32, minimum: i32) -> Result<(), Failure> {
    if index < minimum {
        return Err(program_error(
            "Invalid negative triangulation index!",
            &format!("index {index}; expected at least {minimum}"),
        ));
    }
    Ok(())
}

/// The active triangulation (`index` -1) or the one of that index in the face's list;
/// null when the index is out of range.
fn triangulation_at(face: &Face, index: i32) -> Result<Handle<Triangulation>, Failure> {
    if index == -1 {
        return Ok(BRepTool::triangulation(face, mesh_purpose::NONE)?.0);
    }
    let (list, _) = BRepTool::triangulations(face)?;
    Ok(usize::try_from(index)
        .ok()
        .and_then(|position| list.get(position).cloned())
        .unwrap_or_default())
}

/// `BRepTools::LoadTriangulation(theShape, theTriangulationIdx, theToSetAsActive,
/// theFileSystem)`: loads the deferred data of the chosen triangulation of each face
/// (-1: the active one), making it active when asked and `index` is not -1; true when
/// one was loaded. An index below -1 raises Standard_ProgramError;
/// `load_triangulation(&shape, -1, false, None)?`.
pub fn load_triangulation(
    shape: &Shape,
    index: i32,
    to_set_as_active: bool,
    _file_system: Option<&dyn FileSystem>,
) -> Result<bool, Failure> {
    check_index(index, -1)?;
    let mut was_loaded = false;
    for face in faces_of(shape)? {
        let mesh = triangulation_at(&face, index)?;
        let Some(stored) = mesh.get().filter(|m| m.has_deferred_data()) else {
            continue;
        };
        if stored.load_deferred_data() {
            was_loaded = true;
            if to_set_as_active && index != -1 {
                BRepBuilder::default().update_face_triangulation(&face, &mesh, false)?;
            }
        }
    }
    Ok(was_loaded)
}

/// `BRepTools::LoadAllTriangulations(theShape, theFileSystem)`: loads the deferred data
/// of every triangulation of every face; the result is the outcome of the last load
/// attempted, as natively; `load_all_triangulations(&shape, None)?`.
pub fn load_all_triangulations(
    shape: &Shape,
    _file_system: Option<&dyn FileSystem>,
) -> Result<bool, Failure> {
    let mut was_loaded = false;
    for face in faces_of(shape)? {
        for mesh in BRepTool::triangulations(&face)?.0 {
            if let Some(stored) = mesh.get().filter(|m| m.has_deferred_data()) {
                was_loaded = stored.load_deferred_data();
            }
        }
    }
    Ok(was_loaded)
}

/// `BRepTools::UnloadTriangulation(theShape, theTriangulationIdx)`: releases the data of
/// the chosen triangulation of each face that has deferred storage; the result is the
/// outcome of the last release; `unload_triangulation(&shape, -1)?`.
pub fn unload_triangulation(shape: &Shape, index: i32) -> Result<bool, Failure> {
    check_index(index, -1)?;
    let mut was_unloaded = false;
    for face in faces_of(shape)? {
        let mesh = triangulation_at(&face, index)?;
        if let Some(stored) = mesh.get().filter(|m| m.has_deferred_data()) {
            was_unloaded = stored.unload_deferred_data();
        }
    }
    Ok(was_unloaded)
}

/// `BRepTools::UnloadAllTriangulations(theShape)`: releases the data of every
/// triangulation with deferred storage; the result is the outcome of the last release;
/// `unload_all_triangulations(&shape)?`.
pub fn unload_all_triangulations(shape: &Shape) -> Result<bool, Failure> {
    let mut was_unloaded = false;
    for face in faces_of(shape)? {
        for mesh in BRepTool::triangulations(&face)?.0 {
            if let Some(stored) = mesh.get().filter(|m| m.has_deferred_data()) {
                was_unloaded = stored.unload_deferred_data();
            }
        }
    }
    Ok(was_unloaded)
}

/// `BRepTools::ActivateTriangulation(theShape, theTriangulationIdx,
/// theToActivateStrictly)`: makes the triangulation of that index active on each face
/// (the last one when the index is out of range, unless `strictly`); an index below 0
/// raises Standard_ProgramError; `activate_triangulation(&shape, 1, false)?`.
pub fn activate_triangulation(shape: &Shape, index: i32, strictly: bool) -> Result<bool, Failure> {
    check_index(index, 0)?;
    let mut was_activated = false;
    for face in faces_of(shape)? {
        let (list, _) = BRepTool::triangulations(&face)?;
        let count = list.len() as i32;
        if index >= count && strictly {
            continue;
        }
        let position = if index >= count { count - 1 } else { index };
        let Some(active) = usize::try_from(position).ok().and_then(|p| list.get(p)) else {
            continue;
        };
        if active.is_null() {
            continue;
        }
        BRepBuilder::default().update_face_triangulation(&face, active, false)?;
        was_activated = true;
    }
    Ok(was_activated)
}
