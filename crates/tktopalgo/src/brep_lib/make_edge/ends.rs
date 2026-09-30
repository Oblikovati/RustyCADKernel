// Created on: 1993-07-23
// Created by: Joelle CHAUVET
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
// Rust translation of the vertex checks shared by the two main Init members of
// BRepLib_MakeEdge.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parameter and vertex checks of the two main `BRepLib_MakeEdge::Init` members
//! (3d curve and curve on surface), which run the same code natively.
use crate::brep_lib::errors::BRepLibEdgeError;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Point3;

/// A step of Init: an exception, a recorded error, or the value.
pub(crate) type Step<T> = Result<Result<T, BRepLibEdgeError>, Failure>;

/// The ordered range and vertices of an Init, `reversed` when the non-periodic
/// parameters were swapped.
pub(crate) struct EdgeRange {
    pub(crate) p1: f64,
    pub(crate) p2: f64,
    pub(crate) v1: Vertex,
    pub(crate) v2: Vertex,
    pub(crate) reversed: bool,
}

/// The periodic adjustment or the reordering of `(pp1, pp2)` with their vertices, and
/// the range check against `[cf, cl]` (`epsilon` is Precision::PConfusion() for
/// BRepLib_MakeEdge, Precision::Confusion() for BRepLib_MakeEdge2d).
pub(crate) fn ordered_range(
    periodic: bool,
    bounds: (f64, f64),
    params: (f64, f64),
    vertices: (&Vertex, &Vertex),
    epsilon: f64,
) -> Result<EdgeRange, BRepLibEdgeError> {
    let (cf, cl) = bounds;
    if periodic {
        let (p1, p2) = elclib::adjust_periodic(cf, cl, epsilon, params.0, params.1);
        let (v1, v2) = (vertices.0.clone(), vertices.1.clone());
        return Ok(EdgeRange {
            p1,
            p2,
            v1,
            v2,
            reversed: false,
        });
    }
    let range = if params.0 < params.1 {
        EdgeRange {
            p1: params.0,
            p2: params.1,
            v1: vertices.0.clone(),
            v2: vertices.1.clone(),
            reversed: false,
        }
    } else {
        EdgeRange {
            p1: params.1,
            p2: params.0,
            v1: vertices.1.clone(),
            v2: vertices.0.clone(),
            reversed: true,
        }
    };
    if cf - range.p1 > epsilon || range.p2 - cl > epsilon {
        return Err(BRepLibEdgeError::ParameterOutOfRange);
    }
    Ok(range)
}

/// The vertices at the ends of a closed curve: one new vertex for none, the given one
/// for one; two given vertices must be the same one within the larger of `preci` and
/// its tolerance of `p1`. Returns true when both were given and pass.
pub(super) fn closed_vertices(range: &mut EdgeRange, p1: Point3, preci: f64) -> Step<bool> {
    if range.v1.is_null() && range.v2.is_null() {
        BRepBuilder::default().make_vertex_at(&mut range.v1, p1, preci)?;
        range.v2 = range.v1.clone();
        return Ok(Ok(false));
    }
    if range.v1.is_null() {
        range.v1 = range.v2.clone();
        return Ok(Ok(false));
    }
    if range.v2.is_null() {
        range.v2 = range.v1.clone();
        return Ok(Ok(false));
    }
    if !range.v1.is_same(&range.v2) || !within(p1, &range.v1, preci)? {
        return Ok(Err(BRepLibEdgeError::DifferentPointsOnClosedCurve));
    }
    Ok(Ok(true))
}

/// The vertices at the ends of an open curve: a new vertex at a finite end without
/// one; a vertex at an infinite end, or away from its end point, is an error.
pub(super) fn open_vertices(
    range: &mut EdgeRange,
    ends: (Option<Point3>, Option<Point3>),
    preci: f64,
) -> Step<()> {
    if let Err(error) = open_end(&mut range.v1, ends.0, preci)? {
        return Ok(Err(error));
    }
    open_end(&mut range.v2, ends.1, preci)
}

/// One end of `open_vertices`: `end` is None at an infinite parameter.
fn open_end(vertex: &mut Vertex, end: Option<Point3>, preci: f64) -> Step<()> {
    let Some(point) = end else {
        if !vertex.is_null() {
            return Ok(Err(BRepLibEdgeError::PointWithInfiniteParameter));
        }
        return Ok(Ok(()));
    };
    if vertex.is_null() {
        BRepBuilder::default().make_vertex_at(vertex, point, preci)?;
    } else if !within(point, vertex, preci)? {
        return Ok(Err(BRepLibEdgeError::DifferentsPointAndParameter));
    }
    Ok(Ok(()))
}

/// `P.Distance(BRep_Tool::Pnt(V)) <= std::max(preci, BRep_Tool::Tolerance(V))`.
fn within(point: Point3, vertex: &Vertex, preci: f64) -> Result<bool, Failure> {
    let distance: f64 = point.distance(&BRepTool::pnt(vertex)?);
    Ok(distance <= std_max(preci, BRepTool::vertex_tolerance(vertex)?))
}

/// The vertices of the point Init members: a vertex at `p1`, and the same vertex at
/// `p2` when `coincident`, else a vertex at `p2`, both of tolerance `tolerance`.
pub(crate) fn vertex_pair(
    p1: Point3,
    p2: Point3,
    coincident: bool,
    tolerance: f64,
) -> Result<(Vertex, Vertex), Failure> {
    let builder = BRepBuilder::default();
    let mut v1 = Vertex::default();
    builder.make_vertex_at(&mut v1, p1, tolerance)?;
    if coincident {
        return Ok((v1.clone(), v1));
    }
    let mut v2 = Vertex::default();
    builder.make_vertex_at(&mut v2, p2, tolerance)?;
    Ok((v1, v2))
}
