// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of BRepBndLib::AddOptimal (BRepBndLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBndLib::AddOptimal`: each sub-shape is bounded in a local box with the
//! optimal BndLib algorithms, then merged into the result with `Bnd_Box::Update`.
use super::BRepBndLib;
use super::add::{add_mesh_polygon_nodes, add_polygon3d_nodes, mesh_polygon};
use super::face_box::adjust_face_box;
use super::uv_bounds::{can_use_edges, find_exact_uv_bounds};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::bnd_lib::{Add3dCurve, AddSurface};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::poly::mesh_purpose;

impl BRepBndLib {
    /// `AddOptimal(S, B, useTriangulation, useShapeTolerance)`: as [`BRepBndLib::add`],
    /// with the optimal (numerical) boxes of the curves and surfaces: the face of a
    /// surface whose edges bound it gets the boxes of its edges, another face the box of
    /// its exact UV patch shrunk towards its edges; the tolerances of the sub-shapes are
    /// used only with `use_shape_tolerance`, the meshes only with `use_triangulation`;
    /// `BRepBndLib::add_optimal(&shape, &mut bounds, true, false)?`.
    pub fn add_optimal(
        shape: &Shape,
        bounds: &mut BoundingBox3,
        use_triangulation: bool,
        use_shape_tolerance: bool,
    ) -> Result<(), Failure> {
        let flags = (use_triangulation, use_shape_tolerance);
        for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            let local = optimal_face_box(&topods::face(&face)?, flags)?;
            merge(bounds, &local)?;
        }
        for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Face)? {
            let local = optimal_free_edge_box(&topods::edge(&edge)?, flags)?;
            merge(bounds, &local)?;
        }
        for vertex in Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Edge)? {
            let vertex = topods::vertex(&vertex)?;
            let mut local = BoundingBox3::default();
            local.add_point(BRepTool::pnt(&vertex)?);
            local.enlarge(tolerance_if(use_shape_tolerance, || {
                BRepTool::vertex_tolerance(&vertex)
            })?);
            merge(bounds, &local)?;
        }
        Ok(())
    }
}

/// `B.Update(aLocBox.Get())` for a local box that is not void.
fn merge(bounds: &mut BoundingBox3, local: &BoundingBox3) -> Result<(), Failure> {
    if local.is_void() {
        return Ok(());
    }
    let [x0, y0, z0, x1, y1, z1] = local.get()?;
    bounds.update_bounds([x0, y0, z0], [x1, y1, z1]);
    Ok(())
}

/// `useShapeTolerance ? Tolerance(...) : 0.`
fn tolerance_if(
    use_shape_tolerance: bool,
    tolerance: impl FnOnce() -> Result<f64, Failure>,
) -> Result<f64, Failure> {
    if use_shape_tolerance {
        tolerance()
    } else {
        Ok(0.)
    }
}

/// The local box of a face: its mesh (enlarged by its deflection and the face
/// tolerance) when allowed and not void, else its exact geometry (void without a surface).
fn optimal_face_box(
    face: &Face,
    (use_triangulation, use_tol): (bool, bool),
) -> Result<BoundingBox3, Failure> {
    let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
    let mut local = BoundingBox3::default();
    // The mesh is only asked for its bounds when it is wanted and present (`&&`).
    if let Some(mesh) = mesh.get().filter(|_| use_triangulation)
        && mesh.min_max(&mut local, location.transformation(), false)
    {
        local.enlarge(mesh.deflection() + BRepTool::face_tolerance(face)?);
        return Ok(local);
    }
    let (surface, _) = BRepTool::surface_with_location(face)?;
    if surface.is_null() {
        return Ok(local);
    }
    let unrestricted = BRepAdaptorSurface::with_face(face, false)?;
    if can_use_edges(&unrestricted)? {
        edges_face_box(face, use_tol, &mut local)?;
    } else {
        patch_face_box(face, &unrestricted, use_tol, &mut local)?;
    }
    Ok(local)
}

/// A face whose edges bound it: the restricted surface box without edges, else the
/// union of the optimal boxes of its edges.
fn edges_face_box(face: &Face, use_tol: bool, local: &mut BoundingBox3) -> Result<(), Failure> {
    let edges = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    if !edges.more() {
        let surface = BRepAdaptorSurface::with_face(face, true)?;
        let tol = tolerance_if(use_tol, || BRepTool::face_tolerance(face))?;
        return AddSurface::add_optimal(&surface, tol, local);
    }
    local.add_box(&edges_box(face, use_tol)?);
    Ok(())
}

/// The union of the optimal boxes of the face's edges, degenerated and non-geometric
/// edges skipped.
fn edges_box(face: &Face, use_tol: bool) -> Result<BoundingBox3, Failure> {
    let mut union = BoundingBox3::default();
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&edge)?;
        if BRepTool::degenerated(&edge)? || !BRepTool::is_geometric_edge(&edge)? {
            continue;
        }
        union.add_box(&optimal_edge_curve_box(&edge, use_tol)?);
    }
    Ok(union)
}

/// `BndLib_Add3dCurve::AddOptimal(BRepAdaptor_Curve(E), Tol, anEBox)` in a new box.
fn optimal_edge_curve_box(edge: &Edge, use_tol: bool) -> Result<BoundingBox3, Failure> {
    let mut curve_box = BoundingBox3::default();
    let curve = BRepAdaptorCurve::with_edge(edge)?;
    let tol = tolerance_if(use_tol, || BRepTool::edge_tolerance(edge))?;
    Add3dCurve::add_optimal(&curve, tol, &mut curve_box)?;
    Ok(curve_box)
}

/// Another face: the optimal box of its exact UV patch on the unrestricted surface,
/// shrunk towards the box of its edges unless the patch is its natural restriction.
fn patch_face_box(
    face: &Face,
    unrestricted: &BRepAdaptorSurface,
    use_tol: bool,
    local: &mut BoundingBox3,
) -> Result<(), Failure> {
    let tol = tolerance_if(use_tol, || BRepTool::face_tolerance(face))?;
    let (patch, natural) = find_exact_uv_bounds(face, tol)?;
    AddSurface::add_optimal_range(unrestricted, patch, tol, local)?;
    if natural {
        return Ok(());
    }
    let edge_box = edges_box(face, use_tol)?;
    let tol = tolerance_if(use_tol, || BRepTool::face_tolerance(face))?;
    adjust_face_box(unrestricted, patch, local, &edge_box, tol)
}

/// The local box of an edge outside a face: its 3d polygon or its polygon on a
/// triangulation (when allowed, enlarged by the deflection and the edge tolerance),
/// else the optimal box of its curve.
fn optimal_free_edge_box(
    edge: &Edge,
    (use_triangulation, use_tol): (bool, bool),
) -> Result<BoundingBox3, Failure> {
    let mut local = BoundingBox3::default();
    let (polygon, location) = BRepTool::polygon3d(edge)?;
    if let Some(polygon) = polygon
        .get()
        .filter(|p| use_triangulation && p.nb_nodes() > 0)
    {
        add_polygon3d_nodes(polygon, &location, &mut local);
        local.enlarge(
            polygon.deflection() + tolerance_if(use_tol, || BRepTool::edge_tolerance(edge))?,
        );
        return Ok(local);
    }
    let (polygon, mesh, location) = BRepTool::first_polygon_on_triangulation(edge)?;
    if let Some((polygon, mesh)) = mesh_polygon(&polygon, &mesh).filter(|_| use_triangulation) {
        add_mesh_polygon_nodes(polygon, mesh, &location, &mut local)?;
        local.enlarge(
            polygon.deflection() + tolerance_if(use_tol, || BRepTool::edge_tolerance(edge))?,
        );
        return Ok(local);
    }
    if BRepTool::is_geometric_edge(edge)? {
        return optimal_edge_curve_box(edge, use_tol);
    }
    Ok(local)
}
