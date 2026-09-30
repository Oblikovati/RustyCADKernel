// Created on: 1993-05-07
// Created by: Jacques GOUSSARD
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_RstInt.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_RstInt`: the vertices of a walking or restriction line on the restrictions
//! of a surface (docs/geomalgo-design.md section 17).
mod arcs;
mod helpers;
mod vertices;

use super::line::IntPatchLine;
use super::point::IntPatchPoint;
use super::point_line::IntPatchPointLine;
use super::polyline::IntPatchPolyLine;
use crate::int_start::{TheArc, TheTopolTool, TheVertex};
use crate::int_surf::IntSurfTransition;
use helpers::{PointLineRef, point_line_of};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

/// `IntPatch_RstInt`: one static member.
///
/// ```ignore
/// IntPatchRstInt::put_vertex_on_line(&line, &surf, &*domain, &other, true, 1e-6)?;
/// ```
pub struct IntPatchRstInt;

/// The fixed inputs of `PutVertexOnLine` and what it derives from them.
struct RstInput<'a> {
    line: PointLineRef,
    surf: &'a Adaptor3dSurfaceHandle,
    domain: &'a (dyn TheTopolTool + 'static),
    other_surf: &'a Adaptor3dSurfaceHandle,
    on_first: bool,
    tol: f64,
    /// The closed directions of the surface and of the other one, and 1% of their
    /// ranges as tolerances.
    closed: [bool; 4],
    tol_closed: [f64; 4],
    types: (SurfaceType, SurfaceType),
    surfacetype: SurfaceType,
    periodic: bool,
    bi_periodic: bool,
}

impl RstInput<'_> {
    /// Whether one of the surfaces is closed in a direction.
    fn possibly_closed(&self) -> bool {
        self.closed.iter().any(|&c| c)
    }
}

/// The locals of `PutVertexOnLine` that keep their values from one arc (or point) to
/// the next, as natively.
#[derive(Default)]
struct RstState {
    nbvtx: i32,
    u: f64,
    v: f64,
    w: f64,
    u1: f64,
    v1: f64,
    u2: f64,
    v2: f64,
    paramarc: f64,
    paramline: f64,
    ptsommet: Point3,
    tgline: Vector3,
    d2d: rustycad_tkmath::gp::Vector2,
    sommet: IntPatchPoint,
    ptline: IntPatchPoint,
    vtxarc: TheVertex,
    transarc: IntSurfTransition,
    transline: IntSurfTransition,
    locpt: Vec<Point3>,
    locpt2: Vec<rustycad_tkmath::gp::Point2>,
}

/// The closed directions of both surfaces and their tolerances.
fn closed_directions(surf: &Adaptor3dSurfaceHandle, other: &Adaptor3dSurfaceHandle) -> Result<([bool; 4], [f64; 4]), Failure> {
    let closed = [surf.is_u_closed()?, surf.is_v_closed()?, other.is_u_closed()?, other.is_v_closed()?];
    let mut tols = [0.0; 4];
    if closed.iter().any(|&c| c) {
        if closed[0] {
            tols[0] = (surf.last_u_parameter()? - surf.first_u_parameter()?) * 0.01;
        }
        if closed[1] {
            tols[1] = (surf.last_v_parameter()? - surf.first_v_parameter()?) * 0.01;
        }
        if closed[2] {
            tols[2] = (other.last_u_parameter()? - other.first_u_parameter()?) * 0.01;
        }
        if closed[3] {
            tols[3] = (other.last_v_parameter()? - other.first_v_parameter()?) * 0.01;
        }
    }
    Ok((closed, tols))
}

impl IntPatchRstInt {
    /// `PutVertexOnLine(L, Surf, Domain, OtherSurf, OnFirst, Tol)`: intersects each arc
    /// of the domain of `surf` (the first surface of the intersection when `on_first`)
    /// with the line in 2d (Intf_InterferencePolygon2d on the line polygon and the arc
    /// polygon, shifted by periods), refines each crossing on `other_surf`
    /// (IntPatch_CurvIntSurf), and puts it on the line as a vertex or merges it into an
    /// existing one; then sets the periods of a walking line and computes the vertex
    /// parameters. Standard_DomainError for a line that is neither walking nor
    /// restriction. `IntPatchRstInt::put_vertex_on_line(&l, &s, &*d, &o, true, 1e-6)?`.
    pub fn put_vertex_on_line(
        l: &rustycad_tkernel::standard::Handle<dyn IntPatchLine>,
        surf: &Adaptor3dSurfaceHandle,
        domain: &(dyn TheTopolTool + 'static),
        other_surf: &Adaptor3dSurfaceHandle,
        on_first: bool,
        tol: f64,
    ) -> Result<(), Failure> {
        let line: PointLineRef = point_line_of(l)?;
        let mut tol_plin: f64 = surf.u_resolution(precision::confusion())?;
        tol_plin = std_max(tol_plin, surf.v_resolution(precision::confusion())?);
        tol_plin = std_min(tol_plin, precision::confusion());
        let mut plin = IntPatchPolyLine::with_deflection(tol_plin);
        let mut st = RstState::default();
        match &line {
            PointLineRef::Walking(w) => {
                st.nbvtx = w.nb_vertex();
                plin.set_w_line(on_first, w)?;
            }
            PointLineRef::Restriction(r) => {
                st.nbvtx = r.nb_vertex();
                plin.set_r_line(on_first, r)?;
            }
        }
        if !domain.has_3d() {
            // Do not use the computed deflection in a pure geometric intersection.
            plin.reset_error();
        }
        let input = Self::input(line, (surf, domain, other_surf), on_first, tol)?;
        domain.init();
        while domain.more() {
            let arc: TheArc = domain.value()?;
            input.process_arc(&mut st, &plin, &arc)?;
            domain.next();
        }
        input.finish_line()
    }

    fn input<'a>(
        line: PointLineRef,
        (surf, domain, other_surf): (&'a Adaptor3dSurfaceHandle, &'a (dyn TheTopolTool + 'static), &'a Adaptor3dSurfaceHandle),
        on_first: bool,
        tol: f64,
    ) -> Result<RstInput<'a>, Failure> {
        let (closed, tol_closed) = closed_directions(surf, other_surf)?;
        let (surf1, surf2) = if on_first { (surf, other_surf) } else { (other_surf, surf) };
        let types = (surf1.get_type()?, surf2.get_type()?);
        let surfacetype: SurfaceType = if on_first { types.0 } else { types.1 };
        let periodic: bool = matches!(surfacetype, SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Torus | SurfaceType::Sphere);
        let bi_periodic: bool = surfacetype == SurfaceType::Torus;
        Ok(RstInput { line, surf, domain, other_surf, on_first, tol, closed, tol_closed, types, surfacetype, periodic, bi_periodic })
    }
}

/// The periods of a surface for the vertex parameters of a walking line: 2 PI in U on
/// the cylinder, cone and sphere (and in V on the torus), else its period or the
/// range of a closed direction.
fn line_periods(s: &Adaptor3dSurfaceHandle, t: SurfaceType) -> Result<(f64, f64), Failure> {
    Ok(match t {
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere => (PI + PI, 0.0),
        SurfaceType::Torus => (PI + PI, PI + PI),
        _ => {
            let pu = if s.is_u_periodic()? {
                s.u_period()?
            } else if s.is_u_closed()? {
                s.last_u_parameter()? - s.first_u_parameter()?
            } else {
                0.0
            };
            let pv = if s.is_v_periodic()? {
                s.v_period()?
            } else if s.is_v_closed()? {
                s.last_v_parameter()? - s.first_v_parameter()?
            } else {
                0.0
            };
            (pu, pv)
        }
    })
}

impl RstInput<'_> {
    /// The vertex parameters again: the periods of a walking line first.
    fn finish_line(&self) -> Result<(), Failure> {
        match &self.line {
            PointLineRef::Walking(w) => {
                let (surf1, surf2) = if self.on_first { (self.surf, self.other_surf) } else { (self.other_surf, self.surf) };
                let (pu1, pv1) = line_periods(surf1, self.types.0)?;
                let (pu2, pv2) = line_periods(surf2, self.types.1)?;
                w.set_period(pu1, pv1, pu2, pv2);
                w.compute_vertex_parameters(self.tol)
            }
            PointLineRef::Restriction(r) => r.compute_vertex_parameters(self.tol),
        }
    }
}
