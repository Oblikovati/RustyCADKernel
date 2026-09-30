// Rust translation of BRepLib::UpdateTolerances, InternalUpdateTolerances, UpdTolMap and
// UpdShTol (BRepLib.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::UpdateTolerances`: the tolerances harmonized with the rule
//! Tolerance(VERTEX) >= Tolerance(EDGE) >= Tolerance(FACE), on the shape itself or on
//! copies recorded in a re-shaper.
mod vertices;

use crate::brep_bnd_lib::BRepBndLib;
use rustycad_tkbrep::brep::access::vertex_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, TShape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomRectangularTrimmedSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::SurfaceType;

/// `NCollection_DataMap<TopoDS_Shape, double, TopTools_ShapeMapHasher>`: the new
/// tolerance of each shape.
pub(crate) type ShapeTolerances = DataMap<Shape, f64, ShapeMapHasher>;

/// `BRepLib::UpdateTolerances(S, verifyFaceTolerance)`: harmonizes the tolerances of the
/// shape in place: an edge takes at least the tolerance of its faces, a vertex that of
/// its edges and its distance to their curves (plus twice its epsilon); with
/// `verify_face_tolerance`, the face tolerances are first reset from their surface
/// kind and size, and a vertex takes its computed tolerance even when smaller;
/// `brep_lib::update_tolerances(&shape, false)?`.
pub fn update_tolerances(shape: &Shape, verify_face_tolerance: bool) -> Result<(), Failure> {
    let reshaper = BRepToolsReShape::new();
    internal_update_tolerances(shape, verify_face_tolerance, true, &reshaper)
}

/// `BRepLib::UpdateTolerances(S, theReshaper, verifyFaceTolerance)`: the same without
/// changing the shape: the updated shapes are empty copies recorded as replacements in
/// `reshaper`; `brep_lib::update_tolerances_with_reshape(&shape, &reshaper, false)?`.
pub fn update_tolerances_with_reshape(
    shape: &Shape,
    reshaper: &dyn ReShape,
    verify_face_tolerance: bool,
) -> Result<(), Failure> {
    internal_update_tolerances(shape, verify_face_tolerance, false, reshaper)
}

/// The static `InternalUpdateTolerances`.
pub(crate) fn internal_update_tolerances(
    old_shape: &Shape,
    is_verify_tolerance: bool,
    is_mutable_input: bool,
    reshaper: &dyn ReShape,
) -> Result<(), Failure> {
    let mut sh_to_tol = ShapeTolerances::default();
    if is_verify_tolerance {
        face_tolerances(old_shape, &mut sh_to_tol)?;
    }
    edge_tolerances(old_shape, is_verify_tolerance, &mut sh_to_tol)?;
    vertices::vertex_tolerances(old_shape, is_verify_tolerance, &mut sh_to_tol)?;
    upd_sh_tol(&sh_to_tol, is_mutable_input, reshaper, true)
}

/// "Set tolerance to its minimum value": the tolerance of each face from its surface
/// kind, scaled by the size of its box (at most 0.99). The size reads the raw lower Y and
/// Z bounds of a box open in Y or Z, as natively.
fn face_tolerances(shape: &Shape, sh_to_tol: &mut ShapeTolerances) -> Result<(), Failure> {
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let (mut surface, _) = BRepTool::surface_with_location(&Face::unchecked(face.clone()))?;
        if surface.is_null() {
            continue;
        }
        let mut bounds = BoundingBox3::default();
        BRepBndLib::add(&face, &mut bounds, true)?;
        if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
            surface = trimmed.basis_surface();
        }
        let confusion: f64 = precision::confusion();
        let mut tol: f64 = match GeomAdaptorSurface::with_surface(&surface)?.get_type()? {
            SurfaceType::Plane | SurfaceType::Cylinder | SurfaceType::Cone => confusion,
            SurfaceType::Sphere | SurfaceType::Torus => confusion * 2.,
            _ => confusion * 4.,
        };
        if !bounds.is_whole() {
            tol *= box_size(&bounds)?;
            // Do not process tolerances > 1.
            if tol > 1. {
                tol = 0.99;
            }
        }
        sh_to_tol.bind(face, tol);
    }
    Ok(())
}

/// The size of a face box as InternalUpdateTolerances measures it.
fn box_size(bounds: &BoundingBox3) -> Result<f64, Failure> {
    let [x_min, mut y_min, mut z_min, x_max, y_max, z_max] = bounds.get()?;
    let mut d_max: f64 = 1.;
    if !bounds.is_open_xmin() && !bounds.is_open_xmax() {
        d_max = x_max - x_min;
    }
    if !bounds.is_open_ymin() && !bounds.is_open_ymax() {
        y_min = y_max - y_min;
    }
    if !bounds.is_open_zmin() && !bounds.is_open_zmax() {
        z_min = z_max - z_min;
    }
    if y_min > d_max {
        d_max = y_min;
    }
    if z_min > d_max {
        d_max = z_min;
    }
    Ok(d_max)
}

/// "Process edges": an edge takes the largest tolerance of its faces (the computed one
/// with the verification) when it is above its own.
fn edge_tolerances(
    shape: &Shape,
    is_verify_tolerance: bool,
    sh_to_tol: &mut ShapeTolerances,
) -> Result<(), Failure> {
    let mut parents = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut parents)?;
    for index in 1..=parents.extent() {
        let mut tol: f64 = 0.;
        for face in parents.find_from_index(index)?.iter() {
            let face_tol: f64 = match sh_to_tol.seek(face) {
                Some(&computed) if is_verify_tolerance => computed,
                // tolerance have not been updated
                _ => BRepTool::face_tolerance(&Face::unchecked(face.clone()))?,
            };
            tol = std_max(tol, face_tol);
        }
        // Update can only increase tolerance, so if the edge has a greater tolerance
        // than its faces it is not concerned
        let edge = parents.find_key(index)?.clone();
        if tol > BRepTool::edge_tolerance(&Edge::unchecked(edge.clone()))? {
            sh_to_tol.bind(edge, tol);
        }
    }
    Ok(())
}

/// The static `UpdTolMap`: records `new_tol` for a vertex or an edge whose tolerance it
/// exceeds (the larger one when already recorded); other shapes are ignored.
pub(crate) fn upd_tol_map(
    shape: &Shape,
    new_tol: f64,
    sh_to_tol: &mut ShapeTolerances,
) -> Result<(), Failure> {
    let sh_tol: f64 = match shape.shape_type()? {
        ShapeEnum::Vertex => BRepTool::vertex_tolerance(&Vertex::unchecked(shape.clone()))?,
        ShapeEnum::Edge => BRepTool::edge_tolerance(&Edge::unchecked(shape.clone()))?,
        _ => return Ok(()),
    };
    if new_tol <= sh_tol {
        return Ok(());
    }
    match sh_to_tol.change_seek(shape) {
        Some(old) => *old = std_max(*old, new_tol),
        None => {
            sh_to_tol.bind(shape.clone(), new_tol);
        }
    }
    Ok(())
}

/// The static `UpdShTol`: applies the recorded tolerances, to the shapes themselves
/// (mutable input, or a shape already replaced) or to empty copies recorded in the
/// re-shaper; a vertex takes its tolerance (`force_vertex`) or at least it.
pub(crate) fn upd_sh_tol(
    sh_to_tol: &ShapeTolerances,
    is_mutable_input: bool,
    reshaper: &dyn ReShape,
    force_vertex: bool,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    for (shape, &tol) in sh_to_tol.items() {
        let current: Shape = reshaper.value(shape)?;
        let use_old: bool =
            is_mutable_input || reshaper.is_new_shape(shape) || !current.is_same(shape);
        let new_shape: Shape = if use_old {
            current
        } else {
            flagged_copy(shape)?
        };
        match shape.shape_type()? {
            ShapeEnum::Face => {
                builder.update_face_tolerance(&Face::unchecked(new_shape.clone()), tol)?
            }
            ShapeEnum::Edge => {
                builder.update_edge_tolerance(&Edge::unchecked(new_shape.clone()), tol)?
            }
            ShapeEnum::Vertex => {
                let tvertex = vertex_tshape(&new_shape, "BRepLib::UpdateTolerances")?;
                if force_vertex {
                    tvertex.set_tolerance(tol);
                } else {
                    tvertex.update_tolerance(tol);
                }
                tvertex.set_modified(true);
            }
            _ => {}
        }
        if !use_old {
            reshaper.replace(shape, &new_shape)?;
        }
    }
    Ok(())
}

/// An empty copy of the shape with its sub-shapes and its flags.
fn flagged_copy(shape: &Shape) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut copy: Shape = shape.empty_copied()?;
    // add subshapes from the original shape
    let mut children = ShapeIterator::new(shape, true, true);
    while let Ok(child) = children.value() {
        builder.add(&mut copy, child)?;
        children.next();
    }
    copy.set_free(shape.free()?)?;
    copy.set_checked(shape.checked()?)?;
    copy.set_orientable(shape.orientable()?)?;
    copy.set_closed(shape.closed()?)?;
    copy.set_infinite(shape.infinite()?)?;
    copy.set_convex(shape.convex()?)?;
    Ok(copy)
}
