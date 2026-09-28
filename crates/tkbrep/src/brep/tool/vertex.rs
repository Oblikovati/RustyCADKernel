// Created on: 1993-07-07
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKBRep/BRep/BRep_Tool.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertex queries of `BRep_Tool`: the point, the tolerance and the parameters.
use super::BRepTool;
use super::face::floored_tolerance;
use crate::brep::BRepPointRepresentation;
use crate::brep::access::vertex_tshape;
use crate::brep::errors::{no_such_object, null_object};
use crate::topexp::{self, Explorer};
use crate::topods::{Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::toploc::Location;

/// "Search the vertex in the edge": the matching child of the FORWARD edge (the one
/// with the vertex orientation when there are several) and whether the edge is
/// REVERSED with several matches.
fn vertex_in_edge(vertex: &Vertex, edge: &Edge) -> (Shape, bool) {
    let mut found = Shape::default();
    let mut reversed = false;
    let mut children = ShapeIterator::new(&edge.oriented(Orientation::Forward), true, true);
    while let Ok(child) = children.value() {
        if vertex.is_same(child) {
            if found.is_null() {
                found = child.clone();
            } else {
                reversed = edge.orientation() == Orientation::Reversed;
                if child.orientation() == vertex.orientation() {
                    found = child.clone();
                }
            }
        }
        children.next();
    }
    (found, reversed)
}

/// The parameter of a FORWARD (first) or REVERSED (last) vertex in `range`, swapped
/// for a REVERSED edge with several matches; None for the other orientations.
fn range_end(
    orientation: Orientation,
    reversed: bool,
    range: impl FnOnce() -> Result<(f64, f64), Failure>,
) -> Result<Option<f64>, Failure> {
    let pick_last = match orientation {
        Orientation::Forward => reversed,
        Orientation::Reversed => !reversed,
        _ => return Ok(None),
    };
    let (first, last) = range()?;
    Ok(Some(if pick_last { last } else { first }))
}

/// The point representations of a vertex.
fn points(
    vertex: &Vertex,
    member: &str,
) -> Result<Vec<Handle<dyn BRepPointRepresentation>>, Failure> {
    Ok(vertex_tshape(vertex, member)?.points())
}

/// "Closed curves RLE 16 june 94": the parameter `p` of a vertex on a 3D curve, or
/// the range end matching the vertex orientation when both curve ends lie on the
/// vertex.
fn closed_curve_parameter(
    vertex: &Vertex,
    curve: &dyn GeomCurve,
    location: &Location,
    range: (f64, f64),
    parameter: f64,
) -> Result<f64, Failure> {
    let (first, last) = range;
    if precision::is_negative_infinite(first) || precision::is_positive_infinite(last) {
        return Ok(parameter);
    }
    let first_point = curve.value(first)?.transformed(location.transformation());
    let last_point = curve.value(last)?.transformed(location.transformation());
    let tolerance = BRepTool::vertex_tolerance(vertex)?;
    if first_point.distance(&last_point) < tolerance
        && first_point.distance(&BRepTool::pnt(vertex)?) < tolerance
    {
        return Ok(if vertex.orientation() == Orientation::Forward {
            first
        } else {
            last
        });
    }
    Ok(parameter)
}

/// The parameter of an INTERNAL or EXTERNAL vertex on the 3D curve of the edge (or of
/// a degenerated edge); None without a matching point representation.
fn parameter_on_curve(vertex: &Vertex, edge: &Edge) -> Result<Option<f64>, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let location = found.location.predivided(vertex.location())?;
    if found.curve.is_null() && !BRepTool::degenerated(edge)? {
        return Ok(None);
    }
    for point in points(vertex, "BRep_Tool::Parameter")? {
        let Some(point) = point
            .get()
            .filter(|p| p.is_point_on_curve_at(&found.curve, &location))
        else {
            continue;
        };
        let parameter = point.parameter();
        let Some(curve) = found.curve.get() else {
            return Ok(Some(parameter));
        };
        let range = (found.first, found.last);
        return closed_curve_parameter(vertex, curve, &location, range, parameter).map(Some);
    }
    Ok(None)
}

/// The parameter of a vertex on the first p-curve of an edge without 3D curve; a
/// parameter at an end of a closed p-curve becomes the end matching the vertex
/// orientation.
fn parameter_on_first_pcurve(vertex: &Vertex, edge: &Edge) -> Result<Option<f64>, Failure> {
    let found = BRepTool::first_curve_on_surface(edge)?;
    let location = found.location.predivided(vertex.location())?;
    for point in points(vertex, "BRep_Tool::Parameter")? {
        let Some(point) = point.get().filter(|p| {
            p.is_point_on_curve_on_surface_at(&found.pcurve, &found.surface, &location)
        }) else {
            continue;
        };
        let pcurve = found
            .pcurve
            .get()
            .ok_or_else(|| null_object("BRep_Tool::Parameter: the p-curve is null"))?;
        let mut parameter = point.parameter();
        let (first, last) = (pcurve.first_parameter(), pcurve.last_parameter());
        if pcurve.is_closed()? && (parameter == first || parameter == last) {
            parameter = if vertex.orientation() == Orientation::Forward {
                first
            } else {
                last
            };
        }
        return Ok(Some(parameter));
    }
    Ok(None)
}

impl BRepTool {
    /// `Pnt(V)`: the point moved by the vertex location; a null vertex raises
    /// Standard_NullObject ("TopoDS_Vertex hasn't gp_Pnt"); `BRepTool::pnt(&vertex)?`.
    pub fn pnt(vertex: &Vertex) -> Result<Point3, Failure> {
        let point = vertex_tshape(vertex, "BRep_Tool:: TopoDS_Vertex hasn't gp_Pnt")?.pnt();
        if vertex.location().is_identity() {
            return Ok(point);
        }
        Ok(point.transformed(vertex.location().transformation()))
    }

    /// `Tolerance(V)`: the stored tolerance, at least `Precision::Confusion()`;
    /// `BRepTool::vertex_tolerance(&vertex)?`.
    pub fn vertex_tolerance(vertex: &Vertex) -> Result<f64, Failure> {
        Ok(floored_tolerance(
            vertex_tshape(vertex, "BRep_Tool:: TopoDS_Vertex hasn't gp_Pnt")?.tolerance(),
        ))
    }

    /// `Parameter(theV, theE, theParam)`: the range end for a FORWARD or REVERSED vertex
    /// of the edge, else the stored parameter on the 3D curve or on the first p-curve;
    /// None (`false`) when there is none; `BRepTool::find_parameter(&vertex, &edge)?`.
    pub fn find_parameter(vertex: &Vertex, edge: &Edge) -> Result<Option<f64>, Failure> {
        let has_children =
            ShapeIterator::new(&edge.oriented(Orientation::Forward), true, true).more();
        let mut orientation = Orientation::Internal;
        if !has_children && Self::degenerated(edge)? {
            orientation = vertex.orientation();
        }
        let (found, reversed) = vertex_in_edge(vertex, edge);
        if !found.is_null() {
            orientation = found.orientation();
        }
        if let Some(parameter) = range_end(orientation, reversed, || Self::range(edge))? {
            return Ok(Some(parameter));
        }
        let curve = Self::curve_with_location(edge)?.curve;
        if curve.is_some() || Self::degenerated(edge)? {
            return parameter_on_curve(vertex, edge);
        }
        parameter_on_first_pcurve(vertex, edge)
    }

    /// `Parameter(V, E)`: [`BRepTool::find_parameter`], raising Standard_NoSuchObject
    /// ("no parameter on edge") without one; `BRepTool::parameter(&vertex, &edge)?`.
    pub fn parameter(vertex: &Vertex, edge: &Edge) -> Result<f64, Failure> {
        Self::find_parameter(vertex, edge)?.ok_or_else(|| {
            no_such_object(
                "BRep_Tool:: no parameter on edge",
                "the vertex has no parameter on the edge",
            )
        })
    }

    /// `Parameter(V, E, F)`: the parameter on the p-curve on the face's surface;
    /// `BRepTool::parameter_on_face(&vertex, &edge, &face)?`.
    pub fn parameter_on_face(vertex: &Vertex, edge: &Edge, face: &Face) -> Result<f64, Failure> {
        let (surface, location) = Self::surface_with_location(face)?;
        Self::parameter_on_surface(vertex, edge, &surface, &location)
    }

    /// `Parameter(V, E, S, L)`: the range end on the p-curve for a FORWARD or REVERSED
    /// vertex (reading that range sets Modified on the edge, as natively), else the
    /// stored parameter on the p-curve, else on the 3D curve; Standard_NoSuchObject
    /// without one; `BRepTool::parameter_on_surface(&vertex, &edge, &surface, &location)?`.
    pub fn parameter_on_surface(
        vertex: &Vertex,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<f64, Failure> {
        let (found, reversed) = vertex_in_edge(vertex, edge);
        let orientation = if found.is_null() {
            Orientation::Internal
        } else {
            found.orientation()
        };
        let range = || Self::range_on_surface(edge, surface, location);
        if let Some(parameter) = range_end(orientation, reversed, range)? {
            return Ok(parameter);
        }
        let pcurve = Self::curve_on_surface(edge, surface, location)?.pcurve;
        for point in points(vertex, "BRep_Tool::Parameter")? {
            if let Some(point) = point
                .get()
                .filter(|p| p.is_point_on_curve_on_surface_at(&pcurve, surface, location))
            {
                return Ok(point.parameter());
            }
        }
        parameter_on_curve(vertex, edge)?.ok_or_else(|| {
            no_such_object(
                "BRep_Tool:: no parameter on edge",
                "the vertex has no parameter on the surface or the edge",
            )
        })
    }

    /// `Parameters(V, F)`: the stored (U, V) parameters on the face's surface, else the
    /// UV point of the first face edge that has the vertex at an end; Standard_NoSuchObject
    /// ("no parameters on surface") otherwise; `BRepTool::parameters(&vertex, &face)?`.
    pub fn parameters(vertex: &Vertex, face: &Face) -> Result<Point2, Failure> {
        let (surface, location) = Self::surface_with_location(face)?;
        let relative = location.predivided(vertex.location())?;
        for point in points(vertex, "BRep_Tool::Parameters")? {
            if let Some(point) = point
                .get()
                .filter(|p| p.is_point_on_surface_at(&surface, &relative))
            {
                return Ok(Point2::new(point.parameter(), point.parameter2()?));
            }
        }
        let mut explorer = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while explorer.more() {
            let edge = Edge::unchecked(explorer.current().clone());
            let (first, last) = topexp::vertices(&edge, false);
            if vertex.is_same(&first) || vertex.is_same(&last) {
                let (first_uv, last_uv) = Self::uv_points_on_face(&edge, face)?;
                // Ambiguity (natural) for degenerated edges.
                return Ok(if vertex.is_same(&first) {
                    first_uv
                } else {
                    last_uv
                });
            }
            explorer.next()?;
        }
        Err(no_such_object(
            "BRep_Tool:: no parameters on surface",
            "the vertex has no parameters on the face",
        ))
    }

    /// `MaxTolerance(S, SubShape)`: the largest tolerance of the faces, edges or vertices
    /// of `shape`; 0 for another sub-shape type or none;
    /// `BRepTool::max_tolerance(&shape, ShapeEnum::Edge)?`.
    pub fn max_tolerance(shape: &Shape, sub_shape: ShapeEnum) -> Result<f64, Failure> {
        let mut explorer = Explorer::new(shape, sub_shape, ShapeEnum::Shape)?;
        let mut maximum = 0.0_f64;
        while explorer.more() {
            let current = explorer.current().clone();
            let tolerance = match sub_shape {
                ShapeEnum::Face => Self::face_tolerance(&Face::unchecked(current))?,
                ShapeEnum::Edge => Self::edge_tolerance(&Edge::unchecked(current))?,
                ShapeEnum::Vertex => Self::vertex_tolerance(&Vertex::unchecked(current))?,
                _ => return Ok(maximum),
            };
            maximum = maximum.max(tolerance);
            explorer.next()?;
        }
        Ok(maximum)
    }
}
