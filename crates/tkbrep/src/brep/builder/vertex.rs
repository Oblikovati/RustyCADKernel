// Created on: 1991-07-02
// Created by: Remi LEQUETTE
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Builder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertex members of `BRep_Builder`.
use super::update_curves::{
    as_gcurve, update_point_on_curve, update_point_on_pcurve, update_point_on_surface,
};
use super::{BRepBuilder, check_unlocked, relative_to};
use crate::brep::access::{edge_tshape, vertex_tshape};
use crate::brep::errors::domain_error;
use crate::brep::{BRepGCurve, BRepTEdge, BRepTVertex, BRepTool};
use crate::topods::{Edge, Face, ShapeIterator, TShape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;

/// `Standard_DomainError("BRep_Builder::Infinite parameter")` for an infinite parameter.
fn check_finite(parameter: f64) -> Result<(), Failure> {
    if precision::is_positive_infinite(parameter) || precision::is_negative_infinite(parameter) {
        return Err(domain_error(
            "BRep_Builder::Infinite parameter",
            &format!("parameter {parameter}"),
        ));
    }
    Ok(())
}

/// The vertex and edge TShapes of `UpdateVertex(V, Par, E, ...)`, both unlocked.
fn vertex_and_edge<'a>(
    vertex: &'a Vertex,
    edge: &'a Edge,
) -> Result<(&'a BRepTVertex, &'a BRepTEdge), Failure> {
    let tvertex = vertex_tshape(vertex, "BRep_Builder::UpdateVertex")?;
    let tedge = edge_tshape(edge, "BRep_Builder::UpdateVertex")?;
    check_unlocked(tvertex, "BRep_Builder::UpdateVertex")?;
    check_unlocked(tedge, "BRep_Builder::UpdateVertex")?;
    Ok((tvertex, tedge))
}

/// "Search the vertex in the edge": the orientation of the last child of the FORWARD
/// edge that is the same vertex, stopping at one with the vertex's orientation;
/// INTERNAL when none, and the vertex orientation for a degenerated edge without
/// vertices (RLE, june 94).
fn orientation_in_edge(vertex: &Vertex, edge: &Edge, tedge: &BRepTEdge) -> Orientation {
    let mut orientation = Orientation::Internal;
    let mut children = ShapeIterator::new(&edge.oriented(Orientation::Forward), true, true);
    if !children.more() && tedge.degenerated() {
        orientation = vertex.orientation();
    }
    while let Ok(child) = children.value() {
        if vertex.is_same(child) {
            orientation = child.orientation();
            if orientation == vertex.orientation() {
                break;
            }
        }
        children.next();
    }
    orientation
}

/// Sets the first (FORWARD) or last (REVERSED) parameter of a parametric curve; false
/// for the other orientations.
fn set_range_end(
    gcurve: &dyn BRepGCurve,
    orientation: Orientation,
    parameter: f64,
) -> Result<bool, Failure> {
    match orientation {
        Orientation::Forward => gcurve.set_first(parameter).map(|()| true),
        Orientation::Reversed => gcurve.set_last(parameter).map(|()| true),
        _ => Ok(false),
    }
}

impl BRepBuilder {
    /// `MakeVertex(V)`: an undefined vertex; the inline native member makes no lock
    /// check; `builder.make_vertex(&mut vertex)`.
    pub fn make_vertex(&self, vertex: &mut Vertex) {
        self.make_shape(vertex, &Handle::new(BRepTVertex::new()).into_cast());
    }

    /// `MakeVertex(V, P, Tol)`; `builder.make_vertex_at(&mut vertex, point, 1e-7)?`.
    pub fn make_vertex_at(
        &self,
        vertex: &mut Vertex,
        point: Point3,
        tolerance: f64,
    ) -> Result<(), Failure> {
        self.make_vertex(vertex);
        self.update_vertex_point(vertex, point, tolerance)
    }

    /// `UpdateVertex(V, P, Tol)`: stores the point in the vertex's own frame and raises
    /// the tolerance; `builder.update_vertex_point(&vertex, point, 1e-7)?`.
    pub fn update_vertex_point(
        &self,
        vertex: &Vertex,
        point: Point3,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let tvertex = vertex_tshape(vertex, "BRep_Builder::UpdateVertex")?;
        check_unlocked(tvertex, "BRep_Builder::UpdateVertex")?;
        let inverse = vertex.location().inverted()?;
        tvertex.set_pnt(point.transformed(inverse.transformation()));
        tvertex.update_tolerance(tolerance);
        tvertex.set_modified(true);
        Ok(())
    }

    /// `UpdateVertex(V, Par, E, Tol)`: a FORWARD or REVERSED vertex of the edge sets the
    /// first or last parameter of every parametric curve; another vertex gets a point
    /// representation on each 3D curve and p-curve;
    /// `builder.update_vertex_on_edge(&vertex, 0.0, &edge, 1e-7)?`.
    pub fn update_vertex_on_edge(
        &self,
        vertex: &Vertex,
        parameter: f64,
        edge: &Edge,
        tolerance: f64,
    ) -> Result<(), Failure> {
        check_finite(parameter)?;
        let (tvertex, tedge) = vertex_and_edge(vertex, edge)?;
        let location = edge.location().predivided(vertex.location())?;
        let orientation = orientation_in_edge(vertex, edge, tedge);
        for representation in tedge.curves() {
            let gcurve = as_gcurve(&representation);
            let Some(gcurve) = gcurve.get() else { continue };
            if !set_range_end(gcurve, orientation, parameter)? {
                add_point_on_curve(tvertex, gcurve, parameter, &location)?;
            }
        }
        if orientation != Orientation::Forward && orientation != Orientation::Reversed {
            tvertex.set_modified(true);
        }
        tvertex.update_tolerance(tolerance);
        tedge.set_modified(true);
        Ok(())
    }

    /// `UpdateVertex(V, Par, E, F, Tol)`: the parameter on the p-curve on the face's
    /// located surface; `builder.update_vertex_on_edge_face(&vertex, 0.0, &edge, &face, 1e-7)?`.
    pub fn update_vertex_on_edge_face(
        &self,
        vertex: &Vertex,
        parameter: f64,
        edge: &Edge,
        face: &Face,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.update_vertex_on_edge_surface(vertex, parameter, edge, &surface, &location, tolerance)
    }

    /// `UpdateVertex(V, Par, E, S, L, Tol)`: like the edge overload on the first p-curve
    /// on (`surface`, `location`). The p-curve is searched with `location` itself, not
    /// relative to the edge (xpu020198, BUC60407); without one it raises
    /// Standard_DomainError ("no pcurve");
    /// `builder.update_vertex_on_edge_surface(&vertex, 0.0, &edge, &surface, &location, 1e-7)?`.
    pub fn update_vertex_on_edge_surface(
        &self,
        vertex: &Vertex,
        parameter: f64,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        tolerance: f64,
    ) -> Result<(), Failure> {
        check_finite(parameter)?;
        let relative = relative_to(location, vertex)?;
        let (tvertex, tedge) = vertex_and_edge(vertex, edge)?;
        let orientation = orientation_in_edge(vertex, edge, tedge);
        let found = tedge
            .curves()
            .into_iter()
            .map(|r| as_gcurve(&r))
            .find(|gcurve| {
                gcurve
                    .get()
                    .is_some_and(|g| g.is_curve_on_surface_at(surface, location))
            });
        let Some(gcurve) = found.as_ref().and_then(|g| g.get()) else {
            return Err(domain_error(
                "BRep_Builder:: no pcurve",
                "no p-curve on the surface at the location",
            ));
        };
        if !set_range_end(gcurve, orientation, parameter)? {
            let pcurve = gcurve.pcurve()?;
            tvertex.change_points(|points| {
                update_point_on_pcurve(points, parameter, &pcurve, surface, &relative)
            });
            tvertex.set_modified(true);
        }
        tvertex.update_tolerance(tolerance);
        tedge.set_modified(true);
        Ok(())
    }

    /// `UpdateVertex(Ve, U, V, F, Tol)`: the (U, V) parameters on the face's surface;
    /// `builder.update_vertex_on_face(&vertex, 0.5, 0.5, &face, 1e-7)?`.
    pub fn update_vertex_on_face(
        &self,
        vertex: &Vertex,
        u: f64,
        v: f64,
        face: &Face,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let tvertex = vertex_tshape(vertex, "BRep_Builder::UpdateVertex")?;
        check_unlocked(tvertex, "BRep_Builder::UpdateVertex")?;
        let (surface, location) = BRepTool::surface_with_location(face)?;
        let relative = relative_to(&location, vertex)?;
        tvertex
            .change_points(|points| update_point_on_surface(points, (u, v), &surface, &relative))?;
        tvertex.update_tolerance(tolerance);
        tvertex.set_modified(true);
        Ok(())
    }

    /// `UpdateVertex(V, Tol)`: raises the tolerance; `builder.update_vertex_tolerance(&vertex, 1e-5)?`.
    pub fn update_vertex_tolerance(&self, vertex: &Vertex, tolerance: f64) -> Result<(), Failure> {
        let tvertex = vertex_tshape(vertex, "BRep_Builder::UpdateVertex")?;
        check_unlocked(tvertex, "BRep_Builder::UpdateVertex")?;
        tvertex.update_tolerance(tolerance);
        tvertex.set_modified(true);
        Ok(())
    }

    /// `Transfert(Ein, Eout, Vin, Vout)`: the parameter of `vertex_in` on `edge_in`
    /// becomes the parameter of `vertex_out` on `edge_out`, with the tolerance of
    /// `vertex_in`; `builder.transfert_vertex(&e_in, &e_out, &v_in, &v_out)?`.
    pub fn transfert_vertex(
        &self,
        edge_in: &Edge,
        edge_out: &Edge,
        vertex_in: &Vertex,
        vertex_out: &Vertex,
    ) -> Result<(), Failure> {
        let tolerance = BRepTool::vertex_tolerance(vertex_in)?;
        let parameter = BRepTool::parameter(vertex_in, edge_in)?;
        self.update_vertex_on_edge(vertex_out, parameter, edge_out, tolerance)
    }
}

/// The INTERNAL/EXTERNAL branch of `UpdateVertex(V, Par, E, Tol)`: a point
/// representation on the 3D curve or on the p-curve of `gcurve`, at `location *
/// GC->Location()`.
fn add_point_on_curve(
    tvertex: &BRepTVertex,
    gcurve: &dyn BRepGCurve,
    parameter: f64,
    location: &Location,
) -> Result<(), Failure> {
    let located = (location * &gcurve.location())?;
    if gcurve.is_curve3d() {
        let curve = gcurve.curve3d()?;
        tvertex.change_points(|points| update_point_on_curve(points, parameter, &curve, &located));
    } else if gcurve.is_curve_on_surface() {
        let (pcurve, surface) = (gcurve.pcurve()?, gcurve.surface()?);
        tvertex.change_points(|points| {
            update_point_on_pcurve(points, parameter, &pcurve, &surface, &located)
        });
    }
    Ok(())
}
