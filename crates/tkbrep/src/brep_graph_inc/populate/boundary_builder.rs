// Copyright (c) 2026 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_BoundaryBuilder.pxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraphInc_BoundaryBuilder` (private): the rectangular boundary of a
//! natural-restriction face in its surface's parameter box, with merged vertices and
//! seam and degenerate edges on closed and pole iso-curves.
use crate::brep_graph_inc::std_max;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Point3};

/// `SurfaceBoundaryEdge`: an iso-curve edge between two boundary vertices.
pub(super) struct SurfaceBoundaryEdge {
    pub(super) start_vertex: usize,
    pub(super) end_vertex: usize,
    pub(super) curve3d: Handle<dyn GeomCurve>,
    pub(super) first: f64,
    pub(super) last: f64,
    pub(super) is_degenerate: bool,
}

/// `SurfaceBoundaryCoEdge`: one use of a boundary edge with its 2D line.
pub(super) struct SurfaceBoundaryCoEdge {
    pub(super) edge_index: usize,
    pub(super) orientation: Orientation,
    pub(super) pcurve: Handle<dyn Geom2dCurve>,
    pub(super) first: f64,
    pub(super) last: f64,
}

/// `SurfaceBoundary`: the vertex points, edges and coedges of the boundary.
#[derive(Default)]
pub(super) struct SurfaceBoundary {
    pub(super) vertices: Vec<Point3>,
    pub(super) edges: Vec<SurfaceBoundaryEdge>,
    pub(super) co_edges: Vec<SurfaceBoundaryCoEdge>,
}

/// The parameter box of the boundary.
#[derive(Clone, Copy)]
pub(super) struct ParameterBox {
    pub(super) u_min: f64,
    pub(super) u_max: f64,
    pub(super) v_min: f64,
    pub(super) v_max: f64,
}

// `isFullPeriod`: the range spans the period of a periodic direction.
fn is_full_period(
    surface: &dyn GeomSurface,
    u_direction: bool,
    first: f64,
    last: f64,
) -> Result<bool, Failure> {
    let periodic = if u_direction {
        surface.is_u_periodic()
    } else {
        surface.is_v_periodic()
    };
    if !periodic {
        return Ok(false);
    }
    let period = if u_direction {
        surface.u_period()?
    } else {
        surface.v_period()?
    };
    Ok(((last - first).abs() - period).abs() <= precision::p_confusion())
}

// `addSurfaceVertex`: an existing vertex within the tolerance, else a new one.
fn add_vertex(boundary: &mut SurfaceBoundary, point: Point3, tolerance: f64) -> usize {
    let tolerance2 = tolerance * tolerance;
    if let Some(index) = boundary
        .vertices
        .iter()
        .position(|p| p.square_distance(&point) <= tolerance2)
    {
        return index;
    }
    boundary.vertices.push(point);
    boundary.vertices.len() - 1
}

// `isDegenerateIsoCurve`: a circle whose radius is within the tolerance.
fn is_degenerate_iso_curve(curve: &Handle<dyn GeomCurve>, tolerance: f64) -> Result<bool, Failure> {
    if curve.is_null() {
        return Ok(false);
    }
    let adaptor = GeomAdaptorCurve::with_curve(curve)?;
    Ok(adaptor.get_type()? == CurveType::Circle && adaptor.circle()?.radius() <= tolerance)
}

fn add_edge(boundary: &mut SurfaceBoundary, edge: SurfaceBoundaryEdge) -> usize {
    boundary.edges.push(edge);
    boundary.edges.len() - 1
}

// `addSurfaceCoEdge` with a `Geom2d_Line(origin, direction)` p-curve.
fn add_co_edge(
    boundary: &mut SurfaceBoundary,
    edge_index: usize,
    orientation: Orientation,
    line: (Point2, Direction2),
    range: (f64, f64),
) {
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::new(line.0, line.1))).into_cast();
    boundary.co_edges.push(SurfaceBoundaryCoEdge {
        edge_index,
        orientation,
        pcurve,
        first: range.0,
        last: range.1,
    });
}

/// `BuildSurfaceBoundary(theBoundary, theSurface, UMin, UMax, VMin, VMax, theTolerance)`:
/// the four corner vertices (merged within the tolerance and on closed directions), the
/// bottom, right, top and left iso edges (right and top omitted when U or V is closed)
/// and four coedges; false for a null surface or an empty result.
pub(super) fn build_surface_boundary(
    boundary: &mut SurfaceBoundary,
    surface: &Handle<dyn GeomSurface>,
    parameters: ParameterBox,
    tolerance: f64,
) -> Result<bool, Failure> {
    let Some(geometry) = surface.get() else {
        return Ok(false);
    };
    *boundary = SurfaceBoundary::default();
    let tolerance = std_max(tolerance, precision::confusion());
    let ParameterBox {
        u_min,
        u_max,
        v_min,
        v_max,
    } = parameters;
    let u_closed = is_full_period(geometry, true, u_min, u_max)?;
    let v_closed = is_full_period(geometry, false, v_min, v_max)?;
    let corners = corner_vertices(
        boundary,
        geometry,
        parameters,
        tolerance,
        (u_closed, v_closed),
    )?;
    let edges = iso_edges(
        boundary,
        geometry,
        parameters,
        tolerance,
        corners,
        (u_closed, v_closed),
    )?;
    add_boundary_co_edges(boundary, parameters, edges, (u_closed, v_closed));
    Ok(
        !boundary.vertices.is_empty()
            && !boundary.edges.is_empty()
            && !boundary.co_edges.is_empty(),
    )
}

// The corner vertices V00, V10, V11, V01, merged along closed directions.
fn corner_vertices(
    boundary: &mut SurfaceBoundary,
    surface: &dyn GeomSurface,
    p: ParameterBox,
    tolerance: f64,
    closed: (bool, bool),
) -> Result<[usize; 4], Failure> {
    let v00 = add_vertex(boundary, surface.eval_d0(p.u_min, p.v_min)?, tolerance);
    let mut v10 = add_vertex(boundary, surface.eval_d0(p.u_max, p.v_min)?, tolerance);
    let mut v11 = add_vertex(boundary, surface.eval_d0(p.u_max, p.v_max)?, tolerance);
    let mut v01 = add_vertex(boundary, surface.eval_d0(p.u_min, p.v_max)?, tolerance);
    if closed.0 {
        v10 = v00;
        v11 = v01;
    }
    if closed.1 {
        v01 = v00;
        v11 = v10;
    }
    Ok([v00, v10, v11, v01])
}

// The bottom, right, top and left edges (right when U is open, top when V is open);
// the result is [bottom, right, top, left], 0 for an omitted edge as natively.
fn iso_edges(
    boundary: &mut SurfaceBoundary,
    surface: &dyn GeomSurface,
    p: ParameterBox,
    tolerance: f64,
    corners: [usize; 4],
    closed: (bool, bool),
) -> Result<[usize; 4], Failure> {
    let [v00, v10, v11, v01] = corners;
    let curves = [
        surface.v_iso(p.v_min)?,
        surface.u_iso(p.u_max)?,
        surface.v_iso(p.v_max)?,
        surface.u_iso(p.u_min)?,
    ];
    let mut degenerate = [false; 4];
    for (flag, curve) in degenerate.iter_mut().zip(&curves) {
        *flag = is_degenerate_iso_curve(curve, tolerance)?;
    }
    let [bottom_curve, right_curve, top_curve, left_curve] = curves;
    let edge = |start, end, curve, first, last, is_degenerate| SurfaceBoundaryEdge {
        start_vertex: start,
        end_vertex: end,
        curve3d: curve,
        first,
        last,
        is_degenerate,
    };
    let bottom = add_edge(
        boundary,
        edge(v00, v10, bottom_curve, p.u_min, p.u_max, degenerate[0]),
    );
    let right = if closed.0 {
        0
    } else {
        add_edge(
            boundary,
            edge(v10, v11, right_curve, p.v_min, p.v_max, degenerate[1]),
        )
    };
    let top = if closed.1 {
        0
    } else {
        add_edge(
            boundary,
            edge(v01, v11, top_curve, p.u_min, p.u_max, degenerate[2]),
        )
    };
    let left = add_edge(
        boundary,
        edge(v00, v01, left_curve, p.v_min, p.v_max, degenerate[3]),
    );
    Ok([bottom, right, top, left])
}

// The four coedges: bottom FORWARD; right FORWARD (or the left seam FORWARD when U is
// closed); top REVERSED (or the bottom seam when V is closed); left REVERSED (at UMax
// when U is closed).
fn add_boundary_co_edges(
    boundary: &mut SurfaceBoundary,
    p: ParameterBox,
    edges: [usize; 4],
    closed: (bool, bool),
) {
    let [bottom, right, top, left] = edges;
    let du = Direction2::new(1.0, 0.0).expect("unit direction");
    let dv = Direction2::new(0.0, 1.0).expect("unit direction");
    let (u_range, v_range) = ((p.u_min, p.u_max), (p.v_min, p.v_max));
    let at = |u: f64, v: f64| Point2::new(u, v);
    add_co_edge(
        boundary,
        bottom,
        Orientation::Forward,
        (at(p.u_min, p.v_min), du),
        u_range,
    );
    if closed.0 {
        add_co_edge(
            boundary,
            left,
            Orientation::Forward,
            (at(p.u_min, p.v_min), dv),
            v_range,
        );
    } else {
        add_co_edge(
            boundary,
            right,
            Orientation::Forward,
            (at(p.u_max, p.v_min), dv),
            v_range,
        );
    }
    let top_edge = if closed.1 { bottom } else { top };
    add_co_edge(
        boundary,
        top_edge,
        Orientation::Reversed,
        (at(p.u_min, p.v_max), du),
        u_range,
    );
    let left_origin = if closed.0 {
        at(p.u_max, p.v_min)
    } else {
        at(p.u_min, p.v_min)
    };
    add_co_edge(
        boundary,
        left,
        Orientation::Reversed,
        (left_origin, dv),
        v_range,
    );
}
